use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Request, Method, Uri, HeaderMap, header};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use hyper_rustls::HttpsConnectorBuilder;
use quinn::{SendStream, RecvStream};
use tracing::{info, debug};
use std::time::Duration;

use crate::transport::addr::parse_upstream;
use crate::protocol::rewrite::Rewriter;

type HttpsClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>>;

pub fn create_https_client() -> HttpsClient {
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .unwrap()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();

    Client::builder(TokioExecutor::new())
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .build(https)
}

pub async fn forward_http(
    mut recv: RecvStream,
    mut send: SendStream,
    upstream_addr: &str,
    client: &HttpsClient,
    rewriter: Option<&Rewriter>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    let parsed = parse_upstream(upstream_addr);

    let mut first_buf = vec![0u8; 8192];
    let n = match recv.read(&mut first_buf).await? {
        Some(n) => n,
        None => return Ok(()),
    };
    let first_chunk = &first_buf[..n];

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    let parse_result = req.parse(first_chunk)?;

    let method = req.method.ok_or_else(|| anyhow::anyhow!("no method"))?;
    let mut path = req.path.ok_or_else(|| anyhow::anyhow!("no path"))?.to_string();

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

    let hyper_method = match method {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        "PATCH" => Method::PATCH,
        _ => Method::GET,
    };

    let body_start = parse_result.unwrap();
    let remaining_first = &first_chunk[body_start..];

    let content_length: usize = hyper_headers
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let req_body_stream: std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<hyper::body::Frame<Bytes>, std::io::Error>> + Send>> = if content_length > 0 {
        let stream = futures_util::stream::try_unfold(
            (recv, Some(Bytes::copy_from_slice(remaining_first)), 0usize, content_length),
            |(mut recv, first_chunk, mut read, total)| async move {
                if let Some(chunk) = first_chunk {
                    let len = chunk.len();
                    if len > 0 {
                        read += len;
                        return Ok(Some((hyper::body::Frame::data(chunk), (recv, None, read, total))));
                    }
                }

                if read >= total {
                    return Ok(None);
                }

                let remaining = total - read;
                let to_read = 8192.min(remaining);
                let mut buf = vec![0u8; to_read];

                match recv.read(&mut buf).await {
                    Ok(Some(n)) if n > 0 => {
                        read += n;
                        let chunk = Bytes::copy_from_slice(&buf[..n]);
                        Ok(Some((hyper::body::Frame::data(chunk), (recv, None, read, total))))
                    }
                    Ok(_) => Ok(None),
                    Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
                }
            },
        );
        Box::pin(stream)
    } else {
        Box::pin(futures_util::stream::empty())
    };

    let req_body = http_body_util::StreamBody::new(req_body_stream);
    let req_body = http_body_util::combinators::UnsyncBoxBody::new(req_body);

    let mut request_builder = Request::builder()
        .method(hyper_method)
        .uri(uri);

    if let Some(headers) = request_builder.headers_mut() {
        *headers = hyper_headers;
    }

    let request = request_builder.body(req_body)?;
    let response = client.request(request).await?;

    info!(
        status = response.status().as_u16(),
        elapsed_ms = start_time.elapsed().as_millis(),
        "HTTP response received"
    );

    let status_line = format!("HTTP/1.1 {} {}\r\n", 
        response.status().as_u16(), 
        response.status().canonical_reason().unwrap_or("OK")
    );
    send.write_all(status_line.as_bytes()).await?;

    for (name, value) in response.headers() {
        let header_line = format!("{}: {}\r\n", name, value.to_str()?);
        send.write_all(header_line.as_bytes()).await?;
    }
    send.write_all(b"\r\n").await?;

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
                debug!(error = %e, "error reading response");
                break;
            }
            None => break,
        }
    }

    debug!(bytes = total_bytes, "response complete");
    send.finish()?;
    Ok(())
}
