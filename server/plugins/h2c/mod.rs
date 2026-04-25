use anyhow::Result;
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::body::Body as _;
use hyper::server::conn::http2::Builder as H2Builder;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, Uri, Version};
use hyper_util::rt::TokioIo;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::debug;

use tunnel_lib::plugin::{
    IngressProtocolHandler, PhaseResult, ProtocolHint, ProtocolKind, Route, RouteCtx,
    RouteResolver, ServerCtx,
};
use tunnel_lib::transport::listener::RouteTarget;
use tunnel_lib::{ErrorKind, ErrorSource, ProxyError};

use crate::registry::{SelectedConnection, SharedRegistry};

#[derive(Clone)]
struct CachedSender {
    selected: Arc<SelectedConnection>,
    sender: tunnel_lib::H2Sender,
}

#[derive(Clone)]
struct RetryableRequest {
    method: Method,
    uri: Uri,
    version: Version,
    headers: hyper::HeaderMap,
}

fn error_response(
    err: &ProxyError,
) -> Response<tunnel_lib::proxy::h2_proxy::BoxBody> {
    let status = err.http_status().unwrap_or(StatusCode::BAD_GATEWAY);
    Response::builder()
        .status(status)
        .body(
            Full::new(bytes::Bytes::from(err.to_string()))
                .map_err(|never| match never {})
                .boxed(),
        )
        .unwrap()
}

fn error_kind_label(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::QuicOpenTimeout => "quic_open_timeout",
        ErrorKind::QuicOpenConnection => "quic_open_connection",
        ErrorKind::HttpUpstreamRequest => "http_upstream_request",
        ErrorKind::H2cMissingAuthority => "h2c_missing_authority",
        ErrorKind::H2cMisdirected => "h2c_misdirected",
        ErrorKind::H2cRouteResolve => "h2c_route_resolve",
        ErrorKind::H2cNoRoute => "h2c_no_route",
        ErrorKind::H2cNoClient => "h2c_no_client",
        ErrorKind::H2cForward => "h2c_forward",
    }
}

fn error_source_label(source: ErrorSource) -> &'static str {
    match source {
        ErrorSource::Upstream => "upstream",
        ErrorSource::Downstream => "downstream",
        ErrorSource::Internal => "internal",
    }
}

fn error_status_label(err: &ProxyError) -> &'static str {
    match err.http_status().map(|s| s.as_u16()) {
        Some(400) => "400",
        Some(404) => "404",
        Some(421) => "421",
        Some(502) => "502",
        Some(503) => "503",
        Some(_) => "other",
        None => "000",
    }
}

fn observe_h2c_error(
    metrics: &Arc<dyn tunnel_lib::plugin::MetricsSink>,
    err: &ProxyError,
) {
    metrics.incr(
        "duotunnel_h2c_errors_total",
        &[
            ("status", error_status_label(err)),
            ("type", error_kind_label(err.kind)),
            ("source", error_source_label(err.source)),
        ],
    );
}

/// Serves HTTP/2 cleartext (h2c) connections with per-request vhost routing
/// and authority rewriting via the QUIC tunnel.
///
/// Holds its own `RouteResolver` reference because H2 multiplexes many
/// authorities on one TCP connection. The dispatcher's Phase 4 runs once
/// per connection; this handler re-resolves per request with the request's
/// `:authority`. This is why `IngressDispatcher` skips Phase 4 for
/// `ProtocolKind::H2c` and passes `None` as the route.
pub struct H2cHandler {
    pub registry: SharedRegistry,
    pub route_resolver: Arc<dyn RouteResolver>,
    pub single_authority: bool,
}

fn get_or_create_sender(
    sender_cache: &Mutex<HashMap<RouteTarget, CachedSender>>,
    registry: &SharedRegistry,
    route_target: &RouteTarget,
) -> Option<CachedSender> {
    let mut guard = sender_cache.lock();
    if let Some(entry) = guard.get(route_target) {
        if entry.selected.conn.close_reason().is_none() {
            return Some(entry.clone());
        }
        guard.remove(route_target);
    }

    let selected = registry.select_client_for_group(&route_target.group_id)?;
    let entry = CachedSender {
        selected,
        sender: tunnel_lib::new_h2_sender(),
    };
    guard.insert(route_target.clone(), entry.clone());
    Some(entry)
}

fn invalidate_sender_if_matches(
    sender_cache: &Mutex<HashMap<RouteTarget, CachedSender>>,
    route_target: &RouteTarget,
    conn_id: usize,
) {
    let mut guard = sender_cache.lock();
    if guard
        .get(route_target)
        .is_some_and(|entry| entry.selected.conn.stable_id() == conn_id)
    {
        guard.remove(route_target);
    }
}

fn build_retry_request(
    template: &RetryableRequest,
) -> Request<tunnel_lib::proxy::h2_proxy::BoxBody> {
    let mut req = Request::builder()
        .method(template.method.clone())
        .uri(template.uri.clone())
        .version(template.version)
        .body(
            http_body_util::Empty::<bytes::Bytes>::new()
                .map_err(|never| match never {})
                .boxed(),
        )
        .unwrap();
    *req.headers_mut() = template.headers.clone();
    req
}

#[async_trait]
impl IngressProtocolHandler for H2cHandler {
    fn protocol_kind(&self) -> ProtocolKind {
        ProtocolKind::H2c
    }

    async fn handle(
        &self,
        stream: TcpStream,
        _route: Option<Route>,
        ctx: &ServerCtx,
    ) -> Result<()> {
        debug!("plaintext H2 detected, using L7 proxy");
        let src_addr = ctx.peer_addr.ip().to_string();
        let src_port = ctx.peer_addr.port();
        let listener_port = ctx.listener_port;
        let client_addr = ctx.peer_addr;
        let single_authority = self.single_authority;

        let first_authority: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let route_cache: Arc<Mutex<HashMap<String, Option<RouteTarget>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let sender_cache: Arc<Mutex<HashMap<RouteTarget, CachedSender>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let registry = self.registry.clone();
        let route_resolver = self.route_resolver.clone();
        let metrics = ctx.metrics.clone();

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let registry = registry.clone();
            let first_authority = first_authority.clone();
            let route_cache = route_cache.clone();
            let sender_cache = sender_cache.clone();
            let route_resolver = route_resolver.clone();
            let metrics = metrics.clone();
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
                        let err = ProxyError::h2c_missing_authority();
                        observe_h2c_error(&metrics, &err);
                        return Ok(error_response(&err));
                    }
                };
                let route_host = host.split(':').next().unwrap_or(&host).to_ascii_lowercase();

                if single_authority {
                    let mut fa = first_authority.lock();
                    match fa.as_ref() {
                        None => *fa = Some(route_host.clone()),
                        Some(pinned) if pinned != &route_host => {
                            let err = ProxyError::h2c_misdirected(route_host.clone());
                            observe_h2c_error(&metrics, &err);
                            return Ok(error_response(&err));
                        }
                        Some(_) => {}
                    }
                }

                let cached_route = { route_cache.lock().get(&route_host).cloned() };
                let route_target = match cached_route {
                    Some(route) => route,
                    None => {
                        let resolve_ctx = RouteCtx {
                            listener_port,
                            client_addr,
                            hint: ProtocolHint::new(ProtocolKind::H2c, bytes::Bytes::new())
                                .with_authority(route_host.clone()),
                        };
                        let resolved = match route_resolver.resolve(&resolve_ctx).await {
                            Ok(PhaseResult::Continue(route)) => Some(RouteTarget {
                                group_id: route.group_id,
                                proxy_name: route.proxy_name,
                            }),
                            Ok(PhaseResult::Reject { .. }) => None,
                            Err(e) => {
                                let err = ProxyError::h2c_route_resolve(format!(
                                    "host={}: {}",
                                    route_host, e
                                ));
                                observe_h2c_error(&metrics, &err);
                                tracing::error!(kind = ?err.kind, error = %err, "h2c route resolve failed");
                                return Ok(error_response(&err));
                            }
                        };
                        route_cache
                            .lock()
                            .insert(route_host.clone(), resolved.clone());
                        resolved
                    }
                };
                let route_target = match route_target {
                    Some(r) => r,
                    None => {
                        let err = ProxyError::h2c_no_route(route_host.clone());
                        observe_h2c_error(&metrics, &err);
                        return Ok(error_response(&err));
                    }
                };

                let proxy_name = route_target.proxy_name.clone();
                let sender_entry =
                    match get_or_create_sender(&sender_cache, &registry, &route_target) {
                        Some(entry) => entry,
                        None => {
                            let err = ProxyError::h2c_no_client(
                                route_target.group_id.to_string(),
                            );
                            observe_h2c_error(&metrics, &err);
                            return Ok(error_response(&err));
                        }
                    };

                let retryable_request = req.body().is_end_stream().then(|| RetryableRequest {
                    method: req.method().clone(),
                    uri: req.uri().clone(),
                    version: req.version(),
                    headers: req.headers().clone(),
                });

                let (parts, body) = req.into_parts();
                debug!(
                    "L7 Proxy (plaintext H2): {} {} -> {}",
                    parts.method, parts.uri, host
                );
                let routing_info = tunnel_lib::RoutingInfo {
                    proxy_name: proxy_name.to_string(),
                    src_addr,
                    src_port,
                    protocol: tunnel_lib::proxy::core::Protocol::H2,
                    host: Some(host),
                };
                let boxed_body = body.map_err(std::io::Error::other).boxed();
                let upstream_req = Request::from_parts(parts, boxed_body);
                match tunnel_lib::forward_h2_request(
                    &sender_entry.selected.conn,
                    &sender_entry.sender,
                    routing_info.clone(),
                    upstream_req,
                )
                .await
                {
                    Ok(resp) => Ok::<_, hyper::Error>(resp),
                    Err(first_err) => {
                        invalidate_sender_if_matches(
                            &sender_cache,
                            &route_target,
                            sender_entry.selected.conn.stable_id(),
                        );

                        if let Some(template) = retryable_request.as_ref() {
                            if let Some(retry_entry) =
                                get_or_create_sender(&sender_cache, &registry, &route_target)
                            {
                                metrics.incr(
                                    "duotunnel_h2c_retry_total",
                                    &[("result", "attempt")],
                                );
                                let retry_req = build_retry_request(template);
                                match tunnel_lib::forward_h2_request(
                                    &retry_entry.selected.conn,
                                    &retry_entry.sender,
                                    routing_info.clone(),
                                    retry_req,
                                )
                                .await
                                {
                                    Ok(resp) => {
                                        metrics.incr(
                                            "duotunnel_h2c_retry_total",
                                            &[("result", "success")],
                                        );
                                        return Ok::<_, hyper::Error>(resp);
                                    }
                                    Err(retry_err) => {
                                        metrics.incr(
                                            "duotunnel_h2c_retry_total",
                                            &[("result", "error")],
                                        );
                                        invalidate_sender_if_matches(
                                            &sender_cache,
                                            &route_target,
                                            retry_entry.selected.conn.stable_id(),
                                        );
                                        let err =
                                            ProxyError::h2c_forward(retry_err.to_string());
                                        observe_h2c_error(&metrics, &err);
                                        tracing::error!(
                                            kind = ?err.kind,
                                            error = %err,
                                            "L7 Proxy upstream retry error"
                                        );
                                        return Ok(error_response(&err));
                                    }
                                }
                            }
                        }

                        let err = ProxyError::h2c_forward(first_err.to_string());
                        observe_h2c_error(&metrics, &err);
                        tracing::error!(kind = ?err.kind, error = %err, "L7 Proxy upstream error");
                        Ok(error_response(&err))
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
}
