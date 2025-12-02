use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::io;
use tracing::{info, error, debug, warn};
use quinn::{Connection, SendStream, RecvStream};
use crate::types::ClientState;
use crate::pool::{get_or_create_connection, return_connection_to_pool};
use crate::routing::{match_rule_by_type_and_host, resolve_upstream, parse_target_addr};
use crate::types::PooledConnection;
use tunnel_lib::protocol::{read_data_stream_header, write_data_stream_header};
use tunnel_lib::proto::tunnel::{Rule, Upstream};

/// Handle reverse streams initiated by Server (Server -> Client -> Upstream)
pub async fn handle_reverse_streams(
    connection: Arc<Connection>,
    state: Arc<ClientState>,
) -> Result<()> {
    info!("Reverse stream handler started, waiting for server-initiated streams...");
    
    const INITIAL_BACKOFF: std::time::Duration = std::time::Duration::from_millis(100);
    const MAX_BACKOFF: std::time::Duration = std::time::Duration::from_secs(10);
    let mut backoff = INITIAL_BACKOFF;
    
    loop {
        match connection.accept_bi().await {
            Ok((send, recv)) => {
                info!("Accepted reverse bidirectional stream from server");
                // Reset backoff on successful accept
                backoff = INITIAL_BACKOFF;
                
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_reverse_stream(send, recv, state).await {
                        error!("Reverse stream error: {}", e);
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                // Connection closed gracefully, this is normal
                info!("Connection closed, reverse stream handler exiting");
                break;
            }
            Err(quinn::ConnectionError::TimedOut) => {
                // Connection timeout, but keep trying
                warn!("Connection timeout, but continuing to wait for streams...");
                continue;
            }
            Err(e) => {
                // Other errors might be transient, use exponential backoff
                warn!("Reverse stream accept error (will retry after {:?}): {}", backoff, e);
                tokio::time::sleep(backoff).await;
                
                // Increase backoff exponentially
                backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
            }
        }
    }
    
    Ok(())
}

/// Handle a single reverse stream from Server
async fn handle_reverse_stream(
    mut send: SendStream,
    mut recv: RecvStream,
    state: Arc<ClientState>,
) -> Result<()> {
    // 1. Read DataStreamHeader
    info!("Reading DataStreamHeader from reverse stream...");
    let header_start = std::time::Instant::now();
    let header = read_data_stream_header(&mut recv).await?;
    info!(
        "[{}] Received reverse DataStreamHeader: type={}, host={} (read in {:?})",
        header.request_id, header.r#type, header.host, header_start.elapsed()
    );
    
    // 2. Match rules based on type and host
    let rules: Vec<Rule> = state.rules.iter().map(|r| r.value().clone()).collect();
    let upstreams: Vec<Upstream> = state.upstreams.iter().map(|u| u.value().clone()).collect();
    
    info!("[{}] Matching rules for type={}, host={} (total rules: {}, total upstreams: {})", 
        header.request_id, header.r#type, header.host, rules.len(), upstreams.len());
    info!("[{}] Available rules: {:?}", 
        header.request_id, 
        rules.iter().map(|r| format!("{}:{}->{}", r.r#type, r.match_host, r.action_proxy_pass)).collect::<Vec<_>>());
    info!("[{}] Available upstreams: {:?}", 
        header.request_id,
        upstreams.iter().map(|u| {
            let addresses: Vec<String> = u.servers.iter().map(|s| s.address.clone()).collect();
            format!("{}:[{}]", u.name, addresses.join(","))
        }).collect::<Vec<_>>());
    
    let matched_rule = match_rule_by_type_and_host(&rules, &header.r#type, &header.host)?;
    
    let (final_target_addr, is_target_ssl) = if let Some(rule) = matched_rule {
        info!("[{}] Matched rule: {} -> {} (action_proxy_pass)", 
            header.request_id, header.host, rule.action_proxy_pass);
        
        // Resolve upstream or use direct address
        let resolved = resolve_upstream(&rule.action_proxy_pass, &upstreams)?;
        info!("[{}] Resolved upstream '{}' to address: {} (SSL: {})", 
            header.request_id, rule.action_proxy_pass, resolved.0, resolved.1);
        resolved
    } else {
        error!("[{}] No matching rule for type={}, host={} (available rules: {:?})", 
            header.request_id, header.r#type, header.host, 
            rules.iter().map(|r| format!("{}:{}", r.r#type, r.match_host)).collect::<Vec<_>>());
        return Err(anyhow::anyhow!("No matching rule for type={}, host={}", header.r#type, header.host));
    };
    
    info!("[{}] Forwarding to upstream: {} (SSL: {})", 
        header.request_id, final_target_addr, is_target_ssl);
    
    // 3. Connect to upstream (preserve protocol, don't parse HTTP)
    // Parse address to extract hostname and port
    let (hostname, port) = parse_target_addr(&final_target_addr, is_target_ssl)?;
    
    // 4. Get or create connection from pool
    let pool_key = format!("{}:{}:{}", hostname, port, is_target_ssl);
    let upstream_stream = match get_or_create_connection(
        &state,
        &pool_key,
        &hostname,
        port,
        is_target_ssl,
        &header.request_id,
        &header.r#type,
    ).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("[{}] Failed to connect to upstream {}: {}", 
                header.request_id, final_target_addr, e);
            // Try to send error response back to server before closing
            if let Err(send_err) = send.finish().await {
                warn!("[{}] Failed to finish QUIC send stream after upstream error: {}", 
                    header.request_id, send_err);
            }
            return Err(e);
        }
    };
    
    // 5. Bidirectional copy: QUIC stream <-> Upstream stream
    // Note: Due to tokio::io::split consuming the stream, connections cannot be reused after use
    // The connection pool infrastructure is in place, but true reuse requires HTTP parsing
    // For now, connections are consumed after use
    match copy_bidirectional_and_return(recv, send, upstream_stream, &header.request_id, &state, &pool_key).await {
        Ok(_conn) => {
            // This branch is currently unreachable due to split consuming the connection
            // Future: If we implement HTTP parsing, we can detect request/response boundaries
            // and reuse connections without splitting
            unreachable!("Connection reuse not yet implemented")
        }
        Err(e) => {
            // Connection consumed or error occurred
            // Note: Even successful copies consume the connection due to split
            if e.to_string().contains("consumed after split") {
                debug!("[{}] Connection consumed after use (expected)", header.request_id);
            } else {
                warn!("[{}] Bidirectional copy error: {}", header.request_id, e);
                // Don't return error for copy failures - connection might have been interrupted
                // This is normal when server or client closes connection
            }
        }
    }
    
    Ok(())
}

/// Bidirectional copy between QUIC stream and upstream stream (TLS or TCP)
/// Note: Due to tokio::io::split consuming the stream, we cannot reuse connections after split
/// This function will consume the connection, so connection pooling is limited
async fn copy_bidirectional_and_return(
    mut recv: RecvStream,
    mut send: SendStream,
    upstream_stream: PooledConnection,
    request_id: &str,
    _state: &ClientState,
    _pool_key: &str,
) -> Result<PooledConnection> {
    debug!("[{}] Starting bidirectional copy: QUIC stream <-> Upstream socket (raw data forwarding, no HTTP parsing)", 
        request_id);
    
    let copy_start = std::time::Instant::now();
    
    // Note: tokio::io::split consumes the stream, so we cannot reuse connections
    // For true connection reuse, we would need HTTP parsing to detect request/response boundaries
    // For now, connections are consumed after use
    
    match upstream_stream {
        PooledConnection::Tcp(tcp_stream) => {
            let (mut upstream_read, mut upstream_write) = tokio::io::split(tcp_stream);
            
            let request_id_clone = request_id.to_string();
            let quic_to_upstream = async move {
                debug!("[{}] Starting QUIC -> Upstream copy...", request_id_clone);
                match io::copy(&mut recv, &mut upstream_write).await {
                    Ok(bytes_copied) => {
                        debug!("[{}] QUIC -> Upstream: copied {} bytes in {:?}", 
                            request_id_clone, bytes_copied, copy_start.elapsed());
                        Ok::<(), anyhow::Error>(())
                    }
                    Err(e) => {
                        // Connection might be closed by peer, this is normal
                        debug!("[{}] QUIC -> Upstream copy ended: {}", request_id_clone, e);
                        Ok::<(), anyhow::Error>(()) // Don't propagate error
                    }
                }
            };
            
            let request_id_clone2 = request_id.to_string();
            let upstream_to_quic = async move {
                debug!("[{}] Starting Upstream -> QUIC copy...", request_id_clone2);
                match io::copy(&mut upstream_read, &mut send).await {
                    Ok(bytes_copied) => {
                        debug!("[{}] Upstream -> QUIC: copied {} bytes in {:?}", 
                            request_id_clone2, bytes_copied, copy_start.elapsed());
                    }
                    Err(e) => {
                        // Connection might be closed by peer, this is normal
                        debug!("[{}] Upstream -> QUIC copy ended: {}", request_id_clone2, e);
                    }
                }
                // Try to finish send stream gracefully
                if let Err(e) = send.finish().await {
                    debug!("[{}] QUIC stream finish error (might be already closed): {}", request_id_clone2, e);
                }
                Ok::<_, anyhow::Error>(())
            };
            
            // Use select instead of try_join to handle partial failures gracefully
            tokio::select! {
                _ = quic_to_upstream => {},
                _ = upstream_to_quic => {},
            }
            
            info!("[{}] Bidirectional copy completed in {:?}", request_id, copy_start.elapsed());
            
            // Connection consumed after split, cannot reuse
            Err(anyhow::anyhow!("Connection consumed after split, cannot reuse"))
        }
        PooledConnection::Tls(tls_stream) => {
            let (mut upstream_read, mut upstream_write) = tokio::io::split(tls_stream);
            
            let request_id_clone = request_id.to_string();
            let quic_to_upstream = async move {
                debug!("[{}] Starting QUIC -> Upstream copy...", request_id_clone);
                match io::copy(&mut recv, &mut upstream_write).await {
                    Ok(bytes_copied) => {
                        debug!("[{}] QUIC -> Upstream: copied {} bytes in {:?}", 
                            request_id_clone, bytes_copied, copy_start.elapsed());
                        Ok::<(), anyhow::Error>(())
                    }
                    Err(e) => {
                        debug!("[{}] QUIC -> Upstream copy ended: {}", request_id_clone, e);
                        Ok::<(), anyhow::Error>(())
                    }
                }
            };
            
            let request_id_clone2 = request_id.to_string();
            let upstream_to_quic = async move {
                debug!("[{}] Starting Upstream -> QUIC copy...", request_id_clone2);
                match io::copy(&mut upstream_read, &mut send).await {
                    Ok(bytes_copied) => {
                        debug!("[{}] Upstream -> QUIC: copied {} bytes in {:?}", 
                            request_id_clone2, bytes_copied, copy_start.elapsed());
                    }
                    Err(e) => {
                        debug!("[{}] Upstream -> QUIC copy ended: {}", request_id_clone2, e);
                    }
                }
                if let Err(e) = send.finish().await {
                    debug!("[{}] QUIC stream finish error (might be already closed): {}", request_id_clone2, e);
                }
                Ok::<_, anyhow::Error>(())
            };
            
            // Use select instead of try_join to handle partial failures gracefully
            tokio::select! {
                _ = quic_to_upstream => {},
                _ = upstream_to_quic => {},
            }
            
            info!("[{}] Bidirectional copy completed in {:?}", request_id, copy_start.elapsed());
            
            // Connection consumed after split, cannot reuse
            Err(anyhow::anyhow!("Connection consumed after split, cannot reuse"))
        }
    }
}

