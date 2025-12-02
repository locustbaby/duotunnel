use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, error, warn};
use tunnel_lib::quic_transport::QuicClient;
use rustls;
use tokio_rustls;
use webpki_roots;

mod config;
mod http_handler;
mod http_forwarder;
mod types;
mod pool;
mod routing;
mod reverse;
mod control;
mod forward;
mod warmup;

use types::ClientState;
use control::handle_control_stream;
use reverse::handle_reverse_streams;
use forward::{start_http_listener, start_grpc_listener, start_wss_listener};

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
    
    // Initialize logging with configured log level
    let log_level = config.log_level.to_lowercase();
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(&log_level)
        });
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("Starting tunnel client...");
    info!("Config file: {}", args.config);
    info!("Log level: {}", config.log_level);
    let server_addr_str = config.server_addr();
    info!("Server address: {}", server_addr_str);

    // Create TLS connector (reused for all HTTPS connections)
    let mut root_certs = rustls::RootCertStore::empty();
    root_certs.add_trust_anchors(
        webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        })
    );
    
    let tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certs)
        .with_no_client_auth();
    
    let tls_connector = Arc::new(tokio_rustls::TlsConnector::from(Arc::new(tls_config)));
    
    // Create Hyper HTTP client (with built-in connection pooling)
    let http_connector = hyper::client::HttpConnector::new();
    let http_client = Arc::new(
        hyper::Client::builder()
            .build(http_connector)
    );
    
    // Create Hyper HTTPS client (with built-in connection pooling)
    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();
    let https_client = Arc::new(
        hyper::Client::builder()
            .build(https_connector)
    );
    
    // Initialize state
    let state = Arc::new(ClientState {
        rules: Arc::new(dashmap::DashMap::new()),
        upstreams: Arc::new(dashmap::DashMap::new()),
        config_version: Arc::new(tokio::sync::RwLock::new("0".to_string())),
        config_hash: Arc::new(tokio::sync::RwLock::new(String::new())),
        tls_connector,
        connection_pool: Arc::new(dashmap::DashMap::new()),
        quic_connection: Arc::new(tokio::sync::RwLock::new(None)),
        sessions: Arc::new(dashmap::DashMap::new()),
        http_client,
        https_client,
    });

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Start Listeners (HTTP, gRPC, WSS)
    if let Some(http_port) = config.http_entry_port {
        info!("Starting HTTP listener on port {}", http_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_http_listener(http_port, state_clone).await {
                error!("HTTP listener error: {}", e);
            }
        });
    }

    if let Some(grpc_port) = config.grpc_entry_port {
        info!("Starting gRPC listener on port {}", grpc_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_grpc_listener(grpc_port, state_clone).await {
                error!("gRPC listener error: {}", e);
            }
        });
    }

    if let Some(wss_port) = config.wss_entry_port {
        info!("Starting WSS listener on port {}", wss_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_wss_listener(wss_port, state_clone).await {
                error!("WSS listener error: {}", e);
            }
        });
    }
    
    // CONNECTION LOOP
    let client_id = config.client_id();
    let client_group_id = config.client_group_id.clone();
    let instance_id = uuid::Uuid::new_v4().to_string();
    info!("Client Instance ID: {}", instance_id);
    
    let state_clone = state.clone();
    let server_addr_str_clone = server_addr_str.clone(); // Renamed to avoid shadowing
    let mut shutdown_rx_clone = shutdown_rx.resubscribe();
    
    tokio::spawn(async move {
        let mut backoff = std::time::Duration::from_secs(1);
        
        loop {
            // Check shutdown
            if shutdown_rx_clone.try_recv().is_ok() {
                break;
            }
            
            info!("Connecting to server at {}...", server_addr_str_clone);
            let client = match QuicClient::new() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create QUIC client: {}", e);
                    tokio::time::sleep(backoff).await;
                    continue;
                }
            };
            
            let server_addr: SocketAddr = match server_addr_str_clone.parse() {
                Ok(a) => a,
                Err(e) => {
                    error!("Invalid server address: {}", e);
                    break; 
                }
            };
            
            match client.connect(server_addr, "localhost").await {
                Ok(conn) => {
                    info!("Connected to QUIC server");
                    let conn = Arc::new(conn);
                    
                    // Update shared state
                    {
                        let mut lock = state_clone.quic_connection.write().await;
                        *lock = Some(conn.clone());
                    }
                    
                    backoff = std::time::Duration::from_secs(1);
                    
                    // Spawn Control Stream
                    let conn_clone = conn.clone();
                    let state_clone2 = state_clone.clone();
                    let client_id = client_id.clone();
                    let client_group_id = client_group_id.clone();
                    let instance_id = instance_id.clone();
                    let shutdown_rx_control = shutdown_rx_clone.resubscribe();
                    
                    let control_handle = tokio::spawn(async move {
                        if let Err(e) = handle_control_stream(conn_clone, client_id, client_group_id, instance_id, state_clone2, shutdown_rx_control).await {
                            error!("Control stream error: {}", e);
                        }
                    });
                    
                    // Spawn Reverse Stream
                    let conn_clone = conn.clone();
                    let state_clone2 = state_clone.clone();
                    let reverse_handle = tokio::spawn(async move {
                        if let Err(e) = handle_reverse_streams(conn_clone, state_clone2).await {
                            error!("Reverse stream handler error: {}", e);
                        }
                    });
                    
                    // Wait for connection to close or critical error
                    let reason = conn.closed().await;
                    warn!("Connection closed: {:?}", reason);
                    
                    // Cleanup
                    {
                        let mut lock = state_clone.quic_connection.write().await;
                        *lock = None;
                    }
                    
                    // Abort handlers if they are still running (they should have exited due to connection close)
                    control_handle.abort();
                    reverse_handle.abort();
                }
                Err(e) => {
                    error!("Failed to connect to server: {}", e);
                }
            }
            
            info!("Reconnecting in {:?}...", backoff);
            tokio::time::sleep(backoff).await;
            backoff = std::cmp::min(backoff * 2, std::time::Duration::from_secs(30));
        }
    });

    // Start Listeners (HTTP, gRPC, WSS)
    if let Some(http_port) = config.http_entry_port {
        info!("Starting HTTP listener on port {}", http_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_http_listener(http_port, state_clone).await {
                error!("HTTP listener error: {}", e);
            }
        });
    }

    if let Some(grpc_port) = config.grpc_entry_port {
        info!("Starting gRPC listener on port {}", grpc_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_grpc_listener(grpc_port, state_clone).await {
                error!("gRPC listener error: {}", e);
            }
        });
    }

    if let Some(wss_port) = config.wss_entry_port {
        info!("Starting WSS listener on port {}", wss_port);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_wss_listener(wss_port, state_clone).await {
                error!("WSS listener error: {}", e);
            }
        });
    }


    // Keep main alive
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    
    // Send shutdown signal
    let _ = shutdown_tx.send(());
    
    // Wait a bit for graceful shutdown messages to be sent
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    Ok(())
}
