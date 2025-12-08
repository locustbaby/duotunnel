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
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use url::Url;
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

    /// Handle WebSocket stream: bidirectional streaming between tunnel and backend
    async fn handle_websocket_stream(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
        routing_info: RoutingInfo,
        session_id: u64,
        _initial_request: BytesMut,
        target_addr: String,
        is_ssl: bool,
        request_id: String,
        stream_start: std::time::Instant,
    ) -> Result<()> {
        info!("[{}] Handling WebSocket stream: {} -> {}", request_id, routing_info.host, target_addr);
        
        // Parse target URI
        let parsed = {
            let uri = if target_addr.starts_with("ws://") || target_addr.starts_with("wss://") {
                target_addr.clone()
            } else {
                format!("{}://{}", if is_ssl { "wss" } else { "ws" }, target_addr)
            };
            url::Url::parse(&uri)
                .map_err(|e| anyhow::anyhow!("Invalid WebSocket target URI: {} ({})", target_addr, e))?
        };
        
        let ws_url = format!("{}://{}:{}", 
            if parsed.scheme() == "wss" || is_ssl { "wss" } else { "ws" },
            parsed.host_str().ok_or_else(|| anyhow::anyhow!("Missing host in URI"))?,
            parsed.port().unwrap_or(if is_ssl { 443 } else { 80 })
        );
        
        debug!("[{}] Connecting to WebSocket backend: {}", request_id, ws_url);
        
        // Connect to backend WebSocket using tokio_tungstenite
        // connect_async will perform WebSocket handshake automatically
        let (ws_stream, response) = connect_async(&ws_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to WebSocket {}: {}", ws_url, e))?;
        
        info!("[{}] WebSocket backend connected to {}", request_id, ws_url);
        
        // Immediately forward HTTP upgrade response back to server
        let mut response_bytes = Vec::new();
        write!(response_bytes, "HTTP/1.1 {} {}\r\n", 
            response.status().as_u16(), 
            response.status().canonical_reason().unwrap_or("Switching Protocols"))
            .map_err(|e| anyhow::anyhow!("Failed to write response status: {}", e))?;
        
        // Write response headers
        for (name, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                write!(response_bytes, "{}: {}\r\n", name, value_str)
                    .map_err(|e| anyhow::anyhow!("Failed to write response header: {}", e))?;
            }
        }
        write!(response_bytes, "\r\n")
            .map_err(|e| anyhow::anyhow!("Failed to write response end: {}", e))?;
        
        // Send HTTP upgrade response frame to server
        let response_frame = TunnelFrame::new(
            session_id,
            ProtocolType::WssFrame,
            false,  // Not the last frame, more WebSocket data will follow
            response_bytes,
        );
        write_frame(&mut send, &response_frame).await?;
        info!("[{}] Sent WebSocket upgrade response to server ({} bytes)", request_id, response_frame.payload.len());
        
        // Split WebSocket stream into send and receive halves
        let (mut ws_sink, mut ws_stream_recv) = ws_stream.split();
        
        // Spawn bidirectional forwarding tasks
        let mut send_clone = send;
        let mut recv_clone = recv;
        let session_id_clone = session_id;
        let request_id_clone1 = request_id.clone();
        let request_id_clone2 = request_id.clone();
        
        // Task 1: Forward frames from tunnel to backend WebSocket
        let tunnel_to_backend_task = tokio::spawn(async move {
            let mut ws_send = ws_sink;
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
                        
                        // Forward payload to backend WebSocket
                        if !frame.payload.is_empty() {
                            if let Err(e) = ws_send.send(tokio_tungstenite::tungstenite::Message::Binary(frame.payload)).await {
                                error!("[{}] Error sending to backend WebSocket: {}", request_id_clone1, e);
                                break;
                            }
                        }
                        
                        session_complete = frame.end_of_stream;
                        if session_complete {
                            debug!("[{}] Received end of stream from tunnel", request_id_clone1);
                            // Send close frame to backend
                            let _ = ws_send.close().await;
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
        
        // Task 2: Forward messages from backend WebSocket to tunnel
        let backend_to_tunnel_task = tokio::spawn(async move {
            let mut ws_recv = ws_stream_recv;
            const MAX_FRAME_SIZE: usize = 64 * 1024;
            
            loop {
                match ws_recv.next().await {
                    Some(Ok(msg)) => {
                        let payload = match msg {
                            tokio_tungstenite::tungstenite::Message::Binary(data) => data,
                            tokio_tungstenite::tungstenite::Message::Text(text) => text.into_bytes(),
                            tokio_tungstenite::tungstenite::Message::Close(_) => {
                                debug!("[{}] Backend WebSocket closed", request_id_clone2);
                                // Send end frame to tunnel
                                let end_frame = TunnelFrame::new(
                                    session_id_clone,
                                    ProtocolType::WssFrame,
                                    true,
                                    Vec::new(),
                                );
                                if let Err(e) = write_frame(&mut send_clone, &end_frame).await {
                                    error!("[{}] Error sending end frame: {}", request_id_clone2, e);
                                }
                                break;
                            }
                            tokio_tungstenite::tungstenite::Message::Ping(_) | 
                            tokio_tungstenite::tungstenite::Message::Pong(_) => {
                                // Ignore ping/pong for now
                                continue;
                            }
                            _ => continue,
                        };
                        
                        // Send payload in chunks if needed
                        let mut offset = 0;
                        while offset < payload.len() {
                            let chunk_size = std::cmp::min(MAX_FRAME_SIZE, payload.len() - offset);
                            let chunk = payload[offset..offset + chunk_size].to_vec();
                            let is_last = offset + chunk_size >= payload.len();
                            
                            let frame = TunnelFrame::new(
                                session_id_clone,
                                ProtocolType::WssFrame,
                                is_last,
                                chunk,
                            );
                            
                            if let Err(e) = write_frame(&mut send_clone, &frame).await {
                                error!("[{}] Error sending frame to tunnel: {}", request_id_clone2, e);
                                return;
                            }
                            
                            offset += chunk_size;
                        }
                    }
                    Some(Err(e)) => {
                        error!("[{}] WebSocket error from backend: {}", request_id_clone2, e);
                        break;
                    }
                    None => {
                        debug!("[{}] Backend WebSocket stream ended", request_id_clone2);
                        break;
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
