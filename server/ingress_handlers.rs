use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tracing::{info, error, debug, warn};
use uuid::Uuid;
use bytes::BytesMut;
use httparse::Request;
use crate::types::ServerState;
use crate::control::{select_client_from_group, probe_client};
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, write_frame, RoutingInfo, create_routing_frame};
use tunnel_lib::frame::TunnelFrame as Frame;

pub struct HttpIngressHandler {
    state: Arc<ServerState>,
}

impl HttpIngressHandler {
    pub fn new(state: Arc<ServerState>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl tunnel_lib::listener::ConnectionHandler for HttpIngressHandler {
    async fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let request_start = std::time::Instant::now();
        

        let complete_request = read_complete_http_request(&mut stream).await?;
        info!("Read complete HTTP request ({} bytes) in {:?}", complete_request.len(), request_start.elapsed());
        

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
        

        let host_with_port = req.headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("host"))
            .and_then(|h| std::str::from_utf8(h.value).ok())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Missing Host header"))?;
        
        debug!("HTTP request Host: {}", host_with_port);
        
        let host = host_with_port.split(':').next().unwrap_or(&host_with_port).trim();
        debug!("Extracted hostname (without port): {}", host);
        

        // Use RuleMatcher for O(1) lookup
        let matched_rule = {
            let matcher = self.state.rule_matcher.read().await;
            matcher.match_ingress_rule("http", &host)
        };
        
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
        

        info!("Selecting client from group '{}' for host '{}'", client_group, host);
        let client_id = select_client_from_group(&self.state, &client_group)?;
        

        let client_conn = self.state.clients.get(&client_id)
            .ok_or_else(|| {
                error!("Client '{}' not found in clients registry", client_id);
                anyhow::anyhow!("Client {} not found", client_id)
            })?
            .clone();
        
        info!("Selected client '{}' from group '{}', opening QUIC stream...", client_id, client_group);
        

        let (mut send, mut recv) = match client_conn.open_bi().await {
            Ok(streams) => {
                info!("Successfully opened QUIC bidirectional stream to client '{}'", client_id);
                streams
            }
            Err(e) => {
                error!("Failed to open QUIC stream to client '{}': {}", client_id, e);
                
                if !probe_client(&self.state, &client_id).await {
                    return Err(anyhow::anyhow!("Client {} is dead (probe failed)", client_id));
                }
                
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
        

        let request_id = Uuid::new_v4().to_string();
        let session_id = Frame::session_id_from_uuid(&request_id);
        
        let method = req.method.unwrap_or("GET").to_string();
        let path = req.path.unwrap_or("/").to_string();
        
        let routing_info = RoutingInfo {
            r#type: "http".to_string(),
            host: host_with_port.clone(),
            method,
            path,
        };
        
        let routing_frame = create_routing_frame(session_id, &routing_info);
        write_frame(&mut send, &routing_frame).await?;
        info!("[{}] Sent routing frame to client {}: session_id={}, host={}", 
            request_id, client_id, session_id, host_with_port);
        

        const MAX_FRAME_SIZE: usize = 64 * 1024;
        let request_bytes = complete_request.to_vec();
        let mut offset = 0;
        
        while offset < request_bytes.len() {
            let chunk_size = std::cmp::min(MAX_FRAME_SIZE, request_bytes.len() - offset);
            let chunk = request_bytes[offset..offset + chunk_size].to_vec();
            let is_last = offset + chunk_size >= request_bytes.len();
            
            let data_frame = TunnelFrame::new(
                session_id,
                ProtocolType::Http11,
                is_last,
                chunk,
            );
            
            write_frame(&mut send, &data_frame).await?;
            offset += chunk_size;
        }
        
        info!("[{}] Sent {} frames (total {} bytes) to client", 
            request_id, (request_bytes.len() + MAX_FRAME_SIZE - 1) / MAX_FRAME_SIZE, request_bytes.len());
        
        // Finish the send stream to signal that request is complete
        // This allows the client to know when to close the TCP write side
        // The recv stream remains open to receive the response
        send.finish()?;
        info!("[{}] Finished send stream, waiting for response", request_id);

        let mut response_buffer = BytesMut::new();
        let mut session_complete = false;
        const RESPONSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
        
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
        
        stream.write_all(&response_buffer).await?;
        stream.flush().await?;
        info!("[{}] Sent response to external client ({} bytes) in {:?}", 
            request_id, response_buffer.len(), request_start.elapsed());
        
        Ok(())
    }
}

async fn read_complete_http_request(socket: &mut TcpStream) -> Result<BytesMut> {
    
    let mut buffer = BytesMut::new();
    
    let mut header_end = false;
    while !header_end {
        let mut buf = vec![0u8; 4096];
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            anyhow::bail!("Connection closed before headers");
        }
        
        buffer.extend_from_slice(&buf[..n]);
        
        if buffer.len() >= 4 {
            for i in 0..=buffer.len().saturating_sub(4) {
                if &buffer[i..i+4] == b"\r\n\r\n" {
                    header_end = true;
                    break;
                }
            }
        }
        
        if buffer.len() > 8192 {
            anyhow::bail!("HTTP headers too large");
        }
    }
    
    let header_end_pos = buffer.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(buffer.len());
    
    let header_bytes = &buffer[..header_end_pos];
    
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    
    let parse_result = req.parse(header_bytes)?;
    match parse_result {
        httparse::Status::Complete(_) => {}
        httparse::Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    let body_length = determine_body_length(&req.headers)?;
    
    if let Some(len) = body_length {
        let remaining_in_buffer = buffer.len() - header_end_pos;
        if remaining_in_buffer < len {
            let needed = len - remaining_in_buffer;
            let mut body_buf = vec![0u8; needed];
            socket.read_exact(&mut body_buf).await?;
            buffer.extend_from_slice(&body_buf);
        }
    } else {
        read_chunked_body(socket, &mut buffer).await?;
    }
    
    Ok(buffer)
}

fn determine_body_length(headers: &[httparse::Header]) -> Result<Option<usize>> {
    for header in headers {
        if header.name.eq_ignore_ascii_case("transfer-encoding") {
            let value = std::str::from_utf8(header.value)?;
            if value.eq_ignore_ascii_case("chunked") {
                return Ok(None);
            }
        }
    }
    
    for header in headers {
        if header.name.eq_ignore_ascii_case("content-length") {
            let value = std::str::from_utf8(header.value)?;
            let len = value.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("Invalid Content-Length: {} ({})", value, e))?;
            return Ok(Some(len));
        }
    }
    
    Ok(Some(0))
}

async fn read_chunked_body(socket: &mut TcpStream, buffer: &mut BytesMut) -> Result<()> {
    use tokio::io::AsyncReadExt;
    
    loop {
        let mut chunk_size_line = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            socket.read_exact(&mut byte).await?;
            chunk_size_line.push(byte[0]);
            
            if chunk_size_line.len() >= 2 && chunk_size_line[chunk_size_line.len()-2..] == [b'\r', b'\n'] {
                break;
            }
        }
        
        let size_str = std::str::from_utf8(&chunk_size_line[..chunk_size_line.len()-2])?;
        let chunk_size = usize::from_str_radix(size_str.trim(), 16)
            .map_err(|e| anyhow::anyhow!("Invalid chunk size: {} ({})", size_str, e))?;
        
        if chunk_size == 0 {
            let mut trailer = [0u8; 2];
            socket.read_exact(&mut trailer).await?;
            if trailer != [b'\r', b'\n'] {
                warn!("Invalid chunked encoding trailer");
            }
            break;
        }
        
        let mut chunk_data = vec![0u8; chunk_size];
        socket.read_exact(&mut chunk_data).await?;
        buffer.extend_from_slice(&chunk_data);
        
        let mut trailer = [0u8; 2];
        socket.read_exact(&mut trailer).await?;
        if trailer != [b'\r', b'\n'] {
            warn!("Invalid chunked encoding trailer after chunk data");
        }
    }
    
    Ok(())
}


pub struct WssIngressHandler {
    state: Arc<ServerState>,
}

impl WssIngressHandler {
    pub fn new(state: Arc<ServerState>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl tunnel_lib::listener::ConnectionHandler for WssIngressHandler {
    async fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let request_start = std::time::Instant::now();
        let request_id = Uuid::new_v4().to_string();

        // Read WebSocket HTTP upgrade request
        let mut header_buffer = BytesMut::new();
        let mut header_complete = false;
        let mut header_end_pos = 0;

        while !header_complete && header_buffer.len() < 8192 {
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                return Err(anyhow::anyhow!("Connection closed before headers"));
            }
            header_buffer.extend_from_slice(&buf[..n]);

            for i in 0..=header_buffer.len().saturating_sub(4) {
                if &header_buffer[i..i+4] == b"\r\n\r\n" {
                    header_complete = true;
                    header_end_pos = i + 4;
                    break;
                }
            }
        }

        if !header_complete {
            return Err(anyhow::anyhow!("WebSocket headers too large or incomplete"));
        }

        // Parse HTTP headers
        let header_bytes = &header_buffer[..header_end_pos];
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = Request::new(&mut headers);
        
        let parse_result = req.parse(header_bytes)?;
        match parse_result {
            httparse::Status::Complete(_) => {
                debug!("Parsed WebSocket HTTP request: {} {}", req.method.unwrap_or(""), req.path.unwrap_or(""));
            }
            httparse::Status::Partial => {
                anyhow::bail!("Incomplete WebSocket HTTP headers");
            }
        }
        
        // Verify this is a WebSocket upgrade request
        let is_websocket = req.headers.iter().any(|h| {
            h.name.eq_ignore_ascii_case("upgrade") && 
            std::str::from_utf8(h.value).map(|v| v.eq_ignore_ascii_case("websocket")).unwrap_or(false)
        }) && req.headers.iter().any(|h| {
            h.name.eq_ignore_ascii_case("connection") && 
            std::str::from_utf8(h.value).map(|v| v.to_lowercase().contains("upgrade")).unwrap_or(false)
        });
        
        if !is_websocket {
            // Check if this might be a TLS handshake (WSS attempt)
            if header_buffer.len() > 0 && header_buffer[0] == 0x16 {
                return Err(anyhow::anyhow!("TLS handshake detected. This server only supports WS (non-encrypted WebSocket). Use ws:// instead of wss://"));
            }
            return Err(anyhow::anyhow!("Not a WebSocket upgrade request. Missing Upgrade: websocket or Connection: Upgrade headers"));
        }

        let host_with_port = req.headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("host"))
            .and_then(|h| std::str::from_utf8(h.value).ok())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Missing Host header"))?;

        let host = host_with_port.split(':').next().unwrap_or(&host_with_port).trim();

        // Match ingress rule using RuleMatcher for O(1) lookup
        let matched_rule = {
            let matcher = self.state.rule_matcher.read().await;
            matcher.match_ingress_rule("wss", &host)
        };
        
        let client_group = match matched_rule {
            Some(rule) => {
                debug!("Matched WebSocket ingress rule: {} -> group {}", host, rule.action_client_group);
                rule.action_client_group.clone()
            }
            None => {
                warn!("No matching WebSocket ingress rule for host: {}", host);
                return Err(anyhow::anyhow!("No matching ingress rule for host: {}", host));
            }
        };

        // Select client
        let client_id = select_client_from_group(&self.state, &client_group)?;
        let client_conn = self.state.clients.get(&client_id)
            .ok_or_else(|| anyhow::anyhow!("Client {} not found", client_id))?
            .clone();

        // Open QUIC stream
        let (mut send, mut recv) = match client_conn.open_bi().await {
            Ok(streams) => streams,
            Err(e) => {
                error!("Failed to open QUIC stream to client '{}': {}", client_id, e);
                if !probe_client(&self.state, &client_id).await {
                    return Err(anyhow::anyhow!("Client {} is dead", client_id));
                }
                client_conn.open_bi().await?
            }
        };

        let session_id = Frame::session_id_from_uuid(&request_id);
        
        let routing_info = RoutingInfo {
            r#type: "wss".to_string(),
            host: host_with_port.clone(),
            method: req.method.unwrap_or("GET").to_string(),
            path: req.path.unwrap_or("/").to_string(),
        };

        let routing_frame = create_routing_frame(session_id, &routing_info);
        write_frame(&mut send, &routing_frame).await?;
        info!("[{}] Sent WebSocket routing frame to client {}: session_id={}, host={}", 
            request_id, client_id, session_id, host_with_port);

        // Forward HTTP upgrade request (direct forwarding, no encapsulation)
        send.write_all(&header_buffer[..header_end_pos]).await?;
        info!("[{}] Sent WebSocket upgrade request to client (direct forwarding)", request_id);

        // Split TCP stream for bidirectional forwarding (direct forwarding, no encapsulation)
        let (mut stream_recv, mut stream_send) = tokio::io::split(stream);
        
        // Spawn bidirectional forwarding tasks
        let mut send_clone = send;
        let request_id_clone1 = request_id.clone();
        let request_id_clone2 = request_id.clone();
        const FORWARD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

        // Task 1: Forward from TCP (WebSocket) to QUIC (tunnel) - direct forwarding
        let tcp_to_quic_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer
            
            loop {
                match tokio::time::timeout(FORWARD_TIMEOUT, stream_recv.read(&mut buf)).await {
                    Ok(Ok(n)) => {
                        if n == 0 {
                            debug!("[{}] WebSocket stream ended", request_id_clone1);
                            break;
                        }
                        
                        if let Err(e) = send_clone.write_all(&buf[..n]).await {
                            error!("[{}] Error writing to QUIC: {}", request_id_clone1, e);
                            break;
                        }
                    }
                    Ok(Err(e)) => {
                        error!("[{}] Error reading from WebSocket: {}", request_id_clone1, e);
                        break;
                    }
                    Err(_) => {
                        warn!("[{}] WebSocket to QUIC forward timeout", request_id_clone1);
                        break;
                    }
                }
            }
            
            // Finish QUIC send stream
            let _ = send_clone.finish();
        });

        // Task 2: Forward from QUIC (tunnel) to TCP (WebSocket) - direct forwarding
        let quic_to_tcp_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer
            
            loop {
                match tokio::time::timeout(FORWARD_TIMEOUT, recv.read(&mut buf)).await {
                    Ok(Ok(Some(n))) => {
                        if n == 0 {
                            debug!("[{}] QUIC stream ended", request_id_clone2);
                            break;
                        }
                        
                        if let Err(e) = stream_send.write_all(&buf[..n]).await {
                            error!("[{}] Error writing to WebSocket: {}", request_id_clone2, e);
                            break;
                        }
                        if let Err(e) = stream_send.flush().await {
                            error!("[{}] Error flushing WebSocket: {}", request_id_clone2, e);
                            break;
                        }
                    }
                    Ok(Ok(None)) => {
                        debug!("[{}] QUIC stream closed", request_id_clone2);
                        break;
                    }
                    Ok(Err(e)) => {
                        error!("[{}] Error reading from QUIC: {}", request_id_clone2, e);
                        break;
                    }
                    Err(_) => {
                        warn!("[{}] QUIC to WebSocket forward timeout", request_id_clone2);
                        break;
                    }
                }
            }
            
            // Close TCP send side
            let _ = stream_send.shutdown().await;
        });

        // Wait for either task to complete
        tokio::select! {
            result = tcp_to_quic_task => {
                if let Err(e) = result {
                    error!("[{}] TCP to QUIC task error: {}", request_id, e);
                }
            }
            result = quic_to_tcp_task => {
                if let Err(e) = result {
                    error!("[{}] QUIC to TCP task error: {}", request_id, e);
                }
            }
        }
        
        info!("[{}] WebSocket bidirectional forwarding completed (direct forwarding)", request_id);
        Ok(())
    }
}
