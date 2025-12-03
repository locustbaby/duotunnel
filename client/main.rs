use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use tokio::sync::broadcast;

mod config;
mod http_handler;
mod http_forwarder;
mod wss_forwarder;
mod grpc_forwarder;
mod types;
mod forwarder;
mod client_listener;
mod egress_pool;
mod register;
mod config_manager;
mod quic_tunnel_manager;
mod reverse_handler;
mod listener_manager;

use types::{ClientState, ClientIdentity};
use egress_pool::EgressPool;
use quic_tunnel_manager::QuicTunnelManager;
use listener_manager::ListenerManager;
use config_manager::ConfigManager;
use reverse_handler::ReverseRequestHandler;
use forwarder::Forwarder;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "../config/client.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::ClientConfig::load(&args.config)?;

    initialize_logging(&config.log_level);

    info!("=== Tunnel Client Starting ===");
    info!("Config file: {}", args.config);
    info!("Log level: {}", config.log_level);
    info!("Server: {}", config.server_addr());
    info!("Client group: {}", config.client_group_id);

    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(16);

    let identity = ClientIdentity {
        client_id: config.client_id(),
        group_id: config.client_group_id.clone(),
        instance_id: uuid::Uuid::new_v4().to_string(),
    };

    info!("Client Identity: {:?}", identity);

    let server_addr: SocketAddr = config.server_addr().parse()?;

    info!("=== Step 1: Initializing EgressPool ===");
    let egress_pool = Arc::new(EgressPool::new());

    info!("=== Step 2: Initializing ClientState ===");
    let state = initialize_client_state(egress_pool.clone());

    info!("=== Step 3: Initializing Forwarder ===");
    let forwarder = Arc::new(Forwarder::new(state.clone()));

    info!("=== Step 4: Starting ListenerManager ===");
    let listener_manager = ListenerManager::new(
        config.http_entry_port,
        config.grpc_entry_port,
        config.wss_entry_port,
        state.clone(),
        forwarder.clone(),
    );
    let listener_handles = listener_manager.start_all().await?;

    info!("=== Step 5: Starting QuicTunnelManager ===");
    let quic_manager = QuicTunnelManager::new(
        server_addr,
        "localhost".to_string(),
        identity.clone(),
        state.clone(),
        egress_pool.clone(),
        forwarder.clone(),
    );

    let shutdown_rx_quic = shutdown_rx.resubscribe();
    let quic_handle = tokio::spawn(async move {
        if let Err(e) = quic_manager.run(shutdown_rx_quic).await {
            tracing::error!("QUIC tunnel manager error: {}", e);
        }
    });

    info!("=== Step 6: Starting ConfigManager ===");
    let config_manager = ConfigManager::new(
        identity.clone(),
        state.clone(),
        egress_pool.clone(),
    );
    let shutdown_rx_config = shutdown_rx.resubscribe();
    let config_handle = tokio::spawn(async move {
        if let Err(e) = config_manager.run(shutdown_rx_config).await {
            tracing::error!("Config manager error: {}", e);
        }
    });

    info!("=== Step 7: Starting ReverseRequestHandler ===");
    let reverse_handler = ReverseRequestHandler::new(state.clone(), forwarder.clone());
    let shutdown_rx_reverse = shutdown_rx.resubscribe();
    let reverse_handle = tokio::spawn(async move {
        if let Err(e) = reverse_handler.run(shutdown_rx_reverse).await {
            tracing::error!("Reverse request handler error: {}", e);
        }
    });

    info!("=== Tunnel Client Started Successfully ===");

    tokio::signal::ctrl_c().await?;
    info!("=== Received shutdown signal, initiating graceful shutdown ===");

    let _ = shutdown_tx.send(());

    for handle in listener_handles {
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            handle
        ).await;
    }

    config_handle.abort();
    reverse_handle.abort();

    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        quic_handle
    ).await;

    info!("=== Tunnel client shutdown complete ===");
    Ok(())
}

fn initialize_logging(log_level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(log_level.to_lowercase())
        });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
}

fn initialize_client_state(egress_pool: Arc<EgressPool>) -> Arc<ClientState> {
    Arc::new(ClientState {
        rules: Arc::new(dashmap::DashMap::new()),
        upstreams: Arc::new(dashmap::DashMap::new()),
        config_version: Arc::new(tokio::sync::RwLock::new("0".to_string())),
        config_hash: Arc::new(tokio::sync::RwLock::new(String::new())),
        quic_connection: Arc::new(tokio::sync::RwLock::new(None)),
        sessions: Arc::new(dashmap::DashMap::new()),
        egress_pool,
    })
}
