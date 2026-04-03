#[cfg(
    all(not(target_os = "macos"), not(target_os = "windows"), not(target_env = "msvc"))
)]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use anyhow::Result;
use arc_swap::ArcSwap;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
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
use tunnel_lib::{HttpClientParams, VhostRouter};
use tunnel_store::{AuthStore, RuleStore};
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
    Token { #[command(subcommand)] action: TokenAction },
}
#[derive(Subcommand, Debug)]
enum TokenAction {
    Create { #[arg(long)] name: String },
    List,
    Revoke { #[arg(long)] name: String },
    Rotate { #[arg(long)] name: String },
}
pub type HttpRouterMap = Arc<HashMap<u16, Arc<VhostRouter<(Arc<str>, Arc<str>)>>>>;
pub struct RoutingSnapshot {
    pub http_routers: HttpRouterMap,
    pub tunnel_management: Arc<TunnelManagement>,
    pub egress_map: Arc<egress::ServerEgressMap>,
}
/// Thread-local free-list for peek buffers.
/// Each tokio worker thread keeps its own pool — no lock, no cross-thread contention.
/// Buffers are returned with their original capacity; `set_len` avoids zeroing.
pub struct PeekBufPool {
    buf_size: usize,
}
impl PeekBufPool {
    /// Max buffers kept per thread. With 8–16 worker threads this caps total idle RAM at
    /// 16 threads × 32 bufs × 16 KiB = ~8 MiB, same order as the old global cap (256 × 16 KiB).
    const MAX_IDLE_PER_THREAD: usize = 32;

    pub fn new(buf_size: usize) -> Self {
        Self { buf_size }
    }

    pub fn take(&self) -> Vec<u8> {
        PEEK_BUF_POOL.with(|cell| {
            let buf = cell.borrow_mut().pop();
            match buf {
                Some(mut b) => {
                    // SAFETY: capacity == buf_size (enforced in put); bytes are
                    // overwritten by peek() before any read occurs.
                    unsafe { b.set_len(self.buf_size); }
                    b
                }
                None => vec![0u8; self.buf_size],
            }
        })
    }

    pub fn put(&self, mut buf: Vec<u8>) {
        if buf.capacity() < self.buf_size {
            return; // undersized — drop
        }
        // Truncate to buf_size so the slice passed to peek() is exactly buf_size bytes.
        unsafe { buf.set_len(self.buf_size); }
        PEEK_BUF_POOL.with(|cell| {
            let mut pool = cell.borrow_mut();
            if pool.len() < Self::MAX_IDLE_PER_THREAD {
                pool.push(buf);
            }
        });
    }
}

thread_local! {
    static PEEK_BUF_POOL: std::cell::RefCell<Vec<Vec<u8>>> = const { std::cell::RefCell::new(Vec::new()) };
}

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
async fn build_stores(
    database_url: &str,
) -> Result<(Arc<dyn AuthStore>, Arc<dyn RuleStore>)> {
    let pool = tunnel_store::open_sqlite_pool(database_url).await?;
    let auth_store = tunnel_store::sqlite::SqliteAuthStore::from_pool(pool.clone());
    auth_store.migrate().await?;
    let rule_store = tunnel_store::sqlite_rules::SqliteRuleStore::new(pool);
    rule_store.migrate().await?;
    Ok((Arc::new(auth_store), Arc::new(rule_store)))
}
fn build_config_source(
    config_path: &str,
    rule_store: Arc<dyn RuleStore>,
) -> Arc<dyn ConfigSource> {
    Arc::new(MergedSource::new(
        Box::new(DbSource::new(rule_store)),
        Box::new(FileSource::new(config_path)),
    ))
}
#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Token { action }) => {
            handle_token_command(&cli.config, action).await
        }
        Some(Commands::Run) | None => run_server(&cli.config, cli.ctld_addr.as_deref()).await,
    }
}
async fn handle_token_command(config_path: &str, action: TokenAction) -> Result<()> {
    let config = ServerConfigFile::load(config_path)?;
    let log_level = config.server.log_level.as_deref().unwrap_or("info");
    tunnel_lib::infra::observability::init_tracing(log_level);
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
async fn run_server(config_path: &str, ctld_addr: Option<&str>) -> Result<()> {
    let config = ServerConfigFile::load(config_path)?;
    // In standalone mode the routing sections are used at runtime, so validate them.
    // In ctld-managed mode the routing sections are ignored by the server, so skip.
    if ctld_addr.is_none() {
        config::validate_server_config(&config)?;
    }
    let log_level = config.server.log_level.as_deref().unwrap_or("info");
    tunnel_lib::infra::observability::init_tracing(log_level);
    info!("Starting DuoTunnel Server");
    info!(tunnel_port = %config.server.tunnel_port, "Configuration loaded");
    tunnel_lib::init_cert_cache(&config.server.pki);
    #[allow(clippy::type_complexity)]
    let (auth_store, rule_store, config_source, local_token_cache): (
        Arc<dyn AuthStore>,
        Arc<dyn RuleStore>,
        Arc<dyn ConfigSource>,
        Option<Arc<local_auth::LocalTokenCache>>,
    ) = if ctld_addr.is_some() {
        // ctld-managed mode: no local SQLite; all config comes from ctld watch stream.
        info!("running in ctld-managed mode; no local SQLite stores");
        let token_cache = Arc::new(local_auth::LocalTokenCache::new());
        let auth: Arc<dyn AuthStore> = token_cache.clone();
        (auth, Arc::new(NullRuleStore), Arc::new(NullConfigSource), Some(token_cache))
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
    let state = Arc::new(ServerState {
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
    });
    info!(
        max_quic_connections = %config.server.max_connections,
        max_tcp_connections = %config.server.max_tcp_connections,
        "Connection limits configured"
    );

    if let Some(addr_str) = ctld_addr {
        let addr: std::net::SocketAddr = addr_str.parse()?;
        info!(addr = %addr, "starting ctld watch client");
        control_client::spawn_control_client(addr, state.clone());
    } else {
        hot_reload::spawn_config_watcher(config_path.to_string(), state.clone());
    }

    let quic_state = state.clone();
    let quic_handle =
        tokio::spawn(async move { handlers::quic::run_quic_server(quic_state).await });
    {
        let listeners: Vec<_> = tm.server_ingress_routing.listeners.to_vec();
        sync_listeners(&state, &listeners);
    }
    if let Some(metrics_port) = config.server.metrics_port {
        tokio::spawn(async move {
            if let Err(e) = handlers::metrics::run_metrics_server(metrics_port).await {
                error!(port = %metrics_port, error = %e, "Metrics server failed");
            }
        });
    }
    quic_handle.await??;
    Ok(())
}
pub fn build_routing_snapshot(
    tm: &TunnelManagement,
    egress: &ServerEgressUpstream,
    http_params: &HttpClientParams,
) -> RoutingSnapshot {
    #[allow(clippy::type_complexity)]
    let mut http_routers: HashMap<u16, Arc<VhostRouter<(Arc<str>, Arc<str>)>>> = HashMap::new();
    for listener in &tm.server_ingress_routing.listeners {
        if let IngressMode::Http(cfg) = &listener.mode {
            let router: VhostRouter<(Arc<str>, Arc<str>)> = VhostRouter::new();
            for rule in &cfg.vhost {
                router.add_route(
                    &rule.match_host,
                    (Arc::from(rule.client_group.as_str()), Arc::from(rule.proxy_name.as_str())),
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
