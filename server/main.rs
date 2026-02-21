use anyhow::Result;
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

use config::ServerConfigFile;
use registry::{SharedRegistry, new_shared_registry};
use tunnel_lib::VhostRouter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config/server.yaml")]
    config: String,
}

pub struct ServerState {
    pub config: Arc<ServerConfigFile>,
    pub registry: SharedRegistry,
    pub vhost_router: Arc<VhostRouter<String>>,
    pub egress_map: Arc<egress::ServerEgressMap>,
    /// Semaphore for limiting QUIC connections
    pub quic_semaphore: Arc<Semaphore>,
    /// Semaphore for limiting TCP connections
    pub tcp_semaphore: Arc<Semaphore>,
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

    let state = Arc::new(ServerState {
        config: Arc::new(config.clone()),
        registry: new_shared_registry(),
        vhost_router: Arc::new(build_vhost_router(&config)),
        egress_map: Arc::new(egress::ServerEgressMap::from_config(&config)),
        quic_semaphore: Arc::new(Semaphore::new(config.server.max_connections)),
        tcp_semaphore: Arc::new(Semaphore::new(config.server.max_tcp_connections)),
    });

    info!(
        max_quic_connections = %config.server.max_connections,
        max_tcp_connections = %config.server.max_tcp_connections,
        "Connection limits configured"
    );

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

    // Start TCP listeners for each rule
    for rule in &config.tunnel_management.server_ingress_routing.rules.tcp {
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

fn build_vhost_router(config: &ServerConfigFile) -> VhostRouter<String> {
    let router = VhostRouter::new();

    for rule in &config.tunnel_management.server_ingress_routing.rules.vhost {
        router.add_route(&rule.match_host, rule.action_client_group.clone());
    }

    info!(routes = router.len(), "vhost router initialized");
    router
}
