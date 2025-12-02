use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, warn};
use quinn::{Connection, SendStream, RecvStream};
use uuid::Uuid;
use bytes::BytesMut;
use crate::types::ClientState;
use crate::routing::{match_rule_by_type_and_host, resolve_upstream};
use crate::http_forwarder::forward_http_request;
use crate::wss_forwarder::forward_wss_request;
use crate::grpc_forwarder::forward_grpc_request;
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, read_frame, write_frame, RoutingInfo};
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

/// Handle a single reverse stream from Server using frame protocol
async fn handle_reverse_stream(
    mut send: SendStream,
    mut recv: RecvStream,
    state: Arc<ClientState>,
) -> Result<()> {
    let request_id = Uuid::new_v4().to_string();
    let stream_start = std::time::Instant::now();
    
    // 1. Read first frame (routing frame)
    info!("[{}] Reading routing frame from reverse stream...", request_id);
    let routing_frame = read_frame(&mut recv).await?;
    
    // Parse routing information from first frame payload
    let routing_info = RoutingInfo::decode(&routing_frame.payload)?;
    info!(
        "[{}] Received routing frame: type={}, host={}, session_id={}",
        request_id, routing_info.r#type, routing_info.host, routing_frame.session_id
    );
    
    let session_id = routing_frame.session_id;
    
    // 2. Read all request frames and reassemble (with timeout)
    let mut request_buffer = BytesMut::new();
    let mut session_complete = false;
    const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30); // 30s timeout
    
    while !session_complete {
        match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(REQUEST_TIMEOUT)).await {
            Ok(frame) => {
                if frame.session_id != session_id {
                    warn!("[{}] Received frame with mismatched session_id: {} (expected {})", 
                        request_id, frame.session_id, session_id);
                    continue;
                }
                
                request_buffer.extend_from_slice(&frame.payload);
                session_complete = frame.end_of_stream;
                
                if session_complete {
                    info!("[{}] Received complete request ({} bytes)", request_id, request_buffer.len());
                }
            }
            Err(e) => {
                if e.to_string().contains("timeout") {
                    error!("[{}] Request timeout after {:?}", request_id, REQUEST_TIMEOUT);
                } else {
                    error!("[{}] Error reading frame: {}", request_id, e);
                }
                return Err(e);
            }
        }
    }
    
    // 3. Match rules based on type and host
    let rules: Vec<Rule> = state.rules.iter().map(|r| r.value().clone()).collect();
    let upstreams: Vec<Upstream> = state.upstreams.iter().map(|u| u.value().clone()).collect();
    
    info!("[{}] Matching rules for type={}, host={} (total rules: {}, total upstreams: {})", 
        request_id, routing_info.r#type, routing_info.host, rules.len(), upstreams.len());
    
    let matched_rule = match_rule_by_type_and_host(&rules, &routing_info.r#type, &routing_info.host)?;
    
    let (final_target_addr, is_target_ssl) = if let Some(rule) = matched_rule {
        info!("[{}] Matched rule: {} -> {} (action_proxy_pass)", 
            request_id, routing_info.host, rule.action_proxy_pass);
        
        // Resolve upstream or use direct address
        let resolved = resolve_upstream(&rule.action_proxy_pass, &upstreams)?;
        info!("[{}] Resolved upstream '{}' to address: {} (SSL: {})", 
            request_id, rule.action_proxy_pass, resolved.0, resolved.1);
        resolved
    } else {
        error!("[{}] No matching rule for type={}, host={}", 
            request_id, routing_info.r#type, routing_info.host);
        return Err(anyhow::anyhow!("No matching rule for type={}, host={}", 
            routing_info.r#type, routing_info.host));
    };
    
    info!("[{}] Forwarding to upstream: {} (SSL: {})", 
        request_id, final_target_addr, is_target_ssl);
    
    // 4. Forward request based on protocol type
    let protocol_type_enum = match routing_info.r#type.as_str() {
        "http" => ProtocolType::Http11,
        "wss" => ProtocolType::WssFrame,
        "grpc" => ProtocolType::Grpc,
        _ => {
            error!("[{}] Unknown protocol type: {}", request_id, routing_info.r#type);
            return Err(anyhow::anyhow!("Unknown protocol type: {}", routing_info.r#type));
        }
    };
    
    let response_bytes = match routing_info.r#type.as_str() {
        "http" => {
            forward_http_request(
                &state.http_client,
                &state.https_client,
                &request_buffer,
                &final_target_addr,
                is_target_ssl,
            ).await
        }
        "wss" => {
            forward_wss_request(
                &state.https_client,
                &request_buffer,
                &final_target_addr,
                is_target_ssl,
            ).await
        }
        "grpc" => {
            forward_grpc_request(
                &request_buffer,
                &final_target_addr,
                is_target_ssl,
            ).await
        }
        _ => {
            anyhow::bail!("Unsupported protocol type: {}", routing_info.r#type);
        }
    };
    
    let response_bytes = match response_bytes {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("[{}] Failed to forward {} request: {}", request_id, routing_info.r#type, e);
            
            // Send error response frame to server
            let error_response = format!(
                "HTTP/1.1 502 Bad Gateway\r\n\
                Content-Length: {}\r\n\
                Content-Type: text/plain\r\n\
                \r\n\
                {}",
                e.to_string().len(),
                e
            );
            
            let error_frame = TunnelFrame::new(
                session_id,
                protocol_type_enum,
                true, // END_OF_STREAM
                error_response.into_bytes(),
            );
            
            if let Err(send_err) = write_frame(&mut send, &error_frame).await {
                error!("[{}] Failed to send error response frame: {}", request_id, send_err);
            }
            
            // Finish the send stream
            if let Err(finish_err) = send.finish().await {
                error!("[{}] Failed to finish send stream: {}", request_id, finish_err);
            }
            
            return Err(e);
        }
    };
    
    info!("[{}] Received response from upstream ({} bytes)", request_id, response_bytes.len());
    
    // 5. Split response into frames and send back to server
    const MAX_FRAME_SIZE: usize = 64 * 1024; // 64KB
    let mut offset = 0;
    
    while offset < response_bytes.len() {
        let chunk_size = std::cmp::min(MAX_FRAME_SIZE, response_bytes.len() - offset);
        let chunk = response_bytes[offset..offset + chunk_size].to_vec();
        let is_last = offset + chunk_size >= response_bytes.len();
        
        let response_frame = TunnelFrame::new(
            session_id,
            protocol_type_enum,
            is_last, // END_OF_STREAM flag on last frame
            chunk,
        );
        
        if let Err(e) = write_frame(&mut send, &response_frame).await {
            error!("[{}] Failed to write response frame: {}", request_id, e);
            return Err(e.into());
        }
        
        offset += chunk_size;
    }
    
    info!("[{}] Sent {} response frames (total {} bytes) to server in {:?}", 
        request_id, (response_bytes.len() + MAX_FRAME_SIZE - 1) / MAX_FRAME_SIZE, 
        response_bytes.len(), stream_start.elapsed());
    
    // 6. Finish the send stream (critical fix for "stream finished early" bug)
    if let Err(e) = send.finish().await {
        error!("[{}] Failed to finish send stream: {}", request_id, e);
        return Err(e.into());
    }
    
    Ok(())
}

