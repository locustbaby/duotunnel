#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    not(target_env = "msvc")
))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use anyhow::Result;
use arc_swap::ArcSwap;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
#[cfg(feature = "dial9-telemetry")]
use std::path::PathBuf;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
mod config;
mod control_client;
mod egress;
mod handlers;
mod hot_reload;
mod listener_mgr;
mod local_auth;
mod metrics;
mod null_stores;
mod registry;
mod tunnel_handler;
use config::{
    ConfigSource, DbSource, FileSource, IngressMode, MergedSource, ServerConfigFile,
    ServerEgressUpstream, TunnelManagement,
};
use null_stores::{NullConfigSource, NullRuleStore};
use registry::{new_shared_registry, SharedRegistry};
use tunnel_lib::{HttpClientParams, RouteTarget, VhostRouter};
use tunnel_store::{AuthStore, RuleStore};

#[cfg(feature = "dial9-telemetry")]
static DIAL9_HANDLE: std::sync::OnceLock<dial9_tokio_telemetry::telemetry::TelemetryHandle> =
    std::sync::OnceLock::new();

// proxy-rt–local spawn helper: used only inside proxy_main (multi-thread runtime).
pub(crate) fn spawn_task<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[cfg(feature = "dial9-telemetry")]
    if std::env::var_os("DIAL9_TRACE_PATH").is_some() {
        if let Some(handle) = DIAL9_HANDLE.get() {
            return handle.spawn(future);
        }
    }

    tokio::task::spawn(future)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, default_value = "config/server.yaml", global = true)]
    config: String,
    /// Address of tunnel-ctld watch endpoint (e.g. 127.0.0.1:7788).
    /// When set, server fetches all routing and token config from ctld via
    /// long-lived watch stream instead of reading SQLite directly.
    #[arg(long, global = true)]
    ctld_addr: Option<String>,
}
#[derive(Subcommand, Debug)]
enum Commands {
    Run,
    Token {
        #[command(subcommand)]
        action: TokenAction,
    },
}
#[derive(Subcommand, Debug)]
enum TokenAction {
    Create {
        #[arg(long)]
        name: String,
    },
    List,
    Revoke {
        #[arg(long)]
        name: String,
    },
    Rotate {
        #[arg(long)]
        name: String,
    },
}
pub type HttpRouterMap = Arc<HashMap<u16, Arc<VhostRouter<RouteTarget>>>>;
pub struct RoutingSnapshot {
    pub http_routers: HttpRouterMap,
    pub tunnel_management: Arc<TunnelManagement>,
    pub egress_map: Arc<egress::ServerEgressMap>,
}
pub use tunnel_lib::PeekBufPool;

pub struct ServerState {
    pub config: Arc<ServerConfigFile>,
    pub registry: SharedRegistry,
    pub quic_semaphore: Arc<Semaphore>,
    pub tcp_semaphore: Arc<Semaphore>,
    pub tcp_params: tunnel_lib::TcpParams,
    pub proxy_buffer_params: tunnel_lib::ProxyBufferParams,
    pub peek_buf_pool: PeekBufPool,
    pub routing: Arc<ArcSwap<RoutingSnapshot>>,
    pub auth_store: Arc<dyn AuthStore>,
    pub rule_store: Arc<dyn RuleStore>,
    pub config_source: Arc<dyn ConfigSource>,
    pub revocation_tx: tokio::sync::broadcast::Sender<String>,
    /// Present when running in ctld-managed mode; None in standalone mode.
    pub local_token_cache: Option<Arc<local_auth::LocalTokenCache>>,
    pub listeners: listener_mgr::ListenerManager,
}
async fn build_stores(database_url: &str) -> Result<(Arc<dyn AuthStore>, Arc<dyn RuleStore>)> {
    let pool = tunnel_store::open_sqlite_pool(database_url).await?;
    let auth_store = tunnel_store::sqlite::SqliteAuthStore::from_pool(pool.clone());
    auth_store.migrate().await?;
    let rule_store = tunnel_store::sqlite_rules::SqliteRuleStore::new(pool);
    rule_store.migrate().await?;
    Ok((Arc::new(auth_store), Arc::new(rule_store)))
}
fn build_config_source(config_path: &str, rule_store: Arc<dyn RuleStore>) -> Arc<dyn ConfigSource> {
    Arc::new(MergedSource::new(
        Box::new(DbSource::new(rule_store)),
        Box::new(FileSource::new(config_path)),
    ))
}
fn main() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    #[cfg(feature = "dial9-telemetry")]
    if let Some(trace_path) = std::env::var_os("DIAL9_TRACE_PATH").map(PathBuf::from) {
        return run_with_dial9(trace_path, async_main());
    }
    run_with_tokio(async_main())
}
async fn async_main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Token { action }) => handle_token_command(&cli.config, action).await,
        Some(Commands::Run) | None => run_server(&cli.config, cli.ctld_addr.as_deref()),
    }
}
fn run_with_tokio(fut: impl Future<Output = Result<()>>) -> Result<()> {
    tunnel_lib::build_proxy_runtime().block_on(fut)
}
#[cfg(feature = "dial9-telemetry")]
fn run_with_dial9(trace_path: PathBuf, fut: impl Future<Output = Result<()>>) -> Result<()> {
    use dial9_tokio_telemetry::telemetry::{
        CpuProfilingConfig, RotatingWriter, SchedEventConfig, TracedRuntime,
    };
    let writer = RotatingWriter::builder()
        .base_path(&trace_path)
        .max_file_size(512 * 1024 * 1024)
        .max_total_size(512 * 1024 * 1024)
        .build()?;
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    tunnel_lib::apply_worker_threads(&mut builder);
    let trace_path_display = trace_path.display().to_string();
    let trace_file_display = {
        let stem = trace_path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("trace");
        trace_path
            .with_file_name(format!("{stem}.0.bin"))
            .display()
            .to_string()
    };
    let (runtime, guard) = TracedRuntime::builder()
        .with_task_tracking(true)
        .with_trace_path(trace_path)
        .with_cpu_profiling(CpuProfilingConfig::default())
        .with_sched_events(SchedEventConfig {
            include_kernel: true,
        })
        .build_and_start_with_writer(builder, writer)?;
    let _ = DIAL9_HANDLE.set(guard.handle());
    info!("dial9 trace started, base path: {trace_path_display}");
    let result = runtime.block_on(async {
        let mut sigterm = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )?;
        tokio::select! {
            r = fut => r,
            _ = sigterm.recv() => {
                info!("SIGTERM received, starting graceful shutdown");
                Ok(())
            }
        }
    });
    info!("runtime stopped, flushing dial9 trace (timeout 30s)");
    drop(runtime);
    match guard.graceful_shutdown(std::time::Duration::from_secs(30)) {
        Ok(()) => info!("dial9 trace flush complete, output: {trace_file_display}"),
        Err(e) => error!("dial9 trace flush error: {e}"),
    }
    result
}
fn init_observability(log_level: &str) {
    tunnel_lib::infra::observability::init_tracing(log_level);
}
async fn handle_token_command(config_path: &str, action: TokenAction) -> Result<()> {
    let config = ServerConfigFile::load(config_path)?;
    let log_level = config.server.log_level.as_deref().unwrap_or("info");
    init_observability(log_level);
    let pool = tunnel_store::open_sqlite_pool(&config.server.database_url).await?;
    let auth = tunnel_store::sqlite::SqliteAuthStore::from_pool(pool);
    auth.migrate().await?;
    let store = Arc::new(auth) as Arc<dyn AuthStore>;
    match action {
        TokenAction::Create { name } => {
            let token = store.create_client(&name).await?;
            println!("{}", token);
        }
        TokenAction::List => {
            let entries = store.list_tokens().await?;
            println!(
                "{:<20} {:<10} {:<8} {:<10} {:<20} REVOKED",
                "NAME", "CLIENT", "TOKEN_ID", "STATUS", "CREATED"
            );
            for e in entries {
                println!(
                    "{:<20} {:<10} {:<8} {:<10} {:<20} {}",
                    e.client_name,
                    e.client_status,
                    e.token_id,
                    e.token_status,
                    e.created_at,
                    e.revoked_at.as_deref().unwrap_or("-")
                );
            }
        }
        TokenAction::Revoke { name } => {
            store.revoke_token(&name).await?;
            println!("token revoked for '{}'", name);
        }
        TokenAction::Rotate { name } => {
            let token = store.rotate_token(&name).await?;
            println!("{}", token);
        }
    }
    Ok(())
}
fn run_server(config_path: &str, ctld_addr: Option<&str>) -> Result<()> {
    let config = ServerConfigFile::load(config_path)?;
    if ctld_addr.is_none() {
        config::validate_server_config(&config)?;
    }
    let log_level = config.server.log_level.as_deref().unwrap_or("info");
    init_observability(log_level);
    info!("Starting DuoTunnel Server");
    info!(tunnel_port = %config.server.tunnel_port, "Configuration loaded");
    tunnel_lib::init_cert_cache(&config.server.pki);

    // Install Prometheus recorder before any runtime starts so no metrics are lost.
    {
        use metrics_exporter_prometheus::PrometheusBuilder;
        let handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install prometheus recorder");
        crate::metrics::set_handle(handle);
    }

    let proxy_rt = tunnel_lib::build_proxy_runtime();
    let proxy_handle = proxy_rt.handle().clone();
    let shutdown = CancellationToken::new();
    let ready = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // ── metrics thread ──────────────────────────────────────────────────────────
    let metrics_thread = if let Some(metrics_port) = config.server.metrics_port {
        let ready2 = ready.clone();
        let shutdown2 = shutdown.clone();
        Some(std::thread::spawn(move || {
            let rt = tunnel_lib::build_single_thread_runtime("metrics-worker");
            rt.block_on(async move {
                if let Err(e) =
                    handlers::metrics::run_metrics_server(metrics_port, ready2, shutdown2).await
                {
                    error!(port = %metrics_port, error = %e, "Metrics server failed");
                }
            });
        }))
    } else {
        None
    };

    // ── background thread ───────────────────────────────────────────────────────
    let config_path_bg = config_path.to_string();
    let ctld_addr_bg = ctld_addr.map(|s| s.to_string());
    let shutdown_bg = shutdown.clone();
    let proxy_handle_bg = proxy_handle.clone();
    let background_thread = {
        // build state synchronously in proxy_rt so async stores are initialised
        let state = proxy_rt.block_on(build_server_state(&config, config_path, ctld_addr))?;
        let state_bg = state.clone();
        let t = std::thread::spawn(move || {
            let rt = tunnel_lib::build_single_thread_runtime("bg-worker");
            rt.block_on(background_main(
                state_bg,
                config_path_bg,
                ctld_addr_bg.as_deref(),
                shutdown_bg,
                proxy_handle_bg,
            ));
        });
        (state, t)
    };
    let (state, background_thread) = background_thread;

    // ── proxy main (blocks main thread) ────────────────────────────────────────
    let result = proxy_rt.block_on(proxy_main(state, shutdown, ready));

    background_thread.join().ok();
    if let Some(t) = metrics_thread {
        t.join().ok();
    }

    result
}

async fn build_server_state(
    config: &ServerConfigFile,
    config_path: &str,
    ctld_addr: Option<&str>,
) -> Result<Arc<ServerState>> {
    #[allow(clippy::type_complexity)]
    let (auth_store, rule_store, config_source, local_token_cache): (
        Arc<dyn AuthStore>,
        Arc<dyn RuleStore>,
        Arc<dyn ConfigSource>,
        Option<Arc<local_auth::LocalTokenCache>>,
    ) = if ctld_addr.is_some() {
        info!("running in ctld-managed mode; no local SQLite stores");
        let token_cache = Arc::new(local_auth::LocalTokenCache::new());
        let auth: Arc<dyn AuthStore> = token_cache.clone();
        (
            auth,
            Arc::new(NullRuleStore),
            Arc::new(NullConfigSource),
            Some(token_cache),
        )
    } else {
        let (auth_store, rule_store) = build_stores(&config.server.database_url).await?;
        info!(url = %config.server.database_url, "auth and rule stores initialized (shared pool)");
        match rule_store.is_routing_empty().await {
            Ok(true) => {
                if let Err(e) = config::sync_file_to_db(config, rule_store.as_ref()).await {
                    tracing::warn!(error = %e, "failed to seed routing DB from YAML (non-fatal)");
                } else {
                    info!("routing rules seeded into DB from config file (first boot)");
                }
            }
            Ok(false) => info!("routing DB already populated, skipping YAML seed"),
            Err(e) => {
                tracing::warn!(error = %e, "could not check routing DB state, skipping YAML seed");
            }
        }
        let cs = build_config_source(config_path, rule_store.clone());
        (auth_store, rule_store, cs, None)
    };

    let http_params = HttpClientParams::from(&config.server.http_pool);
    let (tm, egress) = config_source.load().await?;
    let initial_snapshot = build_routing_snapshot(&tm, &egress, &http_params);
    let (revocation_tx, _) = tokio::sync::broadcast::channel::<String>(64);
    let proxy_buffer_params = tunnel_lib::ProxyBufferParams::from(&config.server.proxy_buffers);
    Ok(Arc::new(ServerState {
        tcp_params: tunnel_lib::TcpParams::from(&config.server.tcp),
        peek_buf_pool: PeekBufPool::new(proxy_buffer_params.peek_buf_size),
        proxy_buffer_params,
        quic_semaphore: Arc::new(Semaphore::new(config.server.max_connections)),
        tcp_semaphore: Arc::new(Semaphore::new(config.server.max_tcp_connections)),
        routing: Arc::new(ArcSwap::from_pointee(initial_snapshot)),
        registry: new_shared_registry(),
        config: Arc::new(config.clone()),
        auth_store,
        rule_store,
        config_source,
        revocation_tx,
        local_token_cache,
        listeners: listener_mgr::ListenerManager::new(),
    }))
}

async fn proxy_main(
    state: Arc<ServerState>,
    shutdown: CancellationToken,
    ready: Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    info!(
        max_quic_connections = %state.config.server.max_connections,
        max_tcp_connections = %state.config.server.max_tcp_connections,
        "Connection limits configured"
    );

    // Sync initial listeners before QUIC starts.
    {
        let routing = state.routing.load();
        let listeners: Vec<_> = routing.tunnel_management.server_ingress_routing.listeners.to_vec();
        sync_listeners(&state, &listeners);
    }

    let quic_state = state.clone();
    let quic_handle =
        crate::spawn_task(async move { handlers::quic::run_quic_server(quic_state, ready).await });

    tokio::select! {
        r = quic_handle => { r??; }
        _ = shutdown.cancelled() => {}
    }
    Ok(())
}

async fn background_main(
    state: Arc<ServerState>,
    config_path: String,
    ctld_addr: Option<&str>,
    shutdown: CancellationToken,
    _proxy_handle: tokio::runtime::Handle,
) {
    if let Some(addr_str) = ctld_addr {
        match addr_str.parse::<std::net::SocketAddr>() {
            Ok(addr) => {
                info!(addr = %addr, "starting ctld watch client");
                control_client::spawn_control_client(addr, state);
            }
            Err(e) => error!(error = %e, "invalid ctld_addr"),
        }
    } else {
        hot_reload::spawn_config_watcher(config_path, state);
    }
    shutdown.cancelled().await;
}
pub fn build_routing_snapshot(
    tm: &TunnelManagement,
    egress: &ServerEgressUpstream,
    http_params: &HttpClientParams,
) -> RoutingSnapshot {
    let mut http_routers: HashMap<u16, Arc<VhostRouter<RouteTarget>>> = HashMap::new();
    for listener in &tm.server_ingress_routing.listeners {
        if let IngressMode::Http(cfg) = &listener.mode {
            let router: VhostRouter<RouteTarget> = VhostRouter::new();
            for rule in &cfg.vhost {
                router.add_route(
                    &rule.match_host,
                    RouteTarget {
                        group_id: Arc::from(rule.client_group.as_str()),
                        proxy_name: Arc::from(rule.proxy_name.as_str()),
                    },
                );
            }
            info!(
                port = listener.port,
                routes = router.len(),
                "http listener router initialized"
            );
            http_routers.insert(listener.port, Arc::new(router));
        }
    }
    let egress_map = egress::ServerEgressMap::from_config(egress, http_params);
    RoutingSnapshot {
        http_routers: Arc::new(http_routers),
        tunnel_management: Arc::new(tm.clone()),
        egress_map: Arc::new(egress_map),
    }
}
pub use listener_mgr::sync_listeners;
