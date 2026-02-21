use anyhow::Result;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn, debug};

use tunnel_lib::proxy;
use tunnel_lib::extract_host_from_http;

use crate::{ServerState, metrics};

pub async fn run_http_listener(state: Arc<ServerState>, port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!(addr = %addr, "entry TCP listener started");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        stream.set_nodelay(true)?;
        debug!(peer_addr = %peer_addr, "new entry connection");

        let permit = match state.tcp_semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                warn!(peer_addr = %peer_addr, "HTTP connection rejected: max connections reached");
                metrics::connection_rejected("http");
                continue;
            }
        };

        let state = state.clone();
        tokio::spawn(async move {
            let _permit = permit;
            metrics::tcp_connection_opened();
            let result = handle_http_connection(state, stream).await;
            if let Err(e) = &result {
                debug!(error = %e, "entry connection error");
                metrics::request_completed("http", "error");
            } else {
                metrics::request_completed("http", "success");
            }
            metrics::tcp_connection_closed();
        });
    }
}

async fn handle_http_connection(state: Arc<ServerState>, stream: TcpStream) -> Result<()> {
    use tunnel_lib::detect_protocol_and_host;
    use tunnel_lib::protocol::detect::extract_tls_sni;

    let peer_addr = stream.peer_addr()?;
    let mut buf = [0u8; 16384]; // stack-allocated; avoids heap alloc per connection
    let n = stream.peek(&mut buf).await?;

    let is_tls = n > 0 && buf[0] == 0x16;

    if is_tls {
        let sni = extract_tls_sni(&buf[..n]);
        let host = sni.ok_or_else(|| anyhow::anyhow!("no SNI in TLS ClientHello"))?;
        handle_tls_connection(state, stream, host, peer_addr).await
    } else {
        let (protocol, detected_host) = detect_protocol_and_host(&buf[..n]);

        if protocol == "h2" {
            handle_plaintext_h2_connection(state, stream).await
        } else {
            let host = detected_host.or_else(|| extract_host_from_http(&buf[..n]))
                .ok_or_else(|| anyhow::anyhow!("no Host header in plaintext request"))?;
            handle_plaintext_h1_connection(state, stream, host, protocol.to_string(), peer_addr, n).await
        }
    }
}

/// Handle TLS-terminated connections with H2 authority rewriting
async fn handle_tls_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    host: String,
    peer_addr: std::net::SocketAddr,
) -> Result<()> {
    use hyper::server::conn::http2::Builder as H2Builder;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use http_body_util::{BodyExt, Full};
    use hyper_util::rt::TokioIo;

    debug!(host = %host, "TLS connection detected, terminating");

    let group_id = state.vhost_router.get(&host)
        .ok_or_else(|| anyhow::anyhow!("no route for host: {}", host))?;

    let client_conn = state.registry.select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;

    let (certs, key) = tunnel_lib::infra::pki::generate_self_signed_cert_for_host(&host)?;
    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(server_config));
    let tls_stream = acceptor.accept(stream).await?;

    info!("TLS terminated, serving H2 with authority rewriting");

    let proxy_name = host.clone();
    let target_host = host.clone();
    let src_addr = peer_addr.ip().to_string();
    let src_port = peer_addr.port();

    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
        let client_conn = client_conn.clone();
        let proxy_name = proxy_name.clone();
        let target_host = target_host.clone();
        let src_addr = src_addr.clone();
        async move {
            let (mut parts, body) = req.into_parts();

            let mut uri_parts = parts.uri.clone().into_parts();
            uri_parts.authority = Some(target_host.parse().unwrap());
            parts.uri = hyper::Uri::from_parts(uri_parts).unwrap_or(parts.uri);
            parts.headers.insert(hyper::header::HOST, target_host.parse().unwrap());

            debug!("L7 Proxy: rewriting authority to {}, forwarding {} {}", target_host, parts.method, parts.uri);

            let routing_info = tunnel_lib::RoutingInfo {
                proxy_name,
                src_addr,
                src_port,
                protocol: "h2".to_string(),
                host: Some(target_host),
            };

            let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
            let upstream_req = Request::from_parts(parts, boxed_body);

            match proxy::forward_h2_request(&client_conn, routing_info, upstream_req).await {
                Ok(resp) => Ok::<_, hyper::Error>(resp),
                Err(e) => {
                    tracing::error!("L7 Proxy upstream error: {}", e);
                    Ok(Response::builder()
                        .status(502)
                        .body(Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                        .unwrap())
                }
            }
        }
    });

    let io = TokioIo::new(tls_stream);
    H2Builder::new(hyper_util::rt::TokioExecutor::new())
        .serve_connection(io, service)
        .await
        .map_err(|e| anyhow::anyhow!("H2 connection error: {}", e))?;

    Ok(())
}

/// Handle plaintext H2 connections with per-request routing
async fn handle_plaintext_h2_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
) -> Result<()> {
    use hyper::server::conn::http2::Builder as H2Builder;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use http_body_util::{BodyExt, Full};
    use hyper_util::rt::TokioIo;

    debug!("plaintext H2 detected, using L7 proxy");

    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
        let state = state.clone();
        async move {
            let authority = req.uri().authority().map(|a| a.to_string())
                .or_else(|| req.headers().get(hyper::header::HOST).and_then(|h| h.to_str().ok()).map(|s| s.to_string()));

            let host = match authority {
                Some(h) => h,
                None => {
                    return Ok::<_, hyper::Error>(Response::builder()
                        .status(400)
                        .body(Full::new(bytes::Bytes::from("Missing authority")).map_err(|_| unreachable!()).boxed_unsync())
                        .unwrap());
                }
            };

            let host_without_port = host.split(':').next().unwrap_or(&host);

            let group_id = match state.vhost_router.get(host_without_port) {
                Some(g) => g,
                None => {
                    return Ok(Response::builder()
                        .status(404)
                        .body(Full::new(bytes::Bytes::from("No route")).map_err(|_| unreachable!()).boxed_unsync())
                        .unwrap());
                }
            };

            let client_conn = match state.registry.select_client_for_group(&group_id) {
                Some(c) => c,
                None => {
                    return Ok(Response::builder()
                        .status(503)
                        .body(Full::new(bytes::Bytes::from("No client")).map_err(|_| unreachable!()).boxed_unsync())
                        .unwrap());
                }
            };

            let (parts, body) = req.into_parts();
            debug!("L7 Proxy (plaintext H2): {} {} -> {}", parts.method, parts.uri, host);

            let routing_info = tunnel_lib::RoutingInfo {
                proxy_name: host.clone(),
                src_addr: "0.0.0.0".to_string(),
                src_port: 0,
                protocol: "h2".to_string(),
                host: Some(host),
            };

            let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
            let upstream_req = Request::from_parts(parts, boxed_body);

            match proxy::forward_h2_request(&client_conn, routing_info, upstream_req).await {
                Ok(resp) => Ok::<_, hyper::Error>(resp),
                Err(e) => {
                    tracing::error!("L7 Proxy upstream error: {}", e);
                    Ok(Response::builder()
                        .status(502)
                        .body(Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                        .unwrap())
                }
            }
        }
    });

    let io = TokioIo::new(stream);
    H2Builder::new(hyper_util::rt::TokioExecutor::new())
        .serve_connection(io, service)
        .await
        .map_err(|e| anyhow::anyhow!("H2 connection error: {}", e))?;

    Ok(())
}

/// Handle plaintext H1/WebSocket connections with byte-level forwarding
async fn handle_plaintext_h1_connection(
    state: Arc<ServerState>,
    mut stream: TcpStream,
    host: String,
    protocol: String,
    peer_addr: std::net::SocketAddr,
    peeked_bytes: usize,
) -> Result<()> {
    use tokio::io::AsyncReadExt;

    debug!(host = %host, protocol = %protocol, "plaintext H1/WS, using byte-level forwarding");

    let group_id = state.vhost_router.get(&host)
        .ok_or_else(|| anyhow::anyhow!("no route for host: {}", host))?;

    let client_conn = state.registry.select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;

    let proxy_name = host.clone();

    let mut data = [0u8; 16384]; // stack-allocated; peeked_bytes <= 16384
    stream.read_exact(&mut data[..peeked_bytes]).await?;

    proxy::forward_with_initial_data(
        &client_conn,
        tunnel_lib::RoutingInfo {
            proxy_name,
            src_addr: peer_addr.ip().to_string(),
            src_port: peer_addr.port(),
            protocol,
            host: Some(host),
        },
        stream,
        &data[..peeked_bytes],
    ).await
}
