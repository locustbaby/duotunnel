use anyhow::{Result, Context};
use bytes::BytesMut;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_rustls::HttpsConnector;
use httparse::{Request, Status};
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tracing::{debug, info, error, warn};
use quinn::Connection;
use uuid::Uuid;
use tokio_tungstenite::connect_async;
use url::Url;
use crate::types::ClientState;
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, write_frame, RoutingInfo, create_routing_frame};
use tunnel_lib::frame::TunnelFrame as Frame;
use crate::forward_strategies::{ForwardStrategy, HttpForwardStrategy, WssForwardStrategy, GrpcForwardStrategy};


mod uri_utils {
    use super::*;


    #[derive(Debug, Clone)]
    pub struct ParsedUri {
        pub scheme: String,
        pub host: String,
        pub port: u16,
        pub is_ssl: bool,
    }



    pub fn normalize_uri(uri: &str, default_scheme: &str) -> Result<String> {
        let uri = uri.trim();
        

        if uri.starts_with("http://") || uri.starts_with("https://") 
            || uri.starts_with("ws://") || uri.starts_with("wss://") {
            return Ok(uri.to_string());
        }
        

        Ok(format!("{}://{}", default_scheme, uri))
    }


    pub fn parse_target(target_uri: &str, protocol_type: &str) -> Result<ParsedUri> {

        let default_scheme = match protocol_type {
            "http" => "http",
            "wss" => "ws",
            "grpc" => "http",
            _ => "http",
        };
        
        let normalized = normalize_uri(target_uri, default_scheme)?;
        let url = Url::parse(&normalized)
            .with_context(|| format!("Failed to parse URI: {}", target_uri))?;
        
        let scheme = url.scheme().to_string();
        let is_ssl = scheme == "https" || scheme == "wss";
        
        let host = url.host_str()
            .ok_or_else(|| anyhow::anyhow!("Missing host in URI: {}", target_uri))?
            .to_string();
        

        let default_port = match scheme.as_str() {
            "http" | "ws" => 80,
            "https" | "wss" => 443,
            "grpc" => 50051,
            _ => 80,
        };
        
        let port = url.port().unwrap_or(default_port);
        
        Ok(ParsedUri {
            scheme,
            host,
            port,
            is_ssl,
        })
    }


    pub fn to_host_port(target_uri: &str, protocol_type: &str) -> Result<String> {
        let parsed = parse_target(target_uri, protocol_type)?;
        Ok(format!("{}:{}", parsed.host, parsed.port))
    }
}


pub type ForwardResult = Result<Vec<u8>>;

#[derive(Clone)]
pub struct Forwarder {
    state: Arc<ClientState>,
    strategies: Arc<HashMap<String, Box<dyn ForwardStrategy>>>,
}

impl Forwarder {
    pub fn new(state: Arc<ClientState>) -> Self {
        let client = state.egress_pool.client();
        let mut strategies: HashMap<String, Box<dyn ForwardStrategy>> = HashMap::new();
        
        // Register strategies
        strategies.insert("http".to_string(), Box::new(HttpForwardStrategy::new(client.clone())));
        strategies.insert("https".to_string(), Box::new(HttpForwardStrategy::new(client.clone())));
        strategies.insert("wss".to_string(), Box::new(WssForwardStrategy::new()));
        strategies.insert("ws".to_string(), Box::new(WssForwardStrategy::new()));
        strategies.insert("grpc".to_string(), Box::new(GrpcForwardStrategy::new()));
        
        Self {
            state,
            strategies: Arc::new(strategies),
        }
    }

    pub fn get_strategy(&self, protocol_type: &str) -> Result<&dyn ForwardStrategy> {
        self.strategies.get(protocol_type)
            .map(|s| s.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Unsupported protocol type: {}", protocol_type))
    }

    pub async fn forward(
        &self,
        protocol_type: &str,
        request_bytes: &[u8],
        target_uri: &str,
        is_ssl: bool,
    ) -> ForwardResult {
        let strategy = self.get_strategy(protocol_type)?;
        strategy.forward(request_bytes, target_uri, is_ssl).await
    }
}


pub mod http {
    use super::*;



    pub async fn read_complete_http_request(
        socket: &mut TcpStream,
    ) -> Result<BytesMut> {
        let mut buffer = BytesMut::new();
        
        let mut header_end = false;
        while !header_end {
            let mut buf = vec![0u8; 4096];
            let n = socket.read(&mut buf).await?;
            if n == 0 {
                anyhow::bail!("Connection closed before headers");
            }
            
            buffer.extend_from_slice(&buf[..n]);
            
            if buffer.len() >= 4 {
                for i in 0..=buffer.len().saturating_sub(4) {
                    if &buffer[i..i+4] == b"\r\n\r\n" {
                        header_end = true;
                        break;
                    }
                }
            }
            
            if buffer.len() > 8192 {
                anyhow::bail!("HTTP headers too large");
            }
        }
        
        let header_end_pos = buffer.windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|i| i + 4)
            .unwrap_or(buffer.len());
        
        let header_bytes = &buffer[..header_end_pos];
        
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = Request::new(&mut headers);
        
        let parse_result = req.parse(header_bytes)?;
        match parse_result {
            Status::Complete(_) => {}
            Status::Partial => {
                anyhow::bail!("Incomplete HTTP headers");
            }
        }
        
        let body_length = determine_body_length(&req.headers)?;
        
        if let Some(len) = body_length {
            let remaining_in_buffer = buffer.len() - header_end_pos;
            if remaining_in_buffer < len {
                let needed = len - remaining_in_buffer;
                let mut body_buf = vec![0u8; needed];
                use tokio::io::AsyncReadExt;
                socket.read_exact(&mut body_buf).await?;
                buffer.extend_from_slice(&body_buf);
            }
        } else {
            read_chunked_body(socket, &mut buffer).await?;
        }
        
        Ok(buffer)
    }


    fn determine_body_length(headers: &[httparse::Header]) -> Result<Option<usize>> {
        for header in headers {
            if header.name.eq_ignore_ascii_case("transfer-encoding") {
                let value = std::str::from_utf8(header.value)?;
                if value.eq_ignore_ascii_case("chunked") {
                    return Ok(None);
                }
            }
        }
        
        for header in headers {
            if header.name.eq_ignore_ascii_case("content-length") {
                let value = std::str::from_utf8(header.value)?;
                let len = value.parse::<usize>()
                    .context(format!("Invalid Content-Length: {}", value))?;
                return Ok(Some(len));
            }
        }
        
        Ok(Some(0))
    }


    async fn read_chunked_body(socket: &mut TcpStream, buffer: &mut BytesMut) -> Result<()> {
        use tokio::io::AsyncReadExt;
        
        loop {
            let mut chunk_size_line = Vec::new();
            loop {
                let mut byte = [0u8; 1];
                socket.read_exact(&mut byte).await?;
                chunk_size_line.push(byte[0]);
                
                if chunk_size_line.len() >= 2 && chunk_size_line[chunk_size_line.len()-2..] == [b'\r', b'\n'] {
                    break;
                }
            }
            
            let size_str = std::str::from_utf8(&chunk_size_line[..chunk_size_line.len()-2])?;
            let chunk_size = usize::from_str_radix(size_str.trim(), 16)
                .context(format!("Invalid chunk size: {}", size_str))?;
            
            if chunk_size == 0 {
                let mut trailer = [0u8; 2];
                socket.read_exact(&mut trailer).await?;
                if trailer != [b'\r', b'\n'] {
                    warn!("Invalid chunked encoding trailer");
                }
                break;
            }
            
            let mut chunk_data = vec![0u8; chunk_size];
            socket.read_exact(&mut chunk_data).await?;
            buffer.extend_from_slice(&chunk_data);
            
            let mut trailer = [0u8; 2];
            socket.read_exact(&mut trailer).await?;
            if trailer != [b'\r', b'\n'] {
                warn!("Invalid chunked encoding trailer after chunk data");
            }
        }
        
        Ok(())
    }


    pub async fn handle_http_forward_connection(
        mut socket: TcpStream,
        connection: Arc<Connection>,
    ) -> Result<()> {
        let request_id = Uuid::new_v4().to_string();
        let stream_start = std::time::Instant::now();
        
        let complete_request = read_complete_http_request(&mut socket).await?;
        info!("[{}] Read complete HTTP request ({} bytes)", request_id, complete_request.len());
        
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        
        let parse_result = req.parse(&complete_request)?;
        match parse_result {
            httparse::Status::Complete(_) => {
                debug!("[{}] Parsed HTTP request: {} {}", 
                    request_id, req.method.unwrap_or(""), req.path.unwrap_or(""));
            }
            httparse::Status::Partial => {
                anyhow::bail!("Incomplete HTTP headers");
            }
        }
        
        let host = req.headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("host"))
            .and_then(|h| std::str::from_utf8(h.value).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "localhost".to_string());
        
        debug!("[{}] Request Host: {}", request_id, host);
        
        let (mut send, mut recv) = connection.open_bi().await?;
        
        let session_id = Frame::session_id_from_uuid(&request_id);
        
        let method = req.method.unwrap_or("GET").to_string();
        let path = req.path.unwrap_or("/").to_string();
        
        let routing_info = RoutingInfo {
            r#type: "http".to_string(),
            host: host.clone(),
            method,
            path,
        };
        
        let routing_frame = create_routing_frame(session_id, &routing_info);
        write_frame(&mut send, &routing_frame).await?;
        info!("[{}] Sent routing frame to server: session_id={}, host={}", 
            request_id, session_id, host);
        
        // Send HTTP request data using TunnelFrame (same as server->client ingress)
        // This ensures consistency with how server handles requests from client
        const MAX_FRAME_SIZE: usize = 64 * 1024;
        let request_bytes = complete_request.to_vec();
        let mut offset = 0;
        
        while offset < request_bytes.len() {
            let chunk_size = std::cmp::min(MAX_FRAME_SIZE, request_bytes.len() - offset);
            let chunk = request_bytes[offset..offset + chunk_size].to_vec();
            let is_last = offset + chunk_size >= request_bytes.len();
            
            let data_frame = TunnelFrame::new(
                session_id,
                ProtocolType::Http11,
                is_last,
                chunk,
            );
            
            write_frame(&mut send, &data_frame).await?;
            offset += chunk_size;
        }
        
        info!("[{}] Sent {} frames (total {} bytes) to server", 
            request_id, (request_bytes.len() + MAX_FRAME_SIZE - 1) / MAX_FRAME_SIZE, request_bytes.len());
        
        // Finish the send stream to signal that request is complete
        // This allows the server to know when to close the TCP write side
        // The recv stream remains open to receive the response
        send.finish()?;
        info!("[{}] Finished send stream, waiting for response", request_id);
        
        // Read response frames from server (same as server->client ingress)
        let mut response_buffer = BytesMut::new();
        let mut session_complete = false;
        const RESPONSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
        
        while !session_complete {
            match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(RESPONSE_TIMEOUT)).await {
                Ok(frame) => {
                    if frame.session_id != session_id {
                        warn!("[{}] Received frame with mismatched session_id: {} (expected {})", 
                            request_id, frame.session_id, session_id);
                        continue;
                    }
                    
                    response_buffer.extend_from_slice(&frame.payload);
                    session_complete = frame.end_of_stream;
                    
                    // Stream response chunks to client as we receive them
                    if !frame.payload.is_empty() {
                        socket.write_all(&frame.payload).await?;
                    }
                    
                    if session_complete {
                        info!("[{}] Received complete response ({} bytes)", request_id, response_buffer.len());
                    }
                }
                Err(e) => {
                    let err_str = e.to_string();
                    // If stream is closed (server called finish), and we have some data, consider it complete
                    if (err_str.contains("stream closed") || err_str.contains("connection closed") || err_str.contains("UnexpectedEof")) 
                        && !response_buffer.is_empty() {
                        warn!("[{}] QUIC stream closed while reading frames, but received {} bytes. Treating as complete response.", 
                            request_id, response_buffer.len());
                        session_complete = true;
                        break;
                    }
                    
                    if err_str.contains("timeout") {
                        error!("[{}] Response timeout after {:?}", request_id, RESPONSE_TIMEOUT);
                    } else {
                        error!("[{}] Error reading frame: {}", request_id, e);
                    }
                    return Err(e);
                }
            }
        }
        
        socket.flush().await?;
        info!("[{}] Received and sent HTTP response ({} bytes) in {:?}", 
            request_id, response_buffer.len(), stream_start.elapsed());
        
        Ok(())
    }



    pub async fn forward_http_request(
        client: &Client<HttpsConnector<HttpConnector>, http_body_util::Full<bytes::Bytes>>,
        request_bytes: &[u8],
        target_uri: &str,
        _is_ssl: bool,
    ) -> Result<Vec<u8>> {
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
        
        let method_str = req.method.unwrap_or("GET");
        
        let original_path = req.path.unwrap_or("/");
        let uri_str = if original_path.starts_with("http://") || original_path.starts_with("https://") {
            original_path.to_string()
        } else {
            let base_uri = hyper::Uri::from_str(target_uri)?;
            let base_path = base_uri.path();
            let base_scheme = base_uri.scheme_str().unwrap_or("http");
            let base_authority = base_uri.authority()
                .map(|a| a.as_str())
                .unwrap_or("localhost");
            
            let full_path = if original_path.starts_with('/') {
                original_path.to_string()
            } else {
                format!("{}/{}", base_path.trim_end_matches('/'), original_path)
            };
            
            format!("{}://{}{}", base_scheme, base_authority, full_path)
        };
        
        let uri = hyper::Uri::from_str(&uri_str)?;
        
        let mut builder = hyper::Request::builder()
            .method(method_str)
            .uri(uri);
        
        for header in req.headers.iter() {
            let name = header.name;
            if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("connection") {
                continue;
            }
            
            let value = std::str::from_utf8(header.value)
                .with_context(|| format!("Invalid header value for {}", name))?;
            
            builder = builder.header(name, value);
        }
        
        let header_len = parse_result.unwrap();
        let body_bytes = if request_bytes.len() > header_len {
            &request_bytes[header_len..]
        } else {
            &[]
        };
        
        let body = http_body_util::Full::new(bytes::Bytes::copy_from_slice(body_bytes));
        let hyper_request = builder.body(body)?;
        
        debug!("Sending HTTP request: {} {}", hyper_request.method(), hyper_request.uri());
        
        let response = client.request(hyper_request).await
            .with_context(|| "Failed to send HTTP/HTTPS request")?;
        
        debug!("Received HTTP response: {}", response.status());
        
        // Detect HTTP version from original request
        let http_version = tunnel_lib::http_version::HttpVersion::detect_from_request(request_bytes)?;
        debug!("Detected HTTP version: {:?}", http_version);
        
        let status_line = format!(
            "{} {} {}\r\n",
            http_version.to_status_line_string(),
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        );
        
        let mut response_bytes = BytesMut::new();
        response_bytes.extend_from_slice(status_line.as_bytes());
        
        for (name, value) in response.headers() {
            let header_line = format!("{}: {}\r\n", name, value.to_str().unwrap_or(""));
            response_bytes.extend_from_slice(header_line.as_bytes());
        }
        
        response_bytes.extend_from_slice(b"\r\n");
        
        let body = response.into_body();
        use http_body_util::BodyExt;
        let mut body_bytes = Vec::new();
        let mut body_stream = body;
        while let Some(frame_result) = body_stream.frame().await {
            if let Ok(frame) = frame_result {
                if let Some(chunk) = frame.data_ref() {
                    body_bytes.extend_from_slice(chunk);
                }
            }
        }
        
        response_bytes.extend_from_slice(&body_bytes);
        
        Ok(response_bytes.to_vec())
    }
}


pub mod wss {
    use super::*;
    use futures_util::{SinkExt, StreamExt};

    pub async fn forward_wss_request(
        request_bytes: &[u8],
        target_uri: &str,
        is_ssl: bool,
    ) -> Result<Vec<u8>> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        
        let parse_result = req.parse(request_bytes)?;
        if !matches!(parse_result, httparse::Status::Complete(_)) {
            anyhow::bail!("Incomplete HTTP headers");
        }

        let parsed = uri_utils::parse_target(target_uri, "wss")?;
        let ws_url = format!("{}://{}:{}", 
            if parsed.is_ssl || is_ssl { "wss" } else { "ws" },
            parsed.host,
            parsed.port
        );

        debug!("Connecting to WebSocket: {}", ws_url);

        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .with_context(|| format!("Failed to connect to WebSocket: {}", ws_url))?;

        debug!("WebSocket connected");

        // Parse HTTP upgrade request to extract headers
        let header_end = request_bytes.windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|i| i + 4)
            .unwrap_or(request_bytes.len());

        // Send HTTP upgrade request to backend
        let mut backend_stream = ws_stream;
        backend_stream.send(tokio_tungstenite::tungstenite::Message::Binary(request_bytes[..header_end].to_vec())).await?;

        // Read WebSocket upgrade response
        let mut response_buffer = BytesMut::new();
        
        // For WebSocket, we need to handle bidirectional streaming
        // This is a simplified implementation that reads the initial response
        // In practice, WebSocket forwarding should be handled differently (see reverse_handler)
        
        match backend_stream.next().await {
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

        Ok(response_buffer.to_vec())
    }
}


pub mod grpc {
    use super::*;
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    pub async fn forward_grpc_request(
        request_bytes: &[u8],
        target_uri: &str,
        is_ssl: bool,
    ) -> Result<Vec<u8>> {
        let parsed = uri_utils::parse_target(target_uri, "grpc")?;
        
        debug!("Connecting to gRPC endpoint: {}:{} (SSL: {})", parsed.host, parsed.port, parsed.is_ssl || is_ssl);

        // For gRPC over HTTP/2, we need to establish a TCP connection
        // and forward HTTP/2 frames. This is a simplified implementation.
        // In production, you'd use a proper HTTP/2 client library.
        
        let addr = format!("{}:{}", parsed.host, parsed.port);
        let mut stream = TcpStream::connect(&addr).await
            .with_context(|| format!("Failed to connect to gRPC endpoint: {}", addr))?;

        debug!("gRPC TCP connection established");

        // Forward the entire request (HTTP/2 headers + gRPC messages)
        stream.write_all(request_bytes).await?;
        stream.flush().await?;

        debug!("Sent gRPC request ({} bytes)", request_bytes.len());

        // Read response
        let mut response_buffer = BytesMut::new();
        let mut buf = vec![0u8; 4096];
        
        // Read HTTP/2 response headers first
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
            
            // For gRPC, we typically read until connection closes or timeout
            // In a more sophisticated implementation, you'd parse gRPC frames
            // and handle streaming responses properly
        }

        debug!("Received gRPC response ({} bytes)", response_buffer.len());
        
        Ok(response_buffer.to_vec())
    }
}
