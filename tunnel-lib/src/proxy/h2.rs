use super::http_connector::SharedHttpConnector;
use super::peers::HttpPeerSpec;
use crate::transport::quinn_io::{PrefixedReadWrite, QuinnStream};
use crate::ProxyError;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::server::conn::http1::Builder as H1Builder;
use hyper::server::conn::http2::Builder as H2Builder;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use quinn::{RecvStream, SendStream};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::debug;

// Anti-abuse bounds for H2 servers that accept downstream connections
// (rapid-reset / stream-flood / header-flood mitigation, CVE-2023-44487 class).
// Explicit values rather than hyper defaults: defaults drift across versions
// and the safety property must not depend on them.
//
// 200 pins hyper 1.x's current server default. It must not be raised above it:
// every call site here previously inherited that default, so a larger number
// would widen the stream-flood budget instead of bounding it.
pub const H2_SERVER_MAX_CONCURRENT_STREAMS: u32 = 200;
pub const H2_SERVER_MAX_HEADER_LIST_SIZE: u32 = 16 * 1024;
// hyper/h2's current defaults, pinned so a future default change cannot
// silently widen the reset-stream budget.
pub const H2_SERVER_MAX_PENDING_ACCEPT_RESET_STREAMS: usize = 20;
pub const H2_SERVER_MAX_LOCAL_ERROR_RESET_STREAMS: usize = 1024;
pub const H2_SERVER_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(30);
pub const H2_SERVER_KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(20);

// Bounds for the HTTP/1.1 fallback served to downstream peers that negotiate
// `http/1.1` over ALPN (or send no ALPN at all). Same intent as the H2 bounds
// above: header-flood and slowloris budgets pinned here, not inherited.
pub const H1_SERVER_MAX_HEADERS: usize = 100;
pub const H1_SERVER_MAX_BUF_SIZE: usize = 64 * 1024;
pub const H1_SERVER_HEADER_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// H2 server builder with explicit anti-abuse bounds. Use this for every
/// H2 connection accepted from a downstream peer instead of a bare
/// `H2Builder::new`.
pub fn hardened_h2_server_builder<E>(exec: E) -> H2Builder<E> {
    let mut builder = H2Builder::new(exec);
    builder
        .max_concurrent_streams(H2_SERVER_MAX_CONCURRENT_STREAMS)
        .max_header_list_size(H2_SERVER_MAX_HEADER_LIST_SIZE)
        .max_pending_accept_reset_streams(H2_SERVER_MAX_PENDING_ACCEPT_RESET_STREAMS)
        .max_local_error_reset_streams(H2_SERVER_MAX_LOCAL_ERROR_RESET_STREAMS)
        // keep-alive needs a timer; without one hyper panics at runtime.
        .timer(TokioTimer::new())
        .keep_alive_interval(H2_SERVER_KEEP_ALIVE_INTERVAL)
        .keep_alive_timeout(H2_SERVER_KEEP_ALIVE_TIMEOUT);
    builder
}

/// HTTP/1.1 server builder with explicit anti-abuse bounds, for downstream
/// peers that did not negotiate `h2`. Lives beside the H2 builder so both
/// hardening policies stay in one place.
pub fn hardened_h1_server_builder() -> H1Builder {
    let mut builder = H1Builder::new();
    builder
        // Costs a per-request heap allocation for the header slots; acceptable
        // on the legacy-client fallback path.
        .max_headers(H1_SERVER_MAX_HEADERS)
        .max_buf_size(H1_SERVER_MAX_BUF_SIZE)
        // header_read_timeout needs a timer; without one hyper panics at runtime.
        .timer(TokioTimer::new())
        .header_read_timeout(H1_SERVER_HEADER_READ_TIMEOUT);
    builder
}

pub async fn serve_h2_forward<IO>(
    io: IO,
    connector: SharedHttpConnector,
    spec: HttpPeerSpec,
) -> Result<()>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
        let connector = connector.clone();
        let spec = spec.clone();
        async move {
            let (mut parts, body) = req.into_parts();
            let target_uri = match format!(
                "{}://{}{}",
                spec.scheme,
                spec.target_host,
                parts
                    .uri
                    .path_and_query()
                    .map(|pq| pq.as_str())
                    .unwrap_or("/")
            )
            .parse::<hyper::Uri>()
            {
                Ok(uri) => uri,
                Err(error) => {
                    let body = http_body_util::Full::new(Bytes::from(format!(
                        "invalid request target: {error}"
                    )))
                    .map_err(|never| match never {})
                    .boxed();
                    return Ok(Response::builder()
                        .status(hyper::StatusCode::BAD_REQUEST)
                        .header(hyper::header::CONTENT_TYPE, "text/plain")
                        .body(body)
                        .expect("static response builder"));
                }
            };
            parts.uri = target_uri;
            if let Ok(hv) = spec.target_host.parse() {
                parts.headers.insert(hyper::header::HOST, hv);
            }
            debug!("H2 forward: {} {}", parts.method, parts.uri);
            let boxed_body = body.map_err(std::io::Error::other).boxed();
            let upstream_req = Request::from_parts(parts, boxed_body);
            match connector.request(&spec, upstream_req).await {
                Ok(resp) => Ok::<_, hyper::Error>(resp),
                Err(e) => {
                    let proxy_err = ProxyError::http_upstream_request(e.to_string());
                    debug!(
                        kind = ?proxy_err.kind,
                        retry = ?proxy_err.retry(),
                        error = %proxy_err,
                        "H2 forward: upstream request failed"
                    );
                    let payload = serde_json::json!({
                        "error_code": proxy_err.error_code(),
                        "message": proxy_err.to_string(),
                    });
                    let body_bytes = Bytes::from(serde_json::to_vec(&payload).unwrap_or_default());

                    Ok(Response::builder()
                        .status(
                            proxy_err
                                .http_status()
                                .unwrap_or(hyper::StatusCode::BAD_GATEWAY),
                        )
                        .header(hyper::header::CONTENT_TYPE, "application/json")
                        .body(
                            http_body_util::Full::new(body_bytes)
                                .map_err(|never| match never {})
                                .boxed(),
                        )
                        .expect("failed to build default response"))
                }
            }
        }
    });
    hardened_h2_server_builder(TokioExecutor::new())
        .serve_connection(TokioIo::new(io), service)
        .await
        .map_err(|e| anyhow::anyhow!("H2 connection error: {}", e))?;
    Ok(())
}
pub struct H2Peer {
    pub connector: SharedHttpConnector,
    pub spec: HttpPeerSpec,
}
impl H2Peer {
    pub async fn connect_inner(
        self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        debug!(target = % self.spec.target_host, scheme = % self.spec.scheme, "H2 proxy starting");
        let stream = QuinnStream { send, recv };
        if let Some(init) = initial_data.filter(|b| !b.is_empty()) {
            let io = PrefixedReadWrite::new(stream, init);
            serve_h2_forward(io, self.connector, self.spec).await
        } else {
            serve_h2_forward(stream, self.connector, self.spec).await
        }
    }
}
