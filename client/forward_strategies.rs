use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_rustls::HttpsConnector;
use std::sync::Arc;

#[async_trait]
pub trait ForwardStrategy: Send + Sync {
    async fn forward(
        &self,
        request_bytes: &[u8],
        target_uri: &str,
        is_ssl: bool,
    ) -> Result<Vec<u8>>;
}

pub struct HttpForwardStrategy {
    client: Arc<Client<HttpsConnector<HttpConnector>, http_body_util::Full<Bytes>>>,
}

impl HttpForwardStrategy {
    pub fn new(client: Arc<Client<HttpsConnector<HttpConnector>, http_body_util::Full<Bytes>>>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ForwardStrategy for HttpForwardStrategy {
    async fn forward(
        &self,
        request_bytes: &[u8],
        target_uri: &str,
        _is_ssl: bool,
    ) -> Result<Vec<u8>> {
        crate::forwarder::http::forward_http_request(
            &self.client,
            request_bytes,
            target_uri,
            _is_ssl,
        ).await
    }
}

pub struct WssForwardStrategy;

impl WssForwardStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ForwardStrategy for WssForwardStrategy {
    async fn forward(
        &self,
        request_bytes: &[u8],
        target_uri: &str,
        is_ssl: bool,
    ) -> Result<Vec<u8>> {
        crate::forwarder::wss::forward_wss_request(
            request_bytes,
            target_uri,
            is_ssl,
        ).await
    }
}

pub struct GrpcForwardStrategy;

impl GrpcForwardStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ForwardStrategy for GrpcForwardStrategy {
    async fn forward(
        &self,
        request_bytes: &[u8],
        target_uri: &str,
        is_ssl: bool,
    ) -> Result<Vec<u8>> {
        crate::forwarder::grpc::forward_grpc_request(
            request_bytes,
            target_uri,
            is_ssl,
        ).await
    }
}

impl Default for WssForwardStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GrpcForwardStrategy {
    fn default() -> Self {
        Self::new()
    }
}

