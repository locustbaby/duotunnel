use anyhow::Result;
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http2::Builder as H2Builder;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tracing::{debug, info};

use tunnel_lib::ProxyError;
use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};

use crate::registry::SharedRegistry;

pub struct TlsHandler {
    pub registry: SharedRegistry,
}

#[async_trait]
impl IngressProtocolHandler for TlsHandler {
    fn protocol_kind(&self) -> ProtocolKind {
        ProtocolKind::Tls
    }

    async fn handle(&self, stream: TcpStream, route: Option<Route>, ctx: &ServerCtx) -> Result<()> {
        let route = route.ok_or_else(ProxyError::routing_missing_info)?;
        let host = ctx
            .hint
            .as_ref()
            .and_then(|h| h.sni.clone())
            .ok_or_else(ProxyError::routing_missing_host)?;

        debug!(host = %host, "TLS connection: terminating");
        let server_config = tunnel_lib::infra::pki::get_or_create_server_config(&host)
            .map_err(|e| ProxyError::tls_handshake(format!("server config for {host}: {e}")))?;
        let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
        let tls_stream = acceptor
            .accept(stream)
            .await
            .map_err(|e| ProxyError::tls_handshake(e.to_string()))?;
        info!("TLS terminated, serving H2 with authority rewriting");

        let (group_id, proxy_name) = (route.group_id.clone(), route.proxy_name.clone());
        let peer_addr = ctx.peer_addr;
        let src_addr = peer_addr.ip().to_string();
        let src_port = peer_addr.port();
        let target_host = host.clone();

        let selected = self
            .registry
            .select_client_for_group(&group_id)
            .ok_or_else(|| ProxyError::no_client_available(group_id.to_string()))?;
        let client_conn = selected.conn.clone();
        let sender_cache = tunnel_lib::new_h2_sender();
        let metrics = ctx.metrics.clone();

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let client_conn = client_conn.clone();
            let sender_cache = sender_cache.clone();
            let metrics = metrics.clone();
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
                    protocol: tunnel_lib::proxy::core::Protocol::H2,
                    host: Some(target_host),
                };
                let boxed_body = body.map_err(std::io::Error::other).boxed();
                let upstream_req = Request::from_parts(parts, boxed_body);
                match tunnel_lib::forward_h2_request(
                    &client_conn,
                    &sender_cache,
                    routing_info,
                    upstream_req,
                )
                .await
                {
                    Ok(resp) => Ok::<_, hyper::Error>(resp),
                    Err(e) => {
                        let err = ProxyError::upstream_forward(e.to_string());
                        tunnel_lib::plugin::observe_proxy_error(
                            metrics.as_ref(),
                            ProtocolKind::Tls.as_label(),
                            &err,
                        );
                        tracing::error!(kind = ?err.kind, error = %err, "L7 Proxy upstream error");
                        Ok(Response::builder()
                            .status(err.http_status().unwrap_or(StatusCode::BAD_GATEWAY))
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

        let io = TokioIo::new(tls_stream);
        H2Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .map_err(|e| ProxyError::downstream_connection(format!("H2 connection error: {e}")))?;
        Ok(())
    }
}
