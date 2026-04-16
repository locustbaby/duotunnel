use crate::protocol::driver::h1::Http1Driver;
use crate::protocol::driver::ProtocolDriver;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Request;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use quinn::{RecvStream, SendStream};
use std::time::Duration;
use tracing::debug;
type HttpsClient = Client<HttpsConnector<HttpConnector>, BoxBody<Bytes, std::io::Error>>;
const KEEPALIVE_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
pub struct HttpPeer {
    pub client: HttpsClient,
    pub target_host: String,
    pub scheme: String,
}
impl HttpPeer {
    pub async fn connect_inner(
        self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        let upstream = self.target_host.clone();
        let mut driver = Http1Driver::new(
            send,
            recv,
            self.scheme.clone(),
            self.target_host.clone(),
            initial_data,
        );
        loop {
            let req =
                match tokio::time::timeout(KEEPALIVE_IDLE_TIMEOUT, driver.read_request()).await {
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
            debug!(upstream = %self.target_host, uri = %request.uri(), "H1 sending request to upstream");
            let response = match self.client.request(request).await {
                Ok(resp) => resp,
                Err(e) => {
                    debug!(upstream = %self.target_host, error = %e, "H1 upstream request failed, sending 502");
                    let _ = driver.write_502().await;
                    break;
                }
            };
            debug!(upstream = %self.target_host, status = %response.status(), "H1 received response");
            if let Err(e) = driver.write_response(response).await {
                debug!(upstream = %self.target_host, error = %e, "H1 write_response error, closing");
                break;
            }
            if driver.should_close || should_close_after {
                debug!(upstream = %self.target_host, "H1 keep-alive: Connection: close, closing");
                break;
            }
            debug!(upstream = %self.target_host, "H1 keep-alive: request complete, waiting for next");
        }
        let _ = driver.finish().await;
        Ok(())
    }
}
