use anyhow::Result;
use arc_swap::ArcSwap;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, error};

mod config;
mod registry;
mod egress;
mod tunnel_handler;
mod handlers;
mod metrics;
mod hot_reload;

use config::{ServerConfigFile, TunnelManagement, ServerEgressUpstream};
use registry::{SharedRegistry, new_shared_registry};
use tunnel_lib::{VhostRouter, HttpClientParams};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config/server.yaml")]
    config: String,
}

/// Snapshot of all hot-reloadable routing state.
///
/// Handlers call `state.routing.load()` to get a consistent view; the returned
/// `Guard` keeps the old snapshot alive for in-flight requests while a hot reload
/// atomically swaps in a new one.
pub struct RoutingSnapshot {
    pub vhost_router: Arc<VhostRouter<String>>,
    pub tunnel_management: Arc<TunnelManagement>,
    pub egress_map: Arc<egress::ServerEgressMap>,
}

pub struct ServerState {
    /// Static config (restart required to change).
    pub config: Arc<ServerConfigFile>,
    pub registry: SharedRegistry,
    /// Semaphore for limiting QUIC connections
    pub quic_semaphore: Arc<Semaphore>,
    /// Semaphore for limiting TCP connections
    pub tcp_semaphore: Arc<Semaphore>,
    /// TCP socket parameters applied to accepted/connected streams
    pub tcp_params: tunnel_lib::TcpParams,
    /// Proxy buffer sizes for protocol detection and HTTP parsing
    pub proxy_buffer_params: tunnel_lib::ProxyBufferParams,

    /// Hot-reloadable routing snapshot (vhost router + tunnel management + egress map).
    /// Use `.load()` for a lock-free read; `.store()` to atomically replace on reload.
    pub routing: Arc<ArcSwap<RoutingSnapshot>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install the ring crypto provider for rustls
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let args = Args::parse();

    let config = ServerConfigFile::load(&args.config)?;

    let log_level = config.server.log_level.as_deref().unwrap_or("info");
    tunnel_lib::infra::observability::init_tracing(log_level);

    info!("Starting DuoTunnel Server");
    info!(tunnel_port = %config.server.tunnel_port, "Configuration loaded");

    // Initialise cert cache with configured TTL before any TLS connections.
    // config.server.pki is already a PkiParams — pass it directly.
    tunnel_lib::init_cert_cache(&config.server.pki);

    let http_params = HttpClientParams::from(&config.server.http_pool);
    let initial_snapshot = build_routing_snapshot(
        &config.tunnel_management,
        &config.server_egress_upstream,
        &http_params,
    );

    let state = Arc::new(ServerState {
        tcp_params: tunnel_lib::TcpParams::from(&config.server.tcp),
        proxy_buffer_params: tunnel_lib::ProxyBufferParams::from(&config.server.proxy_buffers),
        quic_semaphore: Arc::new(Semaphore::new(config.server.max_connections)),
        tcp_semaphore: Arc::new(Semaphore::new(config.server.max_tcp_connections)),
        routing: Arc::new(ArcSwap::from_pointee(initial_snapshot)),
        registry: new_shared_registry(),
        config: Arc::new(config.clone()),
    });

    info!(
        max_quic_connections = %config.server.max_connections,
        max_tcp_connections = %config.server.max_tcp_connections,
        "Connection limits configured"
    );

    // Spawn hot-reload watcher for server_egress_upstream + tunnel_management
    hot_reload::spawn_config_watcher(args.config.clone(), state.clone());

    // Start QUIC server
    let quic_state = state.clone();
    let quic_handle = tokio::spawn(async move {
        handlers::quic::run_quic_server(quic_state).await
    });

    // Start HTTP entry listener
    if let Some(entry_port) = config.server.entry_port {
        let http_state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handlers::http::run_http_listener(http_state, entry_port).await {
                error!(port = %entry_port, error = %e, "Entry listener failed");
            }
        });
    }

    // Start TCP listeners for each rule — snapshot current rules at startup
    {
        let routing = state.routing.load();
        for rule in &routing.tunnel_management.server_ingress_routing.rules.tcp {
            let tcp_state = state.clone();
            let port = rule.match_port;
            let proxy_name = rule.action_proxy_name.clone();
            let group_id = rule.action_client_group.clone();
            tokio::spawn(async move {
                if let Err(e) = handlers::tcp::run_tcp_listener(tcp_state, port, proxy_name, group_id).await {
                    error!(port = %port, error = %e, "TCP listener failed");
                }
            });
        }
    }

    // Start Prometheus metrics server
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

/// Build a fresh `RoutingSnapshot` from the hot-reloadable config sections.
/// Called at startup and on each successful hot reload.
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

