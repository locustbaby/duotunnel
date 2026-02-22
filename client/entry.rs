use anyhow::Result;
use quinn::Connection;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, debug};
use tunnel_lib::{send_routing_info, RoutingInfo, relay_quic_to_tcp, detect_protocol_and_host, TcpParams};
use bytes::Bytes;

pub async fn start_entry_listener(
    conn: Connection,
    port: u16,
    cancel_token: CancellationToken,
    max_connections: u32,
    tcp_params: TcpParams,
    peek_buf_size: usize,
) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let semaphore = Arc::new(Semaphore::new(max_connections as usize));
    let tcp_params = Arc::new(tcp_params);

    info!(addr = %addr, max_connections = %max_connections, "client entry listener started");

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!(port = port, "entry listener cancelled");
                break Ok(());
            }
            result = listener.accept() => {
                let (stream, peer_addr) = result?;
                tcp_params.apply(&stream)?;

                let permit = match semaphore.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => {
                        warn!(port = port, "entry connection rejected: max connections reached");
                        continue;
                    }
                };

                debug!(peer_addr = %peer_addr, "new entry connection");
                let conn = conn.clone();
                let tcp_params = tcp_params.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    let _ = tcp_params; // keep alive for the duration of the connection
                    if let Err(e) = handle_entry_connection(conn, stream, peek_buf_size).await {
                        debug!(error = %e, "entry connection error");
                    }
                });
            }
        }
    }
}

async fn handle_entry_connection(
    conn: Connection,
    local_stream: TcpStream,
    peek_buf_size: usize,
) -> Result<()> {
    let peer_addr = local_stream.peer_addr()?;

    let mut buf = vec![0u8; peek_buf_size];
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

    // Use relay_quic_to_tcp which calls TcpStream::into_split() — zero-cost split,
    // no Arc<Mutex> overhead compared to the generic relay_bidirectional.
    let (sent, received) = relay_quic_to_tcp(recv, send, local_stream).await?;

    debug!(sent = sent, received = received, protocol = %protocol, "entry relay completed");

    Ok(())
}
