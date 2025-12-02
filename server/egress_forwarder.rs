use anyhow::{Result, Context};
use bytes::BytesMut;
use hyper::{Body, Client, Method, Request};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use httparse::{Request as HttpRequest, Status};
use std::str::FromStr;
use tracing::{debug, error};

/// Forward egress HTTP request using Hyper client (with connection pooling)
/// Parses frame data, converts to hyper::Request, and sends via pre-warmed connection pool
pub async fn forward_egress_http_request(
    http_client: &Client<HttpConnector, Body>,
    https_client: &Client<HttpsConnector<HttpConnector>, Body>,
    request_bytes: &[u8],
    target_uri: &str,
    is_ssl: bool,
) -> Result<Vec<u8>> {
    // 1. Parse HTTP request from frame data
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
    
    // 2. Extract method, path, headers, and body
    let method_str = req.method.unwrap_or("GET");
    let original_path = req.path.unwrap_or("/");
    
    // 3. Build target URI: use target_uri as base, preserve path and query from original request
    let uri_str = if original_path.starts_with("http://") || original_path.starts_with("https://") {
        // Full URL in path
        original_path.to_string()
    } else {
        // Relative path, combine with target_uri
        let base_uri = hyper::Uri::from_str(target_uri)
            .with_context(|| format!("Invalid target URI: {}", target_uri))?;
        let base_scheme = base_uri.scheme_str().unwrap_or(if is_ssl { "https" } else { "http" });
        let base_authority = base_uri.authority()
            .map(|a| a.as_str())
            .unwrap_or_else(|| {
                // Extract host:port from target_uri if no authority
                target_uri.trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .split('/')
                    .next()
                    .unwrap_or("localhost")
            });
        
        // Combine base with original path
        let full_path = if original_path.starts_with('/') {
            original_path.to_string()
        } else {
            format!("/{}", original_path)
        };
        
        format!("{}://{}{}", base_scheme, base_authority, full_path)
    };
    
    let uri = hyper::Uri::from_str(&uri_str)
        .with_context(|| format!("Invalid URI: {}", uri_str))?;
    
    debug!("Forwarding request: {} {} -> {} (using {} connection pool)", 
        method_str, original_path, uri_str, if is_ssl { "HTTPS" } else { "HTTP" });
    
    // 4. Convert to hyper::Request
    let method = Method::from_str(method_str)?;
    
    // Extract body
    let body_bytes = if request_bytes.len() > header_len {
        &request_bytes[header_len..]
    } else {
        &[]
    };
    
    // Build hyper::Request
    let mut builder = Request::builder()
        .method(method)
        .uri(uri);
    
    // Copy headers (skip Host and Connection as Hyper will handle them automatically)
    for header in req.headers.iter() {
        let name = header.name;
        if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("connection") {
            continue; // Skip Host and Connection headers - Hyper handles these
        }
        
        if let Ok(value) = std::str::from_utf8(header.value) {
            builder = builder.header(name, value);
        }
    }
    
    let hyper_request = builder.body(Body::from(body_bytes.to_vec()))
        .with_context(|| "Failed to build hyper request")?;
    
    // 5. Send request using pre-warmed Hyper connection pool
    // Hyper automatically reuses connections from the pool
    let response = if is_ssl {
        https_client.request(hyper_request).await
            .with_context(|| format!("Failed to send HTTPS request to {} (connection pool)", uri_str))?
    } else {
        http_client.request(hyper_request).await
            .with_context(|| format!("Failed to send HTTP request to {} (connection pool)", uri_str))?
    };
    
    debug!("Received HTTP response: {} from {}", response.status(), uri_str);
    
    // 6. Convert hyper::Response to raw HTTP response bytes
    let status = response.status();
    let (parts, body) = response.into_parts();
    
    let mut response_bytes = BytesMut::new();
    response_bytes.extend_from_slice(format!("HTTP/1.1 {} {}\r\n", 
        status.as_u16(), 
        status.canonical_reason().unwrap_or("Unknown")).as_bytes());
    
    // Write headers
    for (name, value) in &parts.headers {
        if let Ok(value_str) = value.to_str() {
            response_bytes.extend_from_slice(format!("{}: {}\r\n", name, value_str).as_bytes());
        }
    }
    response_bytes.extend_from_slice(b"\r\n");
    
    // Read body
    use hyper::body::HttpBody;
    let mut body_stream = body;
    while let Some(chunk_result) = body_stream.data().await {
        match chunk_result {
            Ok(chunk) => {
                response_bytes.extend_from_slice(&chunk);
            }
            Err(e) => {
                error!("Error reading response body: {}", e);
                return Err(e.into());
            }
        }
    }
    
    Ok(response_bytes.to_vec())
}

