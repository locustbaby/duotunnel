use crate::runtime::engine::ClientService;
use crate::tunnel::conn_pool::EntryConnPool;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{
    default_client_detectors, maybe_slow_path, relay_quic_to_tcp, run_accept_worker, AcceptedConn,
    ErrorKind, OpenStreamRequest, OverloadLimits, PeekBufPool, ProxyError, RoutingInfo,
    SniffPolicy, SniffRuntime, TcpParams,
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
    pub sniff_timeout: Duration,
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
    let sniff_timeout = cfg.sniff_timeout;
    info!(addr = %addr, accept_workers = %accept_workers, "client entry listener started");

    let mut handles = Vec::with_capacity(accept_workers);
    for _ in 0..accept_workers {
        let listener = Arc::new(tunnel_lib::build_reuseport_listener(addr)?);
        let pool = pool.clone();
        let tcp_params = tcp_params.clone();
        let cancel_token = cancel_token.clone();
        let overload = overload.clone();
        handles.push(crate::runtime::spawn_task(async move {
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
                            sniff_timeout,
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
    sniff_timeout: Duration,
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
        (tunnel_lib::proxy::core::Protocol::Unknown, None, None)
    };

    debug!(protocol = ? protocol, host = ? host, "detected protocol from entry");

    if let Some(ref host_raw) = host {
        if pool.is_rejected_host(host_raw) {
            warn!(target: "client::egress", "Target '{}' rejected locally by egress allowlist", host_raw);
            crate::metrics::egress_rejection("no_egress_route", host_raw);

            if protocol == tunnel_lib::proxy::core::Protocol::H1 {
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
    let mut tried_conn_ids = Vec::with_capacity(pool_size.min(8));
    let mut last_err = anyhow::anyhow!("no QUIC connections available in pool");
    for _ in 0..pool_size.max(1) {
        let conn = match pool.next_conn_for_shard_excluding(preferred_shard, &tried_conn_ids) {
            Some(c) => c,
            None => break,
        };
        tried_conn_ids.push(conn.handle.stable_id());
        maybe_slow_path(
            conn.handle.inflight_table(),
            conn.handle.slot_id(),
            overload,
        )
        .await;
        let routing_info = RoutingInfo {
            proxy_name: "entry".into(),
            src_addr: peer_addr.ip().to_string(),
            src_port: peer_addr.port(),
            protocol,
            host: host.clone(),
        };
        match conn
            .handle
            .open_stream(OpenStreamRequest {
                routing_info,
                initial_bytes: initial_bytes.clone(),
                overload_limits: overload.clone(),
                stream_timeout: open_stream_timeout,
                on_wait_done: None,
            })
            .await
        {
            Ok(opened) => {
                let send = opened.send;
                let recv = opened.recv;
                let _inflight_guard = opened.inflight;
                let (sent, received) = relay_quic_to_tcp(recv, send, local_stream).await?;
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
                    if protocol == tunnel_lib::proxy::core::Protocol::H1 {
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
                    pool.remove_stable_id(conn.handle.stable_id()).await;
                    last_err = with_conn_detail(conn.handle.stable_id(), e).into();
                }
                ErrorKind::QuicConnectionFatal => {
                    warn!(error = %e, conn_id = conn.handle.stable_id(), "open_bi hit fatal connection error");
                    pool.remove_stable_id(conn.handle.stable_id()).await;
                    return Err(with_conn_detail(conn.handle.stable_id(), e).into());
                }
                _ => {
                    warn!(error = %e, conn_id = conn.handle.stable_id(), "open_bi failed, trying next connection");
                    last_err = with_conn_detail(conn.handle.stable_id(), e).into();
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tunnel_lib::EgressVhostRuleDef;

    #[tokio::test]
    async fn test_handle_entry_connection_http_reject() {
        let pool = EntryConnPool::new(100, 25, 1, 1);
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
        let overload = OverloadLimits::resolve(
            tunnel_lib::SharedOverloadMode::Burst,
            100,
            0,
            0,
            None,
            Some(0.0),
            Some(0.0),
            0,
            tunnel_lib::BackoffStrategy::Exponential,
        );

        let res = handle_entry_connection(
            pool,
            server_stream,
            1024,
            tcp_params,
            Duration::from_secs(1),
            &overload,
            Duration::from_secs(1),
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
        let pool = EntryConnPool::new(100, 25, 1, 1);
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
        let overload = OverloadLimits::resolve(
            tunnel_lib::SharedOverloadMode::Burst,
            100,
            0,
            0,
            None,
            Some(0.0),
            Some(0.0),
            0,
            tunnel_lib::BackoffStrategy::Exponential,
        );

        let res = handle_entry_connection(
            pool,
            server_stream,
            1024,
            tcp_params,
            Duration::from_secs(1),
            &overload,
            Duration::from_secs(1),
        )
        .await;

        assert!(res.is_ok());

        let client_response = client_handle.await.unwrap();
        assert!(client_response.is_empty());
    }
}
