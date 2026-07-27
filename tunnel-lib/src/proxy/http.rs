use super::http_connector::SharedHttpConnector;
use super::peers::HttpPeerSpec;
use crate::protocol::driver::h1::Http1Driver;
use crate::protocol::driver::ProtocolDriver;
use crate::timeout as lazy_timeout;
use crate::ProxyError;
use anyhow::Result;
use bytes::Bytes;
use hyper::Request;
use quinn::{RecvStream, SendStream};
use std::time::Duration;
use tracing::debug;
const KEEPALIVE_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const UPSTREAM_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
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
            let req = match lazy_timeout(KEEPALIVE_IDLE_TIMEOUT, driver.read_request()).await {
                Ok(Ok(Some(r))) => r,
                Ok(Ok(None)) => {
                    debug!(upstream = %upstream, "H1 keep-alive: clean EOF, closing");
                    break;
                }
                Ok(Err(e)) => {
                    debug!(upstream = %upstream, error = %e, "H1 keep-alive: read_request error, closing");
                    break;
                }
                Err(_) => {
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
            let response = match tokio::time::timeout(
                UPSTREAM_REQUEST_TIMEOUT,
                self.connector.request(&self.spec, request),
            )
            .await
            {
                Ok(Ok(resp)) => resp,
                Ok(Err(e)) => {
                    let proxy_err = ProxyError::http_upstream_request(format!(
                        "{}: {}",
                        self.spec.target_host, e
                    ));
                    debug!(
                        upstream = %self.spec.target_host,
                        kind = ?proxy_err.kind,
                        retry = ?proxy_err.retry(),
                        error = %proxy_err,
                        "H1 upstream request failed, sending 502"
                    );
                    let _ = driver.write_502(&proxy_err.to_string()).await;
                    break;
                }
                Err(_) => {
                    let proxy_err = ProxyError::http_upstream_request(format!(
                        "{}: upstream request timed out after {:?}",
                        self.spec.target_host, UPSTREAM_REQUEST_TIMEOUT
                    ));
                    debug!(
                        upstream = %self.spec.target_host,
                        error = %proxy_err,
                        "H1 upstream request timed out, sending 502"
                    );
                    let _ = driver.write_502(&proxy_err.to_string()).await;
                    break;
                }
            };
            debug!(upstream = %self.spec.target_host, status = %response.status(), "H1 received response");
            match tokio::time::timeout(UPSTREAM_REQUEST_TIMEOUT, driver.write_response(response))
                .await
            {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    debug!(upstream = %self.spec.target_host, error = %e, "H1 write_response error, closing");
                    break;
                }
                Err(_) => {
                    debug!(upstream = %self.spec.target_host, "H1 response body timed out, closing");
                    break;
                }
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
