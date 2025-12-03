use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tracing::{info, error, debug, warn};
use quinn::Connection;
use uuid::Uuid;
use bytes::BytesMut;
use crate::types::ClientState;
use crate::http_handler::read_complete_http_request;
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, write_frame, RoutingInfo, create_routing_frame};
use tunnel_lib::frame::TunnelFrame as Frame;

async fn handle_forward_connection(
    mut socket: TcpStream,
    _peer_addr: SocketAddr,
    connection: Arc<Connection>,
    _state: Arc<ClientState>,
) -> Result<()> {
    let request_id = Uuid::new_v4().to_string();
    let stream_start = std::time::Instant::now();
    
    // 1. Read complete HTTP request from local client
    let complete_request = read_complete_http_request(&mut socket).await?;
    info!("[{}] Read complete HTTP request ({} bytes)", request_id, complete_request.len());
    
    // 2. Parse HTTP headers to extract Host
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    
    let parse_result = req.parse(&complete_request)?;
    match parse_result {
        httparse::Status::Complete(_) => {
            debug!("[{}] Parsed HTTP request: {} {}", 
                request_id, req.method.unwrap_or(""), req.path.unwrap_or(""));
        }
        httparse::Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Extract Host header
    let host = req.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case("host"))
        .and_then(|h| std::str::from_utf8(h.value).ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "localhost".to_string());
    
    debug!("[{}] Request Host: {}", request_id, host);
    
    // 3. Open QUIC bidirectional stream to server
    let (mut send, mut recv) = connection.open_bi().await?;
    
    // 4. Create Session ID and routing frame
    let session_id = Frame::session_id_from_uuid(&request_id);
    
    // Extract method and path from HTTP request
    let method = req.method.unwrap_or("GET").to_string();
    let path = req.path.unwrap_or("/").to_string();
    
    let routing_info = RoutingInfo {
        r#type: "http".to_string(),
        host: host.clone(),
        method,
        path,
    };
    
    // Send routing frame (first frame)
    let routing_frame = create_routing_frame(session_id, &routing_info);
    write_frame(&mut send, &routing_frame).await?;
    info!("[{}] Sent routing frame to server: session_id={}, host={}", 
        request_id, session_id, host);
    
    // 5. Split request into frames and send to server
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
    
    info!("[{}] Sent {} request frames (total {} bytes) to server", 
        request_id, (request_bytes.len() + MAX_FRAME_SIZE - 1) / MAX_FRAME_SIZE, request_bytes.len());
    
    // 6. Receive response frames and reassemble (with timeout)
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
    
    // 7. Send response to local client
    socket.write_all(&response_buffer).await?;
    socket.flush().await?;
    info!("[{}] Sent response to local client ({} bytes) in {:?}", 
        request_id, response_buffer.len(), stream_start.elapsed());
    
    Ok(())
}

/// Start HTTP listener (uses frame protocol)
pub async fn start_http_listener(
    port: u16,
    state: Arc<ClientState>,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP listener listening on 0.0.0.0:{}", port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        debug!("Accepted HTTP connection from {}", peer_addr);
        let state = state.clone();

        tokio::spawn(async move {
            // Get current active connection
            let connection = {
                let lock = state.quic_connection.read().await;
                lock.clone()
            };

            if let Some(conn) = connection {
                if let Err(e) = handle_forward_connection(socket, peer_addr, conn, state).await {
                    error!("HTTP connection handling error: {}", e);
                }
            } else {
                warn!("No active QUIC connection, dropping HTTP request");
            }
        });
    }
}

/// Start gRPC listener (placeholder - to be implemented)
pub async fn start_grpc_listener(
    _port: u16,
    _state: Arc<ClientState>,
) -> Result<()> {
    // TODO: Implement gRPC listener
    warn!("gRPC listener not yet implemented");
    Ok(())
}

/// Start WSS listener (placeholder - to be implemented)
pub async fn start_wss_listener(
    _port: u16,
    _state: Arc<ClientState>,
) -> Result<()> {
    // TODO: Implement WSS listener
    warn!("WSS listener not yet implemented");
    Ok(())
}
