use crate::conn_pool::EntryConnPool;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{
    detect_protocol_and_host, relay_quic_to_tcp, send_routing_info, RoutingInfo, TcpParams,
};

thread_local! {
    static PEEK_BUF: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub async fn start_entry_listener(
    pool: Arc<EntryConnPool>,
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
                let pool = pool.clone();
                let tcp_params = tcp_params.clone();
                crate::spawn_task(async move {
                    let _permit = permit;
                    if let Err(e) = handle_entry_connection(pool, stream, peek_buf_size, tcp_params, open_stream_timeout).await {
                        debug!(error = %e, "entry connection error");
                    }
                });
            }
        }
    }
}

async fn handle_entry_connection(
    pool: Arc<EntryConnPool>,
    mut local_stream: TcpStream,
    peek_buf_size: usize,
    tcp_params: Arc<TcpParams>,
    open_stream_timeout: Duration,
) -> Result<()> {
    let peer_addr = local_stream.peer_addr()?;
    tcp_params.apply(&local_stream)?;

    let mut tl_buf = PEEK_BUF.with(|c| std::mem::take(&mut *c.borrow_mut()));
    if tl_buf.capacity() < peek_buf_size {
        tl_buf.reserve(peek_buf_size - tl_buf.capacity());
    }
    unsafe { tl_buf.set_len(peek_buf_size) };
    let n = local_stream.peek(&mut tl_buf).await?;
    let initial_bytes = bytes::Bytes::copy_from_slice(&tl_buf[..n]);
    tl_buf.truncate(0);
    PEEK_BUF.with(|c| *c.borrow_mut() = tl_buf);

    let (protocol, host) = detect_protocol_and_host(&initial_bytes);
    debug!(protocol = % protocol, host = ? host, "detected protocol from entry");

    let pool_size = pool.pool_size();
    let mut last_err = anyhow::anyhow!("no QUIC connections available in pool");
    for _ in 0..pool_size.max(1) {
        let conn = match pool.next_conn() {
            Some(c) => c,
            None => break,
        };
        match tokio::time::timeout(open_stream_timeout, conn.open_bi()).await {
            Ok(Ok((mut send, recv))) => {
                let routing_info = RoutingInfo {
                    proxy_name: "entry".to_string(),
                    src_addr: peer_addr.ip().to_string(),
                    src_port: peer_addr.port(),
                    protocol: protocol.to_string(),
                    host,
                };
                send_routing_info(&mut send, &routing_info).await?;
                if !initial_bytes.is_empty() {
                    send.write_all(&initial_bytes).await?;
                    let mut discard = vec![0u8; initial_bytes.len()];
                    local_stream.read_exact(&mut discard).await?;
                }
                let (sent, received) = relay_quic_to_tcp(recv, send, local_stream).await?;
                debug!(
                    sent = sent, received = received, protocol = % protocol,
                    "entry relay completed"
                );
                return Ok(());
            }
            Ok(Err(e)) => {
                warn!(error = %e, "open_bi failed, trying next connection");
                last_err = e.into();
            }
            Err(_) => {
                warn!(timeout_ms = open_stream_timeout.as_millis(), "open_bi timed out, trying next connection");
                last_err = anyhow::anyhow!("open_bi timed out after {:?}", open_stream_timeout);
            }
        }
    }
    Err(last_err)
}
