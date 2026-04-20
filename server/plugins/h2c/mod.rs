use anyhow::Result;
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http2::Builder as H2Builder;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tracing::debug;

use tunnel_lib::plugin::{
    IngressProtocolHandler, PhaseResult, ProtocolHint, ProtocolKind, Route, RouteCtx,
    RouteResolver, ServerCtx,
};
use tunnel_lib::transport::listener::RouteTarget;

use crate::registry::SharedRegistry;

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
        let sender_cache: Arc<
            Mutex<HashMap<RouteTarget, (quinn::Connection, tunnel_lib::H2Sender)>>,
        > = Arc::new(Mutex::new(HashMap::new()));

        let registry = self.registry.clone();
        let route_resolver = self.route_resolver.clone();

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let registry = registry.clone();
            let first_authority = first_authority.clone();
            let route_cache = route_cache.clone();
            let sender_cache = sender_cache.clone();
            let route_resolver = route_resolver.clone();
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
                                    .boxed(),
                            )
                            .unwrap());
                    }
                };
                let route_host = host.split(':').next().unwrap_or(&host).to_ascii_lowercase();

                if single_authority {
                    let mut fa = first_authority.lock().unwrap();
                    match fa.as_ref() {
                        None => *fa = Some(route_host.clone()),
                        Some(pinned) if pinned != &route_host => {
                            return Ok(Response::builder()
                                .status(421)
                                .body(
                                    Full::new(bytes::Bytes::from("Misdirected Request"))
                                        .map_err(|_| unreachable!())
                                        .boxed(),
                                )
                                .unwrap());
                        }
                        Some(_) => {}
                    }
                }

                let cached_route = { route_cache.lock().unwrap().get(&route_host).cloned() };
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
                                tracing::error!(error = %e, host = %route_host, "h2c route resolve failed");
                                return Ok(Response::builder()
                                    .status(502)
                                    .body(
                                        Full::new(bytes::Bytes::from("Bad Gateway"))
                                            .map_err(|_| unreachable!())
                                            .boxed(),
                                    )
                                    .unwrap());
                            }
                        };
                        route_cache
                            .lock()
                            .unwrap()
                            .insert(route_host.clone(), resolved.clone());
                        resolved
                    }
                };
                let route_target = match route_target {
                    Some(r) => r,
                    None => {
                        return Ok(Response::builder()
                            .status(404)
                            .body(
                                Full::new(bytes::Bytes::from("No route"))
                                    .map_err(|_| unreachable!())
                                    .boxed(),
                            )
                            .unwrap());
                    }
                };

                let (conn_group_id, proxy_name) =
                    (route_target.group_id.clone(), route_target.proxy_name.clone());
                let (client_conn, h2_sender) = {
                    let mut guard = sender_cache.lock().unwrap();
                    if !guard.contains_key(&route_target) {
                        if let Some(selected) =
                            registry.select_client_for_group(&conn_group_id)
                        {
                            guard.insert(
                                route_target.clone(),
                                (selected.conn, tunnel_lib::new_h2_sender()),
                            );
                        }
                    }
                    match guard.get(&route_target) {
                        Some(pair) => (pair.0.clone(), pair.1.clone()),
                        None => {
                            return Ok(Response::builder()
                                .status(502)
                                .body(
                                    Full::new(bytes::Bytes::from("No client available"))
                                        .map_err(|_| unreachable!())
                                        .boxed(),
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
                    protocol: tunnel_lib::proxy::core::Protocol::H2,
                    host: Some(host),
                };
                let boxed_body = body.map_err(std::io::Error::other).boxed();
                let upstream_req = Request::from_parts(parts, boxed_body);
                match tunnel_lib::forward_h2_request(&client_conn, &h2_sender, routing_info, upstream_req)
                    .await
                {
                    Ok(resp) => Ok::<_, hyper::Error>(resp),
                    Err(e) => {
                        tracing::error!("L7 Proxy upstream error: {}", e);
                        sender_cache.lock().unwrap().remove(&route_target);
                        Ok(Response::builder()
                            .status(502)
                            .body(
                                Full::new(bytes::Bytes::from("Bad Gateway"))
                                    .map_err(|_| unreachable!())
                                    .boxed(),
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
}
