use tunnel_lib::tunnel::tunnel_service_client::TunnelServiceClient;
use tunnel_lib::tunnel::*;
use tonic::Request;
use tokio_stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyper::{Body, Request as HyperRequest, Response as HyperResponse, Server as HyperServer, Method};
use hyper::service::{make_service_fn, service_fn};
use uuid::Uuid;
mod config;
use anyhow::Result;
use crate::config::{ClientConfig, Upstream};
use tracing::{info, error, debug};
use tracing_subscriber;
use chrono;
use tracing::Instrument;
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;
mod proxy;
use tunnel_lib::http_forward::set_host_header;
use tokio_util::sync::CancellationToken;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc as StdArc;
use tokio::sync::RwLock;
use hyper::Uri;
use url::Url;
use dashmap::DashMap;
use serde_json;
use tunnel_lib::proxy::{HttpTunnelContext, http_entry_handler};
use proxy::ClientHttpEntryTarget;

// 客户端本地规则引擎
#[derive(Clone)]
struct ClientRulesEngine {
    http_rules: Vec<Rule>,
    grpc_rules: Vec<Rule>,
    upstreams: HashMap<String, Upstream>,
}

impl ClientRulesEngine {
    fn new() -> Self {
        Self {
            http_rules: Vec::new(),
            grpc_rules: Vec::new(),
            upstreams: HashMap::new(),
        }
    }
    
    async fn update_rules(&mut self, rules: Vec<Rule>, upstreams: Vec<tunnel_lib::tunnel::Upstream>) {
        let old_http = self.http_rules.clone();
        let old_grpc = self.grpc_rules.clone();
        let old_upstreams = self.upstreams.clone();
        // 构造新规则和 upstreams
        let mut new_http = Vec::new();
        let mut new_grpc = Vec::new();
        let mut new_upstreams = HashMap::new();
        for rule in rules {
            if !rule.match_service.is_empty() {
                new_grpc.push(rule);
            } else {
                new_http.push(rule);
            }
        }
        for upstream in upstreams {
            let up = crate::config::Upstream {
                servers: upstream.servers.into_iter().map(|s| crate::config::ServerAddr {
                    address: s.address,
                    resolve: s.resolve,
                }).collect(),
                lb_policy: if upstream.lb_policy.is_empty() { None } else { Some(upstream.lb_policy) },
            };
            new_upstreams.insert(upstream.name, up);
        }
        // 聚合 debug 日志，减少遮盖力
        debug!(
            "rules_update: old_http_rules={:?}, new_http_rules={:?}, old_grpc_rules={:?}, new_grpc_rules={:?}, old_upstreams={:?}, new_upstreams={:?}",
            old_http, new_http, old_grpc, new_grpc, old_upstreams, new_upstreams
        );
        // 只有内容变化时才更新和打印日志
        if old_http != new_http || old_grpc != new_grpc || old_upstreams != new_upstreams {
            self.http_rules = new_http;
            self.grpc_rules = new_grpc;
            self.upstreams = new_upstreams;
            info!(
                event = "client_config_changed",
                old_rules_count = old_http.len() + old_grpc.len(),
                new_rules_count = self.http_rules.len() + self.grpc_rules.len(),
                old_upstreams_count = old_upstreams.len(),
                new_upstreams_count = self.upstreams.len(),
                message = "client config changed"
            );
        }
    }
    
    fn match_http_rule(&self, host: &str, path: &str) -> Option<&Rule> {
        self.http_rules.iter().find(|rule| {
            (rule.match_host.is_empty() || rule.match_host == host) &&
            (rule.match_path_prefix.is_empty() || path.starts_with(&rule.match_path_prefix))
        })
    }
    
    fn pick_backend(&self, upstream_name: &str) -> Option<String> {
        self.upstreams.get(upstream_name).and_then(|up| up.servers.get(0)).map(|s| s.address.clone())
    }

    pub fn debug_print_upstreams(&self) {
        println!("{:#?}", self.upstreams);
    }
}

struct TunnelClient {
    client_id: String,
    group_id: String,
    server_addr: String,
    tx: mpsc::Sender<TunnelMessage>,
    pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    rules_engine: Arc<Mutex<ClientRulesEngine>>,
    trace_enabled: bool,
    http_client: hyper::Client<HttpConnector>,
    https_client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl TunnelClient {
    fn new(client_id: String, group_id: String, server_addr: String, trace_enabled: bool, tx: mpsc::Sender<TunnelMessage>) -> Self {
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

    async fn register_group(&self, grpc_client: &mut TunnelServiceClient<tonic::transport::Channel>) -> Result<()> {
        let register_req = RegisterRequest {
            client_id: self.client_id.clone(),
            group: self.group_id.clone(),
            version: "v1.0.0".to_string(),
        };
        
        let response = grpc_client.register(Request::new(register_req)).await?;
        let resp = response.into_inner();
        
        if resp.success {
            info!("Successfully registered to group '{}': {}", self.group_id, resp.message);
        } else {
            error!("Failed to register to group '{}': {}", self.group_id, resp.message);
        }
        
        Ok(())
    }

    async fn sync_config(&self, grpc_client: &mut TunnelServiceClient<tonic::transport::Channel>) -> Result<()> {
        let config_req = ConfigSyncRequest {
            client_id: self.client_id.clone(),
            group: self.group_id.clone(),
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

    async fn connect_with_retry(&self, mut grpc_client: TunnelServiceClient<tonic::transport::Channel>, rx: &mut mpsc::Receiver<TunnelMessage>) -> Result<()> {
        // 注册到 group
        self.register_group(&mut grpc_client).await?;
        loop {
            let cancel_token = CancellationToken::new();
            let child_token = cancel_token.child_token();
            let heartbeat_handle = tokio::spawn(Self::heartbeat_task(child_token.clone(), self.tx.clone(), self.client_id.clone()));
            let config_sync_handle = tokio::spawn(Self::config_sync_task(child_token.clone(), self.tx.clone(), self.client_id.clone(), self.group_id.clone(), self.trace_enabled));
            // 每次重连时取出 rx 的所有权
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
                                // 1. tonic::Status 直接用 e
                                match e.code() {
                                    Code::Unknown | Code::Unavailable => {
                                        should_break_outer = true;
                                    }
                                    _ => {}
                                }
                                // 额外检查 message
                                let msg = e.message().to_ascii_lowercase();
                                if msg.contains("broken pipe") || msg.contains("connection refused") {
                                    should_break_outer = true;
                                }
                                // 2. 其他 IO 错误
                                let err_str = e.to_string().to_ascii_lowercase();
                                if err_str.contains("broken pipe") || err_str.contains("connection refused") {
                                    should_break_outer = true;
                                }
                                error!("Error receiving message: {}", e);
                                if should_break_outer {
                                    // 让 connect_with_retry 返回错误，main 外层 loop 负责重连
                                    return Err(anyhow::anyhow!("Critical tunnel error: {}", e));
                                } else {
                                    break; // 只重连 stream
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Tunnel stream error: {}", e);
                }
            }
            cancel_token.cancel();
            let _ = heartbeat_handle.await;
            let _ = config_sync_handle.await;
            info!("Tunnel stream closed, will reconnect");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    // 新增：带 CancellationToken 和 client_id 的心跳任务
    async fn heartbeat_task(cancel_token: CancellationToken, tx: mpsc::Sender<TunnelMessage>, client_id: String) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let heartbeat = TunnelMessage {
                        client_id: client_id.clone(), // 用真实 client_id
                        request_id: Uuid::new_v4().to_string(),
                        direction: Direction::ClientToServer as i32,
                        payload: Some(tunnel_message::Payload::Heartbeat(Heartbeat {
                            timestamp: chrono::Utc::now().timestamp(),
                        })),
                        trace_id: String::new(),
                    };
                    if tx.send(heartbeat).await.is_err() {
                        debug!("Heartbeat channel closed, exit heartbeat task");
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

    // 新增：带 CancellationToken 和 client_id 的配置同步任务
    async fn config_sync_task(cancel_token: CancellationToken, tx: mpsc::Sender<TunnelMessage>, client_id: String, group_id: String, trace_enabled: bool) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let config_sync_msg = TunnelMessage {
                        client_id: client_id.clone(), // 用真实 client_id
                        request_id: Uuid::new_v4().to_string(),
                        direction: Direction::ClientToServer as i32,
                        payload: Some(tunnel_message::Payload::ConfigSync(ConfigSyncRequest {
                            client_id: client_id.clone(),
                            group: group_id.clone(),
                            config_version: "".to_string(),
                        })),
                        trace_id: String::new(),
                    };
                    if tx.send(config_sync_msg).await.is_err() {
                        debug!("Config sync channel closed, exit config sync task");
                        break;
                    }
                    debug!("Sent config sync request via tunnel");
                }
                _ = cancel_token.cancelled() => {
                    debug!("Config sync task cancelled");
                    break;
                }
            }
        }
    }

    async fn handle_tunnel_message(&self, msg: TunnelMessage, tx: &mpsc::Sender<TunnelMessage>) {
        let trace_id = msg.trace_id.clone();
        let request_id = msg.request_id.clone();
        let span = tracing::info_span!("client_handle_tunnel", trace_id = %trace_id, request_id = %request_id);
        async {
            match msg.payload {
                Some(tunnel_message::Payload::HttpRequest(req)) => {
                    // 处理来自 server 的 HTTP 请求（反向代理）
                    if msg.direction == Direction::ServerToClient as i32 {
                        let host = req.host.clone();
                        let path = req.url.parse::<Uri>().map(|u| u.path().to_string()).unwrap_or_else(|_| "/".to_string());
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
                            client_id: self.client_id.clone(),
                            request_id: request_id.clone(),
                            direction: Direction::ClientToServer as i32,
                            payload: Some(tunnel_message::Payload::HttpResponse(response)),
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
                Some(tunnel_message::Payload::HttpResponse(resp)) => {
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
                Some(tunnel_message::Payload::ConfigSyncResponse(resp)) => {
                    let mut rules_engine = self.rules_engine.lock().await;
                    debug!(
                        "configsync_response: old_rules={:?}, new_rules={:?}, old_upstreams={:?}, new_upstreams={:?}",
                        rules_engine.http_rules, resp.rules, rules_engine.upstreams, resp.upstreams
                    );
                    let old_rules = rules_engine.http_rules.clone();
                    let old_upstreams = rules_engine.upstreams.clone();
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
                Some(tunnel_message::Payload::Heartbeat(_)) => {
                    tracing::info!(
                        event = "client_received_heartbeat",
                        trace_id = %trace_id,
                        request_id = %request_id,
                        message = "Received heartbeat from server"
                    );
                }
                _ => {}
            }
        }
        .instrument(span)
        .await;
    }

    async fn handle_tunnel_http_request(&self, req: HttpRequest, _request_id: String, trace_id: String) -> HttpResponse {
        let host = req.host.as_str();
        // 用 hyper::Uri 解析 path
        let path = req.url.parse::<Uri>().map(|u| u.path().to_string()).unwrap_or_else(|_| "/".to_string());

        let rules_engine = self.rules_engine.lock().await;
        let maybe_rule = rules_engine.match_http_rule(host, &path);
        match maybe_rule {
            Some(rule) => {
                info!("Processing tunnel request: {} {} host={} matched_rule=host:{} path_prefix:{} upstream:{} proxy_pass:{}", req.method, req.url, host, rule.match_host, rule.match_path_prefix, rule.action_upstream, rule.action_proxy_pass);
                // Handle action_proxy_pass
                if !rule.action_proxy_pass.is_empty() {
                    info!("Forwarding to backend URL: {}", rule.action_proxy_pass);
                    return self.forward_to_backend(&req, &rule.action_proxy_pass, trace_id.clone()).await;
                }
                // Handle action_upstream
                if !rule.action_upstream.is_empty() {
                    if let Some(backend) = rules_engine.pick_backend(&rule.action_upstream) {
                        let mut req = req.clone();
                        // 解析 backend host
                        let backend_host = match Url::parse(&backend) {
                            Ok(url) => url.host_str().unwrap_or("").to_string(),
                            Err(_) => String::new(),
                        };
                        // 优先用 action_set_host，否则用 backend host
                        let set_host = if !rule.action_set_host.is_empty() {
                            rule.action_set_host.as_str()
                        } else {
                            backend_host.as_str()
                        };
                        set_host_header(&mut req.headers, set_host);
                        // 用 url::Url::join 拼接 backend 和 req.url
                        let full_url = match Url::parse(&backend).and_then(|b| b.join(&req.url)) {
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
        // 没有匹配规则，直接返回 404
        return tunnel_lib::response::resp_404(
            Some(&trace_id),
            Some(&_request_id),
            Some(&self.client_id),
        );
    }

    async fn forward_to_backend(&self, req: &HttpRequest, target_url: &str, trace_id: String) -> HttpResponse {
        let client = &self.https_client;
        let method = match req.method.parse::<Method>() {
            Ok(m) => m,
            Err(_) => return HttpResponse {
                status_code: 400,
                headers: HashMap::new(),
                body: b"Invalid method".to_vec(),
            },
        };
        let url = match Url::parse(target_url).and_then(|base| base.join(&req.url)) {
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
                    Some(&self.client_id),
                );
            }
        }
    }

    async fn connect_with_retry_with_token(&self, mut grpc_client: TunnelServiceClient<tonic::transport::Channel>, rx: &mut mpsc::Receiver<TunnelMessage>, token: CancellationToken) -> Result<()> {
        self.register_group(&mut grpc_client).await?;
        loop {
            let child_token = token.child_token();
            let heartbeat_handle = tokio::spawn(Self::heartbeat_task(child_token.clone(), self.tx.clone(), self.client_id.clone()));
            let config_sync_handle = tokio::spawn(Self::config_sync_task(child_token.clone(), self.tx.clone(), self.client_id.clone(), self.group_id.clone(), self.trace_enabled));
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ClientConfig::load("../config/client.toml")?;
    let log_level = config.log_level.as_str();
    let filter = match log_level {
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    tracing_subscriber::fmt()
        .with_max_level(filter)
        .json()
        .init();
    debug!("Loaded config: {:?}", config);
    info!("Loaded client config: {:?}", config);

    let server_addr = format!("http://{}:{}", config.server_addr, config.server_port);
    let client_group_id = config.client_group_id.clone();
    let trace_enabled = config.trace_enabled.unwrap_or(false);
    let http_port = config.http_entry_port.unwrap_or(8003);
    let http_addr = format!("0.0.0.0:{}", http_port);

    // HTTP入口监听只启动一次，使用动态 tunnel_tx、pending_requests
    let tunnel_tx_holder = StdArc::new(RwLock::new(None));
    let pending_requests_holder = StdArc::new(RwLock::new(None));
    let client_id_holder = StdArc::new(RwLock::new(None));
    let token_holder = StdArc::new(RwLock::new(None::<CancellationToken>));

    // 启动 HTTP 入口监听，只启动一次
    {
        let tunnel_tx_holder = tunnel_tx_holder.clone();
        let pending_requests_holder = pending_requests_holder.clone();
        let client_id_holder = client_id_holder.clone();
        tokio::spawn(async move {
            let tunnel_tx = tunnel_tx_holder.clone();
            let client_id = client_id_holder.clone();
            let target = Arc::new(ClientHttpEntryTarget {
                tunnel_tx,
                pending_requests: pending_requests_holder.clone(),
                client_id,
            });
            let ctx = HttpTunnelContext {
                client_id: "client".to_string(), // 仅作标识
                tunnel_tx: Arc::new(mpsc::channel(1).0), // dummy
                pending_requests: Arc::new(DashMap::new()),
                direction: tunnel_lib::tunnel::Direction::ClientToServer,
            };
            let make_svc = make_service_fn(move |_| {
                let ctx = ctx.clone();
                let target = target.clone();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        let ctx = ctx.clone();
                        let target = target.clone();
                        async move {
                            http_entry_handler(req, &ctx, &*target).await
                        }
                    }))
                }
            });
            let server = hyper::Server::bind(&http_addr.parse().unwrap()).serve(make_svc);
            info!("Client HTTP entry listening on http://{} (tunnel-lib handler)", http_addr);
            if let Err(e) = server.await {
                error!("Client HTTP entry server error: {}", e);
            }
        });
    }

    // main loop，每次重建所有资源
    loop {
        // 1. 新建 channel
        let (tx, rx) = mpsc::channel(128);
        // 2. 新建 TunnelClient
        let client_id = format!("client-{}", Uuid::new_v4());
        let group_id = client_group_id.clone();
        let tunnel_client = Arc::new(TunnelClient::new(client_id.clone(), group_id.clone(), server_addr.clone(), trace_enabled, tx.clone()));
        // 3. 更新 HTTP入口监听用的 tunnel_tx、pending_requests、client_id、token
        {
            let mut t = tunnel_tx_holder.write().await;
            *t = Some(tx.clone());
            let mut p = pending_requests_holder.write().await;
            *p = Some(tunnel_client.pending_requests.clone());
            let mut c = client_id_holder.write().await;
            *c = Some(client_id.clone());
            let mut token_w = token_holder.write().await;
            *token_w = None; // 先清空
        }
        // 4. 打印 upstreams（可选）
        let rules_engine = tunnel_client.rules_engine.lock().await;
        rules_engine.debug_print_upstreams();
        drop(rules_engine);
        // 5. 建立 gRPC client
        let mut rx = rx;
        info!("==== [MAIN] Creating new gRPC client connection ====");
        match TunnelServiceClient::connect(server_addr.clone()).await {
            Ok(grpc_client) => {
                // 新建 token
                let token = CancellationToken::new();
                {
                    let mut token_w = token_holder.write().await;
                    *token_w = Some(token.clone());
                }
                if let Err(e) = tunnel_client.connect_with_retry_with_token(grpc_client, &mut rx, token.clone()).await {
                    error!("Tunnel connection lost: {e}, retrying in 5s...");
                }
            }
            Err(e) => {
                error!("Failed to connect to server: {e}, retrying in 5s...");
            }
        }
        // 清理 token/资源
        {
            let mut token_w = token_holder.write().await;
            *token_w = None;
            let mut t = tunnel_tx_holder.write().await;
            *t = None;
            let mut p = pending_requests_holder.write().await;
            if let Some(pending_requests_arc) = p.as_ref() {
                let client_id: String = client_id_holder.read().await.clone().unwrap();
                let keys: Vec<_> = pending_requests_arc.iter().map(|entry| entry.key().clone()).collect();
                for request_id in keys {
                    if let Some((_, sender)) = pending_requests_arc.remove(&request_id) {
                        let resp = tunnel_lib::response::resp_502(
                            Some("Tunnel closed"),
                            None,
                            Some(client_id.as_str()),
                        );
                        let _ = sender.send(resp);
                    }
                }
            }
            *p = None;
            let mut c = client_id_holder.write().await;
            *c = None;
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
} 