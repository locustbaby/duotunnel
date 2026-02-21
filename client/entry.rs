use anyhow::Result;
use quinn::Connection;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, debug};
use tunnel_lib::{send_routing_info, RoutingInfo, relay_bidirectional, detect_protocol_and_host};
use bytes::Bytes;

pub async fn start_entry_listener(
    conn: Connection,
    port: u16,
    cancel_token: CancellationToken,
    max_connections: u32,
) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let semaphore = Arc::new(Semaphore::new(max_connections as usize));

    info!(addr = %addr, max_connections = %max_connections, "client entry listener started");

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!(port = port, "entry listener cancelled");
                break Ok(());
            }
            result = listener.accept() => {
                let (stream, peer_addr) = result?;
                stream.set_nodelay(true)?;

                let permit = match semaphore.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => {
                        warn!(port = port, "entry connection rejected: max connections reached");
                        continue;
                    }
                };

                debug!(peer_addr = %peer_addr, "new entry connection");
                let conn = conn.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(e) = handle_entry_connection(conn, stream).await {
                        debug!(error = %e, "entry connection error");
                    }
                });
            }
        }
    }
}

async fn handle_entry_connection(
    conn: Connection,
    mut local_stream: TcpStream,
) -> Result<()> {
    use tokio::io::AsyncReadExt;
    
    let peer_addr = local_stream.peer_addr()?;
    
    let mut buf = vec![0u8; 4096];
    let n = local_stream.peek(&mut buf).await?;
    let initial_bytes = Bytes::copy_from_slice(&buf[..n]);
    
    let (protocol, host) = detect_protocol_and_host(&initial_bytes);
    
    debug!(protocol = %protocol, host = ?host, "detected protocol from entry");
    
    let (mut send, recv) = conn.open_bi().await?;
    
    let routing_info = RoutingInfo {
        proxy_name: "entry".to_string(),
        src_addr: peer_addr.ip().to_string(),
        src_port: peer_addr.port(),
        protocol: protocol.to_string(),
        host,
    };
    
    send_routing_info(&mut send, &routing_info).await?;

    let (sent, received) = relay_bidirectional(recv, send, local_stream).await?;
    
    debug!(sent = sent, received = received, protocol = %protocol, "entry relay completed");

    Ok(())
}

