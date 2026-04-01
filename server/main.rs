#[cfg(all(not(target_os = "macos"), not(target_os = "windows"), not(target_env = "msvc")))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use anyhow::Result;
use arc_swap::ArcSwap;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info};

mod config;
mod egress;
mod handlers;
mod hot_reload;
mod metrics;
mod registry;
mod tunnel_handler;

use config::{ConfigSource, DbSource, FileSource, MergedSource, ServerConfigFile, ServerEgressUpstream, TunnelManagement};
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

pub struct RoutingSnapshot {
    pub vhost_router: Arc<VhostRouter<String>>,
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
    /// Broadcast channel for immediate token revocation.
    /// Senders emit the `client_name` whose token was revoked;
    /// tunnel loops subscribe and close the connection if the name matches.
    pub revocation_tx: tokio::sync::broadcast::Sender<String>,
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
    // DB is primary (dynamic); file is fallback (MergedSource falls back when DB is empty)
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
        Some(Commands::Token { action }) => handle_token_command(&cli.config, action).await,
        Some(Commands::Run) | None => run_server(&cli.config).await,
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
                "{:<20} {:<10} {:<8} {:<10} {:<20} {}",
                "NAME", "CLIENT", "TOKEN_ID", "STATUS", "CREATED", "REVOKED"
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

async fn run_server(config_path: &str) -> Result<()> {
    let config = ServerConfigFile::load(config_path)?;

    let log_level = config.server.log_level.as_deref().unwrap_or("info");
    tunnel_lib::infra::observability::init_tracing(log_level);

    info!("Starting DuoTunnel Server");
    info!(tunnel_port = %config.server.tunnel_port, "Configuration loaded");

    tunnel_lib::init_cert_cache(&config.server.pki);

    let (auth_store, rule_store) = build_stores(&config.server.database_url).await?;
    info!(url = %config.server.database_url, "auth and rule stores initialized (shared pool)");

    // On first boot, seed the DB from the YAML config so DB has an initial copy.
    // Only seed the DB from YAML on first boot (when DB is empty) to avoid
    // overwriting live rules that may have been updated via the API after boot.
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

    let config_source = build_config_source(config_path, rule_store.clone());

    let http_params = HttpClientParams::from(&config.server.http_pool);
    let (tm, egress) = config_source.load().await?;
    let initial_snapshot = build_routing_snapshot(&tm, &egress, &http_params);

    // Capacity 64: enough for burst revocations. Receivers that lag get
    // RecvError::Lagged, which the tunnel loop treats as a missed revocation
    // (safe: it will re-validate against the DB).
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
    });

    info!(
        max_quic_connections = %config.server.max_connections,
        max_tcp_connections = %config.server.max_tcp_connections,
        "Connection limits configured"
    );

    hot_reload::spawn_config_watcher(config_path.to_string(), state.clone());

    let quic_state = state.clone();
    let quic_handle =
        tokio::spawn(async move { handlers::quic::run_quic_server(quic_state).await });

    if let Some(entry_port) = config.server.entry_port {
        let http_state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handlers::http::run_http_listener(http_state, entry_port).await {
                error!(port = %entry_port, error = %e, "Entry listener failed");
            }
        });
    }

    {
        let routing = state.routing.load();
        for rule in &routing.tunnel_management.server_ingress_routing.rules.tcp {
            let tcp_state = state.clone();
            let port = rule.match_port;
            let proxy_name = rule.action_proxy_name.clone();
            let group_id = rule.action_client_group.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    handlers::tcp::run_tcp_listener(tcp_state, port, proxy_name, group_id).await
                {
                    error!(port = %port, error = %e, "TCP listener failed");
                }
            });
        }
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
    let vhost_router = build_vhost_router(tm);
    let egress_map = egress::ServerEgressMap::from_config(egress, http_params);

    RoutingSnapshot {
        vhost_router: Arc::new(vhost_router),
        tunnel_management: Arc::new(tm.clone()),
        egress_map: Arc::new(egress_map),
    }
}

fn build_vhost_router(tm: &TunnelManagement) -> VhostRouter<String> {
    let router = VhostRouter::new();

    for rule in &tm.server_ingress_routing.rules.vhost {
        router.add_route(&rule.match_host, rule.action_client_group.clone());
    }

    info!(routes = router.len(), "vhost router initialized");
    router
}
