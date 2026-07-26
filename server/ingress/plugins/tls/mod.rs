use anyhow::Result;
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::body::Body;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};
use tunnel_lib::{OpenStreamRequest, ProxyError, RouteTarget};

use crate::ingress::registry::{unregister_if_connection_lost, SelectedConnection, SharedRegistry};

#[derive(Clone)]
struct CachedSender {
    selected: Arc<SelectedConnection>,
    sender: tunnel_lib::H2Sender,
}

fn get_or_create_sender(
    sender_cache: &parking_lot::Mutex<std::collections::HashMap<RouteTarget, CachedSender>>,
    registry: &SharedRegistry,
    route_target: &RouteTarget,
) -> Option<CachedSender> {
    let mut guard = sender_cache.lock();
    if let Some(entry) = guard.get(route_target) {
        if entry.selected.handle.close_reason().is_none() {
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
    sender_cache: &parking_lot::Mutex<std::collections::HashMap<RouteTarget, CachedSender>>,
    route_target: &RouteTarget,
    stable_id: usize,
) {
    let mut guard = sender_cache.lock();
    if guard
        .get(route_target)
        .is_some_and(|entry| entry.selected.handle.stable_id() == stable_id)
    {
        guard.remove(route_target);
    }
}

pub struct TlsHandler {
    pub registry: SharedRegistry,
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
        info!("TLS terminated, serving H2 with authority rewriting");

        let route_target = RouteTarget {
            group_id: route.group_id.clone(),
            proxy_name: route.proxy_name.clone(),
        };
        let peer_addr = ctx.peer_addr;
        let src_addr = peer_addr.ip().to_string();
        let src_port = peer_addr.port();
        let target_host = host.clone();
        let overload = ctx.overload.clone();
        let open_stream_timeout = Duration::from_millis(ctx.timeouts.open_stream_ms);

        let registry = self.registry.clone();
        let sender_cache: Arc<
            parking_lot::Mutex<std::collections::HashMap<RouteTarget, CachedSender>>,
        > = Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new()));
        let metrics = ctx.metrics.clone();

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let registry = registry.clone();
            let sender_cache = sender_cache.clone();
            let metrics = metrics.clone();
            let target_host = target_host.clone();
            let route_target = route_target.clone();
            let src_addr = src_addr.clone();
            let overload = overload.clone();
            async move {
                let retryable_request = req.body().is_end_stream().then(|| RetryableRequest {
                    method: req.method().clone(),
                    uri: req.uri().clone(),
                    version: req.version(),
                    headers: req.headers().clone(),
                });

                let (mut parts, body) = req.into_parts();
                let mut uri_parts = parts.uri.clone().into_parts();
                if let Ok(authority) = target_host.parse() {
                    uri_parts.authority = Some(authority);
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

                let mut attempts = 0;
                let max_attempts = 3;
                let mut current_req = Some(upstream_req);

                loop {
                    attempts += 1;
                    let sender_entry =
                        match get_or_create_sender(&sender_cache, &registry, &route_target) {
                            Some(entry) => entry,
                            None => {
                                let err = ProxyError::no_client_available(
                                    route_target.group_id.to_string(),
                                );
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
                        build_retry_request(template)
                    } else {
                        let err = ProxyError::upstream_forward(
                            "cannot retry request with body".to_string(),
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
        tunnel_lib::proxy::h2::hardened_h2_server_builder(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .map_err(|e| ProxyError::downstream_connection(format!("H2 connection error: {e}")))?;
        Ok(())
    }
}

struct RetryableRequest {
    method: hyper::Method,
    uri: hyper::Uri,
    version: hyper::Version,
    headers: hyper::HeaderMap,
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
        .expect("Failed to build retry request");
    *req.headers_mut() = template.headers.clone();
    req
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
