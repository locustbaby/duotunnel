use crate::{ConnectionHandle, OpenStreamRequest, QuinnStream};
use anyhow::Result;
use arc_swap::ArcSwap;
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http2::{Builder as H2ClientBuilder, SendRequest};
use hyper::{Method, Request, Response, Uri, Version};
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tracing::debug;

pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, std::io::Error>;
type CachedSendRequest = Arc<Option<SendRequest<BoxBody>>>;

#[derive(Clone)]
pub struct EmptyBodyRetryTemplate {
    method: Method,
    uri: Uri,
    version: Version,
    headers: hyper::HeaderMap,
}

impl EmptyBodyRetryTemplate {
    pub fn from_request<B>(request: &Request<B>) -> Option<Self>
    where
        B: hyper::body::Body,
    {
        (is_automatic_retry_safe_method(request.method()) && request.body().is_end_stream()).then(
            || Self {
                method: request.method().clone(),
                uri: request.uri().clone(),
                version: request.version(),
                headers: request.headers().clone(),
            },
        )
    }

    pub fn build(&self) -> Request<BoxBody> {
        let mut request = Request::builder()
            .method(self.method.clone())
            .uri(self.uri.clone())
            .version(self.version)
            .body(
                Empty::<Bytes>::new()
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .expect("failed to rebuild validated retry request");
        *request.headers_mut() = self.headers.clone();
        request
    }
}

fn is_automatic_retry_safe_method(method: &Method) -> bool {
    // TRACE is safe per HTTP semantics, but automatic replay is intentionally
    // opt-in only because it reflects request metadata and this proxy has no
    // explicit TRACE security policy.
    matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
}

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

fn try_get_sender(cache: &H2SenderCache) -> Option<(SendRequest<BoxBody>, CachedSendRequest)> {
    let sender_arc = cache.sender.load_full();
    match sender_arc.as_ref() {
        Some(s) if s.is_ready() => Some((s.clone(), sender_arc)),
        _ => None,
    }
}

fn clear_sender_if_current(cache: &H2SenderCache, expected: &Arc<Option<SendRequest<BoxBody>>>) {
    let previous = cache.sender.compare_and_swap(expected, Arc::new(None));
    if Arc::ptr_eq(&previous, expected) {
        debug!("cleared failed H2 sender");
    }
}

pub async fn forward_h2_request<B>(
    client_conn: &ConnectionHandle,
    sender_cache: &H2Sender,
    stream_request: OpenStreamRequest,
    request: Request<B>,
) -> Result<Response<BoxBody>>
where
    B: hyper::body::Body + Send + Sync + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let sender = try_get_sender(sender_cache);

    let (sender, sender_arc) = match sender {
        Some(pair) => pair,
        None => {
            let _rebuild_guard = sender_cache.rebuild_mu.lock().await;

            match try_get_sender(sender_cache) {
                Some(pair) => pair,
                None => {
                    debug!("H2 sender miss, establishing new connection");

                    let opened = client_conn.open_stream(stream_request).await?;
                    let crate::OpenedStream {
                        send,
                        recv,
                        inflight,
                        ..
                    } = opened;
                    let quic_stream = QuinnStream { send, recv };
                    let io = TokioIo::new(quic_stream);
                    let (new_sender, conn_driver) =
                        H2ClientBuilder::new(hyper_util::rt::TokioExecutor::new())
                            .initial_max_send_streams(usize::MAX)
                            .handshake(io)
                            .await?;

                    let sender = new_sender.clone();
                    let sender_arc = Arc::new(Some(new_sender));
                    sender_cache.sender.store(sender_arc.clone());

                    let cache = sender_cache.clone();
                    let driver_sender_arc = sender_arc.clone();
                    // Untracked by design: the driver's lifetime is bounded by
                    // the QUIC stream it owns — connection close (including
                    // graceful shutdown) errors the stream, so the driver
                    // exits and clears the cached sender.
                    tokio::spawn(async move {
                        let _inflight_guard = inflight;
                        if let Err(e) = conn_driver.await {
                            debug!(error = %e, "H2 connection driver exited");
                        }
                        clear_sender_if_current(&cache, &driver_sender_arc);
                    });

                    (sender, sender_arc)
                }
            }
        }
    };

    match send_via(sender, request).await {
        Ok(response) => Ok(response),
        Err(error) => {
            clear_sender_if_current(sender_cache, &sender_arc);
            Err(error)
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::{Empty, Full};
    use hyper::body::Body;

    fn empty_request(method: Method) -> Request<Empty<Bytes>> {
        Request::builder()
            .method(method)
            .uri("https://example.test/resource")
            .header("x-retry-test", "preserved")
            .body(Empty::new())
            .unwrap()
    }

    #[test]
    fn retry_template_accepts_only_allowlisted_safe_methods() {
        for method in [Method::GET, Method::HEAD, Method::OPTIONS] {
            assert!(
                EmptyBodyRetryTemplate::from_request(&empty_request(method.clone())).is_some(),
                "{method} should be eligible for automatic replay"
            );
        }
    }

    #[test]
    fn retry_template_rejects_unsafe_or_non_allowlisted_methods_even_with_empty_body() {
        for method in [
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::CONNECT,
            Method::TRACE,
        ] {
            assert!(
                EmptyBodyRetryTemplate::from_request(&empty_request(method.clone())).is_none(),
                "{method} should not be eligible for automatic replay"
            );
        }
    }

    #[test]
    fn retry_template_rejects_non_empty_safe_request() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://example.test/resource")
            .body(Full::new(Bytes::from_static(b"body")))
            .unwrap();

        assert!(EmptyBodyRetryTemplate::from_request(&request).is_none());
    }

    #[test]
    fn retry_template_rebuilds_request_metadata_with_empty_body() {
        let request = empty_request(Method::GET);
        let template = EmptyBodyRetryTemplate::from_request(&request).unwrap();
        let rebuilt = template.build();

        assert_eq!(rebuilt.method(), request.method());
        assert_eq!(rebuilt.uri(), request.uri());
        assert_eq!(rebuilt.version(), request.version());
        assert_eq!(
            rebuilt.headers().get("x-retry-test"),
            request.headers().get("x-retry-test")
        );
        assert!(rebuilt.body().is_end_stream());
    }
}
