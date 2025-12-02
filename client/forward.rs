use anyhow::{Result, Context};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tracing::{info, error, debug, warn};
use quinn::Connection;
use uuid::Uuid;
use crate::types::ClientState;
use crate::http_handler;
use tunnel_lib::protocol::write_data_stream_header;

/// Run forward tunnel (Legacy - Listen Local -> Forward to Server)
pub async fn run_forward_tunnel(
    local_port: u16,
    connection: Arc<Connection>,
    state: Arc<ClientState>,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], local_port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on 0.0.0.0:{}", local_port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        debug!("Accepted connection from {}", peer_addr);
        let connection = connection.clone();
        let state = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_forward_connection(socket, peer_addr, connection, state).await {
                error!("Connection handling error: {}", e);
            }
        });
    }
}

async fn handle_forward_connection(
    mut socket: TcpStream,
    peer_addr: SocketAddr,
    connection: Arc<Connection>,
    state: Arc<ClientState>,
) -> Result<()> {
    // 1. Parse HTTP headers and modify if needed
    let rules: Vec<_> = state.rules.iter().map(|r| r.value().clone()).collect();
    let upstreams: Vec<_> = state.upstreams.iter().map(|u| u.value().clone()).collect();
    
    let (modified_request_bytes, original_host, modified_host, final_target_addr, is_target_ssl) =
        http_handler::parse_and_modify_http_request(
            &mut socket,
            &rules,
            &upstreams,
            peer_addr.ip().to_string(),
        )
        .await
        .context("Failed to parse HTTP request")?;
    
    debug!("Final target: {}, SSL: {}", final_target_addr, is_target_ssl);
    
    // 2. Open a new QUIC bidirectional stream
    let (mut send, mut recv) = connection.open_bi().await?;
    
    // 3. Create and send DataStreamHeader
    let request_id = Uuid::new_v4().to_string();
    let header = http_handler::create_data_stream_header(
        request_id.clone(),
        modified_host.unwrap_or_else(|| original_host.unwrap_or_default()),
    );
    
    write_data_stream_header(&mut send, &header).await?;
    debug!("Sent DataStreamHeader for request_id: {}", request_id);
    
    // 4. Send modified HTTP request bytes
    send.write_all(&modified_request_bytes).await?;
    
    // 5. Bidirectional copy: remaining data from socket <-> QUIC stream
    let (mut ri, mut wi) = socket.split();
    
    let client_to_server = async {
        tokio::io::copy(&mut ri, &mut send).await?;
        send.finish().await?;
        Ok::<_, anyhow::Error>(())
    };
    
    let server_to_client = async {
        tokio::io::copy(&mut recv, &mut wi).await?;
        Ok::<_, anyhow::Error>(())
    };
    
    tokio::try_join!(client_to_server, server_to_client)?;
    
    Ok(())
}

/// Start HTTP listener (uses existing HTTP handler)
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

