use anyhow::Result;
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http2::Builder as H2Builder;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tracing::{debug, info};

use tunnel_lib::plugin::{IngressProtocolHandler, ProtocolKind, Route, ServerCtx};

use crate::registry::SharedRegistry;

/// Terminates TLS (self-signed cert via PKI cache), then serves the inner
/// connection as HTTP/2 with authority rewriting via the QUIC tunnel.
pub struct TlsHandler {
    pub registry: SharedRegistry,
}

#[async_trait]
impl IngressProtocolHandler for TlsHandler {
    fn protocol_kind(&self) -> ProtocolKind {
        ProtocolKind::Tls
    }

    async fn handle(&self, stream: TcpStream, route: Route, ctx: &ServerCtx) -> Result<()> {
        let host = ctx
            .hint
            .as_ref()
            .and_then(|h| h.sni.clone())
            .ok_or_else(|| anyhow::anyhow!("TlsHandler: no SNI in ProtocolHint"))?;

        debug!(host = %host, "TLS connection: terminating");
        let server_config = tunnel_lib::infra::pki::get_or_create_server_config(&host)?;
        let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
        let tls_stream = acceptor.accept(stream).await?;
        info!("TLS terminated, serving H2 with authority rewriting");

        let (group_id, proxy_name) = (route.group_id.clone(), route.proxy_name.clone());
        let peer_addr = ctx.peer_addr;
        let src_addr = peer_addr.ip().to_string();
        let src_port = peer_addr.port();
        let target_host = host.clone();

        let selected = self
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
                        tracing::error!("L7 Proxy upstream error: {}", e);
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

        let io = TokioIo::new(tls_stream);
        H2Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .map_err(|e| anyhow::anyhow!("H2 connection error: {}", e))?;
        Ok(())
    }
}
