use crate::egress::http::{H2cClient, HttpsClient};
use crate::transport::quinn_io::{PrefixedReadWrite, QuinnStream};
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
    https_client: HttpsClient,
    h2c_client: H2cClient,
    scheme: String,
    target_host: String,
) -> Result<()>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let use_h2c = scheme == "http";
    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
        let https_client = https_client.clone();
        let h2c_client = h2c_client.clone();
        let scheme = scheme.clone();
        let target_host = target_host.clone();
        async move {
            let (mut parts, body) = req.into_parts();
            let target_uri: hyper::Uri = format!(
                "{}://{}{}",
                scheme,
                target_host,
                parts
                    .uri
                    .path_and_query()
                    .map(|pq| pq.as_str())
                    .unwrap_or("/")
            )
            .parse()
            .unwrap();
            parts.uri = target_uri;
            if let Ok(hv) = target_host.parse() {
                parts.headers.insert(hyper::header::HOST, hv);
            }
            debug!("H2 forward: {} {}", parts.method, parts.uri);
            let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
            let upstream_req = Request::from_parts(parts, boxed_body);
            let result = if use_h2c {
                h2c_client.request(upstream_req).await
            } else {
                https_client.request(upstream_req).await
            };
            match result {
                Ok(resp) => {
                    let (parts, body) = resp.into_parts();
                    let boxed = body.map_err(std::io::Error::other).boxed_unsync();
                    Ok::<_, hyper::Error>(Response::from_parts(parts, boxed))
                }
                Err(e) => {
                    debug!(error = % e, "H2 forward: upstream request failed");
                    Ok(Response::builder()
                        .status(502)
                        .body(
                            http_body_util::Full::new(Bytes::from("Bad Gateway"))
                                .map_err(|_| unreachable!())
                                .boxed_unsync(),
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
    pub target_host: String,
    pub scheme: String,
    pub https_client: HttpsClient,
    pub h2c_client: H2cClient,
}
impl H2Peer {
    pub async fn connect_inner(
        self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        debug!(target = % self.target_host, scheme = % self.scheme, "H2 proxy starting");
        let stream = QuinnStream { send, recv };
        if let Some(init) = initial_data.filter(|b| !b.is_empty()) {
            let io = PrefixedReadWrite::new(stream, init);
            serve_h2_forward(
                io,
                self.https_client,
                self.h2c_client,
                self.scheme,
                self.target_host,
            )
            .await
        } else {
            serve_h2_forward(
                stream,
                self.https_client,
                self.h2c_client,
                self.scheme,
                self.target_host,
            )
            .await
        }
    }
}
