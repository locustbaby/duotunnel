use crate::engine::ClientService;
use crate::tunnel::conn_pool::EntryConnPool;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{
    detect_protocol_and_host, maybe_slow_path, open_bi_guarded, relay_quic_to_tcp,
    run_accept_worker, send_routing_info, AcceptedConn, ErrorKind, OverloadLimits, PeekBufPool,
    ProxyError, RoutingInfo, TcpParams,
};

const EMFILE_BACKOFF: Duration = Duration::from_millis(100);

static ENTRY_PEEK_POOL: OnceLock<PeekBufPool> = OnceLock::new();

fn with_conn_detail(conn_id: impl std::fmt::Display, err: ProxyError) -> ProxyError {
    let detail = format!("conn_id={conn_id}: {err}");
    err.with_detail(detail)
}

#[derive(Clone)]
pub struct EntryListenerConfig {
    pub port: u16,
    pub tcp_params: TcpParams,
    pub peek_buf_size: usize,
    pub open_stream_timeout: Duration,
    pub accept_workers: usize,
    pub overload: Arc<OverloadLimits>,
}

pub async fn start_entry_listener(
    pool: Arc<EntryConnPool>,
    cancel_token: CancellationToken,
    cfg: EntryListenerConfig,
) -> Result<()> {
    let peek_buf_size = cfg.peek_buf_size;
    let open_stream_timeout = cfg.open_stream_timeout;
    let accept_workers = cfg.accept_workers;
    let addr: SocketAddr = format!("127.0.0.1:{}", cfg.port).parse()?;
    let tcp_params = Arc::new(cfg.tcp_params);
    let overload = cfg.overload;
    info!(addr = %addr, accept_workers = %accept_workers, "client entry listener started");

    let mut handles = Vec::with_capacity(accept_workers);
    for _ in 0..accept_workers {
        let listener = Arc::new(tunnel_lib::build_reuseport_listener(addr)?);
        let pool = pool.clone();
        let tcp_params = tcp_params.clone();
        let cancel_token = cancel_token.clone();
        let overload = overload.clone();
        handles.push(crate::spawn_task(async move {
            run_accept_worker(
                listener,
                cancel_token,
                EMFILE_BACKOFF,
                "entry",
                move |accepted| {
                    let pool = pool.clone();
                    let tcp_params = tcp_params.clone();
                    let overload = overload.clone();
                    async move {
                        let AcceptedConn { stream, .. } = accepted;
                        if let Err(e) = handle_entry_connection(
                            pool,
                            stream,
                            peek_buf_size,
                            tcp_params,
                            open_stream_timeout,
                            &overload,
                        )
                        .await
                        {
                            debug!(error = %e, "entry connection error");
                        }
                    }
                },
            )
            .await;
        }));
    }

    futures_util::future::join_all(handles).await;
    Ok(())
}

async fn handle_entry_connection(
    pool: Arc<EntryConnPool>,
    mut local_stream: TcpStream,
    peek_buf_size: usize,
    tcp_params: Arc<TcpParams>,
    open_stream_timeout: Duration,
    overload: &OverloadLimits,
) -> Result<()> {
    let peer_addr = local_stream.peer_addr()?;
    tcp_params.apply(&local_stream)?;

    let peek_pool = ENTRY_PEEK_POOL.get_or_init(|| PeekBufPool::new(peek_buf_size));
    let mut buf = peek_pool.take();
    let n = local_stream.read(&mut buf).await?;
    let initial_bytes = bytes::Bytes::copy_from_slice(&buf[..n]);
    peek_pool.put(buf);

    let (protocol, host) = detect_protocol_and_host(&initial_bytes);
    debug!(protocol = ? protocol, host = ? host, "detected protocol from entry");

    let pool_size = pool.pool_size();
    let mut tried_conn_ids = Vec::with_capacity(pool_size.min(8));
    let mut last_err = anyhow::anyhow!("no QUIC connections available in pool");
    for _ in 0..pool_size.max(1) {
        let conn = match pool.next_conn_excluding(&tried_conn_ids) {
            Some(c) => c,
            None => break,
        };
        tried_conn_ids.push(conn.conn.stable_id());
        maybe_slow_path(&conn.inflight_table, conn.slot_id, overload).await;
        match open_bi_guarded(
            &conn.conn,
            &conn.inflight_table,
            conn.slot_id,
            open_stream_timeout,
            |_elapsed, _outcome| {},
        )
        .await
        {
            Ok(opened) => {
                let mut send = opened.send;
                let recv = opened.recv;
                let _inflight_guard = opened.inflight;
                let routing_info = RoutingInfo {
                    proxy_name: "entry".to_string(),
                    src_addr: peer_addr.ip().to_string(),
                    src_port: peer_addr.port(),
                    protocol,
                    host,
                };
                send_routing_info(&mut send, &routing_info).await?;
                if !initial_bytes.is_empty() {
                    send.write_all(&initial_bytes).await?;
                }
                let (sent, received) = relay_quic_to_tcp(recv, send, local_stream).await?;
                debug!(
                    sent = sent, received = received, protocol = ? protocol,
                    "entry relay completed"
                );
                return Ok(());
            }
            Err(e) => match e.kind {
                ErrorKind::QuicOpenTimedOut => {
                    warn!(error = %e, conn_id = conn.conn.stable_id(), "open_bi timed out, trying next connection");
                    last_err = with_conn_detail(conn.conn.stable_id(), e).into();
                }
                ErrorKind::QuicConnectionLost => {
                    warn!(error = %e, conn_id = conn.conn.stable_id(), "open_bi hit stale connection, evicting pool entry");
                    pool.remove(&conn.conn);
                    last_err = with_conn_detail(conn.conn.stable_id(), e).into();
                }
                ErrorKind::QuicConnectionFatal => {
                    warn!(error = %e, conn_id = conn.conn.stable_id(), "open_bi hit fatal connection error");
                    pool.remove(&conn.conn);
                    return Err(with_conn_detail(conn.conn.stable_id(), e).into());
                }
                _ => {
                    warn!(error = %e, conn_id = conn.conn.stable_id(), "open_bi failed, trying next connection");
                    last_err = with_conn_detail(conn.conn.stable_id(), e).into();
                }
            },
        }
    }
    Err(last_err)
}

pub struct EgressListenerService {
    pub entry_cfg: EntryListenerConfig,
    pub pool: Arc<EntryConnPool>,
}

#[async_trait::async_trait]
impl ClientService for EgressListenerService {
    fn name(&self) -> &'static str {
        "egress-tcp-listener"
    }
    async fn start(&self, shutdown: CancellationToken) -> anyhow::Result<()> {
        start_entry_listener(self.pool.clone(), shutdown, self.entry_cfg.clone())
            .await
            .map_err(|e| anyhow::anyhow!("entry listener failed: {}", e))
    }
}
