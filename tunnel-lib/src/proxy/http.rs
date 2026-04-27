use super::http_connector::SharedHttpConnector;
use super::peers::HttpPeerSpec;
use crate::protocol::driver::h1::Http1Driver;
use crate::protocol::driver::ProtocolDriver;
use crate::ProxyError;
use anyhow::Result;
use bytes::Bytes;
use hyper::Request;
use pin_project_lite::pin_project;
use quinn::{RecvStream, SendStream};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tracing::debug;
const KEEPALIVE_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

pin_project! {
    struct LazyTimeout<F> {
        #[pin]
        future: F,
        deadline: Instant,
        #[pin]
        timer: Option<tokio::time::Sleep>,
    }
}

impl<F, T> Future for LazyTimeout<F>
where
    F: Future<Output = T>,
{
    type Output = Result<T, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if let Poll::Ready(v) = this.future.poll(cx) {
            return Poll::Ready(Ok(v));
        }
        if this.timer.as_ref().as_pin_ref().is_none() {
            this.timer.set(Some(tokio::time::sleep_until(
                tokio::time::Instant::from_std(*this.deadline),
            )));
        }
        if let Some(timer) = this.timer.as_mut().as_pin_mut() {
            if timer.poll(cx).is_ready() {
                return Poll::Ready(Err(()));
            }
        }
        Poll::Pending
    }
}

fn lazy_timeout<F: Future>(duration: Duration, future: F) -> LazyTimeout<F> {
    LazyTimeout {
        future,
        deadline: Instant::now() + duration,
        timer: None,
    }
}
pub struct HttpPeer {
    pub connector: SharedHttpConnector,
    pub spec: HttpPeerSpec,
}
impl HttpPeer {
    pub async fn connect_inner(
        self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        let upstream = self.spec.target_host.clone();
        let mut driver = Http1Driver::new(
            send,
            recv,
            self.spec.scheme.clone(),
            self.spec.target_host.clone(),
            initial_data,
        );
        loop {
            let req =
                match lazy_timeout(KEEPALIVE_IDLE_TIMEOUT, driver.read_request()).await {
                    Ok(Ok(Some(r))) => r,
                    Ok(Ok(None)) => {
                        debug!(upstream = %upstream, "H1 keep-alive: clean EOF, closing");
                        break;
                    }
                    Ok(Err(e)) => {
                        debug!(upstream = %upstream, error = %e, "H1 keep-alive: read_request error, closing");
                        break;
                    }
                    Err(()) => {
                        debug!(upstream = %upstream, "H1 keep-alive: idle timeout, closing");
                        break;
                    }
                };
            let should_close_after = driver.should_close;
            let mut builder = Request::builder()
                .method(req.method)
                .uri(req.uri)
                .version(req.version);
            if let Some(headers) = builder.headers_mut() {
                *headers = req.headers;
            }
            let request = builder.body(req.body)?;
            debug!(upstream = %self.spec.target_host, uri = %request.uri(), "H1 sending request to upstream");
            let response = match self.connector.request(&self.spec, request).await {
                Ok(resp) => resp,
                Err(e) => {
                    let proxy_err = ProxyError::http_upstream_request(format!(
                        "{}: {}",
                        self.spec.target_host, e
                    ));
                    debug!(
                        upstream = %self.spec.target_host,
                        kind = ?proxy_err.kind,
                        retry = ?proxy_err.retry,
                        error = %proxy_err,
                        "H1 upstream request failed, sending 502"
                    );
                    let _ = driver.write_502().await;
                    break;
                }
            };
            debug!(upstream = %self.spec.target_host, status = %response.status(), "H1 received response");
            if let Err(e) = driver.write_response(response).await {
                debug!(upstream = %self.spec.target_host, error = %e, "H1 write_response error, closing");
                break;
            }
            if driver.should_close || should_close_after {
                debug!(upstream = %self.spec.target_host, "H1 keep-alive: Connection: close, closing");
                break;
            }
            debug!(upstream = %self.spec.target_host, "H1 keep-alive: request complete, waiting for next");
        }
        let _ = driver.finish().await;
        Ok(())
    }
}
