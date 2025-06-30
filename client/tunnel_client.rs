use tunnel_lib::tunnel::tunnel_service_client::TunnelServiceClient;
use tunnel_lib::tunnel::*;
use tonic::Request;
use tokio_stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyper::{Body, Request as HyperRequest, Method};
use uuid::Uuid;
use tracing::{info, error, debug};
use chrono;
use tracing::Instrument;
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;
use tokio_util::sync::CancellationToken;
use std::sync::Arc as StdArc;
use dashmap::DashMap;
use tunnel_lib::proxy::HttpTunnelContext;
use crate::rules_engine::ClientRulesEngine;
use tunnel_lib::tunnel::{TunnelMessage, StreamType, Direction};

pub struct TunnelClient {
    pub client_id: Arc<String>,
    pub group_id: Arc<String>,
    pub server_addr: String,
    pub tx: mpsc::Sender<TunnelMessage>,
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    pub rules_engine: Arc<Mutex<ClientRulesEngine>>,
    pub trace_enabled: bool,
    pub http_client: hyper::Client<HttpConnector>,
    pub https_client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl TunnelClient {
    pub fn new(client_id: Arc<String>, group_id: Arc<String>, server_addr: String, trace_enabled: bool, tx: mpsc::Sender<TunnelMessage>) -> Self {
        let http_client = hyper::Client::new();
        let https_connector = HttpsConnector::new();
        let https_client = hyper::Client::builder().build::<_, hyper::Body>(https_connector);
        Self {
            client_id,
            group_id,
            server_addr,
            tx,
            pending_requests: Arc::new(DashMap::new()),
            rules_engine: Arc::new(Mutex::new(ClientRulesEngine::new())),
            trace_enabled,
            http_client,
            https_client,
        }
    }

    pub async fn sync_config(&self, grpc_client: &mut TunnelServiceClient<tonic::transport::Channel>) -> anyhow::Result<()> {
        let config_req = ConfigSyncRequest {
            client_id: (*self.client_id).clone(),
            group: (*self.group_id).clone(),
            config_version: "".to_string(),
        };
        let response = grpc_client.config_sync(Request::new(config_req)).await?;
        let resp = response.into_inner();
        info!("Synced config version: {}, got {} rules, {} upstreams", resp.config_version, resp.rules.len(), resp.upstreams.len());
        debug!(
            "Received config rules: {:?}, upstreams: {:?}",
            resp.rules, resp.upstreams
        );
        self.rules_engine.lock().await.update_rules(resp.rules, resp.upstreams).await;
        Ok(())
    }

    pub async fn connect_with_retry_with_token(&self, mut grpc_client: TunnelServiceClient<tonic::transport::Channel>, rx: &mut mpsc::Receiver<TunnelMessage>, token: CancellationToken) -> anyhow::Result<()> {
        loop {
            let child_token = token.child_token();
            let heartbeat_handle = tokio::spawn(Self::heartbeat_task(child_token.clone(), self.tx.clone(), Arc::clone(&self.client_id), Arc::clone(&self.group_id)));
            let config_sync_handle = tokio::spawn(Self::config_sync_task(child_token.clone(), self.tx.clone(), Arc::clone(&self.client_id), Arc::clone(&self.group_id)));
            let stream_id = Uuid::new_v4().to_string();
            let stream_type = StreamType::Http;
            let connect_msg = TunnelMessage {
                client_id: (*self.client_id).clone(),
                request_id: Uuid::new_v4().to_string(),
                direction: Direction::ClientToServer as i32,
                payload: Some(tunnel_lib::tunnel::tunnel_message::Payload::StreamOpen(
                    StreamOpenRequest {
                        client_id: (*self.client_id).clone(),
                        group: (*self.group_id).clone(),
                        version: "v1.0.0".to_string(),
                        stream_id: stream_id.clone(),
                        stream_type: stream_type as i32,
                        timestamp: chrono::Utc::now().timestamp(),
                    }
                )),
                trace_id: String::new(),
            };
            if let Err(e) = self.tx.send(connect_msg).await {
                error!("Failed to send StreamOpenRequest: {}", e);
                return Err(anyhow::anyhow!("Failed to send StreamOpenRequest: {}", e));
            }
            let outbound = tokio_stream::wrappers::ReceiverStream::new(std::mem::replace(rx, mpsc::channel(128).1));
            let response = grpc_client.proxy(Request::new(outbound)).await;
            match response {
                Ok(resp) => {
                    let mut inbound = resp.into_inner();
                    info!("Tunnel connection established successfully");
                    while let Some(message) = inbound.next().await {
                        match message {
                            Ok(msg) => {
                                self.handle_tunnel_message(msg, &self.tx).await;
                            }
                            Err(e) => {
                                use tonic::Code;
                                let mut should_break_outer = false;
                                match e.code() {
                                    Code::Unknown | Code::Unavailable => {
                                        should_break_outer = true;
                                    }
                                    _ => {}
                                }
                                let msg = e.message().to_ascii_lowercase();
                                if msg.contains("broken pipe") || msg.contains("connection refused") {
                                    should_break_outer = true;
                                }
                                let err_str = e.to_string().to_ascii_lowercase();
                                if err_str.contains("broken pipe") || err_str.contains("connection refused") {
                                    should_break_outer = true;
                                }
                                error!("Error receiving message: {}", e);
                                if should_break_outer {
                                    return Err(anyhow::anyhow!("Critical tunnel error: {}", e));
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Tunnel stream error: {}", e);
                }
            }
            token.cancel();
            let _ = heartbeat_handle.await;
            let _ = config_sync_handle.await;
            info!("Tunnel stream closed, will reconnect");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    async fn heartbeat_task(cancel_token: CancellationToken, tx: mpsc::Sender<TunnelMessage>, client_id: Arc<String>, group_id: Arc<String>) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        interval.tick().await;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let heartbeat = TunnelMessage {
                        client_id: (*client_id).clone(),
                        request_id: Uuid::new_v4().to_string(),
                        direction: Direction::ClientToServer as i32,
                        payload: Some(tunnel_lib::tunnel::tunnel_message::Payload::StreamOpen(
                            StreamOpenRequest {
                                client_id: (*client_id).clone(),
                                group: (*group_id).clone(),
                                version: "v1.0.0".to_string(),
                                stream_id: Uuid::new_v4().to_string(),
                                stream_type: StreamType::Http as i32,
                                timestamp: chrono::Utc::now().timestamp(),
                            }
                        )),
                        trace_id: String::new(),
                    };
                    if tx.send(heartbeat).await.is_err() {
                        debug!("StreamOpenRequest channel closed, exit heartbeat task");
                        break;
                    }
                }
                _ = cancel_token.cancelled() => {
                    debug!("Heartbeat task cancelled");
                    break;
                }
            }
        }
    }

    async fn config_sync_task(cancel_token: CancellationToken, tx: mpsc::Sender<TunnelMessage>, client_id: Arc<String>, group_id: Arc<String>) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let config_sync_msg = TunnelMessage {
                        client_id: (*client_id).clone(),
                        request_id: Uuid::new_v4().to_string(),
                        direction: Direction::ClientToServer as i32,
                        payload: Some(tunnel_lib::tunnel::tunnel_message::Payload::ConfigSync(
                            ConfigSyncRequest {
                                client_id: (*client_id).clone(),
                                group: (*group_id).clone(),
                                config_version: "".to_string(),
                            }
                        )),
                        trace_id: String::new(),
                    };
                    if tx.send(config_sync_msg).await.is_err() {
                        debug!("ConfigSyncRequest channel closed, exit config sync task");
                        break;
                    }
                    debug!("Sent ConfigSyncRequest via tunnel");
                }
                _ = cancel_token.cancelled() => {
                    debug!("ConfigSyncRequest task cancelled");
                    break;
                }
            }
        }
    }

    pub async fn handle_tunnel_message(&self, msg: TunnelMessage, tx: &mpsc::Sender<TunnelMessage>) {
        let trace_id = msg.trace_id.clone();
        let request_id = msg.request_id.clone();
        let span = tracing::info_span!("client_handle_tunnel", trace_id = %trace_id, request_id = %request_id);
        async {
            match msg.payload {
                Some(tunnel_lib::tunnel::tunnel_message::Payload::HttpRequest(req)) => {
                    // 处理来自 server 的 HTTP 请求（反向代理）
                    if msg.direction == Direction::ServerToClient as i32 {
                        let host = req.host.clone();
                        let path = req.url.parse::<hyper::Uri>().map(|u| u.path().to_string()).unwrap_or_else(|_| "/".to_string());
                        tracing::info!(
                            event = "client_received_tunnel_request",
                            trace_id = %trace_id,
                            request_id = %request_id,
                            host = %host,
                            path = %path,
                            message = "client received tunnel request from server"
                        );
                        let response = self.handle_tunnel_http_request(req, request_id.clone(), trace_id.clone()).await;
                        let response_msg = TunnelMessage {
                            client_id: (*self.client_id).clone(),
                            request_id: request_id.clone(),
                            direction: Direction::ClientToServer as i32,
                            payload: Some(tunnel_lib::tunnel::tunnel_message::Payload::HttpResponse(response)),
                            trace_id: trace_id.clone(),
                        };
                        if let Err(e) = tx.send(response_msg).await {
                            tracing::error!(
                                event = "client_send_response_error",
                                trace_id = %trace_id,
                                request_id = %request_id,
                                error = %e,
                                message = "Failed to send response to server"
                            );
                        }
                    }
                }
                Some(tunnel_lib::tunnel::tunnel_message::Payload::HttpResponse(resp)) => {
                    tracing::info!(
                        event = "client_received_tunnel_response",
                        trace_id = %trace_id,
                        request_id = %request_id,
                        status_code = resp.status_code,
                        message = "client received tunnel response from server"
                    );
                    if let Some((_, sender)) = self.pending_requests.remove(&msg.request_id) {
                        let _ = sender.send(resp);
                    }
                }
                Some(tunnel_lib::tunnel::tunnel_message::Payload::ConfigSyncResponse(resp)) => {
                    let mut rules_engine = self.rules_engine.lock().await;
                    debug!(
                        "configsync_response: old_rules={:?}, new_rules={:?}, old_upstreams={:?}, new_upstreams={:?}",
                        rules_engine.http_rules(), resp.rules, rules_engine.upstreams(), resp.upstreams
                    );
                    let old_rules = rules_engine.http_rules().clone();
                    let old_upstreams = rules_engine.upstreams().clone();
                    let old_rules_count = old_rules.len();
                    let old_upstreams_count = old_upstreams.len();
                    let new_rules_count = resp.rules.len();
                    let new_upstreams_count = resp.upstreams.len();
                    if old_rules_count != new_rules_count || old_upstreams_count != new_upstreams_count {
                        tracing::info!(
                            event = "client_config_changed",
                            trace_id = %trace_id,
                            request_id = %request_id,
                            old_rules_count = old_rules_count,
                            new_rules_count = new_rules_count,
                            old_upstreams_count = old_upstreams_count,
                            new_upstreams_count = new_upstreams_count,
                            message = "client config changed"
                        );
                    }
                    rules_engine.update_rules(resp.rules, resp.upstreams).await;
                }
                Some(tunnel_lib::tunnel::tunnel_message::Payload::StreamOpen(
                    StreamOpenRequest {
                        client_id: _,
                        group: _,
                        version: _,
                        stream_id: _,
                        stream_type: _,
                        timestamp: _,
                    }
                )) => {
                    tracing::info!(
                        event = "client_received_stream_open_request",
                        trace_id = %trace_id,
                        request_id = %request_id,
                        message = "Received StreamOpenRequest from server"
                    );
                }
                _ => {}
            }
        }
        .instrument(span)
        .await;
    }

    pub async fn handle_tunnel_http_request(&self, req: HttpRequest, _request_id: String, trace_id: String) -> HttpResponse {
        let host = req.host.as_str();
        let path = req.url.parse::<hyper::Uri>().map(|u| u.path().to_string()).unwrap_or_else(|_| "/".to_string());
        let rules_engine = self.rules_engine.lock().await;
        let maybe_rule = rules_engine.match_http_rule(host, &path);
        match maybe_rule {
            Some(rule) => {
                info!("Processing tunnel request: {} {} host={} matched_rule=host:{} path_prefix:{} upstream:{} proxy_pass:{}", req.method, req.url, host, rule.match_host, rule.match_path_prefix, rule.action_upstream, rule.action_proxy_pass);
                if !rule.action_proxy_pass.is_empty() {
                    info!("Forwarding to backend URL: {}", rule.action_proxy_pass);
                    return self.forward_to_backend(&req, &rule.action_proxy_pass, trace_id.clone()).await;
                }
                if !rule.action_upstream.is_empty() {
                    if let Some(backend) = rules_engine.pick_backend(&rule.action_upstream) {
                        let mut req = req.clone();
                        let backend_host = match url::Url::parse(&backend) {
                            Ok(url) => url.host_str().unwrap_or("").to_string(),
                            Err(_) => String::new(),
                        };
                        let set_host = if !rule.action_set_host.is_empty() {
                            rule.action_set_host.as_str()
                        } else {
                            backend_host.as_str()
                        };
                        tunnel_lib::http_forward::set_host_header(&mut req.headers, set_host);
                        let full_url = match url::Url::parse(&backend).and_then(|b| b.join(&req.url)) {
                            Ok(url) => url.to_string(),
                            Err(e) => {
                                error!("Failed to join backend and req.url: {}", e);
                                backend.clone()
                            }
                        };
                        info!("Forwarding to backend URL: {}", full_url);
                        return self.forward_to_backend(&req, &full_url, trace_id.clone()).await;
                    }
                }
            }
            None => {
                info!("Processing tunnel request: {} {} host={} no_match", req.method, req.url, host);
            }
        }
        return tunnel_lib::response::resp_404(
            Some(&trace_id),
            Some(&_request_id),
            Some(&(*self.client_id)),
        );
    }

    pub async fn forward_to_backend(&self, req: &HttpRequest, target_url: &str, trace_id: String) -> HttpResponse {
        let client = &self.https_client;
        let method = match req.method.parse::<Method>() {
            Ok(m) => m,
            Err(_) => return HttpResponse {
                status_code: 400,
                headers: HashMap::new(),
                body: b"Invalid method".to_vec(),
            },
        };
        let url = match url::Url::parse(target_url).and_then(|base| base.join(&req.url)) {
            Ok(url) => url.to_string(),
            Err(e) => {
                error!("Failed to join target_url and req.url: {}", e);
                format!("{}{}", target_url, req.url)
            }
        };
        debug!("Forwarding HTTP(S) request to: {}", url);
        let mut builder = HyperRequest::builder()
            .method(method)
            .uri(&url);
        for (k, v) in req.headers.iter() {
            builder = builder.header(k, v);
        }
        let hyper_req = match builder.body(Body::from(req.body.clone())) {
            Ok(req) => req,
            Err(_) => return HttpResponse {
                status_code: 400,
                headers: HashMap::new(),
                body: b"Invalid request".to_vec(),
            },
        };
        match client.request(hyper_req).await {
            Ok(resp) => {
                let status = resp.status().as_u16() as i32;
                let headers: HashMap<String, String> = resp
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                let body_bytes = match hyper::body::to_bytes(resp.into_body()).await {
                    Ok(bytes) => bytes.to_vec(),
                    Err(_) => b"Failed to read response body".to_vec(),
                };
                if self.trace_enabled {
                    tracing::info!(
                        event = "client_received_backend_response",
                        trace_id = %trace_id,
                        host = %req.host,
                        path = %req.path,
                        backend = %target_url,
                        status_code = status,
                        message = "client received HTTP(S) response from backend"
                    );
                }
                HttpResponse {
                    status_code: status,
                    headers,
                    body: body_bytes,
                }
            }
            Err(e) => {
                if self.trace_enabled {
                    tracing::error!(
                        event = "client_backend_error",
                        trace_id = %trace_id,
                        host = %req.host,
                        path = %req.path,
                        backend = %target_url,
                        error = %e,
                        message = "HTTP(S) request to backend failed"
                    );
                }
                return tunnel_lib::response::resp_500(
                    Some("Internal server error"),
                    Some(&trace_id),
                    None,
                    Some(&(*self.client_id)),
                );
            }
        }
    }
} 