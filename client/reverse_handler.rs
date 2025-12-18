use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn, debug};
use quinn::{Connection, SendStream, RecvStream};
use uuid::Uuid;
use bytes::BytesMut;
use crate::types::ClientState;
use crate::forwarder::Forwarder;
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, write_frame, RoutingInfo};
use tunnel_lib::proto::tunnel::{Rule, Upstream};
use crate::stream_state::StreamStateMachine;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;
use rustls::ClientConfig;
use webpki_roots::TLS_SERVER_ROOTS;
use std::io::Write;

fn match_rule_by_type_and_host<'a>(rules: &'a [Rule], rule_type: &str, host: &str) -> Result<Option<&'a Rule>> {
    let host_without_port = host.split(':').next().unwrap_or(host).trim();
    
    for rule in rules {
        if rule.r#type != rule_type {
            continue;
        }
        
        let rule_host_without_port = rule.match_host.split(':').next().unwrap_or(&rule.match_host).trim();
        if !rule.match_host.is_empty() && rule_host_without_port.eq_ignore_ascii_case(host_without_port) {
            return Ok(Some(rule));
        }
    }
    
    Ok(None)
}

fn resolve_upstream(
    action_proxy_pass: &str,
    upstreams: &[Upstream],
) -> Result<(String, bool)> {
    let address = if let Some(upstream) = upstreams.iter().find(|u| u.name == action_proxy_pass) {
        if upstream.servers.is_empty() {
            anyhow::bail!("Upstream '{}' has no servers", action_proxy_pass);
        }
        
        let server = &upstream.servers[0];
        server.address.clone()
    } else {
        action_proxy_pass.to_string()
    };
    
    let addr_trimmed = address.trim();
    let is_ssl = addr_trimmed.starts_with("https://") || addr_trimmed.starts_with("wss://");
    Ok((addr_trimmed.to_string(), is_ssl))
}

const INITIAL_BACKOFF: Duration = Duration::from_millis(100);
const MAX_BACKOFF: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECTION_CHECK_INTERVAL: Duration = Duration::from_millis(500);
const CONNECTION_STATUS_CHECK_INTERVAL: Duration = Duration::from_secs(1);
const MAX_CONCURRENT_REQUESTS: usize = 1000;

#[derive(Clone)]
pub struct ReverseRequestHandler {
    state: Arc<ClientState>,
    forwarder: Arc<Forwarder>,
    semaphore: Arc<tokio::sync::Semaphore>,
    stream_state: Arc<StreamStateMachine>,
}

impl ReverseRequestHandler {
    pub fn new(state: Arc<ClientState>, forwarder: Arc<Forwarder>) -> Self {
        Self {
            state: state.clone(),
            forwarder,
            semaphore: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_REQUESTS)),
            stream_state: Arc::new(StreamStateMachine::new()),
        }
    }

    pub async fn run(
        &self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        info!("Reverse request handler started, waiting for QUIC connection...");
        
        loop {
            // Check shutdown via state machine
            if self.state.connection_state.is_shutting_down() || self.stream_state.is_closing() {
                info!("Reverse request handler received shutdown signal");
                return Ok(());
            }

            // Check shutdown via broadcast channel
            if shutdown_rx.try_recv().is_ok() {
                self.stream_state.transition_to_closing();
                info!("Reverse request handler received shutdown signal");
                return Ok(());
            }

            let connection = self.wait_for_connection().await?;

            info!("QUIC connection available, starting reverse stream listener loop");

            // Transition to active state
            self.stream_state.transition_to_active();

            let mut backoff = INITIAL_BACKOFF;
            let mut connection_status_timer = tokio::time::interval(CONNECTION_STATUS_CHECK_INTERVAL);
            connection_status_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            connection_status_timer.tick().await;
            
            let mut error_count = 0;
            const MAX_ERRORS: usize = 10;

            loop {
                // Check shutdown via state machine
                if self.state.connection_state.is_shutting_down() || self.stream_state.is_closing() {
                    info!("Reverse request handler received shutdown signal");
                    return Ok(());
                }

                // Check shutdown via broadcast channel
                if shutdown_rx.try_recv().is_ok() {
                    self.stream_state.transition_to_closing();
                    info!("Reverse request handler received shutdown signal");
                    return Ok(());
                }

                if let Some(reason) = connection.close_reason() {
                    if shutdown_rx.try_recv().is_ok() || self.state.connection_state.is_shutting_down() {
                        info!("Reverse handler: connection closed during shutdown: {:?}", reason);
                        return Ok(());
                    }
                    // Transition to reconnecting state
                    self.stream_state.transition_to_reconnecting();
                    error!("Connection closed detected in reverse handler: {:?}, will wait for reconnection", reason);
                    break;
                }

                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Reverse request handler received shutdown signal");
                        self.stream_state.transition_to_closing();
                        return Ok(());
                    }

                    _ = connection_status_timer.tick() => {
                        if let Some(reason) = connection.close_reason() {
                            if shutdown_rx.try_recv().is_ok() || self.state.connection_state.is_shutting_down() {
                                info!("Reverse handler: connection closed during shutdown: {:?}", reason);
                                return Ok(());
                            }
                            self.stream_state.transition_to_reconnecting();
                            error!("Connection closed detected via periodic check in reverse handler: {:?}, will wait for reconnection", reason);
                            break;
                        }
                    }
                    
                    result = connection.accept_bi() => {
                        if let Some(reason) = connection.close_reason() {
                            if shutdown_rx.try_recv().is_ok() || self.state.connection_state.is_shutting_down() {
                                info!("Reverse handler: connection closed during shutdown: {:?}", reason);
                                return Ok(());
                            }
                            self.stream_state.transition_to_reconnecting();
                            error!("Connection closed detected during accept_bi in reverse handler: {:?}, will wait for reconnection", reason);
                            break;
                        }
                        match result {
                            Ok((send, recv)) => {
                                info!("Accepted reverse bidirectional stream from server");
                                backoff = INITIAL_BACKOFF;
                                error_count = 0;
                                
                                let permit = match self.semaphore.clone().try_acquire_owned() {
                                    Ok(p) => p,
                                    Err(_) => {
                                        warn!("Max concurrent requests reached ({}), rejecting new request", MAX_CONCURRENT_REQUESTS);
                                        continue;
                                    }
                                };
                                
                                let handler_clone = ReverseRequestHandler {
                                    state: self.state.clone(),
                                    forwarder: self.forwarder.clone(),
                                    semaphore: self.semaphore.clone(),
                                    stream_state: self.stream_state.clone(),
                                };
                                
                                tokio::spawn(async move {
                                    let _permit = permit;
                                    if let Err(e) = handler_clone.handle_reverse_stream(send, recv).await {
                                        error!("Reverse stream error: {}", e);
                                    }
                                });
                            }
                            Err(quinn::ConnectionError::ApplicationClosed(reason)) => {
                                if shutdown_rx.try_recv().is_ok() || self.state.connection_state.is_shutting_down() {
                                    info!("Reverse handler: ApplicationClosed during shutdown: {:?}", reason);
                                    return Ok(());
                                }
                                self.stream_state.transition_to_reconnecting();
                                error!("Connection closed (ApplicationClosed) in reverse handler: {:?}, will wait for reconnection", reason);
                                break;
                            }
                            Err(quinn::ConnectionError::ConnectionClosed(reason)) => {
                                if shutdown_rx.try_recv().is_ok() || self.state.connection_state.is_shutting_down() {
                                    info!("Reverse handler: ConnectionClosed during shutdown: {:?}", reason);
                                    return Ok(());
                                }
                                self.stream_state.transition_to_reconnecting();
                                error!("Connection closed (ConnectionClosed) in reverse handler: {:?}, will wait for reconnection", reason);
                                break;
                            }
                            Err(quinn::ConnectionError::TimedOut) => {
                                error_count += 1;
                                if error_count >= MAX_ERRORS {
                                    self.stream_state.transition_to_reconnecting();
                                    error!("Max timeout errors ({}) reached in reverse handler, will reconnect", MAX_ERRORS);
                                    break;
                                }
                                warn!("Reverse handler timeout ({}/{}), continuing", error_count, MAX_ERRORS);
                                continue;
                            }
                            Err(e) => {
                                if let Some(reason) = connection.close_reason() {
                                    if shutdown_rx.try_recv().is_ok() || self.state.connection_state.is_shutting_down() {
                                        info!("Reverse handler: connection closed during shutdown: {:?}", reason);
                                        return Ok(());
                                    }
                                    self.stream_state.transition_to_reconnecting();
                                    error!("Connection closed detected during accept_bi error: {:?}, reverse request handler will wait for reconnection", reason);
                                    break;
                                }
                                error_count += 1;
                                if error_count >= MAX_ERRORS {
                                    self.stream_state.transition_to_reconnecting();
                                    error!("Max errors ({}) reached in reverse handler: {}, will reconnect", MAX_ERRORS, e);
                                    break;
                                }
                                warn!("Reverse stream accept error ({}/{}, will retry after {:?}): {}", error_count, MAX_ERRORS, backoff, e);
                                tokio::time::sleep(backoff).await;
                                backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn wait_for_connection(&self) -> Result<Arc<Connection>> {
        loop {
            // Check shutdown via state machine
            if self.state.connection_state.is_shutting_down() || self.stream_state.is_closing() {
                return Err(anyhow::anyhow!("Shutting down"));
            }

            let connection = {
                let lock = self.state.quic_connection.read().await;
                lock.clone()
            };

            if let Some(conn) = connection {
                if conn.close_reason().is_none() && self.state.connection_state.current() == crate::connection_state::ConnectionState::Connected {
                    return Ok(conn);
                }
            }

            tokio::time::sleep(CONNECTION_CHECK_INTERVAL).await;
        }
    }

    async fn handle_reverse_stream(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> Result<()> {
        let request_id = Uuid::new_v4().to_string();
        let stream_start = std::time::Instant::now();
        
        let routing_frame = match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(REQUEST_TIMEOUT)).await {
            Ok(frame) => frame,
            Err(e) => {
                error!("[{}] Failed to read routing frame: {}", request_id, e);
                return Err(e);
            }
        };
        
        let routing_info = match RoutingInfo::decode(&routing_frame.payload) {
            Ok(info) => info,
            Err(e) => {
                error!("[{}] Failed to decode routing info: {}", request_id, e);
                return Err(e);
            }
        };
        
        info!(
            "[{}] Received routing frame: type={}, host={}, session_id={}",
            request_id, routing_info.r#type, routing_info.host, routing_frame.session_id
        );
        
        let session_id = routing_frame.session_id;
        
        let upstreams: Vec<Upstream> = self.state.upstreams.iter().map(|u| u.value().clone()).collect();
        
        // Use optimized RuleMatcher for O(1) lookup
        let matched_rule = {
            let matcher = self.state.rule_matcher.read().await;
            matcher.match_rule(&routing_info.r#type, &routing_info.host)
        };
        
        info!("[{}] Matching rules for type={}, host={} (total upstreams: {})", 
            request_id, routing_info.r#type, routing_info.host, upstreams.len());
        
        let (final_target_addr, is_target_ssl) = if let Some(rule) = matched_rule {
            info!("[{}] Matched rule: {} -> {} (action_proxy_pass)", 
                request_id, routing_info.host, rule.action_proxy_pass);
            
            let resolved = resolve_upstream(&rule.action_proxy_pass, &upstreams)?;
            info!("[{}] Resolved upstream '{}' to address: {} (SSL: {})", 
                request_id, rule.action_proxy_pass, resolved.0, resolved.1);
            resolved
        } else {
            error!("[{}] No matching rule for type={}, host={}", 
                request_id, routing_info.r#type, routing_info.host);
            return Err(anyhow::anyhow!("No matching rule for type={}, host={}", 
                routing_info.r#type, routing_info.host));
        };
        
        // WebSocket requires special handling: bidirectional streaming
        if routing_info.r#type == "wss" {
            // Read first frame (HTTP upgrade request) only
            let mut initial_request = BytesMut::new();
            match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(REQUEST_TIMEOUT)).await {
                Ok(frame) => {
                    if frame.session_id != session_id {
                        return Err(anyhow::anyhow!("Mismatched session_id"));
                    }
                    initial_request.extend_from_slice(&frame.payload);
                    info!("[{}] Received WebSocket upgrade request ({} bytes)", request_id, initial_request.len());
                }
                Err(e) => {
                    error!("[{}] Failed to read WebSocket upgrade request: {}", request_id, e);
                    return Err(e);
                }
            }
            
            return self.handle_websocket_stream(
                send,
                recv,
                routing_info,
                session_id,
                initial_request,
                final_target_addr,
                is_target_ssl,
                request_id,
                stream_start,
            ).await;
        }
        
        // For HTTP and gRPC: wait for complete request
        let mut request_buffer = BytesMut::new();
        let mut session_complete = false;
        
        while !session_complete {
            match tunnel_lib::frame::read_frame_with_timeout(&mut recv, Some(REQUEST_TIMEOUT)).await {
                Ok(frame) => {
                    if frame.session_id != session_id {
                        warn!("[{}] Received frame with mismatched session_id: {} (expected {})", 
                            request_id, frame.session_id, session_id);
                        continue;
                    }
                    
                    request_buffer.extend_from_slice(&frame.payload);
                    session_complete = frame.end_of_stream;
                    
                    if session_complete {
                        info!("[{}] Received complete request ({} bytes)", request_id, request_buffer.len());
                    }
                }
                Err(e) => {
                    if e.to_string().contains("timeout") {
                        error!("[{}] Request timeout after {:?}", request_id, REQUEST_TIMEOUT);
                    } else {
                        error!("[{}] Error reading frame: {}", request_id, e);
                    }
                    return Err(e);
                }
            }
        }
        
        // For HTTP and gRPC: wait for complete request, forward, return response
        let protocol_type = match routing_info.r#type.as_str() {
            "http" => ProtocolType::Http11,
            "grpc" => ProtocolType::Grpc,
            _ => {
                error!("[{}] Unknown protocol type: {}", request_id, routing_info.r#type);
                return Err(anyhow::anyhow!("Unknown protocol type: {}", routing_info.r#type));
            }
        };
        
        let forward_result = self.forwarder.forward(
            &routing_info.r#type,
            request_buffer.as_ref(),
            &final_target_addr,
            is_target_ssl,
        ).await;
        
        match forward_result {
            Ok(response_bytes) => {
                info!("[{}] Received response from upstream ({} bytes)", request_id, response_bytes.len());
                
                const MAX_FRAME_SIZE: usize = 64 * 1024;
                let mut offset = 0;
                
                while offset < response_bytes.len() {
                    let chunk_size = std::cmp::min(MAX_FRAME_SIZE, response_bytes.len() - offset);
                    let chunk = response_bytes[offset..offset + chunk_size].to_vec();
                    let is_last = offset + chunk_size >= response_bytes.len();
                    
                    let response_frame = TunnelFrame::new(
                        session_id,
                        protocol_type,
                        is_last,
                        chunk,
                    );
                    
                    if let Err(e) = write_frame(&mut send, &response_frame).await {
                        error!("[{}] Failed to write response frame: {}", request_id, e);
                        return Err(e.into());
                    }
                    
                    offset += chunk_size;
                }
                
                info!("[{}] Sent {} response frames (total {} bytes) to server in {:?}", 
                    request_id, (response_bytes.len() + MAX_FRAME_SIZE - 1) / MAX_FRAME_SIZE, 
                    response_bytes.len(), stream_start.elapsed());
            }
            Err(e) => {
                error!("[{}] Failed to forward {} request: {}", request_id, routing_info.r#type, e);
                
                // Detect HTTP version from request
                let http_version = tunnel_lib::http_version::HttpVersion::detect_from_request(request_buffer.as_ref())
                    .unwrap_or(tunnel_lib::http_version::HttpVersion::Http11);
                
                let error_response = format!(
                    "{} 502 Bad Gateway\r\n\
                    Content-Length: {}\r\n\
                    Content-Type: text/plain\r\n\
                    \r\n\
                    {}",
                    http_version.to_status_line_string(),
                    e.to_string().len(),
                    e
                );
                
                let error_frame = TunnelFrame::new(
                    session_id,
                    protocol_type,
                    true,
                    error_response.into_bytes(),
                );
                
                if let Err(send_err) = write_frame(&mut send, &error_frame).await {
                    error!("[{}] Failed to send error response frame: {}", request_id, send_err);
                }
                
                return Err(e);
            }
        }
        
        if let Err(e) = send.finish() {
            error!("[{}] Failed to finish send stream: {}", request_id, e);
            return Err(e.into());
        }
        
        Ok(())
    }

    /// Handle WebSocket stream: bidirectional streaming between tunnel and backend using raw TCP
    async fn handle_websocket_stream(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
        routing_info: RoutingInfo,
        session_id: u64,
        initial_request: BytesMut,
        target_addr: String,
        is_ssl: bool,
        request_id: String,
        stream_start: std::time::Instant,
    ) -> Result<()> {
        info!("[{}] Handling WebSocket stream: {} -> {}", request_id, routing_info.host, target_addr);
        
        // Parse target URI to get host and port
        let uri = if target_addr.starts_with("ws://") || target_addr.starts_with("wss://") {
            target_addr.clone()
        } else {
            format!("{}://{}", if is_ssl { "wss" } else { "ws" }, target_addr)
        };
        let parsed = url::Url::parse(&uri)
            .map_err(|e| anyhow::anyhow!("Invalid WebSocket target URI: {} ({})", target_addr, e))?;
        
        let backend_host = parsed.host_str()
            .ok_or_else(|| anyhow::anyhow!("Missing host in URI"))?
            .to_string();
        let backend_port = parsed.port().unwrap_or(if is_ssl { 443 } else { 80 });
        let backend_addr = format!("{}:{}", backend_host, backend_port);
        
        debug!("[{}] Connecting to WebSocket backend: {} (SSL: {})", request_id, backend_addr, is_ssl);
        
        // Parse original HTTP upgrade request
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        let parse_result = req.parse(&initial_request)
            .map_err(|e| anyhow::anyhow!("Failed to parse WebSocket upgrade request: {}", e))?;
        
        if !matches!(parse_result, httparse::Status::Complete(_)) {
            anyhow::bail!("Incomplete WebSocket upgrade request");
        }
        
        // Modify Host header to match backend
        let mut modified_request = Vec::new();
        let host_header = if backend_port == (if is_ssl { 443 } else { 80 }) {
            backend_host.clone()
        } else {
            format!("{}:{}", backend_host, backend_port)
        };
        
        // Rebuild request with modified Host header
        write!(modified_request, "{} {} HTTP/1.1\r\n", 
            req.method.unwrap_or("GET"), 
            req.path.unwrap_or("/"))
            .map_err(|e| anyhow::anyhow!("Failed to write request line: {}", e))?;
        
        // Copy headers, replacing Host header
        for header in req.headers {
            if header.name.eq_ignore_ascii_case("host") {
                write!(modified_request, "Host: {}\r\n", host_header)
                    .map_err(|e| anyhow::anyhow!("Failed to write Host header: {}", e))?;
            } else {
                write!(modified_request, "{}: {}\r\n", 
                    header.name,
                    std::str::from_utf8(header.value).unwrap_or(""))
                    .map_err(|e| anyhow::anyhow!("Failed to write header: {}", e))?;
            }
        }
        write!(modified_request, "\r\n")
            .map_err(|e| anyhow::anyhow!("Failed to write request end: {}", e))?;
        
        // Connect to backend - use TLS if SSL is required
        if is_ssl {
            // Establish TLS connection
            let tcp_stream = TcpStream::connect(&backend_addr).await
                .map_err(|e| anyhow::anyhow!("Failed to connect to backend {}: {}", backend_addr, e))?;
            
            // Create TLS connector
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(TLS_SERVER_ROOTS.iter().cloned());
            
            let config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            
            let connector = TlsConnector::from(Arc::new(config));
            // Create server name - use Box::leak to create 'static string
            use rustls::pki_types::{ServerName, DnsName};
            let host_for_tls: &'static str = Box::leak(backend_host.clone().into_boxed_str());
            let dns_name = DnsName::try_from(host_for_tls)
                .map_err(|e| anyhow::anyhow!("Invalid DNS name '{}': {:?}", host_for_tls, e))?;
            let server_name = ServerName::DnsName(dns_name);
            
            let tls_stream = connector.connect(server_name, tcp_stream).await
                .map_err(|e| anyhow::anyhow!("TLS handshake failed: {}", e))?;
            
            info!("[{}] Connected to backend TLS: {}", request_id, backend_addr);
            
            // Send modified HTTP upgrade request
            let (mut tls_read, mut tls_write) = tokio::io::split(tls_stream);
            
            tls_write.write_all(&modified_request).await?;
            tls_write.flush().await?;
            debug!("[{}] Sent modified HTTP upgrade request to backend", request_id);
            
            // Read HTTP upgrade response
            let mut response_buffer = BytesMut::new();
            let mut header_complete = false;
            let mut header_end_pos = 0;
            
            while !header_complete && response_buffer.len() < 8192 {
                let mut buf = vec![0u8; 4096];
                let n = tls_read.read(&mut buf).await?;
                if n == 0 {
                    return Err(anyhow::anyhow!("Backend closed connection before response"));
                }
                response_buffer.extend_from_slice(&buf[..n]);
                
                for i in 0..=response_buffer.len().saturating_sub(4) {
                    if &response_buffer[i..i+4] == b"\r\n\r\n" {
                        header_complete = true;
                        header_end_pos = i + 4;
                        break;
                    }
                }
            }
            
            if !header_complete {
                return Err(anyhow::anyhow!("Backend response headers incomplete"));
            }
            
            // Forward HTTP upgrade response to server
            let response_frame = TunnelFrame::new(
                session_id,
                ProtocolType::WssFrame,
                false,
                response_buffer[..header_end_pos].to_vec(),
            );
            write_frame(&mut send, &response_frame).await?;
            info!("[{}] Sent WebSocket upgrade response to server ({} bytes)", request_id, header_end_pos);
            
            // Use TLS streams for bidirectional forwarding
            let mut send_clone = send;
            let mut recv_clone = recv;
            let session_id_clone = session_id;
            let request_id_clone1 = request_id.clone();
            let request_id_clone2 = request_id.clone();
            const MAX_FRAME_SIZE: usize = 64 * 1024;
            
            // Task 1: Forward from tunnel to backend (TLS)
            let tunnel_to_backend_task = tokio::spawn(async move {
                let mut session_complete = false;
                const FRAME_TIMEOUT: Duration = Duration::from_secs(300);
                
                while !session_complete {
                    match tunnel_lib::frame::read_frame_with_timeout(&mut recv_clone, Some(FRAME_TIMEOUT)).await {
                        Ok(frame) => {
                            if frame.session_id != session_id_clone {
                                warn!("[{}] Received frame with mismatched session_id: {} (expected {})", 
                                    request_id_clone1, frame.session_id, session_id_clone);
                                continue;
                            }
                            
                            if !frame.payload.is_empty() {
                                if let Err(e) = tls_write.write_all(&frame.payload).await {
                                    error!("[{}] Error writing to backend TLS: {}", request_id_clone1, e);
                                    break;
                                }
                                if let Err(e) = tls_write.flush().await {
                                    error!("[{}] Error flushing to backend TLS: {}", request_id_clone1, e);
                                    break;
                                }
                            }
                            
                            session_complete = frame.end_of_stream;
                            if session_complete {
                                debug!("[{}] Received end of stream from tunnel", request_id_clone1);
                                break;
                            }
                        }
                        Err(e) => {
                            if e.to_string().contains("timeout") {
                                warn!("[{}] Tunnel frame read timeout", request_id_clone1);
                            } else {
                                error!("[{}] Error reading frame from tunnel: {}", request_id_clone1, e);
                            }
                            break;
                        }
                    }
                }
            });
            
            // Task 2: Forward from backend (TLS) to tunnel
            let backend_to_tunnel_task = tokio::spawn(async move {
                let mut buffer = BytesMut::new();
                let mut read_buf = vec![0u8; 4096];
                
                loop {
                    match tls_read.read(&mut read_buf).await {
                        Ok(0) => {
                            debug!("[{}] Backend TLS connection closed", request_id_clone2);
                            if !buffer.is_empty() {
                                let frame = TunnelFrame::new(
                                    session_id_clone,
                                    ProtocolType::WssFrame,
                                    true,
                                    buffer.to_vec(),
                                );
                                let _ = write_frame(&mut send_clone, &frame).await;
                            } else {
                                let end_frame = TunnelFrame::new(
                                    session_id_clone,
                                    ProtocolType::WssFrame,
                                    true,
                                    Vec::new(),
                                );
                                let _ = write_frame(&mut send_clone, &end_frame).await;
                            }
                            break;
                        }
                        Ok(n) => {
                            buffer.extend_from_slice(&read_buf[..n]);
                            
                            // For WebSocket, send data more aggressively:
                            // - If buffer reaches MAX_FRAME_SIZE, send immediately
                            // - If we got less data than read buffer size (partial read), send what we have
                            // - Otherwise, send if buffer has accumulated enough data
                            let should_send = buffer.len() >= MAX_FRAME_SIZE || 
                                             n < read_buf.len(); // Partial read indicates no more data immediately available
                            
                            if should_send && !buffer.is_empty() {
                                // Send in MAX_FRAME_SIZE chunks first
                                while buffer.len() >= MAX_FRAME_SIZE {
                                    let chunk = buffer[..MAX_FRAME_SIZE].to_vec();
                                    buffer = buffer[MAX_FRAME_SIZE..].into();
                                    
                                    let frame = TunnelFrame::new(
                                        session_id_clone,
                                        ProtocolType::WssFrame,
                                        false,
                                        chunk,
                                    );
                                    
                                    if let Err(e) = write_frame(&mut send_clone, &frame).await {
                                        let err_str = e.to_string();
                                        if err_str.contains("sending stopped by peer") || 
                                           err_str.contains("connection closed") {
                                            debug!("[{}] Tunnel connection closed, breaking", request_id_clone2);
                                            return;
                                        } else {
                                            error!("[{}] Error sending frame to tunnel: {}", request_id_clone2, e);
                                            return;
                                        }
                                    }
                                }
                                
                                // Send remaining buffer (even if small, for WebSocket responsiveness)
                                if !buffer.is_empty() {
                                    let chunk = buffer.to_vec();
                                    buffer.clear();
                                    
                                    let frame = TunnelFrame::new(
                                        session_id_clone,
                                        ProtocolType::WssFrame,
                                        false,
                                        chunk,
                                    );
                                    
                                    if let Err(e) = write_frame(&mut send_clone, &frame).await {
                                        let err_str = e.to_string();
                                        if err_str.contains("sending stopped by peer") || 
                                           err_str.contains("connection closed") {
                                            debug!("[{}] Tunnel connection closed, breaking", request_id_clone2);
                                            return;
                                        } else {
                                            error!("[{}] Error sending frame to tunnel: {}", request_id_clone2, e);
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("[{}] Error reading from backend TLS: {}", request_id_clone2, e);
                            break;
                        }
                    }
                }
                
                if !buffer.is_empty() {
                    let frame = TunnelFrame::new(
                        session_id_clone,
                        ProtocolType::WssFrame,
                        true,
                        buffer.to_vec(),
                    );
                    if let Err(e) = write_frame(&mut send_clone, &frame).await {
                        let err_str = e.to_string();
                        if !err_str.contains("sending stopped by peer") && 
                           !err_str.contains("connection closed") {
                            debug!("[{}] Error sending final frame: {}", request_id_clone2, e);
                        }
                    }
                }
            });
            
            // Wait for either task to complete
            tokio::select! {
                result = tunnel_to_backend_task => {
                    if let Err(e) = result {
                        error!("[{}] Tunnel to backend task error: {}", request_id, e);
                    }
                }
                result = backend_to_tunnel_task => {
                    if let Err(e) = result {
                        error!("[{}] Backend to tunnel task error: {}", request_id, e);
                    }
                }
            }
            
            info!("[{}] WebSocket stream completed in {:?}", request_id, stream_start.elapsed());
            return Ok(());
        } else {
            // Plain TCP connection
            let mut backend_stream = TcpStream::connect(&backend_addr).await
                .map_err(|e| anyhow::anyhow!("Failed to connect to backend {}: {}", backend_addr, e))?;
            
            info!("[{}] Connected to backend TCP: {}", request_id, backend_addr);
            
            // Send modified HTTP upgrade request
            backend_stream.write_all(&modified_request).await?;
            backend_stream.flush().await?;
            debug!("[{}] Sent modified HTTP upgrade request to backend", request_id);
            
            // Read HTTP upgrade response
            let mut response_buffer = BytesMut::new();
            let mut header_complete = false;
            let mut header_end_pos = 0;
            
            while !header_complete && response_buffer.len() < 8192 {
                let mut buf = vec![0u8; 4096];
                let n = backend_stream.read(&mut buf).await?;
                if n == 0 {
                    return Err(anyhow::anyhow!("Backend closed connection before response"));
                }
                response_buffer.extend_from_slice(&buf[..n]);
                
                for i in 0..=response_buffer.len().saturating_sub(4) {
                    if &response_buffer[i..i+4] == b"\r\n\r\n" {
                        header_complete = true;
                        header_end_pos = i + 4;
                        break;
                    }
                }
            }
            
            if !header_complete {
                return Err(anyhow::anyhow!("Backend response headers incomplete"));
            }
            
            // Forward HTTP upgrade response to server
            let response_frame = TunnelFrame::new(
                session_id,
                ProtocolType::WssFrame,
                false,
                response_buffer[..header_end_pos].to_vec(),
            );
            write_frame(&mut send, &response_frame).await?;
            info!("[{}] Sent WebSocket upgrade response to server ({} bytes)", request_id, header_end_pos);
            
            // Split streams for bidirectional forwarding
            let (mut backend_recv, mut backend_send) = tokio::io::split(backend_stream);
            let mut send_clone = send;
            let mut recv_clone = recv;
            let session_id_clone = session_id;
            let request_id_clone1 = request_id.clone();
            let request_id_clone2 = request_id.clone();
            const MAX_FRAME_SIZE: usize = 64 * 1024;
            
            // Task 1: Forward from tunnel to backend (TCP)
        let tunnel_to_backend_task = tokio::spawn(async move {
            let mut session_complete = false;
            const FRAME_TIMEOUT: Duration = Duration::from_secs(300);
            
            while !session_complete {
                match tunnel_lib::frame::read_frame_with_timeout(&mut recv_clone, Some(FRAME_TIMEOUT)).await {
                    Ok(frame) => {
                        if frame.session_id != session_id_clone {
                            warn!("[{}] Received frame with mismatched session_id: {} (expected {})", 
                                request_id_clone1, frame.session_id, session_id_clone);
                            continue;
                        }
                        
                        // Forward payload directly to backend TCP stream
                        if !frame.payload.is_empty() {
                            if let Err(e) = backend_send.write_all(&frame.payload).await {
                                error!("[{}] Error writing to backend: {}", request_id_clone1, e);
                                break;
                            }
                            if let Err(e) = backend_send.flush().await {
                                error!("[{}] Error flushing to backend: {}", request_id_clone1, e);
                                break;
                            }
                        }
                        
                        session_complete = frame.end_of_stream;
                        if session_complete {
                            debug!("[{}] Received end of stream from tunnel", request_id_clone1);
                            break;
                        }
                    }
                    Err(e) => {
                        if e.to_string().contains("timeout") {
                            warn!("[{}] Tunnel frame read timeout", request_id_clone1);
                        } else {
                            error!("[{}] Error reading frame from tunnel: {}", request_id_clone1, e);
                        }
                        break;
                    }
                }
            }
        });
        
        // Task 2: Forward from backend (TCP) to tunnel
        let backend_to_tunnel_task = tokio::spawn(async move {
            let mut buffer = BytesMut::new();
            let mut read_buf = vec![0u8; 4096];
            
            loop {
                match backend_recv.read(&mut read_buf).await {
                    Ok(0) => {
                        debug!("[{}] Backend connection closed", request_id_clone2);
                        // Send remaining buffer if any
                        if !buffer.is_empty() {
                            let frame = TunnelFrame::new(
                                session_id_clone,
                                ProtocolType::WssFrame,
                                true,
                                buffer.to_vec(),
                            );
                            let _ = write_frame(&mut send_clone, &frame).await;
                        } else {
                            // Send end frame
                            let end_frame = TunnelFrame::new(
                                session_id_clone,
                                ProtocolType::WssFrame,
                                true,
                                Vec::new(),
                            );
                            let _ = write_frame(&mut send_clone, &end_frame).await;
                        }
                        break;
                    }
                    Ok(n) => {
                        buffer.extend_from_slice(&read_buf[..n]);
                        
                        // For WebSocket, send data more aggressively:
                        // - If buffer reaches MAX_FRAME_SIZE, send immediately
                        // - If we got less data than read buffer size (partial read), send what we have
                        // - Otherwise, send if buffer has accumulated enough data
                        let should_send = buffer.len() >= MAX_FRAME_SIZE || 
                                         n < read_buf.len(); // Partial read indicates no more data immediately available
                        
                        if should_send && !buffer.is_empty() {
                            // Send in MAX_FRAME_SIZE chunks first
                            while buffer.len() >= MAX_FRAME_SIZE {
                                let chunk = buffer[..MAX_FRAME_SIZE].to_vec();
                                buffer = buffer[MAX_FRAME_SIZE..].into();
                                
                                let frame = TunnelFrame::new(
                                    session_id_clone,
                                    ProtocolType::WssFrame,
                                    false,
                                    chunk,
                                );
                                
                                if let Err(e) = write_frame(&mut send_clone, &frame).await {
                                    let err_str = e.to_string();
                                    if err_str.contains("sending stopped by peer") || 
                                       err_str.contains("connection closed") {
                                        debug!("[{}] Tunnel connection closed, breaking", request_id_clone2);
                                        return;
                                    } else {
                                        error!("[{}] Error sending frame to tunnel: {}", request_id_clone2, e);
                                        return;
                                    }
                                }
                            }
                            
                            // Send remaining buffer (even if small, for WebSocket responsiveness)
                            if !buffer.is_empty() {
                                let chunk = buffer.to_vec();
                                buffer.clear();
                                
                                let frame = TunnelFrame::new(
                                    session_id_clone,
                                    ProtocolType::WssFrame,
                                    false,
                                    chunk,
                                );
                                
                                if let Err(e) = write_frame(&mut send_clone, &frame).await {
                                    let err_str = e.to_string();
                                    if err_str.contains("sending stopped by peer") || 
                                       err_str.contains("connection closed") {
                                        debug!("[{}] Tunnel connection closed, breaking", request_id_clone2);
                                        return;
                                    } else {
                                        error!("[{}] Error sending frame to tunnel: {}", request_id_clone2, e);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("[{}] Error reading from backend: {}", request_id_clone2, e);
                        break;
                    }
                }
            }
            
            // Send remaining buffer
            if !buffer.is_empty() {
                let frame = TunnelFrame::new(
                    session_id_clone,
                    ProtocolType::WssFrame,
                    true,
                    buffer.to_vec(),
                );
                if let Err(e) = write_frame(&mut send_clone, &frame).await {
                    let err_str = e.to_string();
                    if !err_str.contains("sending stopped by peer") && 
                       !err_str.contains("connection closed") {
                        debug!("[{}] Error sending final frame: {}", request_id_clone2, e);
                    }
                }
            }
        });
        
        // Wait for either task to complete
        tokio::select! {
            result = tunnel_to_backend_task => {
                if let Err(e) = result {
                    error!("[{}] Tunnel to backend task error: {}", request_id, e);
                }
            }
            result = backend_to_tunnel_task => {
                if let Err(e) = result {
                    error!("[{}] Backend to tunnel task error: {}", request_id, e);
                }
            }
        }
        
            info!("[{}] WebSocket stream completed in {:?}", request_id, stream_start.elapsed());
            Ok(())
        }
    }
}
