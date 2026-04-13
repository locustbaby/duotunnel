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
use std::sync::atomic::AtomicBool;
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
use tunnel_lib::{HttpClientParams, RouteTarget, RuleSet};
use tunnel_store::{AuthStore, RuleStore};

#[cfg(feature = "dial9-telemetry")]
static DIAL9_HANDLE: std::sync::OnceLock<dial9_tokio_telemetry::telemetry::TelemetryHandle> =
    std::sync::OnceLock::new();

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
pub type IngressRuleSet = Arc<RuleSet<RouteTarget>>;
pub struct RoutingSnapshot {
    pub ingress_rules: IngressRuleSet,
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
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Token { action }) => {
            let rt = tunnel_lib::build_proxy_runtime();
            rt.block_on(handle_token_command(&cli.config, action))
        }
        Some(Commands::Run) | None => {
            #[cfg(feature = "dial9-telemetry")]
            if let Some(trace_path) = std::env::var_os("DIAL9_TRACE_PATH").map(std::path::PathBuf::from) {
                return run_with_dial9(trace_path, &cli.config, cli.ctld_addr.as_deref());
            }
            run_server(&cli.config, cli.ctld_addr.as_deref())
        }
    }
}
fn run_server(config_path: &str, ctld_addr: Option<&str>) -> Result<()> {
    run_server_with_proxy_runtime(tunnel_lib::build_proxy_runtime(), config_path, ctld_addr)
}

fn run_server_with_proxy_runtime(
    proxy_rt: tokio::runtime::Runtime,
    config_path: &str,
    ctld_addr: Option<&str>,
) -> Result<()> {
    let config = ServerConfigFile::load(config_path)?;
    if ctld_addr.is_none() {
        config::validate_server_config(&config)?;
    }
    let log_level = config.server.log_level.as_deref().unwrap_or("info");
    init_observability(log_level);
    info!("Starting DuoTunnel Server");
    info!(tunnel_port = %config.server.tunnel_port, "Configuration loaded");
    tunnel_lib::init_cert_cache(&config.server.pki);

    // Install Prometheus recorder before any runtime starts.
    let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install prometheus recorder");
    crate::metrics::set_handle(handle);

    let proxy_handle = proxy_rt.handle().clone();

    let shutdown = CancellationToken::new();
    let ready = Arc::new(AtomicBool::new(false));
    let accept_workers = config.server.accept_workers.unwrap_or(4);
    anyhow::ensure!(accept_workers > 0, "server.accept_workers must be >= 1");

    // metrics runtime (1 thread)
    let metrics_thread = if let Some(metrics_port) = config.server.metrics_port {
        let metrics_rt = tunnel_lib::build_single_thread_runtime("metrics");
        let ready2 = ready.clone();
        let shutdown2 = shutdown.clone();
        Some(std::thread::spawn(move || {
            metrics_rt.block_on(async move {
                if let Err(e) = handlers::metrics::run_metrics_server(metrics_port, ready2, shutdown2).await {
                    error!(port = %metrics_port, error = %e, "metrics server failed");
                }
            });
        }))
    } else {
        None
    };

    // background runtime (1 thread): hot_reload or control_client
    let background_rt = tunnel_lib::build_single_thread_runtime("background");
    let config_path_str = config_path.to_string();
    let ctld_addr_owned = ctld_addr.map(|s| s.to_string());
    let background_state = proxy_rt.block_on(async {
        build_server_state(config, &config_path_str, ctld_addr_owned.as_deref()).await
    })?;

    let bg_state = background_state.clone();
    let bg_proxy_handle = proxy_handle.clone();
    let bg_shutdown = shutdown.clone();
    let background_thread = std::thread::spawn(move || {
        background_rt.block_on(async move {
            if let Some(ref addr_str) = ctld_addr_owned {
                let addr: std::net::SocketAddr = addr_str.parse().expect("invalid ctld addr");
                info!(addr = %addr, "starting ctld watch client");
                control_client::spawn_control_client(addr, bg_state.clone(), bg_proxy_handle.clone(), accept_workers, bg_shutdown.clone());
            } else {
                hot_reload::spawn_config_watcher(config_path_str, bg_state.clone(), bg_proxy_handle.clone(), accept_workers, bg_shutdown.clone());
            }
            bg_shutdown.cancelled().await;
        });
    });

    // proxy runtime (N threads): QUIC + ingress listeners
    let proxy_state = background_state;
    let proxy_shutdown = shutdown.clone();
    let signal_shutdown = shutdown.clone();
    proxy_rt.block_on(async move {
        {
            let routing = proxy_state.routing.load();
            let listeners: Vec<_> = routing.tunnel_management.server_ingress_routing.listeners.to_vec();
            listener_mgr::sync_listeners(&proxy_state, &listeners, &proxy_handle, accept_workers);
        }
        let quic_state = proxy_state.clone();
        let quic_ready = ready.clone();
        let quic_handle = tokio::task::spawn(async move {
            if let Err(e) = handlers::quic::run_quic_server(quic_state, quic_ready).await {
                error!(error = %e, "QUIC server failed");
            }
        });

        tokio::select! {
            _ = proxy_shutdown.cancelled() => {}
            _ = tokio::signal::ctrl_c() => {
                info!("SIGINT received, starting graceful shutdown");
                signal_shutdown.cancel();
            }
            _ = async {
                let mut sigterm = tokio::signal::unix::signal(
                    tokio::signal::unix::SignalKind::terminate(),
                ).unwrap();
                sigterm.recv().await
            } => {
                info!("SIGTERM received, starting graceful shutdown");
                signal_shutdown.cancel();
            }
            result = quic_handle => {
                if let Err(e) = result {
                    error!(error = %e, "QUIC server task panicked");
                } else {
                    error!("QUIC server exited unexpectedly");
                }
                signal_shutdown.cancel();
            }
        }
    });

    shutdown.cancel();
    background_thread.join().ok();
    if let Some(t) = metrics_thread {
        t.join().ok();
    }
    Ok(())
}

async fn build_server_state(
    config: ServerConfigFile,
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
                if let Err(e) = config::sync_file_to_db(&config, rule_store.as_ref()).await {
                    tracing::warn!(error = %e, "failed to seed routing DB from YAML (non-fatal)");
                } else {
                    info!("routing rules seeded into DB from config file (first boot)");
                }
            }
            Ok(false) => {
                info!("routing DB already populated, skipping YAML seed");
            }
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
    info!(
        max_quic_connections = %config.server.max_connections,
        max_tcp_connections = %config.server.max_tcp_connections,
        "Connection limits configured"
    );
    Ok(Arc::new(ServerState {
        tcp_params: tunnel_lib::TcpParams::from(&config.server.tcp),
        peek_buf_pool: PeekBufPool::new(proxy_buffer_params.peek_buf_size),
        proxy_buffer_params,
        quic_semaphore: Arc::new(Semaphore::new(config.server.max_connections)),
        tcp_semaphore: Arc::new(Semaphore::new(config.server.max_tcp_connections)),
        routing: Arc::new(ArcSwap::from_pointee(initial_snapshot)),
        registry: new_shared_registry(),
        config: Arc::new(config),
        auth_store,
        rule_store,
        config_source,
        revocation_tx,
        local_token_cache,
        listeners: listener_mgr::ListenerManager::new(),
    }))
}

#[cfg(feature = "dial9-telemetry")]
fn run_with_dial9(trace_path: PathBuf, config_path: &str, ctld_addr: Option<&str>) -> Result<()> {
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
    let result = run_server_with_proxy_runtime(runtime, config_path, ctld_addr);
    info!("runtime stopped, flushing dial9 trace (timeout 30s)");
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
pub fn build_routing_snapshot(
    tm: &TunnelManagement,
    egress: &ServerEgressUpstream,
    http_params: &HttpClientParams,
) -> RoutingSnapshot {
    let mut ingress_rules: RuleSet<RouteTarget> = RuleSet::new();
    for listener in &tm.server_ingress_routing.listeners {
        match &listener.mode {
            IngressMode::Http(cfg) => {
                for rule in &cfg.vhost {
                    ingress_rules.add_port_host_rule(
                        listener.port,
                        &rule.match_host,
                        RouteTarget {
                            group_id: Arc::from(rule.client_group.as_str()),
                            proxy_name: Arc::from(rule.proxy_name.as_str()),
                        },
                    );
                }
                info!(
                    port = listener.port,
                    routes = cfg.vhost.len(),
                    "http listener rule set initialized"
                );
            }
            IngressMode::Tcp(cfg) => ingress_rules.add_port_rule(
                listener.port,
                RouteTarget {
                    group_id: Arc::from(cfg.client_group.as_str()),
                    proxy_name: Arc::from(cfg.proxy_name.as_str()),
                },
            ),
        }
    }
    let egress_map = egress::ServerEgressMap::from_config(egress, http_params);
    RoutingSnapshot {
        ingress_rules: Arc::new(ingress_rules),
        tunnel_management: Arc::new(tm.clone()),
        egress_map: Arc::new(egress_map),
    }
}
pub use listener_mgr::sync_listeners;
