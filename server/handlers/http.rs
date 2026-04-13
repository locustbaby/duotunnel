use crate::{metrics, ServerState};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::extract_host_from_http;
use tunnel_lib::proxy;
use tunnel_lib::RuleMatchContext;
use tunnel_lib::RouteTarget;
pub async fn run_http_listener(
    state: Arc<ServerState>,
    listener: TcpListener,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
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
        tokio::task::spawn(async move {
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
        if protocol == tunnel_lib::proxy::core::Protocol::H2 {
            pool.put(buf);
            handle_plaintext_h2_connection(state, stream, peer_addr, port).await
        } else if protocol == tunnel_lib::proxy::core::Protocol::H1 {
            pool.put(buf);
            handle_plaintext_h1_request_connection(state, stream, peer_addr, port).await
        } else {
            let host = detected_host.or_else(|| extract_host_from_http(&buf[..n]));
            let initial_data: Vec<u8> = buf[..n].to_vec();
            pool.put(buf);
            let host =
                host.ok_or_else(|| anyhow::anyhow!("no Host header in plaintext request"))?;
            handle_plaintext_stream_connection(
                state,
                stream,
                host,
                protocol,
                &initial_data,
                port,
            )
            .await
        }
    }
}
fn lookup_route(state: &ServerState, port: u16, host: &str) -> Option<RouteTarget> {
    state
        .routing
        .load()
        .ingress_rules
        .resolve(&RuleMatchContext::new(Some(port), Some(host), None))
}

fn rewrite_request_for_target(
    parts: &mut hyper::http::request::Parts,
    target_host: &str,
) {
    tunnel_lib::rewrite_request_authority(parts, target_host);
}

type ResponseBody = http_body_util::combinators::UnsyncBoxBody<bytes::Bytes, std::io::Error>;

#[derive(Clone)]
struct H2UpstreamEndpoint {
    conn_id: Arc<str>,
    client_conn: quinn::Connection,
    sender_cache: tunnel_lib::H2Sender,
}

#[derive(Clone)]
struct H1UpstreamEndpoint {
    conn_id: Arc<str>,
    client_conn: quinn::Connection,
}

#[derive(Clone)]
struct ServerIngressH2Resolver {
    state: Arc<ServerState>,
    forced_authority: Option<String>,
    single_authority: bool,
    first_authority: Arc<Mutex<Option<String>>>,
    endpoints: Arc<Mutex<HashMap<RouteTarget, H2UpstreamEndpoint>>>,
}

impl ServerIngressH2Resolver {
    fn new(
        state: Arc<ServerState>,
        forced_authority: Option<String>,
        single_authority: bool,
    ) -> Self {
        Self {
            state,
            forced_authority,
            single_authority,
            first_authority: Arc::new(Mutex::new(None)),
            endpoints: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn invalidate_route(&self, route: &RouteTarget) {
        self.endpoints.lock().unwrap().remove(route);
    }
}

#[derive(Clone)]
struct ServerIngressH1Resolver {
    state: Arc<ServerState>,
}

#[async_trait::async_trait]
impl tunnel_lib::HttpFlowResolver for ServerIngressH1Resolver {
    type Route = RouteTarget;
    type Endpoint = H1UpstreamEndpoint;

    fn select_route(
        &self,
        port: u16,
        ctx: &tunnel_lib::HttpRequestContext,
    ) -> Result<Option<tunnel_lib::RouteDecision<Self::Route>>> {
        let route =
            tunnel_lib::resolve_http_route(&self.state.routing.load().ingress_rules, port, ctx);
        Ok(route.map(|route| tunnel_lib::RouteDecision {
            route: route.clone(),
            route_key: Some(route.proxy_name.to_string()),
            target_host: ctx.authority.clone(),
        }))
    }

    async fn select_endpoint(
        &self,
        _ctx: &tunnel_lib::HttpRequestContext,
        route: &tunnel_lib::RouteDecision<Self::Route>,
    ) -> Result<tunnel_lib::EndpointDecision<Self::Endpoint>> {
        let selected = self
            .state
            .registry
            .select_client_for_group(&route.route.group_id)
            .ok_or_else(|| anyhow::anyhow!("no client for group: {}", route.route.group_id))?;
        Ok(tunnel_lib::EndpointDecision {
            selected_endpoint: Some(selected.conn_id.to_string()),
            endpoint: H1UpstreamEndpoint {
                conn_id: selected.conn_id,
                client_conn: selected.conn,
            },
        })
    }
}

#[async_trait::async_trait]
impl tunnel_lib::HttpFlowResolver for ServerIngressH2Resolver {
    type Route = RouteTarget;
    type Endpoint = H2UpstreamEndpoint;

    async fn request_filter(&self, ctx: &mut tunnel_lib::HttpRequestContext) -> Result<()> {
        if let Some(authority) = self.forced_authority.as_deref() {
            ctx.set_authority(authority.to_string());
        }
        if self.single_authority {
            let Some(route_host) = ctx.route_host.clone() else {
                return Ok(());
            };
            let mut first = self.first_authority.lock().unwrap();
            match first.as_ref() {
                None => *first = Some(route_host),
                Some(pinned) if pinned != &route_host => {
                    anyhow::bail!("misdirected request");
                }
                Some(_) => {}
            }
        }
        Ok(())
    }

    fn select_route(
        &self,
        port: u16,
        ctx: &tunnel_lib::HttpRequestContext,
    ) -> Result<Option<tunnel_lib::RouteDecision<Self::Route>>> {
        let route = tunnel_lib::resolve_http_route(&self.state.routing.load().ingress_rules, port, ctx);
        Ok(route.map(|route| tunnel_lib::RouteDecision {
            route: route.clone(),
            route_key: Some(route.proxy_name.to_string()),
            target_host: ctx.authority.clone(),
        }))
    }

    async fn select_endpoint(
        &self,
        _ctx: &tunnel_lib::HttpRequestContext,
        route: &tunnel_lib::RouteDecision<Self::Route>,
    ) -> Result<tunnel_lib::EndpointDecision<Self::Endpoint>> {
        if let Some(endpoint) = self.endpoints.lock().unwrap().get(&route.route).cloned() {
            if endpoint.client_conn.close_reason().is_none() {
                return Ok(tunnel_lib::EndpointDecision {
                    selected_endpoint: Some(endpoint.conn_id.to_string()),
                    endpoint,
                });
            }
            self.endpoints.lock().unwrap().remove(&route.route);
        }
        let selected = self
            .state
            .registry
            .select_client_for_group(&route.route.group_id)
            .ok_or_else(|| anyhow::anyhow!("no client for group: {}", route.route.group_id))?;
        let endpoint = H2UpstreamEndpoint {
            conn_id: selected.conn_id.clone(),
            client_conn: selected.conn,
            sender_cache: tunnel_lib::new_h2_sender(),
        };
        self.endpoints
            .lock()
            .unwrap()
            .insert(route.route.clone(), endpoint.clone());
        Ok(tunnel_lib::EndpointDecision {
            selected_endpoint: Some(endpoint.conn_id.to_string()),
            endpoint,
        })
    }
}

fn text_response(status: hyper::StatusCode, body: &'static str) -> hyper::Response<ResponseBody> {
    use http_body_util::{BodyExt, Full};
    hyper::Response::builder()
        .status(status)
        .body(
            Full::new(bytes::Bytes::from(body))
                .map_err(|_| unreachable!())
                .boxed_unsync(),
        )
        .unwrap()
}

async fn forward_ingress_h2_request(
    flow: &tunnel_lib::HttpFlow<ServerIngressH2Resolver>,
    port: u16,
    req: hyper::Request<hyper::body::Incoming>,
    src_addr: &str,
) -> std::result::Result<hyper::Response<ResponseBody>, hyper::Error> {
    use http_body_util::BodyExt;
    let authority = tunnel_lib::authority_from_request(&req);
    let resolved = match flow
        .resolve_target_for_request(port, &req, tunnel_lib::proxy::core::Protocol::H2)
        .await
    {
        Ok(Some(resolved)) => resolved,
        Ok(None) => {
            return Ok(if authority.is_none() && flow.resolver().forced_authority.is_none() {
                text_response(hyper::StatusCode::BAD_REQUEST, "Missing authority")
            } else {
                text_response(hyper::StatusCode::NOT_FOUND, "No route")
            });
        }
        Err(err) if err.to_string().contains("misdirected request") => {
            return Ok(text_response(
                hyper::StatusCode::MISDIRECTED_REQUEST,
                "Misdirected Request",
            ));
        }
        Err(err) => {
            tracing::error!("L7 target resolution error: {}", err);
            return Ok(text_response(
                hyper::StatusCode::BAD_GATEWAY,
                "No client available",
            ));
        }
    };

    let target_host = resolved
        .context
        .target_host
        .clone()
        .or_else(|| resolved.context.authority.clone())
        .unwrap_or_default();
    let (mut parts, body) = req.into_parts();
    rewrite_request_for_target(&mut parts, &target_host);
    if let Ok(v) = src_addr.parse::<hyper::header::HeaderValue>() {
        parts.headers.append("x-forwarded-for", v.clone());
        parts.headers.insert("x-real-ip", v);
    }
    debug!(
        "L7 Proxy: rewriting authority to {}, forwarding {} {}",
        target_host, parts.method, parts.uri
    );
    let routing_info = tunnel_lib::RoutingInfo {
        proxy_name: resolved.route.route.proxy_name.to_string(),
        protocol: tunnel_lib::proxy::core::Protocol::H2,
        host: None,
    };
    let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
    let upstream_req = hyper::Request::from_parts(parts, boxed_body);
    match tunnel_lib::forward_h2_request(
        &resolved.endpoint.endpoint.client_conn,
        &resolved.endpoint.endpoint.sender_cache,
        routing_info,
        upstream_req,
    )
    .await
    {
        Ok(resp) => {
            let (mut parts, body) = resp.into_parts();
            if let Err(err) = flow.filter_response_parts(&resolved, &mut parts).await {
                tracing::error!("L7 response filter error: {}", err);
            }
            Ok(hyper::Response::from_parts(parts, body))
        }
        Err(err) => {
            tracing::error!("L7 Proxy upstream error: {}", err);
            flow.resolver().invalidate_route(&resolved.route.route);
            Ok(text_response(hyper::StatusCode::BAD_GATEWAY, "Bad Gateway"))
        }
    }
}

async fn forward_ingress_h1_request(
    state: &Arc<ServerState>,
    flow: &tunnel_lib::HttpFlow<ServerIngressH1Resolver>,
    port: u16,
    req: hyper::Request<hyper::body::Incoming>,
    src_addr: &str,
) -> std::result::Result<hyper::Response<ResponseBody>, hyper::Error> {
    let authority = tunnel_lib::authority_from_request(&req);
    let resolved = match flow
        .resolve_target_for_request(port, &req, tunnel_lib::proxy::core::Protocol::H1)
        .await
    {
        Ok(Some(resolved)) => resolved,
        Ok(None) => {
            return Ok(if authority.is_none() {
                text_response(hyper::StatusCode::BAD_REQUEST, "Missing authority")
            } else {
                text_response(hyper::StatusCode::NOT_FOUND, "No route")
            });
        }
        Err(err) => {
            tracing::error!("L7 H1 target resolution error: {}", err);
            return Ok(text_response(
                hyper::StatusCode::BAD_GATEWAY,
                "No client available",
            ));
        }
    };

    let open_timeout = Duration::from_millis(state.config.server.open_stream_timeout_ms);
    let _open_bi_guard = metrics::open_bi_begin(&resolved.endpoint.endpoint.conn_id);
    let wait_started = Instant::now();
    let (mut send, mut recv) = match tokio::time::timeout(
        open_timeout,
        resolved.endpoint.endpoint.client_conn.open_bi(),
    )
    .await
    {
        Ok(Ok(streams)) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            streams
        }
        Ok(Err(err)) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            tracing::error!(
                proxy_name = %resolved.route.route.proxy_name,
                conn_id = %resolved.endpoint.endpoint.conn_id,
                open_bi_ms = %wait_started.elapsed().as_millis(),
                error = %err,
                "H1 open_bi failed"
            );
            return Ok(text_response(hyper::StatusCode::BAD_GATEWAY, "Bad Gateway"));
        }
        Err(_) => {
            metrics::open_bi_observe_wait_ms(wait_started.elapsed().as_secs_f64() * 1000.0);
            metrics::open_bi_timeout();
            tracing::error!(
                proxy_name = %resolved.route.route.proxy_name,
                conn_id = %resolved.endpoint.endpoint.conn_id,
                open_bi_ms = %wait_started.elapsed().as_millis(),
                "H1 open_bi timed out"
            );
            return Ok(text_response(hyper::StatusCode::GATEWAY_TIMEOUT, "Gateway Timeout"));
        }
    };
    let open_bi_ms = wait_started.elapsed().as_millis();

    let (mut parts, body) = req.into_parts();
    let target_host = resolved
        .context
        .target_host
        .clone()
        .or_else(|| resolved.context.authority.clone());
    if let Ok(v) = src_addr.parse::<hyper::header::HeaderValue>() {
        parts.headers.append("x-forwarded-for", v.clone());
        parts.headers.insert("x-real-ip", v);
    }
    let routing_info = tunnel_lib::RoutingInfo {
        proxy_name: resolved.route.route.proxy_name.to_string(),
        protocol: tunnel_lib::proxy::core::Protocol::H1,
        host: None,
    };
    let request_head = tunnel_lib::HttpRequestHead::from_parts(
        &parts,
        resolved.context.route_key.clone(),
        target_host,
    );

    if let Err(err) = async {
        tunnel_lib::send_routing_info(&mut send, &routing_info).await?;
        tunnel_lib::send_http_request_head(&mut send, &request_head).await?;
        tunnel_lib::send_http_body(&mut send, body).await?;
        send.finish()?;
        Result::<()>::Ok(())
    }
    .await
    {
        tracing::error!(
            proxy_name = %routing_info.proxy_name,
            conn_id = %resolved.endpoint.endpoint.conn_id,
            open_bi_ms = %open_bi_ms,
            send_ms = %(wait_started.elapsed().as_millis() - open_bi_ms),
            error = %err,
            "H1 request forwarding failed"
        );
        return Ok(text_response(hyper::StatusCode::BAD_GATEWAY, "Bad Gateway"));
    }

    let response_head = match tunnel_lib::recv_http_response_head(&mut recv).await {
        Ok(head) => head,
        Err(err) => {
            tracing::error!(
                proxy_name = %routing_info.proxy_name,
                conn_id = %resolved.endpoint.endpoint.conn_id,
                error = %err,
                "H1 response head receive failed"
            );
            return Ok(text_response(hyper::StatusCode::BAD_GATEWAY, "Bad Gateway"));
        }
    };
    let mut response_parts = match response_head.into_parts() {
        Ok(parts) => parts,
        Err(err) => {
            tracing::error!(
                proxy_name = %routing_info.proxy_name,
                conn_id = %resolved.endpoint.endpoint.conn_id,
                error = %err,
                "H1 response head decode failed"
            );
            return Ok(text_response(hyper::StatusCode::BAD_GATEWAY, "Bad Gateway"));
        }
    };
    if let Err(err) = flow.filter_response_parts(&resolved, &mut response_parts).await {
        tracing::error!("H1 response filter error: {}", err);
    }
    Ok(hyper::Response::from_parts(
        response_parts,
        tunnel_lib::recv_http_body(recv),
    ))
}
async fn handle_tls_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    host: String,
    peer_addr: std::net::SocketAddr,
    port: u16,
) -> Result<()> {
    use hyper::server::conn::http1::Builder as H1Builder;
    use hyper::server::conn::http2::Builder as H2Builder;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    debug!(host = %host, "TLS connection detected, terminating");
    let server_config = tunnel_lib::infra::pki::get_or_create_server_config(&host)?;
    let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
    let tls_stream = acceptor.accept(stream).await?;
    let is_h2 = tls_stream.get_ref().1.alpn_protocol() == Some(b"h2");
    let src_addr = peer_addr.ip().to_string();
    if is_h2 {
        info!("TLS terminated, serving H2 with authority rewriting");
        let flow = Arc::new(tunnel_lib::HttpFlow::new(ServerIngressH2Resolver::new(
            state.clone(),
            Some(host),
            false,
        )));
        let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
            let flow = flow.clone();
            let src_addr = src_addr.clone();
            async move { forward_ingress_h2_request(&flow, port, req, &src_addr).await }
        });
        let io = TokioIo::new(tls_stream);
        H2Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .map_err(|e| anyhow::anyhow!("H2 connection error: {}", e))?;
    } else {
        info!("TLS terminated, serving H1 with authority rewriting");
        let flow = Arc::new(tunnel_lib::HttpFlow::new(ServerIngressH1Resolver {
            state: state.clone(),
        }));
        let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
            let flow = flow.clone();
            let state = state.clone();
            let src_addr = src_addr.clone();
            async move { forward_ingress_h1_request(&state, &flow, port, req, &src_addr).await }
        });
        H1Builder::new()
            .keep_alive(true)
            .serve_connection(TokioIo::new(tls_stream), service)
            .await
            .map_err(|e| anyhow::anyhow!("H1 TLS connection error: {}", e))?;
    }
    Ok(())
}
async fn handle_plaintext_h2_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    peer_addr: std::net::SocketAddr,
    port: u16,
) -> Result<()> {
    use hyper::server::conn::http2::Builder as H2Builder;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    debug!("plaintext H2 detected, using L7 proxy");
    let src_addr = peer_addr.ip().to_string();
    let h2_single_authority = state.config.server.h2_single_authority;
    let flow = Arc::new(tunnel_lib::HttpFlow::new(ServerIngressH2Resolver::new(
        state,
        None,
        h2_single_authority,
    )));
    let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
        let flow = flow.clone();
        let src_addr = src_addr.clone();
        async move { forward_ingress_h2_request(&flow, port, req, &src_addr).await }
    });
    let io = TokioIo::new(stream);
    H2Builder::new(hyper_util::rt::TokioExecutor::new())
        .serve_connection(io, service)
        .await
        .map_err(|e| anyhow::anyhow!("H2 connection error: {}", e))?;
    Ok(())
}
async fn handle_plaintext_h1_request_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    peer_addr: std::net::SocketAddr,
    port: u16,
) -> Result<()> {
    use hyper::server::conn::http1::Builder as H1Builder;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;

    let src_addr = peer_addr.ip().to_string();
    let flow = Arc::new(tunnel_lib::HttpFlow::new(ServerIngressH1Resolver {
        state: state.clone(),
    }));
    let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
        let flow = flow.clone();
        let state = state.clone();
        let src_addr = src_addr.clone();
        async move { forward_ingress_h1_request(&state, &flow, port, req, &src_addr).await }
    });
    H1Builder::new()
        .keep_alive(true)
        .serve_connection(TokioIo::new(stream), service)
        .await
        .map_err(|e| anyhow::anyhow!("H1 connection error: {}", e))?;
    Ok(())
}

async fn handle_plaintext_stream_connection(
    state: Arc<ServerState>,
    mut stream: TcpStream,
    host: String,
    protocol: tunnel_lib::proxy::core::Protocol,
    initial_data: &[u8],
    port: u16,
) -> Result<()> {
    use tokio::io::AsyncReadExt;
    debug!(host = %host, protocol = ?protocol, "plaintext H1/WS, using byte-level forwarding");
    let route = lookup_route(&state, port, &host)
        .ok_or_else(|| anyhow::anyhow!("no route for host: {}", host))?;
    let (group_id, proxy_name) = (route.group_id, route.proxy_name);
    let selected = state
        .registry
        .select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;
    let mut discard = vec![0u8; initial_data.len()];
    stream.read_exact(&mut discard).await?;
    let routing_info = tunnel_lib::RoutingInfo {
        proxy_name: proxy_name.to_string(),
        protocol,
        host: None,
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
