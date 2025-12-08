use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use tokio::sync::broadcast;

mod control;
mod config;
mod forwarder;
mod ingress_handlers;
mod quic_tunnel_manager;
mod register;
mod reverse_handler;
mod types;
mod connection_state;
mod stream_state;
mod rule_matcher;
mod session_manager;
mod forward_strategies;
mod http_request_builder;
mod message_handlers;

use control::ConfigManager;
use types::{ClientState, ClientIdentity};
use quic_tunnel_manager::QuicTunnelManager;
use reverse_handler::ReverseRequestHandler;
use forwarder::Forwarder;
use ingress_handlers::{HttpIngressHandler, GrpcIngressHandler, WssIngressHandler};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "../config/client.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    
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
    let egress_pool = Arc::new(tunnel_lib::egress_pool::EgressPool::new());

    info!("=== Step 2: Initializing ClientState ===");
    let state = initialize_client_state(egress_pool.clone());

    info!("=== Step 3: Initializing Forwarder ===");
    let forwarder = Arc::new(Forwarder::new(state.clone()));

    info!("=== Step 4: Starting ListenerManager ===");
    
    let http_handler = config.http_entry_port.map(|_| {
        Arc::new(HttpIngressHandler::new(state.clone()))
    });
    
    let grpc_handler = config.grpc_entry_port.map(|_| {
        Arc::new(GrpcIngressHandler::new(state.clone()))
    });
    
    let wss_handler = config.wss_entry_port.map(|_| {
        Arc::new(WssIngressHandler::new(state.clone()))
    });
    
    let listener_manager = tunnel_lib::listener::ListenerManager::new(
        config.http_entry_port,
        config.grpc_entry_port,
        config.wss_entry_port,
    );
    
    let listener_handles = listener_manager.start_all(
        http_handler,
        grpc_handler,
        wss_handler,
    ).await?;

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

    // Transition connection state to shutting down
    state.connection_state.transition_to_shutting_down();
    info!("Step 1/4: Broadcasting shutdown signal to all components");
    let _ = shutdown_tx.send(());
    

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;


    info!("Step 2/4: Shutting down ingress listeners");
    let listener_shutdown_timeout = std::time::Duration::from_secs(3);
    for (idx, handle) in listener_handles.into_iter().enumerate() {
        match tokio::time::timeout(listener_shutdown_timeout, handle).await {
            Ok(Ok(())) => {
                info!("  ✓ Listener {} shutdown successfully", idx + 1);
            }
            Ok(Err(e)) => {
                tracing::warn!("  ⚠ Listener {} shutdown with error: {}", idx + 1, e);
            }
            Err(_) => {
                tracing::warn!("  ⚠ Listener {} shutdown timeout after {:?}", idx + 1, listener_shutdown_timeout);
            }
        }
    }


    info!("Step 3/4: Waiting for background tasks to complete");
    let task_shutdown_timeout = std::time::Duration::from_secs(10);
    

    let shutdown_result = tokio::time::timeout(
        task_shutdown_timeout,
        async {
            let (config_result, reverse_result, quic_result) = tokio::join!(
                config_handle,
                reverse_handle,
                quic_handle
            );
            
            match config_result {
                Ok(_) => info!("  ✓ Control manager shutdown successfully"),
                Err(e) => tracing::error!("  ✗ Control manager task panicked: {}", e),
            }
            
            match reverse_result {
                Ok(_) => info!("  ✓ Reverse handler shutdown successfully"),
                Err(e) => tracing::error!("  ✗ Reverse handler task panicked: {}", e),
            }
            
            match quic_result {
                Ok(_) => info!("  ✓ QUIC tunnel manager shutdown successfully"),
                Err(e) => tracing::error!("  ✗ QUIC tunnel manager task panicked: {}", e),
            }
        }
    ).await;

    match shutdown_result {
        Ok(_) => {
            info!("Step 4/4: All tasks completed gracefully");
        }
        Err(_) => {
            tracing::warn!("Step 4/4: Graceful shutdown timeout after {:?}, some tasks may still be running", task_shutdown_timeout);
        }
    }


    info!("Performing final cleanup");
    

    if let Some(conn) = state.quic_connection.read().await.as_ref() {
        conn.close(0u32.into(), b"Client shutdown");
        info!("  ✓ QUIC connection closed");
    }
    

    let session_count = state.sessions.len();
    state.sessions.clear();
    if session_count > 0 {
        info!("  ✓ Cleared {} active sessions", session_count);
    }

    // Transition to shutdown state
    state.connection_state.transition_to_shutdown();
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

fn initialize_client_state(egress_pool: Arc<tunnel_lib::egress_pool::EgressPool>) -> Arc<ClientState> {
    let session_manager = Arc::new(session_manager::SessionManager::new(None, None));
    let cleanup_handle = session_manager.clone().start_cleanup_task();
    
    // Keep cleanup task running
    tokio::spawn(async move {
        cleanup_handle.await.ok();
    });
    
    Arc::new(ClientState {
        rules: Arc::new(dashmap::DashMap::new()),
        upstreams: Arc::new(dashmap::DashMap::new()),
        config_version: Arc::new(tokio::sync::RwLock::new("0".to_string())),
        config_hash: Arc::new(tokio::sync::RwLock::new(String::new())),
        quic_connection: Arc::new(tokio::sync::RwLock::new(None)),
        sessions: Arc::new(dashmap::DashMap::new()),  // Legacy
        egress_pool,
        connection_state: Arc::new(connection_state::ConnectionStateMachine::new()),
        rule_matcher: Arc::new(tokio::sync::RwLock::new(rule_matcher::RuleMatcher::new())),
        session_manager,
    })
}
