use anyhow::{Result, Context};
use bytes::BytesMut;
use httparse::{Request, Status};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::warn;

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

