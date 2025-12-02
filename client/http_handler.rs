use anyhow::{Result, Context};
use bytes::{BytesMut, BufMut};
use httparse::{Request, Response, Status};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::{debug, warn};
use tunnel_lib::proto::tunnel::{Rule, Upstream};

/// Parse HTTP request headers from a TCP stream
/// Returns (headers_bytes, original_host, modified_host, final_target_addr, is_target_ssl)
pub async fn parse_and_modify_http_request(
    socket: &mut TcpStream,
    rules: &[Rule],
    upstreams: &[Upstream],
    original_client_ip: String,
) -> Result<(BytesMut, Option<String>, Option<String>, String, bool)> {
    // Read HTTP request headers (up to first \r\n\r\n)
    let mut buffer = BytesMut::new();
    let mut header_end = false;
    let mut header_bytes_read = 0;
    
    // Read until we find \r\n\r\n (end of headers)
    while !header_end {
        let mut buf = vec![0u8; 4096];
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            anyhow::bail!("Connection closed before headers");
        }
        
        buffer.extend_from_slice(&buf[..n]);
        header_bytes_read += n;
        
        // Check for end of headers
        if buffer.len() >= 4 {
            for i in 0..=buffer.len().saturating_sub(4) {
                if &buffer[i..i+4] == b"\r\n\r\n" {
                    header_end = true;
                    break;
                }
            }
        }
        
        // Safety limit: prevent reading too much
        if header_bytes_read > 8192 {
            anyhow::bail!("HTTP headers too large");
        }
    }
    
    // Find the end of headers
    let header_end_pos = buffer.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(buffer.len());
    
    let header_bytes = buffer.split_to(header_end_pos);
    let remaining_body = buffer;
    
    // Parse HTTP headers
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    
    let parse_result = req.parse(&header_bytes)?;
    match parse_result {
        Status::Complete(_header_len) => {
            debug!("Parsed HTTP request: {} {}", req.method.unwrap_or(""), req.path.unwrap_or(""));
        }
        Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Extract original Host header
    let original_host = req.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case("host"))
        .and_then(|h| std::str::from_utf8(h.value).ok())
        .map(|s| s.to_string());
    
    debug!("Original Host: {:?}", original_host);
    
    // Match rule
    let matched_rule = match_rule(rules, &req, &original_host)?;
    
    let (modified_host, final_target_addr, is_target_ssl) = if let Some(rule) = matched_rule {
        debug!("Matched rule: {}", rule.rule_id);
        
        let modified_host = if !rule.action_set_host.is_empty() {
            Some(rule.action_set_host.clone())
        } else {
            original_host.clone()
        };
        
        // Resolve upstream or use direct address
        let (final_target, is_ssl) = resolve_upstream(&rule.action_proxy_pass, upstreams)?;
        
        (modified_host, final_target, is_ssl)
    } else {
        warn!("No matching rule found, using default");
        // Default: use original host as target
        let default_target = original_host.as_ref()
            .map(|h| h.clone())
            .unwrap_or_else(|| "127.0.0.1:80".to_string());
        (original_host.clone(), default_target, false)
    };
    
    // Modify Host header if needed
    let mut modified_header_bytes = BytesMut::new();
    
    // Write request line
    if let Some(method) = req.method {
        modified_header_bytes.put_slice(method.as_bytes());
        modified_header_bytes.put_slice(b" ");
    }
    if let Some(path) = req.path {
        modified_header_bytes.put_slice(path.as_bytes());
        modified_header_bytes.put_slice(b" ");
    }
    modified_header_bytes.put_slice(b"HTTP/1.1\r\n");
    
    // Write headers, modifying Host if needed
    for header in req.headers.iter() {
        let header_name = header.name;
        let header_value = header.value;
        
        if header_name.eq_ignore_ascii_case("host") {
            if let Some(ref new_host) = modified_host {
                modified_header_bytes.put_slice(b"Host: ");
                modified_header_bytes.put_slice(new_host.as_bytes());
                modified_header_bytes.put_slice(b"\r\n");
            } else {
                // Keep original if no modification
                modified_header_bytes.put_slice(header_name.as_bytes());
                modified_header_bytes.put_slice(b": ");
                modified_header_bytes.put_slice(header_value);
                modified_header_bytes.put_slice(b"\r\n");
            }
        } else {
            // Keep other headers as-is
            modified_header_bytes.put_slice(header_name.as_bytes());
            modified_header_bytes.put_slice(b": ");
            modified_header_bytes.put_slice(header_value);
            modified_header_bytes.put_slice(b"\r\n");
        }
    }
    
    // Add X-Forwarded-For header
    modified_header_bytes.put_slice(b"X-Forwarded-For: ");
    modified_header_bytes.put_slice(original_client_ip.as_bytes());
    modified_header_bytes.put_slice(b"\r\n");
    
    // End of headers
    modified_header_bytes.put_slice(b"\r\n");
    
    // Prepend remaining body if any
    if !remaining_body.is_empty() {
        modified_header_bytes.put_slice(&remaining_body);
    }
    
    Ok((modified_header_bytes, original_host, modified_host, final_target_addr, is_target_ssl))
}

/// Match HTTP request against rules
fn match_rule<'a>(
    rules: &'a [Rule],
    req: &Request<'_, '_>,
    host: &Option<String>,
) -> Result<Option<&'a Rule>> {
    for rule in rules {
        // Check type
        if rule.r#type != "http" {
            continue;
        }
        
        // Check host match
        if !rule.match_host.is_empty() {
            if let Some(ref h) = host {
                if !h.eq_ignore_ascii_case(&rule.match_host) {
                    continue;
                }
            } else {
                continue;
            }
        }
        
        // Check path prefix match
        if !rule.match_path_prefix.is_empty() {
            if let Some(path) = req.path {
                if !path.starts_with(&rule.match_path_prefix) {
                    continue;
                }
            } else {
                continue;
            }
        }
        
        // Check header matches
        let mut header_match = true;
        for (key, value) in &rule.match_header {
            let found = req.headers.iter()
                .find(|h| h.name.eq_ignore_ascii_case(key))
                .map(|h| h.value == value.as_bytes())
                .unwrap_or(false);
            
            if !found {
                header_match = false;
                break;
            }
        }
        
        if !header_match {
            continue;
        }
        
        // All checks passed
        return Ok(Some(rule));
    }
    
    Ok(None)
}

/// Resolve upstream name to actual address and determine if SSL is needed
/// Returns (final_target_addr, is_target_ssl)
fn resolve_upstream(
    action_proxy_pass: &str,
    upstreams: &[Upstream],
) -> Result<(String, bool)> {
    // Check if action_proxy_pass is an upstream name or direct address
    if let Some(upstream) = upstreams.iter().find(|u| u.name == action_proxy_pass) {
        // It's an upstream name, select a server using load balancing
        if upstream.servers.is_empty() {
            anyhow::bail!("Upstream '{}' has no servers", action_proxy_pass);
        }
        
        // Simple round-robin: just pick the first server for now
        // TODO: Implement proper load balancing based on lb_policy
        let server = &upstream.servers[0];
        let address = server.address.clone();
        
        // Determine if SSL is needed based on address scheme
        let (target_addr, is_ssl) = parse_address(&address)?;
        
        Ok((target_addr, is_ssl))
    } else {
        // It's a direct address, parse it
        let (target_addr, is_ssl) = parse_address(action_proxy_pass)?;
        Ok((target_addr, is_ssl))
    }
}

/// Parse address string and determine if SSL is needed
/// Supports formats: "http://host:port", "https://host:port", "host:port"
/// Returns (host:port, is_ssl)
fn parse_address(address: &str) -> Result<(String, bool)> {
    let address = address.trim();
    
    // Check if it starts with https://
    if address.starts_with("https://") {
        let addr = address.strip_prefix("https://").unwrap();
        // Extract host:port
        let target = if addr.contains('/') {
            addr.split('/').next().unwrap()
        } else {
            addr
        };
        Ok((target.to_string(), true))
    } else if address.starts_with("http://") {
        let addr = address.strip_prefix("http://").unwrap();
        let target = if addr.contains('/') {
            addr.split('/').next().unwrap()
        } else {
            addr
        };
        Ok((target.to_string(), false))
    } else {
        // Direct address format (host:port)
        Ok((address.to_string(), false))
    }
}

/// Read complete HTTP request (headers + body) from a stream
/// Returns the complete request bytes
pub async fn read_complete_http_request(
    socket: &mut TcpStream,
) -> Result<BytesMut> {
    let mut buffer = BytesMut::new();
    
    // Step 1: Read headers
    let mut header_end = false;
    while !header_end {
        let mut buf = vec![0u8; 4096];
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            anyhow::bail!("Connection closed before headers");
        }
        
        buffer.extend_from_slice(&buf[..n]);
        
        // Check for end of headers
        if buffer.len() >= 4 {
            for i in 0..=buffer.len().saturating_sub(4) {
                if &buffer[i..i+4] == b"\r\n\r\n" {
                    header_end = true;
                    break;
                }
            }
        }
        
        // Safety limit
        if buffer.len() > 8192 {
            anyhow::bail!("HTTP headers too large");
        }
    }
    
    // Find header end position
    let header_end_pos = buffer.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(buffer.len());
    
    let header_bytes = &buffer[..header_end_pos];
    
    // Step 2: Parse headers to determine body length
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    
    let parse_result = req.parse(header_bytes)?;
    match parse_result {
        Status::Complete(_) => {}
        Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Step 3: Determine body length
    let body_length = determine_body_length(&req.headers)?;
    
    // Step 4: Read body if needed
    if let Some(len) = body_length {
        // Content-Length specified
        let remaining_in_buffer = buffer.len() - header_end_pos;
        if remaining_in_buffer < len {
            let needed = len - remaining_in_buffer;
            let mut body_buf = vec![0u8; needed];
            socket.read_exact(&mut body_buf).await?;
            buffer.extend_from_slice(&body_buf);
        }
    } else {
        // Transfer-Encoding: chunked
        read_chunked_body(socket, &mut buffer).await?;
    }
    
    Ok(buffer)
}

/// Read complete HTTP response (headers + body) from a stream
/// Returns the complete response bytes
pub async fn read_complete_http_response(
    socket: &mut TcpStream,
) -> Result<BytesMut> {
    let mut buffer = BytesMut::new();
    
    // Step 1: Read headers
    let mut header_end = false;
    while !header_end {
        let mut buf = vec![0u8; 4096];
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            anyhow::bail!("Connection closed before headers");
        }
        
        buffer.extend_from_slice(&buf[..n]);
        
        // Check for end of headers
        if buffer.len() >= 4 {
            for i in 0..=buffer.len().saturating_sub(4) {
                if &buffer[i..i+4] == b"\r\n\r\n" {
                    header_end = true;
                    break;
                }
            }
        }
        
        // Safety limit
        if buffer.len() > 8192 {
            anyhow::bail!("HTTP headers too large");
        }
    }
    
    // Find header end position
    let header_end_pos = buffer.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(buffer.len());
    
    let header_bytes = &buffer[..header_end_pos];
    
    // Step 2: Parse headers to determine body length
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut resp = Response::new(&mut headers);
    
    let parse_result = resp.parse(header_bytes)?;
    match parse_result {
        Status::Complete(_) => {}
        Status::Partial => {
            anyhow::bail!("Incomplete HTTP headers");
        }
    }
    
    // Step 3: Determine body length
    let body_length = determine_response_body_length(&resp.headers)?;
    
    // Step 4: Read body if needed
    if let Some(len) = body_length {
        // Content-Length specified
        let remaining_in_buffer = buffer.len() - header_end_pos;
        if remaining_in_buffer < len {
            let needed = len - remaining_in_buffer;
            let mut body_buf = vec![0u8; needed];
            socket.read_exact(&mut body_buf).await?;
            buffer.extend_from_slice(&body_buf);
        }
    } else {
        // Transfer-Encoding: chunked or connection close
        read_chunked_body(socket, &mut buffer).await?;
    }
    
    Ok(buffer)
}

/// Determine body length from HTTP headers
fn determine_body_length(headers: &[httparse::Header]) -> Result<Option<usize>> {
    // Check for Transfer-Encoding: chunked
    for header in headers {
        if header.name.eq_ignore_ascii_case("transfer-encoding") {
            let value = std::str::from_utf8(header.value)?;
            if value.eq_ignore_ascii_case("chunked") {
                return Ok(None); // Chunked encoding
            }
        }
    }
    
    // Check for Content-Length
    for header in headers {
        if header.name.eq_ignore_ascii_case("content-length") {
            let value = std::str::from_utf8(header.value)?;
            let len = value.parse::<usize>()
                .context(format!("Invalid Content-Length: {}", value))?;
            return Ok(Some(len));
        }
    }
    
    // No body (GET requests typically)
    Ok(Some(0))
}

/// Determine response body length from HTTP headers
fn determine_response_body_length(headers: &[httparse::Header]) -> Result<Option<usize>> {
    // Check for Transfer-Encoding: chunked
    for header in headers {
        if header.name.eq_ignore_ascii_case("transfer-encoding") {
            let value = std::str::from_utf8(header.value)?;
            if value.eq_ignore_ascii_case("chunked") {
                return Ok(None); // Chunked encoding
            }
        }
    }
    
    // Check for Content-Length
    for header in headers {
        if header.name.eq_ignore_ascii_case("content-length") {
            let value = std::str::from_utf8(header.value)?;
            let len = value.parse::<usize>()
                .context(format!("Invalid Content-Length: {}", value))?;
            return Ok(Some(len));
        }
    }
    
    // For responses, if no Content-Length and no Transfer-Encoding,
    // body ends when connection closes (for HTTP/1.1)
    Ok(None)
}

/// Read chunked body
async fn read_chunked_body(socket: &mut TcpStream, buffer: &mut BytesMut) -> Result<()> {
    // Read chunked data
    // Format: <chunk-size>\r\n<chunk-data>\r\n...
    // Last chunk: 0\r\n\r\n
    
    loop {
        // Read chunk size line
        let mut chunk_size_line = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            socket.read_exact(&mut byte).await?;
            chunk_size_line.push(byte[0]);
            
            if chunk_size_line.len() >= 2 && chunk_size_line[chunk_size_line.len()-2..] == [b'\r', b'\n'] {
                break;
            }
        }
        
        // Parse chunk size (hex)
        let size_str = std::str::from_utf8(&chunk_size_line[..chunk_size_line.len()-2])?;
        let chunk_size = usize::from_str_radix(size_str.trim(), 16)
            .context(format!("Invalid chunk size: {}", size_str))?;
        
        if chunk_size == 0 {
            // Last chunk, read trailing \r\n
            let mut trailer = [0u8; 2];
            socket.read_exact(&mut trailer).await?;
            if trailer != [b'\r', b'\n'] {
                warn!("Invalid chunked encoding trailer");
            }
            break;
        }
        
        // Read chunk data
        let mut chunk_data = vec![0u8; chunk_size];
        socket.read_exact(&mut chunk_data).await?;
        buffer.extend_from_slice(&chunk_data);
        
        // Read trailing \r\n
        let mut trailer = [0u8; 2];
        socket.read_exact(&mut trailer).await?;
        if trailer != [b'\r', b'\n'] {
            warn!("Invalid chunked encoding trailer after chunk data");
        }
    }
    
    Ok(())
}

