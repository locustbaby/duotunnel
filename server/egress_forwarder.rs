use anyhow::{Result, Context};
use bytes::BytesMut;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_rustls::HttpsConnector;
use httparse::{Request as HttpRequest, Status};
use std::str::FromStr;
use tracing::debug;

pub async fn forward_egress_http_request(
    client: &Client<HttpsConnector<HttpConnector>, http_body_util::Full<bytes::Bytes>>,
    request_bytes: &[u8],
    target_uri: &str,
    _is_ssl: bool,
) -> Result<Vec<u8>> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = HttpRequest::new(&mut headers);
    
    let parse_result = req.parse(request_bytes)?;
    let header_len = match parse_result {
        Status::Complete(len) => {
            debug!("Parsed HTTP request: {} {}", req.method.unwrap_or(""), req.path.unwrap_or(""));
            len
        }
        Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    };
    
    let method_str = req.method.unwrap_or("GET");
    let original_path = req.path.unwrap_or("/");
    
    let uri_str = if original_path.starts_with("http://") || original_path.starts_with("https://") {
        original_path.to_string()
    } else {
        let base_uri = hyper::Uri::from_str(target_uri)
            .with_context(|| format!("Invalid target URI: {}", target_uri))?;
        let base_scheme = base_uri.scheme_str().unwrap_or("http");
        let base_authority = base_uri.authority()
            .map(|a| a.as_str())
            .unwrap_or("localhost");
        
        let full_path = if original_path.starts_with('/') {
            original_path.to_string()
        } else {
            format!("/{}", original_path)
        };
        
        format!("{}://{}{}", base_scheme, base_authority, full_path)
    };
    
    let uri = hyper::Uri::from_str(&uri_str)
        .with_context(|| format!("Invalid URI: {}", uri_str))?;
    
    debug!("Forwarding request: {} {} -> {}", method_str, original_path, uri_str);
    
    let method = hyper::Method::from_str(method_str)?;
    
    let body_bytes = if request_bytes.len() > header_len {
        &request_bytes[header_len..]
    } else {
        &[]
    };
    
    let mut builder = hyper::Request::builder()
        .method(method)
        .uri(uri);
    
    for header in req.headers.iter() {
        let name = header.name;
        if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("connection") {
            continue;
        }
        
        if let Ok(value) = std::str::from_utf8(header.value) {
            builder = builder.header(name, value);
        }
    }
    
    let hyper_request = builder.body(http_body_util::Full::new(bytes::Bytes::copy_from_slice(body_bytes)))
        .with_context(|| "Failed to build hyper request")?;
    
    let response = client.request(hyper_request).await
        .with_context(|| format!("Failed to send request to {}", uri_str))?;
    
    debug!("Received HTTP response: {} from {}", response.status(), uri_str);
    
    let status = response.status();
    let (parts, body) = response.into_parts();
    
    use http_body_util::BodyExt;
    let mut body_bytes = BytesMut::new();
    let mut body_stream = body;
    while let Some(chunk_result) = body_stream.frame().await {
        if let Ok(frame) = chunk_result {
            if let Some(chunk) = frame.data_ref() {
                body_bytes.extend_from_slice(chunk);
            }
        }
    }
    
    let mut response_bytes = BytesMut::new();
    response_bytes.extend_from_slice(format!("HTTP/1.1 {} {}\r\n", 
        status.as_u16(), 
        status.canonical_reason().unwrap_or("Unknown")).as_bytes());
    
    for (name, value) in &parts.headers {
        if name.as_str().eq_ignore_ascii_case("transfer-encoding") || 
           name.as_str().eq_ignore_ascii_case("content-length") {
            continue;
        }
        if let Ok(value_str) = value.to_str() {
            response_bytes.extend_from_slice(format!("{}: {}\r\n", name, value_str).as_bytes());
        }
    }
    
    response_bytes.extend_from_slice(format!("content-length: {}\r\n", body_bytes.len()).as_bytes());
    response_bytes.extend_from_slice(b"\r\n");
    response_bytes.extend_from_slice(&body_bytes);
    
    Ok(response_bytes.to_vec())
}
