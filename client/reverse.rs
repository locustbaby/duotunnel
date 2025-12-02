use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, warn};
use quinn::{Connection, SendStream, RecvStream};
use uuid::Uuid;
use bytes::BytesMut;
use crate::types::ClientState;
use crate::pool::{get_or_create_connection, return_connection_to_pool};
use crate::routing::{match_rule_by_type_and_host, resolve_upstream, parse_target_addr};
use crate::types::PooledConnection;
use crate::http_handler::read_complete_http_response;
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
    
    // 4. Connect to upstream
    let (hostname, port) = parse_target_addr(&final_target_addr, is_target_ssl)?;
    
    // 5. Get or create connection from pool
    let pool_key = format!("{}:{}:{}", hostname, port, is_target_ssl);
    let mut upstream_stream = match get_or_create_connection(
        &state,
        &pool_key,
        &hostname,
        port,
        is_target_ssl,
        &request_id,
        &routing_info.r#type,
    ).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("[{}] Failed to connect to upstream {}: {}", 
                request_id, final_target_addr, e);
            return Err(e);
        }
    };
    
    // 6. Send complete HTTP request to upstream
    match &mut upstream_stream {
        PooledConnection::Tcp(ref mut tcp_stream) => {
            tcp_stream.write_all(&request_buffer).await?;
            tcp_stream.flush().await?;
        }
        PooledConnection::Tls(ref mut tls_stream) => {
            tls_stream.write_all(&request_buffer).await?;
            tls_stream.flush().await?;
        }
    }
    
    info!("[{}] Sent request to upstream ({} bytes)", request_id, request_buffer.len());
    
    // 7. Read complete HTTP response from upstream
    let response_buffer = match &mut upstream_stream {
        PooledConnection::Tcp(ref mut tcp_stream) => {
            read_complete_http_response(tcp_stream).await?
        }
        PooledConnection::Tls(ref mut tls_stream) => {
            // For TLS, we need to read manually since we can't use the helper directly
            // For now, use a simple approach: read until connection closes or Content-Length
            read_complete_http_response_tls(tls_stream).await?
        }
    };
    
    info!("[{}] Received response from upstream ({} bytes)", request_id, response_buffer.len());
    
    // 8. Split response into frames and send back to server
    const MAX_FRAME_SIZE: usize = 64 * 1024; // 64KB
    let response_bytes = response_buffer.to_vec();
    let mut offset = 0;
    
    while offset < response_bytes.len() {
        let chunk_size = std::cmp::min(MAX_FRAME_SIZE, response_bytes.len() - offset);
        let chunk = response_bytes[offset..offset + chunk_size].to_vec();
        let is_last = offset + chunk_size >= response_bytes.len();
        
        let response_frame = TunnelFrame::new(
            session_id,
            ProtocolType::Http11,
            is_last, // END_OF_STREAM flag on last frame
            chunk,
        );
        
        write_frame(&mut send, &response_frame).await?;
        offset += chunk_size;
    }
    
    info!("[{}] Sent {} response frames (total {} bytes) to server in {:?}", 
        request_id, (response_bytes.len() + MAX_FRAME_SIZE - 1) / MAX_FRAME_SIZE, 
        response_bytes.len(), stream_start.elapsed());
    
    // 9. Try to return connection to pool (if possible)
    // Note: Currently connections are consumed, but we can try to return them
    // In the future, with proper HTTP parsing, we can reuse connections
    return_connection_to_pool(&state, &pool_key, upstream_stream).await;
    
    Ok(())
}

/// Read complete HTTP response from TLS stream
async fn read_complete_http_response_tls(
    tls_stream: &mut tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
) -> Result<BytesMut> {
    let mut buffer = BytesMut::new();
    
    // Step 1: Read headers
    let mut header_end = false;
    while !header_end {
        let mut buf = vec![0u8; 4096];
        let n = tls_stream.read(&mut buf).await?;
        if n == 0 {
            anyhow::bail!("Connection closed before headers");
        }
        
        buffer.extend_from_slice(&buf[..n]);
        
        // Check for end of headers
        if buffer.len() >= 4 {
            for i in 0..=buffer.len().saturating_sub(4) {
                if &buffer[i..i+4] == b"\r\n\r\n" {
                    header_end = true;
                    break;
                }
            }
        }
        
        // Safety limit
        if buffer.len() > 8192 {
            anyhow::bail!("HTTP headers too large");
        }
    }
    
    // Find header end position
    let header_end_pos = buffer.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(buffer.len());
    
    let header_bytes = &buffer[..header_end_pos];
    
    // Step 2: Parse headers to determine body length
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut resp = httparse::Response::new(&mut headers);
    
    let parse_result = resp.parse(header_bytes)?;
    match parse_result {
        httparse::Status::Complete(_) => {}
        httparse::Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Step 3: Determine body length
    let body_length = determine_response_body_length(&resp.headers)?;
    
    // Step 4: Read body if needed
    if let Some(len) = body_length {
        // Content-Length specified
        let remaining_in_buffer = buffer.len() - header_end_pos;
        if remaining_in_buffer < len {
            let needed = len - remaining_in_buffer;
            let mut body_buf = vec![0u8; needed];
            tls_stream.read_exact(&mut body_buf).await?;
            buffer.extend_from_slice(&body_buf);
        }
    } else {
        // Transfer-Encoding: chunked or connection close
        read_chunked_body_tls(tls_stream, &mut buffer).await?;
    }
    
    Ok(buffer)
}

/// Determine response body length from HTTP headers
fn determine_response_body_length(headers: &[httparse::Header]) -> Result<Option<usize>> {
    // Check for Transfer-Encoding: chunked
    for header in headers {
        if header.name.eq_ignore_ascii_case("transfer-encoding") {
            let value = std::str::from_utf8(header.value)?;
            if value.eq_ignore_ascii_case("chunked") {
                return Ok(None); // Chunked encoding
            }
        }
    }
    
    // Check for Content-Length
    for header in headers {
        if header.name.eq_ignore_ascii_case("content-length") {
            let value = std::str::from_utf8(header.value)?;
            let len = value.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("Invalid Content-Length: {} ({})", value, e))?;
            return Ok(Some(len));
        }
    }
    
    // For responses, if no Content-Length and no Transfer-Encoding,
    // body ends when connection closes (for HTTP/1.1)
    Ok(None)
}

/// Read chunked body from TLS stream
async fn read_chunked_body_tls(
    tls_stream: &mut tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
    buffer: &mut BytesMut,
) -> Result<()> {
    loop {
        // Read chunk size line
        let mut chunk_size_line = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            tls_stream.read_exact(&mut byte).await?;
            chunk_size_line.push(byte[0]);
            
            if chunk_size_line.len() >= 2 && chunk_size_line[chunk_size_line.len()-2..] == [b'\r', b'\n'] {
                break;
            }
        }
        
        // Parse chunk size (hex)
        let size_str = std::str::from_utf8(&chunk_size_line[..chunk_size_line.len()-2])?;
        let chunk_size = usize::from_str_radix(size_str.trim(), 16)
            .map_err(|e| anyhow::anyhow!("Invalid chunk size: {} ({})", size_str, e))?;
        
        if chunk_size == 0 {
            // Last chunk, read trailing \r\n
            let mut trailer = [0u8; 2];
            tls_stream.read_exact(&mut trailer).await?;
            if trailer != [b'\r', b'\n'] {
                warn!("Invalid chunked encoding trailer");
            }
            break;
        }
        
        // Read chunk data
        let mut chunk_data = vec![0u8; chunk_size];
        tls_stream.read_exact(&mut chunk_data).await?;
        buffer.extend_from_slice(&chunk_data);
        
        // Read trailing \r\n
        let mut trailer = [0u8; 2];
        tls_stream.read_exact(&mut trailer).await?;
        if trailer != [b'\r', b'\n'] {
            warn!("Invalid chunked encoding trailer after chunk data");
        }
    }
    
    Ok(())
}
