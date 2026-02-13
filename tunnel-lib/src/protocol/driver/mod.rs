use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Incoming;

pub mod h1;
pub mod h2;

pub struct ProxyRequest {
    pub method: http::Method,
    pub uri: http::Uri,
    pub headers: http::HeaderMap,
    pub version: http::Version,
    pub body: UnsyncBoxBody<Bytes, std::io::Error>,
}

#[async_trait]
pub trait ProtocolDriver {
    /// Read the next request from the downstream connection
    async fn read_request(&mut self) -> Result<Option<ProxyRequest>>;

    /// Write a response back to the downstream connection
    async fn write_response(&mut self, response: http::Response<Incoming>) -> Result<()>;
}
