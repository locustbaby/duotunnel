use crate::protocol::driver::{h1::Http1Driver, ProtocolDriver};
use anyhow::Result;
use bytes::Bytes;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::Request;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use quinn::{RecvStream, SendStream};
use tracing::{debug, info};

type HttpsClient = Client<HttpsConnector<HttpConnector>, UnsyncBoxBody<Bytes, std::io::Error>>;

pub struct HttpPeer {
    pub client: HttpsClient,
    pub target_host: String,
    pub scheme: String,
}

// TODO: support HTTP/1.1 keep-alive — loop over multiple requests per connection
//       instead of handling one request then calling finish().
impl HttpPeer {
    pub async fn connect_inner(
        self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        let mut driver = Http1Driver::new(
            send,
            recv,
            self.scheme.clone(),
            self.target_host.clone(),
            initial_data,
        );

        let req = match driver.read_request().await? {
            Some(r) => r,
            None => return Ok(()),
        };

        let mut builder = Request::builder()
            .method(req.method)
            .uri(req.uri)
            .version(req.version);

        if let Some(headers) = builder.headers_mut() {
            *headers = req.headers;
        }

        let request = builder.body(req.body)?;
        debug!(uri = %request.uri(), "sending http request");

        let response = self.client.request(request).await?;

        info!(status = %response.status(), "received http response");

        driver.write_response(response).await?;

        let _ = driver.finish().await;

        Ok(())
    }
}
