use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};
use quinn::{Connection, SendStream, RecvStream};
use uuid::Uuid;
use bytes::BytesMut;
use crate::types::ClientState;
use crate::forwarder::Forwarder;
use tunnel_lib::protocol::{TunnelFrame, ProtocolType, write_frame, RoutingInfo};
use tunnel_lib::proto::tunnel::{Rule, Upstream};
use crate::stream_state::StreamStateMachine;

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
    
    let is_ssl = address.trim().starts_with("https://");
    Ok((address.trim().to_string(), is_ssl))
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
        
        let protocol_type = match routing_info.r#type.as_str() {
            "http" => ProtocolType::Http11,
            "wss" => ProtocolType::WssFrame,
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
                
                let error_response = format!(
                    "HTTP/1.1 502 Bad Gateway\r\n\
                    Content-Length: {}\r\n\
                    Content-Type: text/plain\r\n\
                    \r\n\
                    {}",
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
}
