use anyhow::{Result, Context};
use bytes::BytesMut;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_rustls::HttpsConnector;
use httparse::{Request as HttpRequest, Status};
use std::str::FromStr;
use tracing::{debug, warn};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

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
    
    // Detect HTTP version from original request
    let http_version = tunnel_lib::http_version::HttpVersion::detect_from_request(request_bytes)?;
    debug!("Detected HTTP version: {:?}", http_version);
    
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
    response_bytes.extend_from_slice(format!("{} {} {}\r\n", 
        http_version.to_status_line_string(),
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

pub async fn forward_egress_grpc_request(
    request_bytes: &[u8],
    target_uri: &str,
    is_ssl: bool,
) -> Result<Vec<u8>> {
    debug!("Forwarding gRPC request to: {} (SSL: {})", target_uri, is_ssl);
    
    // Parse target URI
    let uri = if target_uri.starts_with("http://") || target_uri.starts_with("https://") {
        target_uri.to_string()
    } else {
        format!("{}://{}", if is_ssl { "https" } else { "http" }, target_uri)
    };
    
    let url = url::Url::parse(&uri)
        .with_context(|| format!("Invalid gRPC target URI: {}", target_uri))?;
    
    let host = url.host_str()
        .ok_or_else(|| anyhow::anyhow!("Missing host in URI: {}", target_uri))?;
    let port = url.port().unwrap_or(if is_ssl { 443 } else { 80 });
    let addr = format!("{}:{}", host, port);
    
    debug!("Connecting to gRPC endpoint: {}", addr);
    
    // Establish TCP connection
    let mut stream = TcpStream::connect(&addr).await
        .with_context(|| format!("Failed to connect to gRPC endpoint: {}", addr))?;
    
    // Forward the entire request (HTTP/2 headers + gRPC messages)
    stream.write_all(request_bytes).await?;
    stream.flush().await?;
    
    debug!("Sent gRPC request ({} bytes)", request_bytes.len());
    
    // Read response
    let mut response_buffer = BytesMut::new();
    let mut buf = vec![0u8; 4096];
    
    // Read HTTP/2 response headers
    let mut header_complete = false;
    while !header_complete && response_buffer.len() < 8192 {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        response_buffer.extend_from_slice(&buf[..n]);
        
        // Look for end of HTTP headers
        for i in 0..=response_buffer.len().saturating_sub(4) {
            if &response_buffer[i..i+4] == b"\r\n\r\n" {
                header_complete = true;
                break;
            }
        }
    }
    
    // Continue reading gRPC response messages
    loop {
        let n = match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    break;
                }
                return Err(e.into());
            }
        };
        response_buffer.extend_from_slice(&buf[..n]);
    }
    
    debug!("Received gRPC response ({} bytes)", response_buffer.len());
    
    Ok(response_buffer.to_vec())
}

pub async fn forward_egress_wss_request(
    request_bytes: &[u8],
    target_uri: &str,
    is_ssl: bool,
) -> Result<Vec<u8>> {
    debug!("Forwarding WebSocket request to: {} (SSL: {})", target_uri, is_ssl);
    
    // Parse target URI
    let ws_url = if target_uri.starts_with("ws://") || target_uri.starts_with("wss://") {
        target_uri.to_string()
    } else {
        format!("{}://{}", if is_ssl { "wss" } else { "ws" }, target_uri)
    };
    
    debug!("Connecting to WebSocket: {}", ws_url);
    
    // Connect to WebSocket backend
    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .with_context(|| format!("Failed to connect to WebSocket: {}", ws_url))?;
    
    debug!("WebSocket connected");
    
    // Parse HTTP upgrade request to extract headers
    let header_end = request_bytes.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(request_bytes.len());
    
    // Send HTTP upgrade request to backend
    if header_end > 0 {
        ws_stream.send(tokio_tungstenite::tungstenite::Message::Binary(
            request_bytes[..header_end].to_vec()
        )).await?;
    }
    
    // Read WebSocket upgrade response
    let mut response_buffer = BytesMut::new();
    
    // Read initial response (upgrade response)
    match ws_stream.next().await {
        Some(Ok(msg)) => {
            match msg {
                tokio_tungstenite::tungstenite::Message::Binary(data) => {
                    response_buffer.extend_from_slice(&data);
                }
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    response_buffer.extend_from_slice(text.as_bytes());
                }
                tokio_tungstenite::tungstenite::Message::Close(_) => {
                    debug!("WebSocket connection closed");
                }
                _ => {}
            }
        }
        Some(Err(e)) => {
            return Err(anyhow::anyhow!("WebSocket error: {}", e));
        }
        None => {
            debug!("WebSocket stream ended");
        }
    }
    
    // Note: For full bidirectional WebSocket forwarding, this should be handled
    // differently (see reverse_handler.rs for streaming implementation)
    warn!("WebSocket egress forwarding: Simplified implementation - full bidirectional streaming handled in reverse_handler");
    
    Ok(response_buffer.to_vec())
}
