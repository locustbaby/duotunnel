use anyhow::Result;
use bytes::BytesMut;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{watch, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{
    build_reuseport_tcp_listener, detect_protocol_and_host, relay_quic_to_tcp, send_routing_info,
    RoutingInfo, TcpParams,
};

/// Per-thread entry listener.
///
/// Each OS thread calls this with SO_REUSEPORT — the kernel distributes incoming
/// TCP connections across threads without any cross-thread coordination.
///
/// `conn_rx` is a watch channel written by `run_client` in the **same**
/// current_thread runtime. The listener reads the current connection with a
/// simple borrow — no lock, no cross-runtime call. `open_bi()` is therefore
/// a same-thread quinn call with zero cross-runtime overhead.
pub async fn start_entry_listener_local(
    conn_rx: watch::Receiver<Option<quinn::Connection>>,
    port: u16,
    cancel_token: CancellationToken,
    max_connections: u32,
    tcp_params: TcpParams,
    peek_buf_size: usize,
    open_stream_timeout: Duration,
) -> Result<()> {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let listener = build_reuseport_tcp_listener(addr)?;
    let semaphore = Arc::new(Semaphore::new(max_connections as usize));
    let tcp_params = Arc::new(tcp_params);
    info!(port = port, "per-thread entry listener started");
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
                // Borrow the current connection — same runtime, no cross-thread call.
                let conn = conn_rx.borrow().clone();
                let Some(conn) = conn else {
                    warn!("no live QUIC connection yet, dropping entry connection");
                    continue;
                };
                let tcp_params = tcp_params.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(e) = handle_entry_connection(
                        conn,
                        stream,
                        peek_buf_size,
                        tcp_params,
                        open_stream_timeout,
                    )
                    .await
                    {
                        debug!(error = %e, "entry connection error");
                    }
                });
            }
        }
    }
}

async fn handle_entry_connection(
    conn: quinn::Connection,
    local_stream: TcpStream,
    peek_buf_size: usize,
    tcp_params: Arc<TcpParams>,
    open_stream_timeout: Duration,
) -> Result<()> {
    let peer_addr = local_stream.peer_addr()?;
    tcp_params.apply(&local_stream)?;
    let mut buf = BytesMut::zeroed(peek_buf_size);
    let n = local_stream.peek(&mut buf).await?;
    let initial_bytes = buf.freeze().slice(..n);
    let (protocol, host) = detect_protocol_and_host(&initial_bytes);
    debug!(protocol = %protocol, host = ?host, "detected protocol from entry");

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
        sent = sent,
        received = received,
        protocol = %protocol,
        "entry relay completed"
    );
    Ok(())
}
