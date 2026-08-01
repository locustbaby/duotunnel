use crate::health::ClientHealth;
use crate::runtime::engine::ClientService;
use crate::tunnel::conn_pool::EntryConnPool;
use anyhow::Result;
use duotunnel_lib::{
    default_client_detectors, relay_quic_to_tcp_with_buffer_size, run_accept_worker, AcceptedConn,
    ErrorKind, OpenStreamRequest, PeekBufPool, ProxyError, RoutingInfo, SniffPolicy, SniffRuntime,
    TcpParams,
};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

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
    pub sniff_timeout: Duration,
    pub relay_buf_size: usize,
}

struct EntryListenerHealthGuard {
    health: Arc<ClientHealth>,
}

impl Drop for EntryListenerHealthGuard {
    fn drop(&mut self) {
        self.health.set_entry_listener_up(false);
    }
}

pub async fn start_entry_listener(
    pool: Arc<EntryConnPool>,
    cancel_token: CancellationToken,
    cfg: EntryListenerConfig,
    health: Arc<ClientHealth>,
) -> Result<()> {
    let peek_buf_size = cfg.peek_buf_size;
    let open_stream_timeout = cfg.open_stream_timeout;
    let accept_workers = cfg.accept_workers;
    let addr: SocketAddr = format!("127.0.0.1:{}", cfg.port).parse()?;
    let tcp_params = Arc::new(cfg.tcp_params);
    let sniff_timeout = cfg.sniff_timeout;
    let relay_buf_size = cfg.relay_buf_size;
    anyhow::ensure!(
        accept_workers > 0,
        "entry accept_workers must be at least 1"
    );
    let mut listeners = Vec::with_capacity(accept_workers);
    for _ in 0..accept_workers {
        listeners.push(Arc::new(duotunnel_lib::build_reuseport_listener(addr)?));
    }
    health.set_entry_listener_up(true);
    let _health_guard = EntryListenerHealthGuard {
        health: health.clone(),
    };
    info!(addr = %addr, accept_workers = %accept_workers, "client entry listener started");
    let worker_cancel = cancel_token.child_token();

    let mut handles = Vec::with_capacity(accept_workers);
    for listener in listeners {
        let pool = pool.clone();
        let tcp_params = tcp_params.clone();
        let worker_cancel = worker_cancel.clone();
        handles.push(crate::runtime::spawn_task(async move {
            run_accept_worker(
                listener,
                worker_cancel,
                EMFILE_BACKOFF,
                "entry",
                move |accepted| {
                    let pool = pool.clone();
                    let tcp_params = tcp_params.clone();
                    async move {
                        let AcceptedConn { stream, .. } = accepted;
                        if let Err(e) = handle_entry_connection(
                            pool,
                            stream,
                            peek_buf_size,
                            tcp_params,
                            open_stream_timeout,
                            sniff_timeout,
                            relay_buf_size,
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

    let (first_result, _, remaining) = futures_util::future::select_all(handles).await;
    health.set_entry_listener_up(false);
    worker_cancel.cancel();
    for handle in &remaining {
        handle.abort();
    }
    let _ = futures_util::future::join_all(remaining).await;
    classify_accept_worker_exit(cancel_token.is_cancelled(), first_result.err())
}

fn classify_accept_worker_exit(
    shutdown_requested: bool,
    task_error: Option<tokio::task::JoinError>,
) -> Result<()> {
    match (shutdown_requested, task_error) {
        (true, None) => Ok(()),
        (_, Some(error)) => Err(anyhow::anyhow!("entry accept worker task failed: {error}")),
        (false, None) => Err(anyhow::anyhow!(
            "UnexpectedExit: entry accept worker stopped before shutdown"
        )),
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_entry_connection(
    pool: Arc<EntryConnPool>,
    mut local_stream: TcpStream,
    peek_buf_size: usize,
    tcp_params: Arc<TcpParams>,
    open_stream_timeout: Duration,
    sniff_timeout: Duration,
    relay_buf_size: usize,
) -> Result<()> {
    let peer_addr = local_stream.peer_addr()?;
    tcp_params.apply(&local_stream)?;

    let peek_pool = ENTRY_PEEK_POOL.get_or_init(|| PeekBufPool::new(peek_buf_size));
    let runtime = SniffRuntime::new(SniffPolicy::default(), default_client_detectors());
    let sniffed = match tokio::time::timeout(
        sniff_timeout,
        runtime.sniff(&mut local_stream, peek_pool),
    )
    .await
    {
        Ok(res) => res?,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "protocol sniffing timed out (Slowloris protection)"
            ));
        }
    };
    let (protocol, host, initial_bytes) = if sniffed.bytes_read > 0 {
        (
            sniffed.hint.protocol,
            sniffed.hint.sni.clone().or(sniffed.hint.authority.clone()),
            Some(sniffed.prefix.into_bytes()),
        )
    } else {
        (duotunnel_lib::proxy::core::Protocol::Unknown, None, None)
    };

    debug!(protocol = ? protocol, host = ? host, "detected protocol from entry");

    if let Some(ref host_raw) = host {
        if pool.is_rejected_host(host_raw) {
            warn!(target: "client::egress", "Target '{}' rejected locally by egress allowlist", host_raw);
            crate::metrics::egress_rejection("no_egress_route");

            if protocol == duotunnel_lib::proxy::core::Protocol::H1 {
                let body = "502 Bad Gateway - No Egress Route\n";
                let resp = format!(
                    "HTTP/1.1 502 Bad Gateway\r\n\
                     Connection: close\r\n\
                     Content-Type: text/plain\r\n\
                     Content-Length: {}\r\n\
                     X-DuoTunnel-Reject: no-egress-route\r\n\
                     \r\n\
                     {}",
                    body.len(),
                    body
                );
                let _ = local_stream.write_all(resp.as_bytes()).await;
                let _ = local_stream.shutdown().await;
            } else {
                let _ = local_stream.shutdown().await;
            }
            return Ok(());
        }
    }

    let pool_size = pool.pool_size();
    let preferred_shard = pool.shard_for_hash(&(host.clone(), "entry"));
    let mut tried_conn_ids = HashSet::with_capacity(pool_size.min(8));
    let mut last_err = anyhow::anyhow!("no QUIC connections available in pool");
    for _ in 0..pool_size.max(1) {
        let conn = match pool.next_conn_for_shard_excluding(preferred_shard, &tried_conn_ids) {
            Some(c) => c,
            None => break,
        };
        tried_conn_ids.insert(conn.handle.stable_id());
        let routing_info = RoutingInfo {
            proxy_name: "entry".into(),
            src_addr: peer_addr.ip(),
            src_port: peer_addr.port(),
            protocol,
            host: host.clone(),
        };
        match conn
            .handle
            .open_stream(OpenStreamRequest {
                routing_info,
                initial_bytes: initial_bytes.clone(),
                stream_timeout: open_stream_timeout,
                on_wait_done: None,
            })
            .await
        {
            Ok(opened) => {
                let send = opened.send;
                let recv = opened.recv;
                let _inflight_guard = opened.inflight;
                let (sent, received) =
                    relay_quic_to_tcp_with_buffer_size(recv, send, local_stream, relay_buf_size)
                        .await?;
                conn.mark_business_completed();
                let extra = initial_bytes.as_ref().map(|b| b.len() as u64).unwrap_or(0);
                debug!(
                    sent = sent + extra, received = received, protocol = ? protocol,
                    "entry relay completed"
                );
                return Ok(());
            }
            Err(e) => match e.kind {
                ErrorKind::QuicOpenTimedOut => {
                    warn!(error = %e, conn_id = conn.handle.stable_id(), "open_bi timed out, trying next connection");
                    last_err = with_conn_detail(conn.handle.stable_id(), e).into();
                }
                ErrorKind::QuicOpenRejectedOverloaded => {
                    warn!(error = %e, conn_id = conn.handle.stable_id(), "open_bi rejected due to overload");
                    if protocol == duotunnel_lib::proxy::core::Protocol::H1 {
                        let body = "503 Service Unavailable - Open stream queue full\n";
                        let resp = format!(
                            "HTTP/1.1 503 Service Unavailable\r\n\
                             Connection: close\r\n\
                             Content-Type: text/plain\r\n\
                             Content-Length: {}\r\n\
                             Retry-After: 1\r\n\
                             X-DuoTunnel-Reject: overload\r\n\
                             \r\n\
                             {}",
                            body.len(),
                            body
                        );
                        let _ = local_stream.write_all(resp.as_bytes()).await;
                        let _ = local_stream.shutdown().await;
                        return Ok(());
                    }
                    let _ = local_stream.shutdown().await;
                    return Err(with_conn_detail(conn.handle.stable_id(), e).into());
                }
                ErrorKind::QuicConnectionLost => {
                    warn!(error = %e, conn_id = conn.handle.stable_id(), "open_bi hit stale connection, evicting pool entry");
                    pool.remove_stable_id(conn.handle.stable_id())
                        .await
                        .map_err(|actor_error| {
                            anyhow::anyhow!(
                                "failed to evict stale connection {}: {actor_error}",
                                conn.handle.stable_id()
                            )
                        })?;
                    last_err = with_conn_detail(conn.handle.stable_id(), e).into();
                }
                ErrorKind::QuicConnectionFatal => {
                    warn!(error = %e, conn_id = conn.handle.stable_id(), "open_bi hit fatal connection error");
                    pool.remove_stable_id(conn.handle.stable_id())
                        .await
                        .map_err(|actor_error| {
                            anyhow::anyhow!(
                                "failed to evict fatal connection {}: {actor_error}",
                                conn.handle.stable_id()
                            )
                        })?;
                    return Err(with_conn_detail(conn.handle.stable_id(), e).into());
                }
                _ => {
                    warn!(error = %e, conn_id = conn.handle.stable_id(), "open_bi failed, trying next connection");
                    last_err = with_conn_detail(conn.handle.stable_id(), e).into();
                }
            },
        }
    }
    if protocol == duotunnel_lib::proxy::core::Protocol::H1 {
        let body = format!("502 Bad Gateway - Egress stream failed: {}\n", last_err);
        let resp = format!(
            "HTTP/1.1 502 Bad Gateway\r\n\
             Connection: close\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             X-DuoTunnel-Reject: egress-stream-failed\r\n\
             \r\n\
             {}",
            body.len(),
            body
        );
        let _ = local_stream.write_all(resp.as_bytes()).await;
        let _ = local_stream.shutdown().await;
        return Ok(());
    }
    Err(last_err)
}

pub struct EgressListenerService {
    pub entry_cfg: EntryListenerConfig,
    pub pool: Arc<EntryConnPool>,
    pub health: Arc<ClientHealth>,
}

#[async_trait::async_trait]
impl ClientService for EgressListenerService {
    fn name(&self) -> &'static str {
        "egress-tcp-listener"
    }
    async fn start(&self, shutdown: CancellationToken) -> anyhow::Result<()> {
        start_entry_listener(
            self.pool.clone(),
            shutdown,
            self.entry_cfg.clone(),
            self.health.clone(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("entry listener failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duotunnel_lib::EgressVhostRuleDef;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn accept_worker_exit_is_only_normal_during_shutdown() {
        assert!(classify_accept_worker_exit(true, None).is_ok());
        let error = classify_accept_worker_exit(false, None).unwrap_err();
        assert!(error.to_string().contains("UnexpectedExit"));
    }

    #[tokio::test]
    async fn test_handle_entry_connection_http_reject() {
        let pool =
            EntryConnPool::new(100, 25, 1, 1, Arc::new(ClientHealth::new(false, 1, 1))).unwrap();
        pool.set_egress_rules(vec![EgressVhostRuleDef {
            match_host: "allowed.com".to_string(),
            action_upstream: "backend".to_string(),
        }]);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client_handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream
                .write_all(b"GET / HTTP/1.1\r\nHost: blocked.com\r\n\r\n")
                .await
                .unwrap();
            let mut response = Vec::new();
            stream.read_to_end(&mut response).await.unwrap();
            response
        });

        let (server_stream, _) = listener.accept().await.unwrap();
        let tcp_params = Arc::new(TcpParams::default());
        let res = handle_entry_connection(
            pool,
            server_stream,
            1024,
            tcp_params,
            Duration::from_secs(1),
            Duration::from_secs(1),
            duotunnel_lib::ProxyBufferParams::default().relay_buf_size,
        )
        .await;

        assert!(res.is_ok());

        let client_response = client_handle.await.unwrap();
        let resp_str = String::from_utf8_lossy(&client_response);
        assert!(resp_str.contains("HTTP/1.1 502 Bad Gateway"));
        assert!(resp_str.contains("X-DuoTunnel-Reject: no-egress-route"));
    }

    #[tokio::test]
    async fn test_handle_entry_connection_tls_reject() {
        let pool =
            EntryConnPool::new(100, 25, 1, 1, Arc::new(ClientHealth::new(false, 1, 1))).unwrap();
        pool.set_egress_rules(vec![EgressVhostRuleDef {
            match_host: "allowed.com".to_string(),
            action_upstream: "backend".to_string(),
        }]);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client_handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            let mut client_hello = vec![
                0x16, // Handshake
                0x03, 0x01, // TLS 1.0 (version in record header)
                0x00, 0x43, // Record Length: 67 bytes
                0x01, // Client Hello
                0x00, 0x00, 0x3f, // Handshake length: 63 bytes
                0x03, 0x03, // Version: TLS 1.2
                // Random: 32 bytes
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0x00, // Session ID length: 0
                0x00, 0x02, // Cipher suite length: 2
                0x00, 0x2f, // Cipher suite: TLS_RSA_WITH_AES_128_CBC_SHA
                0x01, // Compression method length: 1
                0x00, // Compression: null
                0x00, 0x14, // Extensions length: 20
                0x00, 0x00, // Extension type: server_name (SNI)
                0x00, 0x10, // SNI extension length: 16
                0x00, 0x0e, // Server name list length: 14
                0x00, // Server name type: host_name
                0x00, 0x0b, // Host name length: 11
            ];
            client_hello.extend_from_slice(b"blocked.com");
            stream.write_all(&client_hello).await.unwrap();
            let mut response = Vec::new();
            stream.read_to_end(&mut response).await.unwrap();
            response
        });

        let (server_stream, _) = listener.accept().await.unwrap();
        let tcp_params = Arc::new(TcpParams::default());
        let res = handle_entry_connection(
            pool,
            server_stream,
            1024,
            tcp_params,
            Duration::from_secs(1),
            Duration::from_secs(1),
            duotunnel_lib::ProxyBufferParams::default().relay_buf_size,
        )
        .await;

        assert!(res.is_ok());

        let client_response = client_handle.await.unwrap();
        assert!(client_response.is_empty());
    }
}
