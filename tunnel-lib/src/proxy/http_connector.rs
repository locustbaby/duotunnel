use super::core::Protocol;
use super::h2::serve_h2_forward;
use super::http::HttpPeer;
use super::peers::HttpPeerSpec;
use crate::egress::http::{H2cClient, HttpsClient};
use crate::transport::quinn_io::{PrefixedReadWrite, QuinnStream};
use crate::ProxyError;
use anyhow::Result;
use bytes::Bytes;
use dashmap::DashMap;
use http_body_util::{BodyExt, Empty};
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::debug;

const PREFER_H1_TTL: Duration = Duration::from_secs(300);

pub type SharedHttpConnector = Arc<HttpConnector>;

#[derive(Clone)]
struct RetryableRequest {
    method: hyper::Method,
    uri: hyper::Uri,
    version: hyper::Version,
    headers: hyper::HeaderMap,
}

pub struct HttpConnector {
    https_client: HttpsClient,
    h2c_client: H2cClient,
    prefer_h1: DashMap<String, Instant>,
}

impl HttpConnector {
    pub fn new(https_client: HttpsClient, h2c_client: H2cClient) -> SharedHttpConnector {
        Arc::new(Self {
            https_client,
            h2c_client,
            prefer_h1: DashMap::new(),
        })
    }

    fn cache_key(spec: &HttpPeerSpec) -> String {
        format!("{}://{}", spec.scheme, spec.target_host)
    }

    fn gc_prefer_h1(&self, key: &str) -> bool {
        match self.prefer_h1.get(key) {
            Some(ts) if ts.elapsed() <= PREFER_H1_TTL => true,
            Some(_) => {
                self.prefer_h1.remove(key);
                false
            }
            None => false,
        }
    }

    fn mark_prefer_h1(&self, spec: &HttpPeerSpec) {
        let key = Self::cache_key(spec);
        self.prefer_h1.insert(key, Instant::now());
    }

    fn build_retry_request(
        template: &RetryableRequest,
    ) -> hyper::Request<super::h2_proxy::BoxBody> {
        let mut req = hyper::Request::builder()
            .method(template.method.clone())
            .uri(template.uri.clone())
            .version(template.version)
            .body(
                Empty::<Bytes>::new()
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap();
        *req.headers_mut() = template.headers.clone();
        req
    }

    fn box_response<RespBody>(
        resp: hyper::Response<RespBody>,
    ) -> hyper::Response<super::h2_proxy::BoxBody>
    where
        RespBody: hyper::body::Body + Send + Sync + 'static,
        RespBody::Data: Into<Bytes> + Send,
        RespBody::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let (parts, body) = resp.into_parts();
        hyper::Response::from_parts(
            parts,
            body.map_frame(|f| f.map_data(Into::into))
                .map_err(|e| std::io::Error::other(e.into()))
                .boxed(),
        )
    }

    pub async fn connect(
        self: &Arc<Self>,
        spec: HttpPeerSpec,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        match spec.protocol {
            Protocol::H2 => self.connect_h2(spec, send, recv, initial_data).await,
            _ => self.connect_h1(spec, send, recv, initial_data).await,
        }
    }

    async fn connect_h1(
        self: &Arc<Self>,
        spec: HttpPeerSpec,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        HttpPeer {
            connector: self.clone(),
            spec,
        }
        .connect_inner(send, recv, initial_data)
        .await
    }

    async fn connect_h2(
        self: &Arc<Self>,
        spec: HttpPeerSpec,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        debug!(target = %spec.target_host, scheme = %spec.scheme, "HTTP connector starting H2 proxy");
        let stream = QuinnStream { send, recv };
        if let Some(init) = initial_data.filter(|b| !b.is_empty()) {
            let io = PrefixedReadWrite::new(stream, init);
            self.serve_h2(io, spec).await
        } else {
            self.serve_h2(stream, spec).await
        }
    }

    pub async fn serve_h2<IO>(self: &Arc<Self>, io: IO, spec: HttpPeerSpec) -> Result<()>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        serve_h2_forward(io, self.clone(), spec).await
    }

    pub async fn request<B>(
        &self,
        spec: &HttpPeerSpec,
        request: hyper::Request<B>,
    ) -> Result<hyper::Response<super::h2_proxy::BoxBody>>
    where
        B: hyper::body::Body + Send + Sync + 'static,
        B::Data: Into<Bytes> + Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let prefer_h1 = spec.scheme == "http" && self.gc_prefer_h1(&Self::cache_key(spec));
        let retryable_request = (spec.scheme == "http" && !prefer_h1 && request.body().is_end_stream())
            .then(|| RetryableRequest {
                method: request.method().clone(),
                uri: request.uri().clone(),
                version: request.version(),
                headers: request.headers().clone(),
            });
        let (parts, body) = request.into_parts();
        let boxed_body = body
            .map_frame(|f| f.map_data(Into::into))
            .map_err(|e| std::io::Error::other(e.into()))
            .boxed();
        let request = hyper::Request::from_parts(parts, boxed_body);

        let result = if spec.scheme == "http" && !prefer_h1 {
            self.h2c_client.request(request).await
        } else {
            self.https_client.request(request).await
        };

        match result {
            Ok(resp) => Ok(Self::box_response(resp)),
            Err(e) => {
                if spec.scheme == "http" && !prefer_h1 {
                    // Phase 1 policy: when cleartext H2 fails once, pin this upstream to H1 for a TTL window.
                    self.mark_prefer_h1(spec);
                    if let Some(template) = retryable_request.as_ref() {
                        debug!(target = %spec.target_host, "cleartext h2c request failed; retrying once with H1");
                        match self
                            .https_client
                            .request(Self::build_retry_request(template))
                            .await
                        {
                            Ok(resp) => return Ok(Self::box_response(resp)),
                            Err(retry_err) => {
                                return Err(
                                    ProxyError::http_upstream_request(retry_err.to_string())
                                        .into(),
                                )
                            }
                        }
                    }
                }
                Err(ProxyError::http_upstream_request(e.to_string()).into())
            }
        }
    }
}
