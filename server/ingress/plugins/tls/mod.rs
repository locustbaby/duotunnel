use anyhow::Result;
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};
use tunnel_lib::{EmptyBodyRetryTemplate, OpenStreamRequest, ProxyError, RouteTarget};

use crate::ingress::registry::{unregister_if_connection_lost, SelectedConnection, SharedRegistry};
use crate::RuntimeGeneration;

#[derive(Clone)]
struct CachedSender {
    selected: Arc<SelectedConnection>,
    sender: tunnel_lib::H2Sender,
}

type SenderCacheKey = (u64, RouteTarget);
const MAX_SENDER_CACHE_ENTRIES: usize = 64;
const PROTOCOL_DRAIN_TIMEOUT: Duration = Duration::from_secs(15);

fn get_or_create_sender(
    sender_cache: &parking_lot::Mutex<std::collections::HashMap<SenderCacheKey, CachedSender>>,
    registry: &SharedRegistry,
    generation: u64,
    route_target: &RouteTarget,
) -> Option<CachedSender> {
    let mut guard = sender_cache.lock();
    guard.retain(|(cached_generation, _), entry| {
        *cached_generation >= generation.saturating_sub(1)
            && entry.selected.handle.close_reason().is_none()
    });
    let key = (generation, route_target.clone());
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
    sender_cache: &parking_lot::Mutex<std::collections::HashMap<SenderCacheKey, CachedSender>>,
    generation: u64,
    route_target: &RouteTarget,
    stable_id: usize,
) {
    let mut guard = sender_cache.lock();
    if guard
        .get(&(generation, route_target.clone()))
        .is_some_and(|entry| entry.selected.handle.stable_id() == stable_id)
    {
        guard.remove(&(generation, route_target.clone()));
    }
}

pub struct TlsHandler {
    pub registry: SharedRegistry,
    pub generation: Arc<arc_swap::ArcSwap<RuntimeGeneration>>,
    pub(crate) health: Arc<crate::runtime::health::ServerHealthFacts>,
}

#[async_trait]
impl IngressProtocolHandler for TlsHandler {
    fn protocol_kind(&self) -> ProtocolKind {
        ProtocolKind::Tls
    }

    async fn handle(
        &self,
        stream: tunnel_lib::PrefixedReadWrite<tokio::net::TcpStream>,
        route: Option<Route>,
        ctx: &ServerCtx,
    ) -> Result<()> {
        let route = route.ok_or_else(ProxyError::routing_missing_info)?;
        let host = ctx
            .hint
            .as_ref()
            .and_then(|h| h.sni.clone())
            .ok_or_else(ProxyError::routing_missing_host)?;

        debug!(host = %host, "TLS connection: terminating");
        let server_config = tunnel_lib::infra::pki::get_or_create_server_config(&host)
            .await
            .map_err(|e| ProxyError::tls_handshake(format!("server config for {host}: {e}")))?;
        let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
        let tls_stream = acceptor
            .accept(stream)
            .await
            .map_err(|e| ProxyError::tls_handshake(e.to_string()))?;

        // We advertise both `h2` and `http/1.1` in ALPN, so the wire protocol
        // must follow what rustls actually negotiated. No ALPN extension at all
        // (older clients) means HTTP/1.1 too.
        let negotiated_h2 = tls_stream
            .get_ref()
            .1
            .alpn_protocol()
            .is_some_and(|proto| proto == b"h2");
        info!(
            alpn = if negotiated_h2 { "h2" } else { "http/1.1" },
            "TLS terminated, serving with authority rewriting"
        );

        let pinned_route_target = RouteTarget {
            group_id: route.group_id,
            proxy_name: route.proxy_name,
        };
        let pinned_generation = ctx.runtime_generation;
        let peer_addr = ctx.peer_addr;
        let src_addr = peer_addr.ip().to_string();
        let src_port = peer_addr.port();
        let target_host = host.clone();
        let overload = ctx.overload.clone();
        let open_stream_timeout = Duration::from_millis(ctx.timeouts.open_stream_ms);
        let listener_port = ctx.listener_port;

        let registry = self.registry.clone();
        let generation = self.generation.clone();
        let health = self.health.clone();
        let sender_cache: Arc<
            parking_lot::Mutex<std::collections::HashMap<SenderCacheKey, CachedSender>>,
        > = Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new()));
        let metrics = ctx.metrics.clone();

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let registry = registry.clone();
            let sender_cache = sender_cache.clone();
            let metrics = metrics.clone();
            let target_host = target_host.clone();
            let generation = generation.clone();
            let health = health.clone();
            let pinned_route_target = pinned_route_target.clone();
            let src_addr = src_addr.clone();
            let overload = overload.clone();
            async move {
                let _tracked = tunnel_lib::track_resource(tunnel_lib::TrackedResource::HttpRequest);
                if !health.admits_new_work() {
                    let err = ProxyError::no_client_available("server-not-ready");
                    tunnel_lib::plugin::observe_proxy_error(
                        metrics.as_ref(),
                        ProtocolKind::Tls.as_label(),
                        &err,
                    );
                    return Ok(error_response(&err));
                }
                let (runtime_generation, route_target) = if negotiated_h2 {
                    let runtime = generation.load_full();
                    let runtime_generation = runtime.sequence();
                    let Some(route_target) =
                        runtime.routing().route_target(listener_port, &target_host)
                    else {
                        let err = ProxyError::route_not_found(target_host.clone());
                        tunnel_lib::plugin::observe_proxy_error(
                            metrics.as_ref(),
                            ProtocolKind::Tls.as_label(),
                            &err,
                        );
                        return Ok(error_response(&err));
                    };
                    (runtime_generation, route_target)
                } else {
                    (pinned_generation, pinned_route_target)
                };
                let (mut parts, body) = req.into_parts();
                let mut uri_parts = parts.uri.clone().into_parts();
                if let Ok(authority) = target_host.parse() {
                    uri_parts.authority = Some(authority);
                }
                // HTTP/1.1 requests arrive in origin-form with no scheme. Both
                // `Uri::from_parts` (authority + path without scheme) and the
                // upstream H2 hop (missing `:scheme` is malformed) reject that,
                // and this listener always terminates TLS.
                if uri_parts.scheme.is_none() && uri_parts.path_and_query.is_some() {
                    uri_parts.scheme = Some(hyper::http::uri::Scheme::HTTPS);
                }
                parts.uri = hyper::Uri::from_parts(uri_parts).unwrap_or(parts.uri);
                if let Ok(host_value) = target_host.parse() {
                    parts.headers.insert(hyper::header::HOST, host_value);
                }

                let routing_info = tunnel_lib::RoutingInfo {
                    proxy_name: route_target.proxy_name.clone(),
                    src_addr: src_addr.clone(),
                    src_port,
                    protocol: tunnel_lib::proxy::core::Protocol::H2,
                    host: Some(target_host.clone()),
                };

                let boxed_body = body.map_err(std::io::Error::other).boxed();
                let upstream_req = Request::from_parts(parts, boxed_body);
                let retryable_request = EmptyBodyRetryTemplate::from_request(&upstream_req);

                let mut attempts = 0;
                let max_attempts = 3;
                let mut current_req = Some(upstream_req);

                loop {
                    attempts += 1;
                    let sender_entry = match get_or_create_sender(
                        &sender_cache,
                        &registry,
                        runtime_generation,
                        &route_target,
                    ) {
                        Some(entry) => entry,
                        None => {
                            let err =
                                ProxyError::no_client_available(route_target.group_id.to_string());
                            tunnel_lib::plugin::observe_proxy_error(
                                metrics.as_ref(),
                                ProtocolKind::Tls.as_label(),
                                &err,
                            );
                            return Ok(error_response(&err));
                        }
                    };

                    let req_to_send = if attempts == 1 {
                        current_req
                            .take()
                            .expect("current_req should be available on attempt 1")
                    } else if let Some(template) = &retryable_request {
                        template.build()
                    } else {
                        let err = ProxyError::upstream_forward(
                            "request is not eligible for automatic retry".to_string(),
                        );
                        return Ok(error_response(&err));
                    };

                    debug!(
                        "L7 Proxy (TLS terminated): rewriting authority to {}, forwarding {} {} (attempt {})",
                        target_host,
                        req_to_send.method(),
                        req_to_send.uri(),
                        attempts
                    );

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
                        req_to_send,
                    )
                    .await
                    {
                        Ok(resp) => return Ok::<_, hyper::Error>(resp),
                        Err(e) => {
                            invalidate_sender_if_matches(
                                &sender_cache,
                                runtime_generation,
                                &route_target,
                                sender_entry.selected.handle.stable_id(),
                            );
                            unregister_if_connection_lost(&registry, &sender_entry.selected, &e);

                            if attempts >= max_attempts {
                                let err = ProxyError::upstream_forward(e.to_string());
                                tunnel_lib::plugin::observe_proxy_error(
                                    metrics.as_ref(),
                                    ProtocolKind::Tls.as_label(),
                                    &err,
                                );
                                tracing::error!(kind = ?err.kind, error = %err, "L7 Proxy upstream error after all attempts");
                                return Ok(error_response(&err));
                            }
                        }
                    }
                }
            }
        });

        let io = TokioIo::new(tls_stream);
        if negotiated_h2 {
            let connection = tunnel_lib::proxy::h2::hardened_h2_server_builder(
                hyper_util::rt::TokioExecutor::new(),
            )
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
        } else {
            let connection =
                tunnel_lib::proxy::h2::hardened_h1_server_builder().serve_connection(io, service);
            tokio::pin!(connection);
            tokio::select! {
                result = &mut connection => {
                    result.map_err(|e| {
                        ProxyError::downstream_connection(format!("HTTP/1 connection error: {e}"))
                    })?;
                }
                _ = ctx.quiesce.cancelled() => {
                    connection.as_mut().graceful_shutdown();
                    match tokio::time::timeout(PROTOCOL_DRAIN_TIMEOUT, connection.as_mut()).await {
                        Ok(result) => result.map_err(|e| {
                            ProxyError::downstream_connection(format!("HTTP/1 drain error: {e}"))
                        })?,
                        Err(_) => {
                            return Err(ProxyError::downstream_connection(
                                "HTTP/1 drain timed out after 15s",
                            )
                            .into());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn error_response(err: &ProxyError) -> Response<tunnel_lib::proxy::h2_proxy::BoxBody> {
    let payload = serde_json::json!({
        "error_code": err.error_code(),
        "message": err.to_string(),
    });
    let body_bytes = bytes::Bytes::from(serde_json::to_vec(&payload).unwrap_or_default());

    Response::builder()
        .status(err.http_status().unwrap_or(StatusCode::BAD_GATEWAY))
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .body(Full::new(body_bytes).map_err(|_| unreachable!()).boxed())
        .expect("Failed to build error response")
}
