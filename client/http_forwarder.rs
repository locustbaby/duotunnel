use anyhow::{Result, Context};
use bytes::BytesMut;
use hyper::{Body, Client};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use httparse::{Request, Status};
use std::str::FromStr;
use tracing::debug;

/// Forward HTTP request using Hyper client (with connection pooling)
pub async fn forward_http_request(
    http_client: &Client<HttpConnector, Body>,
    https_client: &Client<HttpsConnector<HttpConnector>, Body>,
    request_bytes: &[u8],
    target_uri: &str,
    is_ssl: bool,
) -> Result<Vec<u8>> {
    // 1. Parse HTTP request from bytes
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    
    let parse_result = req.parse(request_bytes)?;
    match parse_result {
        Status::Complete(_header_len) => {
            debug!("Parsed HTTP request: {} {}", req.method.unwrap_or(""), req.path.unwrap_or(""));
        }
        Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // 2. Extract method, path, headers, and body
    let method_str = req.method.unwrap_or("GET");
    
    // Use target_uri as base, but preserve path and query from original request
    let original_path = req.path.unwrap_or("/");
    let uri_str = if original_path.starts_with("http://") || original_path.starts_with("https://") {
        // Full URL in path
        original_path.to_string()
    } else {
        // Relative path, combine with target_uri
        let base_uri = hyper::Uri::from_str(target_uri)?;
        let base_path = base_uri.path();
        let base_scheme = base_uri.scheme_str().unwrap_or("http");
        let base_authority = base_uri.authority()
            .map(|a| a.as_str())
            .unwrap_or("localhost");
        
        // Combine base with original path
        let full_path = if original_path.starts_with('/') {
            original_path.to_string()
        } else {
            format!("{}/{}", base_path.trim_end_matches('/'), original_path)
        };
        
        format!("{}://{}{}", base_scheme, base_authority, full_path)
    };
    
    let uri = hyper::Uri::from_str(&uri_str)?;
    
    // 3. Build hyper::Request
    let mut builder = hyper::Request::builder()
        .method(method_str)
        .uri(uri);
    
    // Copy headers (skip Host header as it will be set by Hyper)
    for header in req.headers.iter() {
        let name = header.name;
        if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("connection") {
            continue; // Skip Host and Connection headers
        }
        
        let value = std::str::from_utf8(header.value)
            .with_context(|| format!("Invalid header value for {}", name))?;
        
        builder = builder.header(name, value);
    }
    
    // Extract body
    let header_len = parse_result.unwrap();
    let body_bytes = if request_bytes.len() > header_len {
        &request_bytes[header_len..]
    } else {
        &[]
    };
    
    let body = Body::from(body_bytes.to_vec());
    let hyper_request = builder.body(body)?;
    
    debug!("Sending HTTP request: {} {} (SSL: {})", hyper_request.method(), hyper_request.uri(), is_ssl);
    
    // 4. Send request using Hyper client (HTTP or HTTPS) - clients have built-in connection pooling
    let response = if is_ssl {
        https_client.request(hyper_request).await
            .with_context(|| "Failed to send HTTPS request")?
    } else {
        http_client.request(hyper_request).await
            .with_context(|| "Failed to send HTTP request")?
    };
    
    debug!("Received HTTP response: {}", response.status());
    
    // 5. Convert hyper::Response to raw HTTP response bytes
    let status_line = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("Unknown")
    );
    
    let mut response_bytes = BytesMut::new();
    response_bytes.extend_from_slice(status_line.as_bytes());
    
    // Write headers
    for (name, value) in response.headers() {
        let header_line = format!("{}: {}\r\n", name, value.to_str().unwrap_or(""));
        response_bytes.extend_from_slice(header_line.as_bytes());
    }
    
    // End of headers
    response_bytes.extend_from_slice(b"\r\n");
    
    // Read body
    let body = response.into_body();
    use hyper::body::HttpBody;
    let mut body_bytes = Vec::new();
    let mut body_stream = body;
    while let Some(chunk) = body_stream.data().await {
        let chunk = chunk?;
        body_bytes.extend_from_slice(&chunk);
    }
    
    response_bytes.extend_from_slice(&body_bytes);
    
    Ok(response_bytes.to_vec())
}

