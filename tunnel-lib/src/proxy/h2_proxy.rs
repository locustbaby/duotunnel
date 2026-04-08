use crate::{send_routing_info, QuinnStream, RoutingInfo};
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::client::conn::http2::{Builder as H2ClientBuilder, SendRequest};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use quinn::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>;

pub type H2Sender = Arc<Mutex<Option<SendRequest<BoxBody>>>>;

pub fn new_h2_sender() -> H2Sender {
    Arc::new(Mutex::new(None))
}

pub async fn forward_h2_request<B>(
    client_conn: &Connection,
    sender_cache: &H2Sender,
    routing_info: RoutingInfo,
    request: Request<B>,
) -> Result<Response<BoxBody>>
where
    B: hyper::body::Body + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let mut guard = sender_cache.lock().await;

    let mut sender = match guard.as_mut() {
        Some(s) if s.is_ready() => s.clone(),
        _ => {
            debug!("H2 sender miss, establishing new connection");
            *guard = None;

            let (send, recv) = client_conn.open_bi().await?;
            let mut routing_send = send;
            send_routing_info(&mut routing_send, &routing_info).await?;

            let quic_stream = QuinnStream {
                send: routing_send,
                recv,
            };
            let io = TokioIo::new(quic_stream);
            let (sender, conn) = H2ClientBuilder::new(hyper_util::rt::TokioExecutor::new())
                .initial_max_send_streams(usize::MAX)
                .handshake(io)
                .await?;

            let cache = sender_cache.clone();
            tokio::spawn(async move {
                if let Err(e) = conn.await {
                    debug!(error = %e, "H2 connection driver exited");
                }
                *cache.lock().await = None;
            });

            *guard = Some(sender.clone());
            sender
        }
    };

    drop(guard);

    let (parts, body) = request.into_parts();
    let boxed_body = body
        .map_frame(|f| f.map_data(Into::into))
        .map_err(|e| std::io::Error::other(e.into()))
        .boxed_unsync();
    let req = Request::from_parts(parts, boxed_body);
    debug!("H2 forwarding request: {} {}", req.method(), req.uri());
    let resp = sender.send_request(req).await?;
    let (parts, body) = resp.into_parts();
    Ok(Response::from_parts(
        parts,
        body.map_err(std::io::Error::other).boxed_unsync(),
    ))
}
