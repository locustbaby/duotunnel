use crate::{send_routing_info, QuinnStream, RoutingInfo};
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::client::conn::http2::{Builder as H2ClientBuilder, SendRequest};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use quinn::Connection;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use tracing::debug;

type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>;

/// Cached H2 sender with fast read path and serialised rebuild path.
///
/// Fast path (cache hit):
///   `try_get_sender()` acquires a sync Mutex, clones the `SendRequest`, and releases
///   the lock — all synchronously, before any `.await`. The lock guard never lives
///   inside the async state machine, so the Future remains `Send`.
///
/// Slow path (cache miss or stale sender):
///   `rebuild_mu` (async Mutex) — serialises rebuilds so only one task runs
///   `open_bi` + H2 handshake at a time. Others queue here and then reuse the result.
///
/// Compared to the original `Arc<Mutex<Option<SendRequest>>>`:
/// - fast-path requests no longer serialise (they only share the sync lock for a
///   non-blocking clone, not for the entire request lifetime)
/// - rebuild races are eliminated (only one task ever calls `open_bi` at a time)
pub struct H2SenderCache {
    sender: Mutex<Option<SendRequest<BoxBody>>>,
    rebuild_mu: AsyncMutex<()>,
}

pub type H2Sender = Arc<H2SenderCache>;

pub fn new_h2_sender() -> H2Sender {
    Arc::new(H2SenderCache {
        sender: Mutex::new(None),
        rebuild_mu: AsyncMutex::new(()),
    })
}

/// Try to read a ready sender from the cache without going async.
/// Returns `Some(sender)` on hit, `None` on miss or stale.
/// The Mutex guard is acquired and released entirely within this function —
/// it never crosses an await point.
fn try_get_sender(cache: &H2SenderCache) -> Option<SendRequest<BoxBody>> {
    let guard = cache.sender.lock().unwrap();
    match guard.as_ref() {
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
    B: hyper::body::Body + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    // Fast path: sync call that returns an owned SendRequest — no guard in this future.
    let sender = try_get_sender(sender_cache);

    let sender = match sender {
        Some(s) => s,
        None => {
            // Slow path: serialise rebuilds with an async lock.
            let _rebuild_guard = sender_cache.rebuild_mu.lock().await;

            // Double-check: a previous waiter may have installed a ready sender.
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

                    // Install: sync write (no await), then spawn connection driver.
                    let sender = new_sender.clone();
                    *sender_cache.sender.lock().unwrap() = Some(new_sender);

                    let cache = sender_cache.clone();
                    tokio::spawn(async move {
                        if let Err(e) = conn_driver.await {
                            debug!(error = %e, "H2 connection driver exited");
                        }
                        *cache.sender.lock().unwrap() = None;
                    });

                    sender
                    // _rebuild_guard dropped here
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
    B: hyper::body::Body + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
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
