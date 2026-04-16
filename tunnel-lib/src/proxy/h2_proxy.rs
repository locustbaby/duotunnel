use crate::{send_routing_info, QuinnStream, RoutingInfo};
use anyhow::Result;
use arc_swap::ArcSwap;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::client::conn::http2::{Builder as H2ClientBuilder, SendRequest};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use quinn::Connection;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tracing::debug;

pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, std::io::Error>;

pub struct H2SenderCache {
    sender: ArcSwap<Option<SendRequest<BoxBody>>>,
    rebuild_mu: AsyncMutex<()>,
}

pub type H2Sender = Arc<H2SenderCache>;

pub fn new_h2_sender() -> H2Sender {
    Arc::new(H2SenderCache {
        sender: ArcSwap::new(Arc::new(None)),
        rebuild_mu: AsyncMutex::new(()),
    })
}

fn try_get_sender(cache: &H2SenderCache) -> Option<SendRequest<BoxBody>> {
    match cache.sender.load().as_ref() {
        Some(s) if s.is_ready() => Some(s.clone()),
        _ => None,
    }
}

pub async fn forward_h2_request<B>(
    client_conn: &Connection,
    sender_cache: &H2Sender,
    routing_info: RoutingInfo,
    request: Request<B>,
) -> Result<Response<BoxBody>>
where
    B: hyper::body::Body + Send + Sync + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let sender = try_get_sender(sender_cache);

    let sender = match sender {
        Some(s) => s,
        None => {
            let _rebuild_guard = sender_cache.rebuild_mu.lock().await;

            match try_get_sender(sender_cache) {
                Some(s) => s,
                None => {
                    debug!("H2 sender miss, establishing new connection");

                    let (send, recv) = client_conn.open_bi().await?;
                    let mut routing_send = send;
                    send_routing_info(&mut routing_send, &routing_info).await?;

                    let quic_stream = QuinnStream {
                        send: routing_send,
                        recv,
                    };
                    let io = TokioIo::new(quic_stream);
                    let (new_sender, conn_driver) =
                        H2ClientBuilder::new(hyper_util::rt::TokioExecutor::new())
                            .initial_max_send_streams(usize::MAX)
                            .handshake(io)
                            .await?;

                    let sender = new_sender.clone();
                    sender_cache.sender.store(Arc::new(Some(new_sender)));

                    let cache = sender_cache.clone();
                    tokio::spawn(async move {
                        if let Err(e) = conn_driver.await {
                            debug!(error = %e, "H2 connection driver exited");
                        }
                        cache.sender.store(Arc::new(None));
                    });

                    sender
                }
            }
        }
    };

    send_via(sender, request).await
}

async fn send_via<B>(
    mut sender: SendRequest<BoxBody>,
    request: Request<B>,
) -> Result<Response<BoxBody>>
where
    B: hyper::body::Body + Send + Sync + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let (parts, body) = request.into_parts();
    let boxed_body = body
        .map_frame(|f| f.map_data(Into::into))
        .map_err(|e| std::io::Error::other(e.into()))
        .boxed();
    let req = Request::from_parts(parts, boxed_body);
    debug!("H2 forwarding request: {} {}", req.method(), req.uri());
    let resp = sender.send_request(req).await?;
    let (parts, body) = resp.into_parts();
    Ok(Response::from_parts(
        parts,
        body.map_err(std::io::Error::other).boxed(),
    ))
}
