use anyhow::Result;
use quinn::Connection;
use hyper::{Request, Response};
use hyper::client::conn::http2::handshake;
use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;
use tracing::{info, debug};
use crate::{RoutingInfo, send_routing_info, QuinnStream};
use bytes::Bytes;

pub async fn forward_h2_request<B>(
    client_conn: &Connection,
    routing_info: RoutingInfo,
    request: Request<B>,
) -> Result<Response<http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>>>
where
    B: hyper::body::Body + Send + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    info!(
        proxy_name = %routing_info.proxy_name,
        "forwarding H2 request via QUIC"
    );

    let (mut send, recv) = client_conn.open_bi().await?;
    
    send_routing_info(&mut send, &routing_info).await?;
    
    debug!("routing info sent, establishing H2 connection to client");
    
    let quic_stream = QuinnStream { send, recv };
    let io = TokioIo::new(quic_stream);
    
    let (mut sender, conn) = handshake(hyper_util::rt::TokioExecutor::new(), io).await?;
    
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            tracing::error!("H2 connection to client failed: {}", e);
        }
    });
    
    let (parts, body) = request.into_parts();
    let boxed_body = body.map_err(|e| std::io::Error::other(e.into())).boxed_unsync();
    
    let req = Request::from_parts(parts, boxed_body);
    
    debug!("sending H2 request to client: {} {}", req.method(), req.uri());
    
    let resp = sender.send_request(req).await?;
    
    let (parts, body) = resp.into_parts();
    let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
    
    Ok(Response::from_parts(parts, boxed_body))
}
