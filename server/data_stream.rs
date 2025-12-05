use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, warn};
use quinn::{SendStream, RecvStream};
use uuid::Uuid;
use bytes::BytesMut;
use crate::types::ServerState;
use crate::egress_forwarder::forward_egress_http_request;
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, read_frame, write_frame, RoutingInfo};

pub async fn handle_data_stream(
    mut send: SendStream,
    mut recv: RecvStream,
    state: Arc<ServerState>,
) -> Result<()> {
    let request_id = Uuid::new_v4().to_string();
    let stream_start = std::time::Instant::now();
    

    info!("[{}] Reading routing frame from client-initiated stream...", request_id);
    let routing_frame = read_frame(&mut recv).await?;
    
    let routing_info = RoutingInfo::decode(&routing_frame.payload)?;
    info!(
        "[{}] Received routing frame: type={}, host={}, session_id={}",
        request_id, routing_info.r#type, routing_info.host, routing_frame.session_id
    );
    
    let session_id = routing_frame.session_id;
    

    let mut request_buffer = BytesMut::new();
    let mut session_complete = false;
    const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
    
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
    

    let matched_upstream = if routing_info.r#type == "http" {
        state.egress_rules_http.iter()
            .find(|r| r.match_host.eq_ignore_ascii_case(&routing_info.host))
            .map(|r| r.action_upstream.clone())
    } else if routing_info.r#type == "grpc" {
        state.egress_rules_grpc.iter()
            .find(|r| r.match_host.eq_ignore_ascii_case(&routing_info.host))
            .map(|r| r.action_upstream.clone())
    } else {
        None
    };
    
    let (final_target_addr, is_target_ssl) = if let Some(upstream_name) = matched_upstream {
        if let Some(upstream) = state.egress_upstreams.get(&upstream_name) {
            info!("[{}] Matched egress rule: {} -> upstream {} ({})", 
                request_id, routing_info.host, upstream_name, upstream.address);
            (upstream.address.clone(), upstream.is_ssl)
        } else {
            error!("[{}] Upstream '{}' not found", request_id, upstream_name);
            return Err(anyhow::anyhow!("Upstream '{}' not found", upstream_name));
        }
    } else {
        error!("[{}] No matching egress rule for type={}, host={}", 
            request_id, routing_info.r#type, routing_info.host);
        return Err(anyhow::anyhow!("No matching egress rule for type={}, host={}", 
            routing_info.r#type, routing_info.host));
    };
    
    info!("[{}] Forwarding to upstream: {} (SSL: {})", 
        request_id, final_target_addr, is_target_ssl);
    

    let protocol_type_enum = match routing_info.r#type.as_str() {
        "http" => ProtocolType::Http11,
        "grpc" => ProtocolType::Grpc,
        "wss" => ProtocolType::WssFrame,
        _ => {
            error!("[{}] Unknown protocol type: {}", request_id, routing_info.r#type);
            return Err(anyhow::anyhow!("Unknown protocol type: {}", routing_info.r#type));
        }
    };
    
    let response_bytes = match routing_info.r#type.as_str() {
        "http" => {
            forward_egress_http_request(
                &state.egress_pool.client(),
                &request_buffer,
                &final_target_addr,
                is_target_ssl,
            ).await
        }
        _ => {
            anyhow::bail!("Protocol {} not yet implemented for egress forwarding", routing_info.r#type);
        }
    };
    
    let response_bytes = match response_bytes {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("[{}] Failed to forward {} request: {}", request_id, routing_info.r#type, e);
            
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
                true,
                error_response.into_bytes(),
            );
            
            if let Err(send_err) = write_frame(&mut send, &error_frame).await {
                error!("[{}] Failed to send error response frame: {}", request_id, send_err);
            }
            
            if let Err(finish_err) = send.finish() {
                error!("[{}] Failed to finish send stream: {}", request_id, finish_err);
            }
            
            return Err(e);
        }
    };
    
    info!("[{}] Received response from upstream ({} bytes)", request_id, response_bytes.len());
    

    const MAX_FRAME_SIZE: usize = 64 * 1024;
    let mut offset = 0;
    
    while offset < response_bytes.len() {
        let chunk_size = std::cmp::min(MAX_FRAME_SIZE, response_bytes.len() - offset);
        let chunk = response_bytes[offset..offset + chunk_size].to_vec();
        let is_last = offset + chunk_size >= response_bytes.len();
        
        let response_frame = TunnelFrame::new(
            session_id,
            protocol_type_enum,
            is_last,
            chunk,
        );
        
        if let Err(e) = write_frame(&mut send, &response_frame).await {
            error!("[{}] Failed to write response frame: {}", request_id, e);
            return Err(e.into());
        }
        
        offset += chunk_size;
    }
    
    info!("[{}] Sent {} response frames (total {} bytes) to client in {:?}", 
        request_id, (response_bytes.len() + MAX_FRAME_SIZE - 1) / MAX_FRAME_SIZE, 
        response_bytes.len(), stream_start.elapsed());
    

    if let Err(e) = send.finish() {
        error!("[{}] Failed to finish send stream: {}", request_id, e);
        return Err(e.into());
    }
    
    Ok(())
}
