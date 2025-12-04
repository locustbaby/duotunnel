use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, debug, warn};
use quinn::Connection;
use crate::types::ServerState;
use crate::control::{handle_control_stream, find_client_id_by_addr, cleanup_client_registration};
use crate::data_stream::handle_data_stream;

/// Accept loop for QUIC server
pub async fn accept_loop(server: tunnel_lib::quic_transport::QuicServer, state: Arc<ServerState>) {
    loop {
        if let Some(conn) = server.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(conn, state).await {
                    error!("Connection error: {}", e);
                }
            });
        }
    }
}

/// Handle a single QUIC connection
pub async fn handle_connection(conn: Connection, state: Arc<ServerState>) -> Result<()> {
    let remote_addr = conn.remote_address();
    info!("New connection from {}", remote_addr);
    
    // Store connection for data stream handling (separate from control stream)
    let conn_for_data_streams = conn.clone();

    // 1. Handle Control Stream: expect bidirectional stream for persistent control
    let (control_send, control_recv) = match conn.accept_bi().await {
        Ok(streams) => streams,
        Err(e) => {
            warn!("Failed to accept control stream: {}", e);
            return Ok(());
        }
    };
    
    // 2. Spawn control stream handler IMMEDIATELY to handle the initial ConfigSyncRequest
    // CRITICAL: Move control_send and control_recv into the spawned task to keep them alive
    let state_for_control = state.clone();
    let conn_for_control = conn.clone();
    
    tokio::spawn(async move {
        if let Err(e) = handle_control_stream(
            control_send,
            control_recv,
            conn_for_control.clone(),
            remote_addr,
            state_for_control,
        ).await {
            error!("Control stream handler error: {}", e);
        }
    });
    
    // 3. Handle incoming data streams (Forward Tunnels from Client)
    // This loop keeps the connection alive and handles data streams from client
    loop {
        match conn_for_data_streams.accept_bi().await {
            Ok((send, recv)) => {
                // Find client_id by connection (reverse lookup)
                let client_id_opt = state.clients.iter()
                    .find(|entry| entry.value().remote_address() == remote_addr)
                    .map(|entry| entry.key().clone());
                
                if let Some(ref client_id) = client_id_opt {
                    debug!("Accepted bidirectional data stream from client {}", client_id);
                } else {
                    debug!("Accepted bidirectional data stream from {} (client not yet registered)", remote_addr);
                }
                
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_data_stream(send, recv, state).await {
                        error!("Data stream error: {}", e);
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                // Find client_id and cleanup
                if let Some(client_id) = find_client_id_by_addr(&state, &remote_addr) {
                    info!("Connection closed gracefully for client {}: ApplicationClosed", client_id);
                    cleanup_client_registration(&state, &client_id);
                } else {
                    info!("Connection closed gracefully for {}", remote_addr);
                }
                break;
            }
            Err(quinn::ConnectionError::ConnectionClosed(e)) => {
                // Find client_id and cleanup
                if let Some(client_id) = find_client_id_by_addr(&state, &remote_addr) {
                    warn!("Connection closed unexpectedly for client {}: {}", client_id, e);
                    cleanup_client_registration(&state, &client_id);
                } else {
                    warn!("Connection closed unexpectedly for {}: {}", remote_addr, e);
                }
                break;
            }
            Err(quinn::ConnectionError::TimedOut) => {
                // Connection timeout - but connection might still be alive
                // Don't break, keep waiting for streams
                if let Some(client_id) = find_client_id_by_addr(&state, &remote_addr) {
                    warn!("Connection timeout for client {}, but keeping connection alive", client_id);
                    // Check if connection is still valid
                    if conn_for_data_streams.close_reason().is_some() {
                        info!("Connection actually closed for client {}", client_id);
                        cleanup_client_registration(&state, &client_id);
                        break;
                    }
                } else {
                    warn!("Connection timeout for {}, but keeping connection alive", remote_addr);
                    if conn_for_data_streams.close_reason().is_some() {
                        break;
                    }
                }
                // Otherwise, continue waiting
                continue;
            }
            Err(e) => {
                // Other errors - log but don't break immediately
                if let Some(client_id) = find_client_id_by_addr(&state, &remote_addr) {
                    warn!("Error accepting data stream for client {}: {} (will retry)", client_id, e);
                    // Check if connection is actually closed
                    if conn_for_data_streams.close_reason().is_some() {
                        info!("Connection closed for client {}: {}", client_id, e);
                        cleanup_client_registration(&state, &client_id);
                        break;
                    }
                } else {
                    warn!("Error accepting data stream for {}: {} (will retry)", remote_addr, e);
                    if conn_for_data_streams.close_reason().is_some() {
                        break;
                    }
                }
                // Wait a bit before retrying
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    Ok(())
}

