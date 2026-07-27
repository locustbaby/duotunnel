use anyhow::Result;
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};
use tunnel_lib::transport::listener::RouteTarget;
use tunnel_lib::{EmptyBodyRetryTemplate, OpenStreamRequest, ProxyError};

use crate::ingress::registry::{unregister_if_connection_lost, SelectedConnection, SharedRegistry};
use crate::RuntimeGeneration;

#[derive(Clone)]
struct CachedSender {
    selected: Arc<SelectedConnection>,
    sender: tunnel_lib::H2Sender,
}

fn error_response(err: &ProxyError) -> Response<tunnel_lib::proxy::h2_proxy::BoxBody> {
    let status = err.http_status().unwrap_or(StatusCode::BAD_GATEWAY);
    let payload = serde_json::json!({
        "error_code": err.error_code(),
        "message": err.to_string(),
    });
    let body_bytes = bytes::Bytes::from(serde_json::to_vec(&payload).unwrap_or_default());

    Response::builder()
        .status(status)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .body(
            Full::new(body_bytes)
                .map_err(|never| match never {})
                .boxed(),
        )
        .expect("Failed to build error response")
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

fn observe_h2c_error(metrics: &Arc<dyn tunnel_lib::plugin::MetricsSink>, err: &ProxyError) {
    tunnel_lib::plugin::observe_proxy_error(metrics.as_ref(), ProtocolKind::H2c.as_label(), err);
    metrics.incr(
        "duotunnel_h2c_errors_total",
        &[
            ("status", error_status_label(err)),
            ("type", err.kind.as_label()),
            ("source", err.source().as_label()),
            ("retry", err.retry().as_label()),
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
    pub generation: Arc<arc_swap::ArcSwap<RuntimeGeneration>>,
    pub(crate) health: Arc<crate::runtime::health::ServerHealthFacts>,
    pub single_authority: bool,
}

type SenderCacheKey = (u64, RouteTarget);
const MAX_ROUTE_CACHE_ENTRIES: usize = 256;
const MAX_SENDER_CACHE_ENTRIES: usize = 64;
const PROTOCOL_DRAIN_TIMEOUT: Duration = Duration::from_secs(15);

fn get_or_create_sender(
    sender_cache: &RwLock<HashMap<SenderCacheKey, CachedSender>>,
    registry: &SharedRegistry,
    generation: u64,
    route_target: &RouteTarget,
) -> Option<CachedSender> {
    let key = (generation, route_target.clone());
    {
        let guard = sender_cache.read();
        if let Some(entry) = guard.get(&key) {
            if entry.selected.handle.close_reason().is_none() {
                return Some(entry.clone());
            }
        }
    }

    let mut guard = sender_cache.write();
    guard.retain(|(cached_generation, _), entry| {
        *cached_generation >= generation.saturating_sub(1)
            && entry.selected.handle.close_reason().is_none()
    });
    if let Some(entry) = guard.get(&key) {
        if entry.selected.handle.close_reason().is_none() {
            return Some(entry.clone());
        }
        guard.remove(&key);
    }

    let selected = registry.select_client_for_group(&route_target.group_id)?;
    let entry = CachedSender {
        selected,
        sender: tunnel_lib::new_h2_sender(),
    };
    if guard.len() >= MAX_SENDER_CACHE_ENTRIES {
        if let Some(oldest) = guard.keys().next().cloned() {
            guard.remove(&oldest);
        }
    }
    guard.insert(key, entry.clone());
    Some(entry)
}

fn invalidate_sender_if_matches(
    sender_cache: &RwLock<HashMap<SenderCacheKey, CachedSender>>,
    generation: u64,
    route_target: &RouteTarget,
    conn_id: usize,
) {
    let mut guard = sender_cache.write();
    if guard
        .get(&(generation, route_target.clone()))
        .is_some_and(|entry| entry.selected.handle.stable_id() == conn_id)
    {
        guard.remove(&(generation, route_target.clone()));
    }
}

#[async_trait]
impl IngressProtocolHandler for H2cHandler {
    fn protocol_kind(&self) -> ProtocolKind {
        ProtocolKind::H2c
    }

    async fn handle(
        &self,
        stream: tunnel_lib::PrefixedReadWrite<tokio::net::TcpStream>,
        _route: Option<Route>,
        ctx: &ServerCtx,
    ) -> Result<()> {
        debug!("plaintext H2 detected, using L7 proxy");
        let src_addr = ctx.peer_addr.ip().to_string();
        let src_port = ctx.peer_addr.port();
        let listener_port = ctx.listener_port;
        let single_authority = self.single_authority;
        let overload = ctx.overload.clone();
        let open_stream_timeout = Duration::from_millis(ctx.timeouts.open_stream_ms);

        let first_authority: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let route_cache: Arc<Mutex<HashMap<(u64, String), Option<RouteTarget>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let sender_cache: Arc<RwLock<HashMap<SenderCacheKey, CachedSender>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let registry = self.registry.clone();
        let generation = self.generation.clone();
        let health = self.health.clone();
        let metrics = ctx.metrics.clone();

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let registry = registry.clone();
            let first_authority = first_authority.clone();
            let route_cache = route_cache.clone();
            let sender_cache = sender_cache.clone();
            let generation = generation.clone();
            let health = health.clone();
            let metrics = metrics.clone();
            let src_addr = src_addr.clone();
            let overload = overload.clone();
            async move {
                let _tracked = tunnel_lib::track_resource(tunnel_lib::TrackedResource::HttpRequest);
                if !health.admits_new_work() {
                    let err = ProxyError::no_client_available("server-not-ready");
                    observe_h2c_error(&metrics, &err);
                    return Ok(error_response(&err));
                }
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
                let runtime = generation.load_full();
                let runtime_generation = runtime.sequence();

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

                let route_cache_key = (runtime_generation, route_host.clone());
                let cached_route = { route_cache.lock().get(&route_cache_key).cloned() };
                let route_target = match cached_route {
                    Some(route) => route,
                    None => {
                        let resolved = runtime.routing().route_target(listener_port, &route_host);
                        let mut cache = route_cache.lock();
                        cache.retain(|(cached_generation, _), _| {
                            *cached_generation >= runtime_generation.saturating_sub(1)
                        });
                        if cache.len() >= MAX_ROUTE_CACHE_ENTRIES {
                            if let Some(oldest) = cache.keys().next().cloned() {
                                cache.remove(&oldest);
                            }
                        }
                        cache.insert(route_cache_key, resolved.clone());
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
                let sender_entry = match get_or_create_sender(
                    &sender_cache,
                    &registry,
                    runtime_generation,
                    &route_target,
                ) {
                    Some(entry) => entry,
                    None => {
                        let err = ProxyError::h2c_no_client(route_target.group_id.to_string());
                        observe_h2c_error(&metrics, &err);
                        return Ok(error_response(&err));
                    }
                };

                let (parts, body) = req.into_parts();
                debug!(
                    "L7 Proxy (plaintext H2): {} {} -> {}",
                    parts.method, parts.uri, host
                );
                let routing_info = tunnel_lib::RoutingInfo {
                    proxy_name: proxy_name.clone(),
                    src_addr,
                    src_port,
                    protocol: tunnel_lib::proxy::core::Protocol::H2,
                    host: Some(host),
                };
                let boxed_body = body.map_err(std::io::Error::other).boxed();
                let upstream_req = Request::from_parts(parts, boxed_body);
                let retryable_request = EmptyBodyRetryTemplate::from_request(&upstream_req);
                match tunnel_lib::forward_h2_request(
                    sender_entry.selected.handle.as_ref(),
                    &sender_entry.sender,
                    OpenStreamRequest {
                        routing_info: routing_info.clone(),
                        initial_bytes: None,
                        overload_limits: overload.clone(),
                        stream_timeout: open_stream_timeout,
                        on_wait_done: None,
                    },
                    upstream_req,
                )
                .await
                {
                    Ok(resp) => Ok::<_, hyper::Error>(resp),
                    Err(first_err) => {
                        invalidate_sender_if_matches(
                            &sender_cache,
                            runtime_generation,
                            &route_target,
                            sender_entry.selected.handle.stable_id(),
                        );
                        unregister_if_connection_lost(
                            &registry,
                            &sender_entry.selected,
                            &first_err,
                        );

                        if let Some(template) = retryable_request.as_ref() {
                            if let Some(retry_entry) = get_or_create_sender(
                                &sender_cache,
                                &registry,
                                runtime_generation,
                                &route_target,
                            ) {
                                metrics.incr("duotunnel_h2c_retry_total", &[("result", "attempt")]);
                                let retry_req = template.build();
                                match tunnel_lib::forward_h2_request(
                                    retry_entry.selected.handle.as_ref(),
                                    &retry_entry.sender,
                                    OpenStreamRequest {
                                        routing_info: routing_info.clone(),
                                        initial_bytes: None,
                                        overload_limits: overload.clone(),
                                        stream_timeout: open_stream_timeout,
                                        on_wait_done: None,
                                    },
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
                                            runtime_generation,
                                            &route_target,
                                            retry_entry.selected.handle.stable_id(),
                                        );
                                        unregister_if_connection_lost(
                                            &registry,
                                            &retry_entry.selected,
                                            &retry_err,
                                        );
                                        let err = ProxyError::h2c_forward(retry_err.to_string());
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
        let connection =
            tunnel_lib::proxy::h2::hardened_h2_server_builder(hyper_util::rt::TokioExecutor::new())
                .serve_connection(io, service);
        tokio::pin!(connection);
        tokio::select! {
            result = &mut connection => {
                result.map_err(|e| {
                    ProxyError::downstream_connection(format!("H2 connection error: {e}"))
                })?;
            }
            _ = ctx.quiesce.cancelled() => {
                connection.as_mut().graceful_shutdown();
                match tokio::time::timeout(PROTOCOL_DRAIN_TIMEOUT, connection.as_mut()).await {
                    Ok(result) => result.map_err(|e| {
                        ProxyError::downstream_connection(format!("H2 drain error: {e}"))
                    })?,
                    Err(_) => {
                        return Err(ProxyError::downstream_connection(
                            "H2 drain timed out after 15s",
                        )
                        .into());
                    }
                }
            }
        }
        Ok(())
    }
}
