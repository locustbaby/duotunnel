use crate::{metrics, ServerState};
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::extract_host_from_http;
use tunnel_lib::proxy;
pub async fn run_http_listener(
    state: Arc<ServerState>,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = tunnel_lib::build_reuseport_listener(addr.parse()?)?;
    info!(addr = %addr, "http listener started");
    loop {
        let (stream, peer_addr) = tokio::select! {
            _ = cancel.cancelled() => {
                info!(addr = %addr, "http listener shutting down");
                return Ok(());
            }
            result = listener.accept() => result?,
        };
        state.tcp_params.apply(&stream)?;
        debug!(peer_addr = %peer_addr, "new http connection");
        let permit = match state.tcp_semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                warn!(peer_addr = %peer_addr, "HTTP connection rejected: max connections reached");
                metrics::connection_rejected("http");
                continue;
            }
        };
        let state = state.clone();
        crate::spawn_task(async move {
            let _permit = permit;
            metrics::tcp_connection_opened();
            let result = handle_http_connection(state, stream, port).await;
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
async fn handle_http_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    port: u16,
) -> Result<()> {
    use tunnel_lib::detect_protocol_and_host;
    use tunnel_lib::protocol::detect::extract_tls_sni;
    let peer_addr = stream.peer_addr()?;
    let pool = &state.peek_buf_pool;
    let mut buf = pool.take();
    let n = match stream.peek(&mut buf).await {
        Ok(n) => n,
        Err(e) => {
            pool.put(buf);
            return Err(e.into());
        }
    };
    let is_tls = n > 0 && buf[0] == 0x16;
    if is_tls {
        let sni = extract_tls_sni(&buf[..n]);
        pool.put(buf);
        let host = sni.ok_or_else(|| anyhow::anyhow!("no SNI in TLS ClientHello"))?;
        handle_tls_connection(state, stream, host, peer_addr, port).await
    } else {
        let (protocol, detected_host) = detect_protocol_and_host(&buf[..n]);
        if protocol == "h2" {
            pool.put(buf);
            handle_plaintext_h2_connection(state, stream, peer_addr, port).await
        } else {
            let host = detected_host.or_else(|| extract_host_from_http(&buf[..n]));
            let initial_data: Vec<u8> = buf[..n].to_vec();
            pool.put(buf);
            let host =
                host.ok_or_else(|| anyhow::anyhow!("no Host header in plaintext request"))?;
            handle_plaintext_h1_connection(
                state,
                stream,
                host,
                protocol.to_string(),
                peer_addr,
                &initial_data,
                port,
            )
            .await
        }
    }
}
fn lookup_route(state: &ServerState, port: u16, host: &str) -> Option<(Arc<str>, Arc<str>)> {
    state
        .routing
        .load()
        .http_routers
        .get(&port)
        .and_then(|router| router.get(host))
}
async fn handle_tls_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    host: String,
    peer_addr: std::net::SocketAddr,
    port: u16,
) -> Result<()> {
    use http_body_util::{BodyExt, Full};
    use hyper::server::conn::http2::Builder as H2Builder;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    debug!(host = %host, "TLS connection detected, terminating");
    let (group_id, proxy_name) = lookup_route(&state, port, &host)
        .ok_or_else(|| anyhow::anyhow!("no route for host: {}", host))?;
    let server_config = tunnel_lib::infra::pki::get_or_create_server_config(&host)?;
    let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
    let tls_stream = acceptor.accept(stream).await?;
    info!("TLS terminated, serving H2 with authority rewriting");
    let target_host = host.clone();
    let src_addr = peer_addr.ip().to_string();
    let src_port = peer_addr.port();
    let selected = state
        .registry
        .select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;
    let client_conn = selected.conn;
    let sender_cache = tunnel_lib::new_h2_sender();
    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
        let client_conn = client_conn.clone();
        let sender_cache = sender_cache.clone();
        let proxy_name = proxy_name.clone();
        let target_host = target_host.clone();
        let src_addr = src_addr.clone();
        async move {
            let (mut parts, body) = req.into_parts();
            let mut uri_parts = parts.uri.clone().into_parts();
            if let Ok(authority) = target_host.parse() {
                uri_parts.authority = Some(authority);
            }
            parts.uri = hyper::Uri::from_parts(uri_parts).unwrap_or(parts.uri);
            if let Ok(host_value) = target_host.parse() {
                parts.headers.insert(hyper::header::HOST, host_value);
            }
            debug!(
                "L7 Proxy: rewriting authority to {}, forwarding {} {}",
                target_host, parts.method, parts.uri
            );
            let routing_info = tunnel_lib::RoutingInfo {
                proxy_name: proxy_name.to_string(),
                src_addr,
                src_port,
                protocol: "h2".to_string(),
                host: Some(target_host),
            };
            let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
            let upstream_req = Request::from_parts(parts, boxed_body);
            match tunnel_lib::forward_h2_request(&client_conn, &sender_cache, routing_info, upstream_req).await {
                Ok(resp) => Ok::<_, hyper::Error>(resp),
                Err(e) => {
                    tracing::error!("L7 Proxy upstream error: {}", e);
                    Ok(Response::builder()
                        .status(502)
                        .body(
                            Full::new(bytes::Bytes::from("Bad Gateway"))
                                .map_err(|_| unreachable!())
                                .boxed_unsync(),
                        )
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
async fn handle_plaintext_h2_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    peer_addr: std::net::SocketAddr,
    port: u16,
) -> Result<()> {
    use http_body_util::{BodyExt, Full};
    use hyper::server::conn::http2::Builder as H2Builder;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use std::collections::HashMap;
    use std::sync::Mutex;
    debug!("plaintext H2 detected, using L7 proxy");
    let src_addr = peer_addr.ip().to_string();
    let src_port = peer_addr.port();
    let h2_single_authority = state.config.server.h2_single_authority;
    let first_authority: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    #[allow(clippy::type_complexity)]
    let route_cache: Arc<Mutex<HashMap<String, Option<(Arc<str>, Arc<str>)>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    #[allow(clippy::type_complexity)]
    let sender_cache: Arc<Mutex<HashMap<(Arc<str>, Arc<str>), (quinn::Connection, tunnel_lib::H2Sender)>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
        let state = state.clone();
        let first_authority = first_authority.clone();
        let route_cache = route_cache.clone();
        let sender_cache = sender_cache.clone();
        let src_addr = src_addr.clone();
        async move {
            let authority = req.uri().authority().map(|a| a.to_string()).or_else(|| {
                req.headers()
                    .get(hyper::header::HOST)
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string())
            });
            let host = match authority {
                Some(h) => h,
                None => {
                    return Ok(Response::builder()
                        .status(400)
                        .body(
                            Full::new(bytes::Bytes::from("Missing authority"))
                                .map_err(|_| unreachable!())
                                .boxed_unsync(),
                        )
                        .unwrap());
                }
            };
            let route_host = host.split(':').next().unwrap_or(&host).to_ascii_lowercase();
            if h2_single_authority {
                let mut fa = first_authority.lock().unwrap();
                match fa.as_ref() {
                    None => *fa = Some(route_host.clone()),
                    Some(pinned) if pinned != &route_host => {
                        return Ok(Response::builder()
                            .status(421)
                            .body(
                                Full::new(bytes::Bytes::from("Misdirected Request"))
                                    .map_err(|_| unreachable!())
                                    .boxed_unsync(),
                            )
                            .unwrap());
                    }
                    Some(_) => {}
                }
            }
            let route = {
                let mut cache = route_cache.lock().unwrap();
                cache
                    .entry(route_host.clone())
                    .or_insert_with(|| lookup_route(&state, port, &route_host))
                    .clone()
            };
            let (group_id, proxy_name) = match route {
                Some(r) => r,
                None => {
                    return Ok::<_, hyper::Error>(
                        Response::builder()
                            .status(404)
                            .body(
                                Full::new(bytes::Bytes::from("No route"))
                                    .map_err(|_| unreachable!())
                                    .boxed_unsync(),
                            )
                            .unwrap(),
                    );
                }
            };
            let cache_key = (group_id.clone(), proxy_name.clone());
            let (client_conn, h2_sender) = {
                let mut guard = sender_cache.lock().unwrap();
                if !guard.contains_key(&cache_key) {
                    if let Some(selected) =
                        state.registry.select_client_for_group(&group_id)
                    {
                        guard.insert(cache_key.clone(), (selected.conn, tunnel_lib::new_h2_sender()));
                    }
                }
                match guard.get(&cache_key) {
                    Some(pair) => (pair.0.clone(), pair.1.clone()),
                    None => {
                        return Ok(Response::builder()
                            .status(502)
                            .body(
                                Full::new(bytes::Bytes::from("No client available"))
                                    .map_err(|_| unreachable!())
                                    .boxed_unsync(),
                            )
                            .unwrap());
                    }
                }
            };
            let (parts, body) = req.into_parts();
            debug!(
                "L7 Proxy (plaintext H2): {} {} -> {}",
                parts.method, parts.uri, host
            );
            let routing_info = tunnel_lib::RoutingInfo {
                proxy_name: proxy_name.to_string(),
                src_addr,
                src_port,
                protocol: "h2".to_string(),
                host: Some(host),
            };
            let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
            let upstream_req = Request::from_parts(parts, boxed_body);
            match tunnel_lib::forward_h2_request(&client_conn, &h2_sender, routing_info, upstream_req).await {
                Ok(resp) => Ok::<_, hyper::Error>(resp),
                Err(e) => {
                    tracing::error!("L7 Proxy upstream error: {}", e);
                    sender_cache.lock().unwrap().remove(&cache_key);
                    Ok(Response::builder()
                        .status(502)
                        .body(
                            Full::new(bytes::Bytes::from("Bad Gateway"))
                                .map_err(|_| unreachable!())
                                .boxed_unsync(),
                        )
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
async fn handle_plaintext_h1_connection(
    state: Arc<ServerState>,
    mut stream: TcpStream,
    host: String,
    protocol: String,
    peer_addr: std::net::SocketAddr,
    initial_data: &[u8],
    port: u16,
) -> Result<()> {
    use tokio::io::AsyncReadExt;
    debug!(host = %host, protocol = %protocol, "plaintext H1/WS, using byte-level forwarding");
    let (group_id, proxy_name) = lookup_route(&state, port, &host)
        .ok_or_else(|| anyhow::anyhow!("no route for host: {}", host))?;
    let selected = state
        .registry
        .select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;
    let mut discard = vec![0u8; initial_data.len()];
    stream.read_exact(&mut discard).await?;
    let routing_info = tunnel_lib::RoutingInfo {
        proxy_name: proxy_name.to_string(),
        src_addr: peer_addr.ip().to_string(),
        src_port: peer_addr.port(),
        protocol,
        host: Some(host),
    };
    let open_timeout = Duration::from_millis(state.config.server.open_stream_timeout_ms);
    let _open_bi_guard = metrics::open_bi_begin(&selected.conn_id);
    let _inflight_guard = selected.begin_inflight();
    let wait_started = Instant::now();
    let (mut send, recv) = match tokio::time::timeout(open_timeout, selected.conn.open_bi()).await {
        Ok(Ok(streams)) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            streams
        }
        Ok(Err(e)) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            return Err(e.into());
        }
        Err(_) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            metrics::open_bi_timeout();
            return Err(anyhow::anyhow!("open_bi timed out after {:?}", open_timeout));
        }
    };
    tunnel_lib::send_routing_info(&mut send, &routing_info).await?;
    proxy::forward_with_initial_data(
        send,
        recv,
        stream,
        initial_data,
        state.proxy_buffer_params.relay_buf_size,
    )
    .await
}
