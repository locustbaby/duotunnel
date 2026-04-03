#[cfg(
    all(not(target_os = "macos"), not(target_os = "windows"), not(target_env = "msvc"))
)]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use anyhow::Result;
use arc_swap::ArcSwap;
use clap::{Parser, Subcommand};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
mod config;
mod control_client;
mod egress;
mod handlers;
mod hot_reload;
mod local_auth;
mod metrics;
mod registry;
mod tunnel_handler;
use config::{
    ConfigSource, DbSource, FileSource, IngressMode, MergedSource, ServerConfigFile,
    ServerEgressUpstream, TunnelManagement,
};
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
struct ListenerEntry {
    id: u64,
    kind: ListenerKind,
    cancel: CancellationToken,
}
enum ListenerKind {
    Http,
    Tcp { group_id: String, proxy_name: String },
}
pub type HttpRouterMap = Arc<HashMap<u16, Arc<VhostRouter<(String, String)>>>>;
pub struct RoutingSnapshot {
    pub http_routers: HttpRouterMap,
    pub tunnel_management: Arc<TunnelManagement>,
    pub egress_map: Arc<egress::ServerEgressMap>,
}
pub struct ServerState {
    pub config: Arc<ServerConfigFile>,
    pub registry: SharedRegistry,
    pub quic_semaphore: Arc<Semaphore>,
    pub tcp_semaphore: Arc<Semaphore>,
    pub tcp_params: tunnel_lib::TcpParams,
    pub proxy_buffer_params: tunnel_lib::ProxyBufferParams,
    pub routing: Arc<ArcSwap<RoutingSnapshot>>,
    pub auth_store: Arc<dyn AuthStore>,
    pub rule_store: Arc<dyn RuleStore>,
    pub config_source: Arc<dyn ConfigSource>,
    pub revocation_tx: tokio::sync::broadcast::Sender<String>,
    /// Present when running in ctld-managed mode; None in standalone mode.
    pub local_token_cache: Option<Arc<local_auth::LocalTokenCache>>,
    listener_next_id: AtomicU64,
    listeners: Mutex<HashMap<u16, ListenerEntry>>,
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
    let log_level = config.server.log_level.as_deref().unwrap_or("info");
    tunnel_lib::infra::observability::init_tracing(log_level);
    info!("Starting DuoTunnel Server");
    info!(tunnel_port = %config.server.tunnel_port, "Configuration loaded");
    tunnel_lib::init_cert_cache(&config.server.pki);
    let (auth_store, rule_store, local_token_cache): (
        Arc<dyn AuthStore>,
        Arc<dyn RuleStore>,
        Option<Arc<local_auth::LocalTokenCache>>,
    ) = if ctld_addr.is_some() {
        // ctld-managed mode: no local SQLite auth; token cache fed by watch stream
        info!("running in ctld-managed mode; skipping local SQLite stores");
        let token_cache = Arc::new(local_auth::LocalTokenCache::new());
        let auth: Arc<dyn AuthStore> = token_cache.clone();
        // Still need a rule_store stub for ServerState (used only in standalone hot_reload)
        let (_, rule_store) = build_stores(&config.server.database_url).await?;
        (auth, rule_store, Some(token_cache))
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
        (auth_store, rule_store, None)
    };

    let config_source = build_config_source(config_path, rule_store.clone());
    let http_params = HttpClientParams::from(&config.server.http_pool);
    let (tm, egress) = config_source.load().await?;
    let initial_snapshot = build_routing_snapshot(&tm, &egress, &http_params);
    let (revocation_tx, _) = tokio::sync::broadcast::channel::<String>(64);
    let state = Arc::new(ServerState {
        tcp_params: tunnel_lib::TcpParams::from(&config.server.tcp),
        proxy_buffer_params: tunnel_lib::ProxyBufferParams::from(&config.server.proxy_buffers),
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
        listener_next_id: AtomicU64::new(1),
        listeners: Mutex::new(HashMap::new()),
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
    let mut http_routers: HashMap<u16, Arc<VhostRouter<(String, String)>>> = HashMap::new();
    for listener in &tm.server_ingress_routing.listeners {
        if let IngressMode::Http(cfg) = &listener.mode {
            let router: VhostRouter<(String, String)> = VhostRouter::new();
            for rule in &cfg.vhost {
                router.add_route(
                    &rule.match_host,
                    (rule.action_client_group.clone(), rule.action_proxy_name.clone()),
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
pub fn sync_listeners(
    state: &Arc<ServerState>,
    desired: &[config::IngressListener],
) {
    let desired_ports: HashSet<u16> = desired.iter().map(|l| l.port).collect();
    let desired_by_port: HashMap<u16, &config::IngressListener> =
        desired.iter().map(|l| (l.port, l)).collect();

    let mut to_cancel = Vec::new();
    let mut map = state.listeners.lock().unwrap();

    map.retain(|port, entry| {
        if !desired_ports.contains(port) {
            info!(port = %port, "listener removed (hot-reload)");
            to_cancel.push(entry.cancel.clone());
            return false;
        }
        let listener = desired_by_port[port];
        let changed = match (&entry.kind, &listener.mode) {
            (ListenerKind::Http, IngressMode::Http(_)) => false,
            (
                ListenerKind::Tcp { group_id, proxy_name },
                IngressMode::Tcp(cfg),
            ) => {
                group_id != &cfg.action_client_group || proxy_name != &cfg.action_proxy_name
            }
            _ => true,
        };
        if changed {
            info!(port = %port, "listener config changed (hot-reload), restarting");
            to_cancel.push(entry.cancel.clone());
            false
        } else {
            true
        }
    });
    drop(map);

    for cancel in to_cancel {
        cancel.cancel();
    }

    let mut map = state.listeners.lock().unwrap();
    for (port, listener) in &desired_by_port {
        if map.contains_key(port) {
            continue;
        }
        let listener_id = state.listener_next_id.fetch_add(1, Ordering::Relaxed);
        let cancel = CancellationToken::new();
        let kind = match &listener.mode {
            IngressMode::Http(_) => ListenerKind::Http,
            IngressMode::Tcp(cfg) => ListenerKind::Tcp {
                group_id: cfg.action_client_group.clone(),
                proxy_name: cfg.action_proxy_name.clone(),
            },
        };
        map.insert(*port, ListenerEntry { id: listener_id, kind, cancel: cancel.clone() });
        match &listener.mode {
            IngressMode::Http(_) => {
                let tcp_state = state.clone();
                let p = *port;
                let cancel_for_task = cancel.clone();
                tokio::spawn(async move {
                    if let Err(e) =
                        handlers::http::run_http_listener(tcp_state.clone(), p, cancel_for_task)
                            .await
                    {
                        error!(port = %p, error = %e, "HTTP listener failed");
                    }
                    let mut map = tcp_state.listeners.lock().unwrap();
                    if map.get(&p).map(|e| e.id == listener_id).unwrap_or(false) {
                        map.remove(&p);
                    }
                });
            }
            IngressMode::Tcp(cfg) => {
                let tcp_state = state.clone();
                let p = *port;
                let group_id = cfg.action_client_group.clone();
                let proxy_name = cfg.action_proxy_name.clone();
                let cancel_for_task = cancel.clone();
                tokio::spawn(async move {
                    if let Err(e) = handlers::tcp::run_tcp_listener(
                        tcp_state.clone(),
                        p,
                        proxy_name,
                        group_id,
                        cancel_for_task,
                    )
                    .await
                    {
                        error!(port = %p, error = %e, "TCP listener failed");
                    }
                    let mut map = tcp_state.listeners.lock().unwrap();
                    if map.get(&p).map(|e| e.id == listener_id).unwrap_or(false) {
                        map.remove(&p);
                    }
                });
            }
        }
    }
}
