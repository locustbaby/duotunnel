use super::core::Protocol;
use super::h2::serve_h2_forward;
use super::h2_proxy::EmptyBodyRetryTemplate;
use super::http::HttpPeer;
use super::peers::HttpPeerSpec;
use crate::egress::http::{H2cClient, HttpsClient};
use crate::transport::quinn_io::{PrefixedReadWrite, QuinnStream};
use crate::ProxyError;
use anyhow::Result;
use arc_swap::ArcSwap;
use bytes::Bytes;
use http_body_util::BodyExt;
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
    prefer_h1: ArcSwap<HashMap<String, Instant>>,
}

impl HttpConnector {
    pub fn new(https_client: HttpsClient, h2c_client: H2cClient) -> SharedHttpConnector {
        Arc::new(Self {
            https_client,
            h2c_client,
            prefer_h1: ArcSwap::from_pointee(HashMap::new()),
        })
    }

    fn mark_upstream_healthy(&self, spec: &HttpPeerSpec) {
        if let Some(ref b_ref) = spec.backend_ref {
            b_ref.mark_success();
        }
    }

    fn mark_upstream_unhealthy(&self, spec: &HttpPeerSpec) {
        if let Some(ref b_ref) = spec.backend_ref {
            b_ref.record_failure();
        }
    }

    fn cache_key(spec: &HttpPeerSpec) -> String {
        format!("{}://{}", spec.scheme, spec.target_host)
    }

    fn gc_prefer_h1(&self, key: &str) -> bool {
        let prefer_h1 = self.prefer_h1.load();
        if let Some(timestamp) = prefer_h1.get(key) {
            if timestamp.elapsed() <= PREFER_H1_TTL {
                return true;
            }
        }
        false
    }

    fn mark_prefer_h1(&self, spec: &HttpPeerSpec) {
        let key = Self::cache_key(spec);
        loop {
            let current = self.prefer_h1.load();
            if let Some(timestamp) = current.get(&key) {
                if timestamp.elapsed() <= PREFER_H1_TTL {
                    return;
                }
            }
            let mut new_map = (**current).clone();
            if !new_map.contains_key(&key) && new_map.len() >= MAX_PREFER_H1_ENTRIES {
                new_map.retain(|_, timestamp| timestamp.elapsed() <= PREFER_H1_TTL);
                if new_map.len() >= MAX_PREFER_H1_ENTRIES {
                    if let Some(oldest) = new_map
                        .iter()
                        .min_by_key(|(_, timestamp)| **timestamp)
                        .map(|(key, _)| key.clone())
                    {
                        new_map.remove(&oldest);
                    }
                }
            }
            new_map.insert(key.clone(), Instant::now());
            let previous = self.prefer_h1.compare_and_swap(&current, Arc::new(new_map));
            if Arc::ptr_eq(&previous, &current) {
                return;
            }
        }
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
                                    tracing::warn!(target = %spec.target_host, error = %retry_err, "HTTP connection to upstream failed on retry, marking unhealthy");
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
                    tracing::warn!(target = %spec.target_host, error = %e, "HTTP connection to upstream failed, marking unhealthy");
                    self.mark_upstream_unhealthy(spec);
                }
                Err(ProxyError::http_upstream_request(e.to_string()).into())
            }
        }
    }
}
