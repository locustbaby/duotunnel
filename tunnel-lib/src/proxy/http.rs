use super::peers::UpstreamPeer;
use crate::protocol::driver::{ProtocolDriver, h1::Http1Driver};
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::combinators::UnsyncBoxBody;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_rustls::HttpsConnector;
use quinn::{SendStream, RecvStream};
use tracing::{debug, info};
use hyper::Request;

type HttpsClient = Client<HttpsConnector<HttpConnector>, UnsyncBoxBody<Bytes, std::io::Error>>;

pub struct HttpPeer {
    pub client: HttpsClient,
    pub target_host: String,
    pub scheme: String,
}

#[async_trait]
impl UpstreamPeer for HttpPeer {
    async fn connect(
        &self, 
        send: SendStream, 
        recv: RecvStream, 
        initial_data: Option<Bytes>
    ) -> Result<()> {
        // Initialize Driver with initial data
        let mut driver = Http1Driver::new(
            send, 
            recv, 
            self.scheme.clone(), 
            self.target_host.clone(), 
            initial_data
        );

        // Read Request
        let req = match driver.read_request().await? {
            Some(r) => r,
            None => return Ok(()),
        };

        // Prepare Hyper Request
        let mut builder = Request::builder()
            .method(req.method)
            .uri(req.uri)
            .version(req.version);

        if let Some(headers) = builder.headers_mut() {
            *headers = req.headers;
        }

        let request = builder.body(req.body)?;
        debug!(uri = %request.uri(), "sending http request");

        // Send via Hyper
        let response = self.client.request(request).await?;
        
        info!(status = %response.status(), "received http response");

        // Write Response
        driver.write_response(response).await?;
        
        // Finish
        let _ = driver.finish().await;

        Ok(())
    }
}
