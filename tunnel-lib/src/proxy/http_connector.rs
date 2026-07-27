use super::core::Protocol;
use super::h2::serve_h2_forward;
use super::h2_proxy::EmptyBodyRetryTemplate;
use super::http::HttpPeer;
use super::peers::HttpPeerSpec;
use super::upstream::UpstreamHealthRegistry;
use crate::egress::http::{H2cClient, HttpsClient};
use crate::transport::quinn_io::{PrefixedReadWrite, QuinnStream};
use crate::ProxyError;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use parking_lot::Mutex;
use quinn::{RecvStream, SendStream};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::debug;

const PREFER_H1_TTL: Duration = Duration::from_secs(300);
const MAX_PREFER_H1_ENTRIES: usize = 1024;

pub type SharedHttpConnector = Arc<HttpConnector>;

pub struct HttpConnector {
    https_client: HttpsClient,
    h2c_client: H2cClient,
    prefer_h1: Mutex<HashMap<String, Instant>>,
    upstream_health: Option<Arc<UpstreamHealthRegistry>>,
}

impl HttpConnector {
    pub fn new(https_client: HttpsClient, h2c_client: H2cClient) -> SharedHttpConnector {
        Self::new_with_health(https_client, h2c_client, None)
    }

    pub fn new_with_health(
        https_client: HttpsClient,
        h2c_client: H2cClient,
        upstream_health: Option<Arc<UpstreamHealthRegistry>>,
    ) -> SharedHttpConnector {
        Arc::new(Self {
            https_client,
            h2c_client,
            prefer_h1: Mutex::new(HashMap::new()),
            upstream_health,
        })
    }

    fn mark_upstream_healthy(&self, spec: &HttpPeerSpec) {
        if let (Some(health), Some(namespace), Some(server)) = (
            &self.upstream_health,
            &spec.upstream_name,
            &spec.upstream_addr_str,
        ) {
            health.mark_healthy(namespace, server);
        }
    }

    fn mark_upstream_unhealthy(&self, spec: &HttpPeerSpec) {
        if let (Some(health), Some(namespace), Some(server)) = (
            &self.upstream_health,
            &spec.upstream_name,
            &spec.upstream_addr_str,
        ) {
            health.mark_unhealthy(namespace, server);
        }
    }

    fn cache_key(spec: &HttpPeerSpec) -> String {
        format!("{}://{}", spec.scheme, spec.target_host)
    }

    fn gc_prefer_h1(&self, key: &str) -> bool {
        let mut prefer_h1 = self.prefer_h1.lock();
        match prefer_h1.get(key) {
            Some(timestamp) if timestamp.elapsed() <= PREFER_H1_TTL => true,
            Some(_) => {
                prefer_h1.remove(key);
                false
            }
            None => false,
        }
    }

    fn mark_prefer_h1(&self, spec: &HttpPeerSpec) {
        let key = Self::cache_key(spec);
        let mut prefer_h1 = self.prefer_h1.lock();
        prefer_h1.retain(|_, timestamp| timestamp.elapsed() <= PREFER_H1_TTL);
        if prefer_h1.len() >= MAX_PREFER_H1_ENTRIES && !prefer_h1.contains_key(&key) {
            let oldest = prefer_h1
                .iter()
                .min_by_key(|(_, timestamp)| **timestamp)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                prefer_h1.remove(&oldest);
            }
        }
        prefer_h1.insert(key, Instant::now());
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
        downstream_protocol: Protocol,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        match downstream_protocol {
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
        let can_try_h2c = spec.scheme == "http" && matches!(spec.upstream_protocol, Protocol::H2);
        let prefer_h1 = can_try_h2c && self.gc_prefer_h1(&Self::cache_key(spec));
        let retryable_request = (can_try_h2c && !prefer_h1)
            .then(|| EmptyBodyRetryTemplate::from_request(&request))
            .flatten();
        let (parts, body) = request.into_parts();
        let boxed_body = body
            .map_frame(|f| f.map_data(Into::into))
            .map_err(|e| std::io::Error::other(e.into()))
            .boxed();
        let request = hyper::Request::from_parts(parts, boxed_body);

        let result = if can_try_h2c && !prefer_h1 {
            self.h2c_client.request(request).await
        } else {
            self.https_client.request(request).await
        };

        match result {
            Ok(resp) => {
                self.mark_upstream_healthy(spec);
                Ok(Self::box_response(resp))
            }
            Err(e) => {
                if can_try_h2c && !prefer_h1 {
                    // Phase 1 policy: when cleartext H2 fails once, pin this upstream to H1 for a TTL window.
                    self.mark_prefer_h1(spec);
                    if let Some(template) = retryable_request.as_ref() {
                        debug!(target = %spec.target_host, "cleartext h2c request failed; retrying once with H1");
                        match self.https_client.request(template.build()).await {
                            Ok(resp) => {
                                self.mark_upstream_healthy(spec);
                                return Ok(Self::box_response(resp));
                            }
                            Err(retry_err) => {
                                if retry_err.is_connect() {
                                    self.mark_upstream_unhealthy(spec);
                                }
                                return Err(ProxyError::http_upstream_request(
                                    retry_err.to_string(),
                                )
                                .into());
                            }
                        }
                    }
                    return Err(ProxyError::http_upstream_request(e.to_string()).into());
                }
                if e.is_connect() {
                    self.mark_upstream_unhealthy(spec);
                }
                Err(ProxyError::http_upstream_request(e.to_string()).into())
            }
        }
    }
}
