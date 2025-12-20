use anyhow::{Result, Context};
use bytes::BytesMut;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_rustls::HttpsConnector;
use httparse::{Request as HttpRequest, Status};
use std::str::FromStr;
use tracing::{debug, info, error, warn};
use quinn::{RecvStream, SendStream};
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, read_frame, write_frame};
use std::time::Duration;

/// Stream-based HTTP forwarding that leverages QUIC's multiplexing
/// Reads request header first, then streams body bidirectionally
/// This takes advantage of QUIC's ability to handle multiple concurrent streams
pub async fn forward_http_streaming(
    client: &Client<HttpsConnector<HttpConnector>, http_body_util::Full<bytes::Bytes>>,
    mut recv: RecvStream,
    mut send: SendStream,
    target_uri: &str,
    session_id: u64,
    request_id: &str,
) -> Result<()> {
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
    const MAX_HEADER_SIZE: usize = 8192;
    
    // Step 1: Read HTTP request header (must be complete)
    let mut header_buffer = BytesMut::new();
    let mut header_complete = false;
    let mut header_end_pos = 0;
    
    // Read frames until we have complete headers
    while !header_complete && header_buffer.len() < MAX_HEADER_SIZE {
        let frame = match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(REQUEST_TIMEOUT)).await {
            Ok(f) => {
                if f.session_id != session_id {
                    warn!("[{}] Mismatched session_id in header frame", request_id);
                    continue;
                }
                f
            }
            Err(e) => {
                error!("[{}] Error reading header frame: {}", request_id, e);
                return Err(e);
            }
        };
        
        header_buffer.extend_from_slice(&frame.payload);
        
        // Look for end of HTTP headers (\r\n\r\n)
        for i in 0..=header_buffer.len().saturating_sub(4) {
            if &header_buffer[i..i+4] == b"\r\n\r\n" {
                header_complete = true;
                header_end_pos = i + 4;
                break;
            }
        }
        
        // If this was the last frame and we still don't have headers, break
        if frame.end_of_stream && !header_complete {
            anyhow::bail!("[{}] Stream ended before HTTP headers complete", request_id);
        }
    }
    
    if !header_complete {
        anyhow::bail!("[{}] HTTP headers incomplete or too large", request_id);
    }
    
    info!("[{}] Received HTTP headers ({} bytes)", request_id, header_end_pos);
    
    // Step 2: Parse headers
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = HttpRequest::new(&mut headers);
    let parse_result = req.parse(&header_buffer[..header_end_pos])?;
    let header_len = match parse_result {
        Status::Complete(len) => {
            debug!("[{}] Parsed HTTP request: {} {}", 
                request_id, req.method.unwrap_or(""), req.path.unwrap_or(""));
            len
        }
        Status::Partial => {
            anyhow::bail!("[{}] Incomplete HTTP headers", request_id);
        }
    };
    
    // Step 3: Build request URI
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
    
    debug!("[{}] Forwarding streaming request: {} {} -> {}", 
        request_id, method_str, original_path, uri_str);
    
    // Step 4: Collect remaining body frames (streaming read)
    let mut body_buffer = BytesMut::new();
    let mut request_complete = false;
    
    // Check if there's body data already in header buffer
    if header_buffer.len() > header_end_pos {
        body_buffer.extend_from_slice(&header_buffer[header_end_pos..]);
    }
    
    // Read remaining body frames
    while !request_complete {
        let frame = match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(REQUEST_TIMEOUT)).await {
            Ok(f) => {
                if f.session_id != session_id {
                    warn!("[{}] Mismatched session_id in body frame", request_id);
                    continue;
                }
                f
            }
            Err(e) => {
                if e.to_string().contains("timeout") {
                    error!("[{}] Body read timeout", request_id);
                } else {
                    error!("[{}] Error reading body frame: {}", request_id, e);
                }
                return Err(e);
            }
        };
        
        body_buffer.extend_from_slice(&frame.payload);
        request_complete = frame.end_of_stream;
        
        if request_complete {
            info!("[{}] Received complete request body ({} bytes)", request_id, body_buffer.len());
        }
    }
    
    // Step 5: Build and send HTTP request
    let method = hyper::Method::from_str(method_str)?;
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
    
    let hyper_request = builder.body(http_body_util::Full::new(bytes::Bytes::copy_from_slice(&body_buffer)))
        .with_context(|| "Failed to build hyper request")?;
    
    // Step 6: Send request and get response (streaming)
    let response = client.request(hyper_request).await
        .with_context(|| format!("Failed to send request to {}", uri_str))?;
    
    info!("[{}] Received HTTP response: {} from {}", request_id, response.status(), uri_str);
    
    // Step 7: Stream response back (chunked)
    let http_version = tunnel_lib::http_version::HttpVersion::detect_from_request(&header_buffer[..header_end_pos])
        .unwrap_or(tunnel_lib::http_version::HttpVersion::Http11);
    
    let status = response.status();
    let (parts, body) = response.into_parts();
    
    // Build and send response header
    let mut response_header = BytesMut::new();
    response_header.extend_from_slice(format!("{} {} {}\r\n", 
        http_version.to_status_line_string(),
        status.as_u16(), 
        status.canonical_reason().unwrap_or("Unknown")).as_bytes());
    
    for (name, value) in &parts.headers {
        if name.as_str().eq_ignore_ascii_case("transfer-encoding") || 
           name.as_str().eq_ignore_ascii_case("content-length") {
            continue;
        }
        if let Ok(value_str) = value.to_str() {
            response_header.extend_from_slice(format!("{}: {}\r\n", name, value_str).as_bytes());
        }
    }
    
    // Stream response body
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
    
    response_header.extend_from_slice(format!("content-length: {}\r\n", body_bytes.len()).as_bytes());
    response_header.extend_from_slice(b"\r\n");
    
    // Send response header frame
    let header_frame = TunnelFrame::new(
        session_id,
        ProtocolType::Http11,
        false,
        response_header.to_vec(),
    );
    write_frame(&mut send, &header_frame).await?;
    
    // Send response body in chunks (streaming)
    const MAX_FRAME_SIZE: usize = 64 * 1024;
    let mut offset = 0;
    
    while offset < body_bytes.len() {
        let chunk_size = std::cmp::min(MAX_FRAME_SIZE, body_bytes.len() - offset);
        let chunk = body_bytes[offset..offset + chunk_size].to_vec();
        let is_last = offset + chunk_size >= body_bytes.len();
        
        let body_frame = TunnelFrame::new(
            session_id,
            ProtocolType::Http11,
            is_last,
            chunk,
        );
        
        if let Err(e) = write_frame(&mut send, &body_frame).await {
            error!("[{}] Failed to write response body frame: {}", request_id, e);
            return Err(e.into());
        }
        
        offset += chunk_size;
    }
    
    info!("[{}] Sent streaming response ({} bytes) in chunks", request_id, body_bytes.len());
    
    Ok(())
}

