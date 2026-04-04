use crate::protocol::rewrite::Rewriter;
use crate::transport::addr::parse_upstream;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{header, HeaderMap, Method, Request, Uri};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use quinn::{RecvStream, SendStream};
use std::time::Duration;
use tracing::debug;
pub type HttpsClient = Client<
    hyper_rustls::HttpsConnector<HttpConnector>,
    http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>,
>;
pub type H2cClient =
    Client<HttpConnector, http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>>;
#[derive(Debug, Clone)]
pub struct HttpClientParams {
    pub pool_idle_timeout_secs: u64,
    pub pool_max_idle_per_host: usize,
}
impl Default for HttpClientParams {
    fn default() -> Self {
        Self {
            pool_idle_timeout_secs: 90,
            pool_max_idle_per_host: 10,
        }
    }
}
pub fn create_https_client() -> HttpsClient {
    create_https_client_with(&HttpClientParams::default())
}
pub fn create_https_client_with(params: &HttpClientParams) -> HttpsClient {
    let mut http = HttpConnector::new();
    http.set_nodelay(true);
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .unwrap()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .wrap_connector(http);
    Client::builder(TokioExecutor::new())
        .pool_idle_timeout(Duration::from_secs(params.pool_idle_timeout_secs))
        .pool_max_idle_per_host(params.pool_max_idle_per_host)
        .build(https)
}
pub fn create_h2c_client() -> H2cClient {
    create_h2c_client_with(&HttpClientParams::default())
}
pub fn create_h2c_client_with(params: &HttpClientParams) -> H2cClient {
    let mut http = HttpConnector::new();
    http.set_nodelay(true);
    Client::builder(TokioExecutor::new())
        .http2_only(true)
        .pool_idle_timeout(Duration::from_secs(params.pool_idle_timeout_secs))
        .pool_max_idle_per_host(params.pool_max_idle_per_host)
        .build(http)
}
pub async fn forward_http(
    mut recv: RecvStream,
    mut send: SendStream,
    upstream_addr: &str,
    client: &HttpsClient,
    rewriter: Option<&Rewriter>,
) -> Result<()> {
    use bytes::BytesMut;
    let start_time = std::time::Instant::now();
    let parsed = parse_upstream(upstream_addr);

    const MAX_HEADER_BUF: usize = 64 * 1024;
    let mut head_buf = BytesMut::with_capacity(8192);

    let header_end = loop {
        let mut tmp = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut tmp);
        match req.parse(&head_buf) {
            Ok(httparse::Status::Complete(n)) => break n,
            Ok(httparse::Status::Partial) => {
                if head_buf.len() >= MAX_HEADER_BUF {
                    return Err(anyhow::anyhow!(
                        "HTTP request headers exceed {} bytes",
                        MAX_HEADER_BUF
                    ));
                }
                let old_len = head_buf.len();
                let spare = (MAX_HEADER_BUF - old_len).min(8192);
                head_buf.resize(old_len + spare, 0);
                match recv.read(&mut head_buf[old_len..]).await? {
                    Some(n) => head_buf.truncate(old_len + n),
                    None => {
                        head_buf.truncate(old_len);
                        if old_len == 0 {
                            return Ok(());
                        }
                        return Err(anyhow::anyhow!("EOF in request headers"));
                    }
                }
            }
            Err(e) => return Err(e.into()),
        }
    };

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(&head_buf)?;

    let method = req.method.ok_or_else(|| anyhow::anyhow!("no method"))?;
    let mut path = req
        .path
        .ok_or_else(|| anyhow::anyhow!("no path"))?
        .to_string();
    let mut hyper_headers = HeaderMap::new();
    for h in req.headers.iter() {
        if !h.name.is_empty() {
            if let Ok(name) = hyper::header::HeaderName::from_bytes(h.name.as_bytes()) {
                if let Ok(value) = hyper::header::HeaderValue::from_bytes(h.value) {
                    hyper_headers.insert(name, value);
                }
            }
        }
    }
    if let Some(rewriter) = rewriter {
        rewriter.apply(&mut hyper_headers, &mut path);
    }
    if let Ok(host_value) = hyper::header::HeaderValue::from_str(&parsed.host) {
        hyper_headers.insert(header::HOST, host_value);
    }
    let scheme = if parsed.is_https { "https" } else { "http" };
    let uri: Uri = format!("{}://{}{}", scheme, parsed.host, path).parse()?;
    let hyper_method = Method::from_bytes(method.as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid HTTP method: {}", method))?;

    let remaining_first = head_buf.split_off(header_end).freeze();
    drop(head_buf);
    let content_length: usize = hyper_headers
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let req_body_stream: std::pin::Pin<
        Box<
            dyn futures_util::Stream<Item = Result<hyper::body::Frame<Bytes>, std::io::Error>>
                + Send,
        >,
    > = if content_length > 0 {
        let stream = futures_util::stream::try_unfold(
            (
                recv,
                Some(remaining_first),
                0usize,
                content_length,
                bytes::BytesMut::with_capacity(8192),
            ),
            |(mut recv, first_chunk, mut read, total, mut buf)| async move {
                if let Some(chunk) = first_chunk {
                    let len = chunk.len();
                    if len > 0 {
                        read += len;
                        return Ok(Some((
                            hyper::body::Frame::data(chunk),
                            (recv, None, read, total, buf),
                        )));
                    }
                }
                if read >= total {
                    return Ok(None);
                }
                let remaining = total - read;
                let to_read = 8192.min(remaining);
                buf.clear();
                buf.resize(to_read, 0);
                match recv.read(&mut buf[..to_read]).await {
                    Ok(Some(n)) if n > 0 => {
                        read += n;
                        buf.truncate(n);
                        let chunk = buf.split().freeze();
                        Ok(Some((
                            hyper::body::Frame::data(chunk),
                            (recv, None, read, total, buf),
                        )))
                    }
                    Ok(_) => Ok(None),
                    Err(e) => Err(std::io::Error::other(e)),
                }
            },
        );
        Box::pin(stream)
    } else {
        Box::pin(futures_util::stream::empty())
    };
    let req_body = http_body_util::StreamBody::new(req_body_stream);
    let req_body = http_body_util::combinators::UnsyncBoxBody::new(req_body);
    let mut request_builder = Request::builder().method(hyper_method).uri(uri);
    if let Some(headers) = request_builder.headers_mut() {
        *headers = hyper_headers;
    }
    let request = request_builder.body(req_body)?;
    let response = client.request(request).await?;
    debug!(
        status = response.status().as_u16(),
        elapsed_ms = start_time.elapsed().as_millis(),
        "HTTP response received"
    );
    {
        use std::fmt::Write as FmtWrite;
        let status = response.status();
        let headers = response.headers();
        let mut header_buf = bytes::BytesMut::with_capacity(32 + headers.len() * 48 + 4);
        write!(
            header_buf,
            "HTTP/1.1 {} {}\r\n",
            status.as_u16(),
            status.canonical_reason().unwrap_or("OK")
        )
        .unwrap();
        for (name, value) in headers {
            header_buf.extend_from_slice(name.as_str().as_bytes());
            header_buf.extend_from_slice(b": ");
            header_buf.extend_from_slice(value.as_bytes());
            header_buf.extend_from_slice(b"\r\n");
        }
        header_buf.extend_from_slice(b"\r\n");
        send.write_all(&header_buf).await?;
    }
    let mut body = response.into_body();
    let mut total_bytes = 0;
    loop {
        match body.frame().await {
            Some(Ok(frame)) => {
                if let Some(chunk) = frame.data_ref() {
                    total_bytes += chunk.len();
                    send.write_all(chunk).await?;
                }
            }
            Some(Err(e)) => {
                debug!(error = % e, "error reading response");
                break;
            }
            None => break,
        }
    }
    debug!(bytes = total_bytes, "response complete");
    send.finish()?;
    Ok(())
}
