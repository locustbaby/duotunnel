use anyhow::Result;
use bytes::BytesMut;
use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
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
    // Shared H2Sender across all entry connections on this listener — reuses the same
    // QUIC H2 stream for all HTTP requests instead of open_bi() per request.
    let sender_cache = tunnel_lib::new_h2_sender();
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
                let sender_cache = sender_cache.clone();
                crate::spawn_task(async move {
                    let _permit = permit;
                    if let Err(e) = handle_entry_connection(conn, stream, peek_buf_size, tcp_params, open_stream_timeout, sender_cache).await {
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
    sender_cache: tunnel_lib::H2Sender,
) -> Result<()> {
    let peer_addr = local_stream.peer_addr()?;
    tcp_params.apply(&local_stream)?;
    let mut buf = BytesMut::zeroed(peek_buf_size);
    let n = local_stream.peek(&mut buf).await?;
    let initial_bytes = buf.freeze().slice(..n);
    let (protocol, host) = detect_protocol_and_host(&initial_bytes);
    debug!(protocol = % protocol, host = ? host, "detected protocol from entry");

    if protocol == "websocket" || protocol == "tcp" {
        // WebSocket and raw TCP require byte-level relay — cannot be multiplexed over H2.
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
        debug!(sent = sent, received = received, protocol = %protocol, "entry relay completed");
        return Ok(());
    }

    // HTTP (h1/h2/unknown): parse with hyper and forward via shared H2Sender.
    let target_host = host.clone().unwrap_or_default();
    let src_addr = peer_addr.ip().to_string();
    let src_port = peer_addr.port();
    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
        let conn = conn.clone();
        let sender_cache = sender_cache.clone();
        let target_host = target_host.clone();
        let src_addr = src_addr.clone();
        async move {
            let (mut parts, body) = req.into_parts();
            let mut uri_parts = parts.uri.clone().into_parts();
            if uri_parts.authority.is_none() {
                if let Ok(authority) = target_host.parse() {
                    uri_parts.authority = Some(authority);
                }
            }
            if uri_parts.scheme.is_none() {
                uri_parts.scheme = Some(hyper::http::uri::Scheme::HTTP);
            }
            parts.uri = hyper::Uri::from_parts(uri_parts).unwrap_or(parts.uri);
            if let Ok(host_value) = target_host.parse() {
                parts.headers.insert(hyper::header::HOST, host_value);
            }
            let routing_info = RoutingInfo {
                proxy_name: "entry".to_string(),
                src_addr,
                src_port,
                protocol: "h2".to_string(),
                host: Some(target_host),
            };
            let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
            let upstream_req = Request::from_parts(parts, boxed_body);
            match tunnel_lib::forward_h2_request(&conn, &sender_cache, routing_info, upstream_req).await {
                Ok(resp) => Ok::<_, hyper::Error>(resp),
                Err(e) => {
                    debug!(error = %e, "entry H2 upstream error");
                    Ok(Response::builder()
                        .status(502)
                        .body(
                            http_body_util::Full::new(bytes::Bytes::from("Bad Gateway"))
                                .map_err(|_| unreachable!())
                                .boxed_unsync(),
                        )
                        .unwrap())
                }
            }
        }
    });
    let io = TokioIo::new(local_stream);
    http1::Builder::new()
        .serve_connection(io, service)
        .with_upgrades()
        .await
        .map_err(|e| anyhow::anyhow!("entry H1 connection error: {}", e))?;
    Ok(())
}
