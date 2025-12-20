use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, warn};
use quinn::{SendStream, RecvStream};
use uuid::Uuid;
use bytes::BytesMut;
use crate::types::ServerState;
use crate::egress_forwarder::{forward_egress_http_request, forward_egress_grpc_request, forward_egress_wss_request};
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, read_frame, write_frame, RoutingInfo};
use tunnel_lib::direct_forward::forward_bidirectional_raw;
use tokio::net::TcpStream;
use std::time::Duration;
use tokio_rustls::TlsConnector;
use rustls::ClientConfig;
use webpki_roots::TLS_SERVER_ROOTS;

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
    
    // Parse host to extract hostname (remove port if present)
    let host_with_port = routing_info.host.as_str();
    let host = host_with_port.split(':').next().unwrap_or(host_with_port).trim();
    if host != host_with_port {
        info!("[{}] Parsed host from '{}' to '{}' (removed port)", request_id, host_with_port, host);
    }
    
    let session_id = routing_frame.session_id;
    
    // Find upstream using RuleMatcher for O(1) lookup
    let matched_upstream = {
        if routing_info.r#type == "http" {
            let matcher = state.rule_matcher.read().await;
            matcher.match_egress_http_rule(&host)
                .map(|r| r.action_upstream)
        } else if routing_info.r#type == "grpc" {
            // For gRPC, we need service name - try to extract from path or use empty string
            let service = routing_info.path.split('/').nth(1).unwrap_or("");
            let matcher = state.rule_matcher.read().await;
            matcher.match_egress_grpc_rule(&host, service)
                .map(|r| r.action_upstream)
                .or_else(|| {
                    // Fallback: try matching by host only (if service is empty in rule)
                    state.egress_rules_grpc.iter()
                        .find(|r| r.match_host.eq_ignore_ascii_case(host) && r.match_service.is_empty())
                        .map(|r| r.action_upstream.clone())
                })
        } else {
            None
        }
    };
    
    let (final_target_addr, is_target_ssl) = if let Some(upstream_name) = matched_upstream {
        if let Some(upstream) = state.egress_upstreams.get(&upstream_name) {
            info!("[{}] Matched egress rule: {} -> upstream {} ({})", 
                request_id, host, upstream_name, upstream.address);
            (upstream.address.clone(), upstream.is_ssl)
        } else {
            error!("[{}] Upstream '{}' not found", request_id, upstream_name);
            return Err(anyhow::anyhow!("Upstream '{}' not found", upstream_name));
        }
    } else {
        error!("[{}] No matching egress rule for type={}, host={} (parsed from: {})", 
            request_id, routing_info.r#type, host, routing_info.host);
        return Err(anyhow::anyhow!("No matching egress rule for type={}, host={}", 
            routing_info.r#type, host));
    };
    
    info!("[{}] Forwarding to upstream: {} (SSL: {})", 
        request_id, final_target_addr, is_target_ssl);
    
    // For HTTP, gRPC, and WSS: read frames first (same as server->client ingress)
    // This ensures consistency with how client handles requests from server
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
    
    // Read first data frame
    let first_frame = match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(REQUEST_TIMEOUT)).await {
        Ok(frame) => {
            if frame.session_id != session_id {
                warn!("[{}] Received frame with mismatched session_id: {} (expected {})", 
                    request_id, frame.session_id, session_id);
                return Err(anyhow::anyhow!("Mismatched session_id"));
            }
            frame
        }
        Err(e) => {
            if e.to_string().contains("timeout") {
                error!("[{}] Request timeout after {:?}", request_id, REQUEST_TIMEOUT);
            } else {
                error!("[{}] Error reading first frame: {}", request_id, e);
            }
            return Err(e);
        }
    };
    
    // Collect all frames
    let mut request_buffer = BytesMut::from(first_frame.payload.as_slice());
    let mut session_complete = first_frame.end_of_stream;
    
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
    

    let protocol_type_enum = match routing_info.r#type.as_str() {
        "http" => ProtocolType::Http11,
        "grpc" => ProtocolType::Grpc,
        "wss" => ProtocolType::WssFrame,
        _ => {
            error!("[{}] Unknown protocol type: {}", request_id, routing_info.r#type);
            return Err(anyhow::anyhow!("Unknown protocol type: {}", routing_info.r#type));
        }
    };
    
    // Non-streaming mode: use existing forwarders
    let response_bytes = match routing_info.r#type.as_str() {
        "http" => {
            forward_egress_http_request(
                &state.egress_pool.client(),
                &request_buffer,
                &final_target_addr,
                is_target_ssl,
            ).await
        }
        "grpc" => {
            forward_egress_grpc_request(
                &request_buffer,
                &final_target_addr,
                is_target_ssl,
            ).await
        }
        "wss" => {
            forward_egress_wss_request(
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
            
            // Detect HTTP version from request
            let http_version = tunnel_lib::http_version::HttpVersion::detect_from_request(&request_buffer)
                .unwrap_or(tunnel_lib::http_version::HttpVersion::Http11);
            
            let error_response = format!(
                "{} 502 Bad Gateway\r\n\
                Content-Length: {}\r\n\
                Content-Type: text/plain\r\n\
                \r\n\
                {}",
                http_version.to_status_line_string(),
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
