use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, debug, warn};
use quinn::Connection;
use crate::types::ServerState;
use crate::control::{handle_control_stream, find_client_id_by_addr, cleanup_client_registration};
use crate::data_stream::handle_data_stream;

const MAX_CONCURRENT_DATA_STREAMS: usize = 10000;

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
    
    let conn_for_data_streams = conn.clone();

    let (control_send, control_recv) = match conn.accept_bi().await {
        Ok(streams) => streams,
        Err(e) => {
            warn!("Failed to accept control stream: {}", e);
            return Ok(());
        }
    };
    
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
    
    let mut client_id_cache: Option<String> = None;
    let mut error_count = 0;
    const MAX_ERRORS: usize = 10;
    
    loop {
        match conn_for_data_streams.accept_bi().await {
            Ok((send, recv)) => {
                error_count = 0;
                
                if client_id_cache.is_none() {
                    client_id_cache = find_client_id_by_addr(&state, &remote_addr);
                }
                
                if let Some(ref client_id) = client_id_cache {
                    debug!("Accepted bidirectional data stream from client {}", client_id);
                } else {
                    debug!("Accepted bidirectional data stream from {} (client not yet registered)", remote_addr);
                }
                
                let permit = match state.data_stream_semaphore.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => {
                        warn!("Max concurrent data streams reached ({}), rejecting new stream", MAX_CONCURRENT_DATA_STREAMS);
                        continue;
                    }
                };
                
                let state = state.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(e) = handle_data_stream(send, recv, state).await {
                        error!("Data stream error: {}", e);
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                if client_id_cache.is_none() {
                    client_id_cache = find_client_id_by_addr(&state, &remote_addr);
                }
                if let Some(client_id) = client_id_cache {
                    info!("Connection closed gracefully for client {}: ApplicationClosed", client_id);
                    cleanup_client_registration(&state, &client_id);
                } else {
                    info!("Connection closed gracefully for {}", remote_addr);
                }
                break;
            }
            Err(quinn::ConnectionError::ConnectionClosed(e)) => {
                if client_id_cache.is_none() {
                    client_id_cache = find_client_id_by_addr(&state, &remote_addr);
                }
                if let Some(client_id) = client_id_cache {
                    warn!("Connection closed unexpectedly for client {}: {}", client_id, e);
                    cleanup_client_registration(&state, &client_id);
                } else {
                    warn!("Connection closed unexpectedly for {}: {}", remote_addr, e);
                }
                break;
            }
            Err(quinn::ConnectionError::TimedOut) => {
                error_count += 1;
                if error_count >= MAX_ERRORS {
                    if client_id_cache.is_none() {
                        client_id_cache = find_client_id_by_addr(&state, &remote_addr);
                    }
                    if let Some(ref client_id) = client_id_cache {
                        error!("Max timeout errors ({}) reached for client {}, closing connection", MAX_ERRORS, client_id);
                        cleanup_client_registration(&state, client_id);
                    } else {
                        error!("Max timeout errors ({}) reached for {}, closing connection", MAX_ERRORS, remote_addr);
                    }
                    break;
                }
                
                if client_id_cache.is_none() {
                    client_id_cache = find_client_id_by_addr(&state, &remote_addr);
                }
                if let Some(ref client_id) = client_id_cache {
                    warn!("Connection timeout for client {} ({}/{})", client_id, error_count, MAX_ERRORS);
                    if conn_for_data_streams.close_reason().is_some() {
                        info!("Connection actually closed for client {}", client_id);
                        cleanup_client_registration(&state, client_id);
                        break;
                    }
                } else {
                    warn!("Connection timeout for {} ({}/{})", remote_addr, error_count, MAX_ERRORS);
                    if conn_for_data_streams.close_reason().is_some() {
                        break;
                    }
                }
                continue;
            }
            Err(e) => {
                error_count += 1;
                if error_count >= MAX_ERRORS {
                    if client_id_cache.is_none() {
                        client_id_cache = find_client_id_by_addr(&state, &remote_addr);
                    }
                    if let Some(ref client_id) = client_id_cache {
                        error!("Max errors ({}) reached for client {}: {}, closing connection", MAX_ERRORS, client_id, e);
                        cleanup_client_registration(&state, client_id);
                    } else {
                        error!("Max errors ({}) reached for {}: {}, closing connection", MAX_ERRORS, remote_addr, e);
                    }
                    break;
                }
                
                if client_id_cache.is_none() {
                    client_id_cache = find_client_id_by_addr(&state, &remote_addr);
                }
                if let Some(ref client_id) = client_id_cache {
                    warn!("Error accepting data stream for client {}: {} ({}/{}, will retry)", client_id, e, error_count, MAX_ERRORS);
                    if conn_for_data_streams.close_reason().is_some() {
                        info!("Connection closed for client {}: {}", client_id, e);
                        cleanup_client_registration(&state, client_id);
                        break;
                    }
                } else {
                    warn!("Error accepting data stream for {}: {} ({}/{}, will retry)", remote_addr, e, error_count, MAX_ERRORS);
                    if conn_for_data_streams.close_reason().is_some() {
                        break;
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    Ok(())
}

