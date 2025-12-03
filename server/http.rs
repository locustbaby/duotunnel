use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, debug, warn};
use uuid::Uuid;
use bytes::BytesMut;
use httparse::Request;
use crate::types::ServerState;
use crate::client_mgr::select_client_from_group;
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, write_frame, RoutingInfo, create_routing_frame};
use tunnel_lib::frame::TunnelFrame as Frame;

/// Start HTTP listener
pub async fn start_http_listener(port: u16, state: Arc<ServerState>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP listener listening on 0.0.0.0:{}", port);

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                debug!("Accepted HTTP connection from {}", peer_addr);
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_http_request(stream, peer_addr, state).await {
                        error!("HTTP request handling error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept HTTP connection: {}", e);
            }
        }
    }
}

/// Handle HTTP request from external client using frame protocol
async fn handle_http_request(
    mut stream: TcpStream,
    _peer_addr: SocketAddr,
    state: Arc<ServerState>,
) -> Result<()> {
    // 1. Read complete HTTP request (headers + body)
    let request_start = std::time::Instant::now();
    let complete_request = read_complete_http_request(&mut stream).await?;
    info!("Read complete HTTP request ({} bytes) in {:?}", complete_request.len(), request_start.elapsed());
    
    // 2. Parse HTTP headers to extract Host
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    
    let parse_result = req.parse(&complete_request)?;
    match parse_result {
        httparse::Status::Complete(_) => {
            debug!("Parsed HTTP request: {} {}", req.method.unwrap_or(""), req.path.unwrap_or(""));
        }
        httparse::Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Extract Host header
    let host_with_port = req.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case("host"))
        .and_then(|h| std::str::from_utf8(h.value).ok())
        .map(|s| s.to_string());
    
    let host_with_port = host_with_port.ok_or_else(|| anyhow::anyhow!("Missing Host header"))?;
    debug!("HTTP request Host: {}", host_with_port);
    
    // Strip port from host header for matching (Host header can include port like "hostname:8001")
    let host = host_with_port.split(':').next().unwrap_or(&host_with_port).trim();
    debug!("Extracted hostname (without port): {}", host);
    
    // 3. Match ingress routing rule
    let matched_rule = state.ingress_rules.iter()
        .find(|r| r.match_host.eq_ignore_ascii_case(host));
    
    let client_group = match matched_rule {
        Some(rule) => {
            debug!("Matched ingress rule: {} -> group {}", host, rule.action_client_group);
            rule.action_client_group.clone()
        }
        None => {
            warn!("No matching ingress rule for host: {}", host);
            return Err(anyhow::anyhow!("No matching ingress rule for host: {}", host));
        }
    };
    
    // 4. Select a client from the group
    info!("Selecting client from group '{}' for host '{}'", client_group, host);
    let client_id = select_client_from_group(&state, &client_group)?;
    
    // 5. Get Connection from registry
    let client_conn = state.clients.get(&client_id)
        .ok_or_else(|| {
            error!("Client '{}' not found in clients registry", client_id);
            anyhow::anyhow!("Client {} not found", client_id)
        })?
        .clone();
    
    info!("Selected client '{}' from group '{}', opening QUIC stream...", client_id, client_group);
    
    // 6. Open QUIC bidirectional stream
    let (mut send, mut recv) = match client_conn.open_bi().await {
        Ok(streams) => {
            info!("Successfully opened QUIC bidirectional stream to client '{}'", client_id);
            streams
        }
        Err(e) => {
            error!("Failed to open QUIC stream to client '{}': {}", client_id, e);
            
            // Active Probing: Check if client is actually dead
            use crate::client_mgr::probe_client;
            if !probe_client(&state, &client_id).await {
                // Probe failed, client removed
                return Err(anyhow::anyhow!("Client {} is dead (probe failed)", client_id));
            }
            
            // Probe succeeded (maybe transient error?), try one more time
            info!("Client {} probe succeeded, retrying stream open...", client_id);
            match client_conn.open_bi().await {
                Ok(streams) => {
                    info!("Successfully opened QUIC stream to client '{}' after probe", client_id);
                    streams
                }
                Err(e2) => {
                    return Err(anyhow::anyhow!("Failed to open QUIC stream after probe: {}", e2));
                }
            }
        }
    };
    
    // 7. Create Session ID and routing frame
    let request_id = Uuid::new_v4().to_string();
    let session_id = Frame::session_id_from_uuid(&request_id);
    
    // Extract method and path from HTTP request
    let method = req.method.unwrap_or("GET").to_string();
    let path = req.path.unwrap_or("/").to_string();
    
    let routing_info = RoutingInfo {
        r#type: "http".to_string(),
        host: host_with_port.clone(), // Use original host with port for routing
        method,
        path,
    };
    
    // Send routing frame (first frame)
    let routing_frame = create_routing_frame(session_id, &routing_info);
    write_frame(&mut send, &routing_frame).await?;
    info!("[{}] Sent routing frame to client {}: session_id={}, host={}", 
        request_id, client_id, session_id, host_with_port);
    
    // 8. Split request into frames (max 64KB per frame to avoid QUIC limits)
    const MAX_FRAME_SIZE: usize = 64 * 1024; // 64KB
    let request_bytes = complete_request.to_vec();
    let mut offset = 0;
    
    while offset < request_bytes.len() {
        let chunk_size = std::cmp::min(MAX_FRAME_SIZE, request_bytes.len() - offset);
        let chunk = request_bytes[offset..offset + chunk_size].to_vec();
        let is_last = offset + chunk_size >= request_bytes.len();
        
        let data_frame = TunnelFrame::new(
            session_id,
            ProtocolType::Http11,
            is_last, // END_OF_STREAM flag on last frame
            chunk,
        );
        
        write_frame(&mut send, &data_frame).await?;
        offset += chunk_size;
    }
    
    info!("[{}] Sent {} frames (total {} bytes) to client", 
        request_id, (request_bytes.len() + MAX_FRAME_SIZE - 1) / MAX_FRAME_SIZE, request_bytes.len());
    
    // 9. Receive response frames and reassemble (with timeout)
    let mut response_buffer = BytesMut::new();
    let mut session_complete = false;
    const RESPONSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60); // 60s timeout
    
    while !session_complete {
        match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(RESPONSE_TIMEOUT)).await {
            Ok(frame) => {
                if frame.session_id != session_id {
                    warn!("[{}] Received frame with mismatched session_id: {} (expected {})", 
                        request_id, frame.session_id, session_id);
                    continue;
                }
                
                response_buffer.extend_from_slice(&frame.payload);
                session_complete = frame.end_of_stream;
                
                if session_complete {
                    info!("[{}] Received complete response ({} bytes)", request_id, response_buffer.len());
                }
            }
            Err(e) => {
                if e.to_string().contains("timeout") {
                    error!("[{}] Response timeout after {:?}", request_id, RESPONSE_TIMEOUT);
                } else {
                    error!("[{}] Error reading frame: {}", request_id, e);
                }
                return Err(e);
            }
        }
    }
    
    // 10. Send response to external client
    stream.write_all(&response_buffer).await?;
    stream.flush().await?;
    info!("[{}] Sent response to external client ({} bytes) in {:?}", 
        request_id, response_buffer.len(), request_start.elapsed());
    
    Ok(())
}

/// Read complete HTTP request (headers + body) from a stream
async fn read_complete_http_request(
    socket: &mut TcpStream,
) -> Result<BytesMut> {
    let mut buffer = BytesMut::new();
    
    // Step 1: Read headers
    let mut header_end = false;
    while !header_end {
        let mut buf = vec![0u8; 4096];
        let n = socket.read(&mut buf).await?;
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
    let mut req = Request::new(&mut headers);
    
    let parse_result = req.parse(header_bytes)?;
    match parse_result {
        httparse::Status::Complete(_) => {}
        httparse::Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Step 3: Determine body length
    let body_length = determine_body_length(&req.headers)?;
    
    // Step 4: Read body if needed
    if let Some(len) = body_length {
        // Content-Length specified
        let remaining_in_buffer = buffer.len() - header_end_pos;
        if remaining_in_buffer < len {
            let needed = len - remaining_in_buffer;
            let mut body_buf = vec![0u8; needed];
            socket.read_exact(&mut body_buf).await?;
            buffer.extend_from_slice(&body_buf);
        }
    } else {
        // Transfer-Encoding: chunked
        read_chunked_body(socket, &mut buffer).await?;
    }
    
    Ok(buffer)
}

/// Determine body length from HTTP headers
fn determine_body_length(headers: &[httparse::Header]) -> Result<Option<usize>> {
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
    
    // No body (GET requests typically)
    Ok(Some(0))
}

/// Read chunked body
async fn read_chunked_body(socket: &mut TcpStream, buffer: &mut BytesMut) -> Result<()> {
    // Read chunked data
    // Format: <chunk-size>\r\n<chunk-data>\r\n...
    // Last chunk: 0\r\n\r\n
    
    loop {
        // Read chunk size line
        let mut chunk_size_line = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            socket.read_exact(&mut byte).await?;
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
            socket.read_exact(&mut trailer).await?;
            if trailer != [b'\r', b'\n'] {
                warn!("Invalid chunked encoding trailer");
            }
            break;
        }
        
        // Read chunk data
        let mut chunk_data = vec![0u8; chunk_size];
        socket.read_exact(&mut chunk_data).await?;
        buffer.extend_from_slice(&chunk_data);
        
        // Read trailing \r\n
        let mut trailer = [0u8; 2];
        socket.read_exact(&mut trailer).await?;
        if trailer != [b'\r', b'\n'] {
            warn!("Invalid chunked encoding trailer after chunk data");
        }
    }
    
    Ok(())
}
