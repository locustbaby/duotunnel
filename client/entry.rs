use anyhow::Result;
use bytes::BytesMut;
use quinn::Connection;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{
    detect_protocol_and_host, relay_quic_to_tcp, send_routing_info, RoutingInfo, TcpParams,
};
pub async fn start_entry_listener(
    conn: Connection,
    port: u16,
    cancel_token: CancellationToken,
    max_connections: u32,
    tcp_params: TcpParams,
    peek_buf_size: usize,
    open_stream_timeout: Duration,
) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let semaphore = Arc::new(Semaphore::new(max_connections as usize));
    let tcp_params = Arc::new(tcp_params);
    info!(
        addr = % addr, max_connections = % max_connections,
        "client entry listener started"
    );
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!(port = port, "entry listener cancelled");
                break Ok(());
            }
            result = listener.accept() => {
                let (stream, peer_addr) = result?;
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
                    if let Err(e) = handle_entry_connection(conn, stream, peek_buf_size, tcp_params, open_stream_timeout).await {
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
    tcp_params: Arc<TcpParams>,
    open_stream_timeout: Duration,
) -> Result<()> {
    let peer_addr = local_stream.peer_addr()?;
    tcp_params.apply(&local_stream)?;
    // Single allocation: peek into zeroed BytesMut, then freeze — avoids copy_from_slice.
    let mut buf = BytesMut::zeroed(peek_buf_size);
    let n = local_stream.peek(&mut buf).await?;
    let initial_bytes = buf.freeze().slice(..n);
    let (protocol, host) = detect_protocol_and_host(&initial_bytes);
    debug!(protocol = % protocol, host = ? host, "detected protocol from entry");
    let (mut send, recv) = tokio::time::timeout(open_stream_timeout, conn.open_bi())
        .await
        .map_err(|_| anyhow::anyhow!("open_bi timed out after {:?}", open_stream_timeout))??;
    let routing_info = RoutingInfo {
        proxy_name: "entry".to_string(),
        src_addr: peer_addr.ip().to_string(),
        src_port: peer_addr.port(),
        protocol: protocol.to_string(),
        host,
    };
    send_routing_info(&mut send, &routing_info).await?;
    let (sent, received) = relay_quic_to_tcp(recv, send, local_stream).await?;
    debug!(
        sent = sent, received = received, protocol = % protocol, "entry relay completed"
    );
    Ok(())
}
