use super::http_connector::SharedHttpConnector;
use super::peers::HttpPeerSpec;
use crate::transport::quinn_io::{PrefixedReadWrite, QuinnStream};
use crate::ProxyError;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::server::conn::http2::Builder as H2Builder;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::debug;
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
            let target_uri: hyper::Uri = format!(
                "{}://{}{}",
                spec.scheme,
                spec.target_host,
                parts
                    .uri
                    .path_and_query()
                    .map(|pq| pq.as_str())
                    .unwrap_or("/")
            )
            .parse()
            .unwrap();
            parts.uri = target_uri;
            if let Ok(hv) = spec.target_host.parse() {
                parts.headers.insert(hyper::header::HOST, hv);
            }
            debug!("H2 forward: {} {}", parts.method, parts.uri);
            let boxed_body = body.map_err(std::io::Error::other).boxed();
            let upstream_req = Request::from_parts(parts, boxed_body);
            match connector.request(&spec, upstream_req).await {
                Ok(resp) => {
                    Ok::<_, hyper::Error>(resp)
                }
                Err(e) => {
                    let proxy_err = ProxyError::http_upstream_request(e.to_string());
                    debug!(
                        kind = ?proxy_err.kind,
                        retry = ?proxy_err.retry,
                        error = %proxy_err,
                        "H2 forward: upstream request failed"
                    );
                    Ok(Response::builder()
                        .status(proxy_err.http_status().unwrap_or(hyper::StatusCode::BAD_GATEWAY))
                        .body(
                            http_body_util::Full::new(Bytes::from("Bad Gateway"))
                                .map_err(|never| match never {})
                                .boxed(),
                        )
                        .unwrap())
                }
            }
        }
    });
    H2Builder::new(TokioExecutor::new())
        .max_concurrent_streams(None::<u32>)
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
