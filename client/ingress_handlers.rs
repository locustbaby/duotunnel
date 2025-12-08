use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{warn, info, error, debug};
use uuid::Uuid;
use bytes::BytesMut;
use crate::types::ClientState;
use crate::forwarder::http::handle_http_forward_connection;
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, write_frame, RoutingInfo, create_routing_frame};
use tunnel_lib::frame::TunnelFrame as Frame;

pub struct HttpIngressHandler {
    state: Arc<ClientState>,
}

impl HttpIngressHandler {
    pub fn new(state: Arc<ClientState>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl tunnel_lib::listener::ConnectionHandler for HttpIngressHandler {
    async fn handle_connection(&self, socket: TcpStream) -> Result<()> {
        let connection = {
            let lock = self.state.quic_connection.read().await;
            lock.clone()
        };

        if let Some(conn) = connection {
            handle_http_forward_connection(socket, conn).await
        } else {
            warn!("No active QUIC connection, dropping HTTP request");
            Ok(())
        }
    }
}

pub struct GrpcIngressHandler {
    state: Arc<ClientState>,
}

impl GrpcIngressHandler {
    pub fn new(state: Arc<ClientState>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl tunnel_lib::listener::ConnectionHandler for GrpcIngressHandler {
    async fn handle_connection(&self, mut socket: TcpStream) -> Result<()> {
        let request_id = Uuid::new_v4().to_string();
        info!("[{}] New gRPC ingress connection", request_id);

        let connection = {
            let lock = self.state.quic_connection.read().await;
            lock.clone()
        };

        let conn = match connection {
            Some(conn) => conn,
            None => {
                warn!("[{}] No active QUIC connection, dropping gRPC request", request_id);
                return Ok(());
            }
        };

        // Read gRPC HTTP/2 headers first (for routing)
        let mut header_buffer = BytesMut::new();
        let mut header_complete = false;
        let mut header_end_pos = 0;

        // Read HTTP/2 headers (up to 8KB)
        while !header_complete && header_buffer.len() < 8192 {
            let mut buf = vec![0u8; 4096];
            let n = socket.read(&mut buf).await?;
            if n == 0 {
                return Err(anyhow::anyhow!("Connection closed before headers"));
            }
            header_buffer.extend_from_slice(&buf[..n]);

            // Look for end of HTTP headers (\r\n\r\n)
            for i in 0..=header_buffer.len().saturating_sub(4) {
                if &header_buffer[i..i+4] == b"\r\n\r\n" {
                    header_complete = true;
                    header_end_pos = i + 4;
                    break;
                }
            }
        }

        if !header_complete {
            return Err(anyhow::anyhow!("gRPC headers too large or incomplete"));
        }

        // Parse HTTP/2 headers to extract host
        let header_bytes = &header_buffer[..header_end_pos];
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        
        let parse_result = req.parse(header_bytes)?;
        if !matches!(parse_result, httparse::Status::Complete(_)) {
            return Err(anyhow::anyhow!("Incomplete gRPC HTTP/2 headers"));
        }

        let host = req.headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("host"))
            .and_then(|h| std::str::from_utf8(h.value).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "localhost".to_string());

        debug!("[{}] gRPC request Host: {}", request_id, host);

        // Open QUIC bidirectional stream
        let (mut send, mut recv) = conn.open_bi().await?;
        let session_id = Frame::session_id_from_uuid(&request_id);

        // Create routing frame
        let routing_info = RoutingInfo {
            r#type: "grpc".to_string(),
            host: host.clone(),
            method: req.method.unwrap_or("POST").to_string(),
            path: req.path.unwrap_or("/").to_string(),
        };

        let routing_frame = create_routing_frame(session_id, &routing_info);
        write_frame(&mut send, &routing_frame).await?;
        info!("[{}] Sent gRPC routing frame: session_id={}, host={}", request_id, session_id, host);

        // Forward HTTP/2 headers
        const MAX_FRAME_SIZE: usize = 64 * 1024;
        let header_vec = header_buffer[..header_end_pos].to_vec();
        let header_frame = TunnelFrame::new(
            session_id,
            ProtocolType::Grpc,
            false, // Not end of stream yet
            header_vec,
        );
        write_frame(&mut send, &header_frame).await?;

        // Forward remaining data (gRPC messages)
        let mut remaining_data = header_buffer[header_end_pos..].to_vec();
        
        // Read and forward gRPC messages
        loop {
            let mut buf = vec![0u8; 4096];
            match socket.read(&mut buf).await {
                Ok(0) => {
                    break;
                }
                Ok(n) => {
                    remaining_data.extend_from_slice(&buf[..n]);
                    
                    // Send chunks if buffer is large enough
                    while remaining_data.len() >= MAX_FRAME_SIZE {
                        let chunk = remaining_data[..MAX_FRAME_SIZE].to_vec();
                        remaining_data = remaining_data[MAX_FRAME_SIZE..].to_vec();
                        
                        let data_frame = TunnelFrame::new(
                            session_id,
                            ProtocolType::Grpc,
                            false,
                            chunk,
                        );
                        write_frame(&mut send, &data_frame).await?;
                    }
                }
                Err(e) => {
                    error!("[{}] Error reading from gRPC socket: {}", request_id, e);
                    return Err(e.into());
                }
            }
        }

        // Send any remaining data and end frame
        if !remaining_data.is_empty() {
            let final_frame = TunnelFrame::new(
                session_id,
                ProtocolType::Grpc,
                true,
                remaining_data,
            );
            write_frame(&mut send, &final_frame).await?;
        } else {
            // Send empty end frame
            let end_frame = TunnelFrame::new(
                session_id,
                ProtocolType::Grpc,
                true,
                Vec::new(),
            );
            write_frame(&mut send, &end_frame).await?;
        }

        // Read response from tunnel and forward to client
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
                    
                    // Forward response chunks to client as they arrive
                    if !frame.payload.is_empty() {
                        socket.write_all(&frame.payload).await?;
                    }
                    
                    if session_complete {
                        info!("[{}] Received complete gRPC response ({} bytes)", request_id, response_buffer.len());
                        socket.flush().await?;
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

        send.finish()?;
        info!("[{}] gRPC ingress connection completed", request_id);
        Ok(())
    }
}

pub struct WssIngressHandler {
    state: Arc<ClientState>,
}

impl WssIngressHandler {
    pub fn new(state: Arc<ClientState>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl tunnel_lib::listener::ConnectionHandler for WssIngressHandler {
    async fn handle_connection(&self, mut socket: TcpStream) -> Result<()> {
        let request_id = Uuid::new_v4().to_string();
        info!("[{}] New WebSocket ingress connection", request_id);

        let connection = {
            let lock = self.state.quic_connection.read().await;
            lock.clone()
        };

        let conn = match connection {
            Some(conn) => conn,
            None => {
                warn!("[{}] No active QUIC connection, dropping WebSocket request", request_id);
                return Ok(());
            }
        };

        // Read WebSocket HTTP upgrade request
        let mut header_buffer = BytesMut::new();
        let mut header_complete = false;
        let mut header_end_pos = 0;

        while !header_complete && header_buffer.len() < 8192 {
            let mut buf = vec![0u8; 4096];
            let n = socket.read(&mut buf).await?;
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

        // Parse HTTP headers to extract host
        let header_bytes = &header_buffer[..header_end_pos];
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        
        let parse_result = req.parse(header_bytes)?;
        if !matches!(parse_result, httparse::Status::Complete(_)) {
            return Err(anyhow::anyhow!("Incomplete WebSocket HTTP headers"));
        }

        let host = req.headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("host"))
            .and_then(|h| std::str::from_utf8(h.value).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "localhost".to_string());

        debug!("[{}] WebSocket request Host: {}", request_id, host);

        // Open QUIC bidirectional stream
        let (mut send, mut recv) = conn.open_bi().await?;
        let session_id = Frame::session_id_from_uuid(&request_id);

        // Create routing frame
        let routing_info = RoutingInfo {
            r#type: "wss".to_string(),
            host: host.clone(),
            method: req.method.unwrap_or("GET").to_string(),
            path: req.path.unwrap_or("/").to_string(),
        };

        let routing_frame = create_routing_frame(session_id, &routing_info);
        write_frame(&mut send, &routing_frame).await?;
        info!("[{}] Sent WebSocket routing frame: session_id={}, host={}", request_id, session_id, host);

        // Forward HTTP upgrade request
        let header_vec = header_buffer[..header_end_pos].to_vec();
        let header_frame = TunnelFrame::new(
            session_id,
            ProtocolType::WssFrame,
            false,
            header_vec,
        );
        write_frame(&mut send, &header_frame).await?;

        // Split TCP stream for bidirectional forwarding
        let (mut socket_recv, mut socket_send) = tokio::io::split(socket);
        
        // Spawn task to forward WebSocket frames from client to tunnel
        let mut send_clone = send;
        let session_id_clone = session_id;
        let request_id_clone = request_id.clone();

        let forward_task = tokio::spawn(async move {
            let mut buffer = BytesMut::new();
            loop {
                let mut buf = vec![0u8; 4096];
                match socket_recv.read(&mut buf).await {
                    Ok(0) => {
                        // Connection closed
                        let end_frame = TunnelFrame::new(
                            session_id_clone,
                            ProtocolType::WssFrame,
                            true,
                            Vec::new(),
                        );
                        if let Err(e) = write_frame(&mut send_clone, &end_frame).await {
                            error!("[{}] Error sending WebSocket end frame: {}", request_id_clone, e);
                        }
                        break;
                    }
                    Ok(n) => {
                        buffer.extend_from_slice(&buf[..n]);
                        
                        // Send WebSocket frames (chunked if needed)
                        const MAX_FRAME_SIZE: usize = 64 * 1024;
                        while buffer.len() >= MAX_FRAME_SIZE {
                            let chunk = buffer[..MAX_FRAME_SIZE].to_vec();
                            buffer = buffer[MAX_FRAME_SIZE..].into();
                            
                            let data_frame = TunnelFrame::new(
                                session_id_clone,
                                ProtocolType::WssFrame,
                                false,
                                chunk,
                            );
                            if let Err(e) = write_frame(&mut send_clone, &data_frame).await {
                                error!("[{}] Error sending WebSocket frame: {}", request_id_clone, e);
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!("[{}] Error reading from WebSocket: {}", request_id_clone, e);
                        break;
                    }
                }
            }
            
            // Send remaining data
            if !buffer.is_empty() {
                let final_frame = TunnelFrame::new(
                    session_id_clone,
                    ProtocolType::WssFrame,
                    true,
                    buffer.to_vec(),
                );
                if let Err(e) = write_frame(&mut send_clone, &final_frame).await {
                    error!("[{}] Error sending final WebSocket frame: {}", request_id_clone, e);
                }
            }
        });

        // Forward WebSocket frames from tunnel to client
        let mut recv_clone = recv;
        let request_id_clone2 = request_id.clone();

        let receive_task = tokio::spawn(async move {
            let mut session_complete = false;
            const RESPONSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300); // 5 min for WebSocket

            while !session_complete {
                match tunnel_lib::frame::read_frame_with_timeout(&mut recv_clone, Some(RESPONSE_TIMEOUT)).await {
                    Ok(frame) => {
                        if frame.session_id != session_id {
                            warn!("[{}] Received frame with mismatched session_id: {} (expected {})", 
                                request_id_clone2, frame.session_id, session_id);
                            continue;
                        }
                        
                        if !frame.payload.is_empty() {
                            if let Err(e) = socket_send.write_all(&frame.payload).await {
                                error!("[{}] Error writing to WebSocket: {}", request_id_clone2, e);
                                break;
                            }
                            if let Err(e) = socket_send.flush().await {
                                error!("[{}] Error flushing WebSocket: {}", request_id_clone2, e);
                                break;
                            }
                        }
                        
                        session_complete = frame.end_of_stream;
                        if session_complete {
                            info!("[{}] WebSocket session completed", request_id_clone2);
                        }
                    }
                    Err(e) => {
                        if e.to_string().contains("timeout") {
                            warn!("[{}] WebSocket frame read timeout", request_id_clone2);
                        } else {
                            error!("[{}] Error reading WebSocket frame: {}", request_id_clone2, e);
                        }
                        break;
                    }
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            result = forward_task => {
                if let Err(e) = result {
                    error!("[{}] Forward task error: {}", request_id, e);
                }
            }
            result = receive_task => {
                if let Err(e) = result {
                    error!("[{}] Receive task error: {}", request_id, e);
                }
            }
        }

        info!("[{}] WebSocket ingress connection completed", request_id);
        Ok(())
    }
}
