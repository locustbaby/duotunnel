use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, debug, warn};
use uuid::Uuid;
use bytes::BytesMut;
use httparse::Request;
use crate::types::ServerState;
use crate::client_mgr::select_client_from_group;
use tunnel_lib::protocol::write_data_stream_header;
use tunnel_lib::proto::tunnel::DataStreamHeader;

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

/// Handle HTTP request from external client
async fn handle_http_request(
    mut stream: TcpStream,
    peer_addr: SocketAddr,
    state: Arc<ServerState>,
) -> Result<()> {
    // 1. Read HTTP request headers
    let mut buffer = BytesMut::new();
    let mut header_end = false;
    
    while !header_end {
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await?;
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
    
    // Find the end of headers
    let header_end_pos = buffer.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(buffer.len());
    
    let header_bytes = buffer.split_to(header_end_pos);
    let remaining_body = buffer;
    
    // Parse HTTP headers
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    
    let parse_result = req.parse(&header_bytes)?;
    match parse_result {
        httparse::Status::Complete(_) => {
            debug!("Parsed HTTP request: {} {}", req.method.unwrap_or(""), req.path.unwrap_or(""));
        }
        httparse::Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Extract Host header
    let host = req.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case("host"))
        .and_then(|h| std::str::from_utf8(h.value).ok())
        .map(|s| s.to_string());
    
    let host = host.ok_or_else(|| anyhow::anyhow!("Missing Host header"))?;
    debug!("HTTP request Host: {}", host);
    
    // 2. Match ingress routing rule
    let matched_rule = state.ingress_rules.iter()
        .find(|r| r.match_host.eq_ignore_ascii_case(&host));
    
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
    
    // 3. Select a client from the group
    info!("Selecting client from group '{}' for host '{}'", client_group, host);
    let client_id = select_client_from_group(&state, &client_group)?;
    
    // 4. Get Connection from registry
    let client_conn = state.clients.get(&client_id)
        .ok_or_else(|| {
            error!("Client '{}' not found in clients registry", client_id);
            anyhow::anyhow!("Client {} not found", client_id)
        })?
        .clone();
    
    info!("Selected client '{}' from group '{}', opening QUIC stream...", client_id, client_group);
    
    // 5. Open QUIC bidirectional stream
    // 5. Open QUIC bidirectional stream
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
    
    // 6. Create and send DataStreamHeader
    let request_id = Uuid::new_v4().to_string();
    let header = DataStreamHeader {
        request_id: request_id.clone(),
        r#type: "http".to_string(),
        host: host.clone(),
        metadata: HashMap::new(),
    };
    
    let header_send_start = std::time::Instant::now();
    write_data_stream_header(&mut send, &header).await?;
    info!("[{}] Sent DataStreamHeader to client {}: request_id={}, host={}", 
        request_id, client_id, request_id, host);
    
    // 7. Send HTTP request headers and body
    info!("[{}] Sending HTTP headers ({} bytes) and body ({} bytes) to client...", 
        request_id, header_bytes.len(), remaining_body.len());
    send.write_all(&header_bytes).await?;
    if !remaining_body.is_empty() {
        send.write_all(&remaining_body).await?;
    }
    // Flush to ensure data is sent immediately
    send.flush().await?;
    info!("[{}] Sent HTTP headers and body to client in {:?}, starting bidirectional copy...", 
        request_id, header_send_start.elapsed());
    
    // 8. Bidirectional copy: External client <-> QUIC stream
    let copy_start = std::time::Instant::now();
    info!("[{}] Starting bidirectional copy: External client <-> QUIC stream", request_id);
    let (mut ri, mut wi) = stream.split();
    
    let request_id_clone = request_id.clone();
    let client_to_quic = async move {
        let bytes_copied = tokio::io::copy(&mut ri, &mut send).await?;
        info!("[{}] External client -> QUIC: copied {} bytes in {:?}", 
            request_id_clone, bytes_copied, copy_start.elapsed());
        send.finish().await?;
        info!("[{}] QUIC stream send side finished", request_id_clone);
        Ok::<_, anyhow::Error>(())
    };
    
    let request_id_clone2 = request_id.clone();
    let quic_to_client = async move {
        let bytes_copied = tokio::io::copy(&mut recv, &mut wi).await?;
        info!("[{}] QUIC -> External client: copied {} bytes in {:?}", 
            request_id_clone2, bytes_copied, copy_start.elapsed());
        Ok::<_, anyhow::Error>(())
    };
    
    tokio::try_join!(client_to_quic, quic_to_client)?;
    info!("[{}] Bidirectional copy completed in {:?}", request_id, copy_start.elapsed());
    
    Ok(())
}

