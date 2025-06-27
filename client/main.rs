use tunnel_lib::tunnel::tunnel_service_client::TunnelServiceClient;
use tunnel_lib::tunnel::*;
use tonic::Request;
use tokio_stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    
    fn update_rules(&mut self, rules: Vec<Rule>, upstreams: Vec<tunnel_lib::tunnel::Upstream>) {
        let old_http_count = self.http_rules.len();
        let old_grpc_count = self.grpc_rules.len();
        let old_upstreams_count = self.upstreams.len();
        self.http_rules.clear();
        self.grpc_rules.clear();
        self.upstreams.clear();
        for rule in rules {
            if !rule.match_service.is_empty() {
                self.grpc_rules.push(rule);
            } else {
                self.http_rules.push(rule);
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
            self.upstreams.insert(upstream.name, up);
        }
        let new_http_count = self.http_rules.len();
        let new_grpc_count = self.grpc_rules.len();
        let new_upstreams_count = self.upstreams.len();
        if old_http_count != new_http_count || old_grpc_count != new_grpc_count || old_upstreams_count != new_upstreams_count {
            info!("Updated local rules: {} HTTP, {} gRPC, {} upstreams", new_http_count, new_grpc_count, new_upstreams_count);
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
}

struct TunnelClient {
    client_id: String,
    group_id: String,
    server_addr: String,
    tx: mpsc::Sender<TunnelMessage>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<HttpResponse>>>>,
    rules_engine: Arc<Mutex<ClientRulesEngine>>,
    trace_enabled: bool,
    http_client: hyper::Client<HttpConnector>,
    https_client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl TunnelClient {
    fn new(client_id: String, group_id: String, server_addr: String, trace_enabled: bool) -> (Self, mpsc::Receiver<TunnelMessage>) {
        let (tx, rx) = mpsc::channel(128);
        let http_client = hyper::Client::new();
        let https_connector = HttpsConnector::new();
        let https_client = hyper::Client::builder().build::<_, hyper::Body>(https_connector);
        let client = Self {
            client_id,
            group_id,
            server_addr,
            tx,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            rules_engine: Arc::new(Mutex::new(ClientRulesEngine::new())),
            trace_enabled,
            http_client,
            https_client,
        };
        (client, rx)
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
        
        info!("Synced config version: {}, got {} rules, {} upstreams", 
                resp.config_version, resp.rules.len(), resp.upstreams.len());
        
        // 更新本地规则引擎
        self.rules_engine.lock().unwrap().update_rules(resp.rules, resp.upstreams);
        
        Ok(())
    }

    async fn start_config_sync(&self) {
        let client_id = self.client_id.clone();
        let group_id = self.group_id.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                // 通过tunnel连接发送配置同步请求
                let config_sync_msg = TunnelMessage {
                    client_id: client_id.clone(),
                    request_id: Uuid::new_v4().to_string(),
                    direction: Direction::ClientToServer as i32,
                    payload: Some(tunnel_message::Payload::ConfigSync(ConfigSyncRequest {
                        client_id: client_id.clone(),
                        group: group_id.clone(),
                        config_version: "".to_string(),
                    })),
                    trace_id: String::new(),
                };
                
                // 这里需要通过tunnel连接发送，而不是重新连接
                // 暂时跳过，避免中断连接
                debug!("Config sync via tunnel (not implemented yet)");
            }
        });
    }

    async fn start(&self) -> Result<()> {
        loop {
            info!("Connecting to server at {}...", self.server_addr);
            match self.connect_with_retry().await {
                Ok(_) => info!("Connection lost, reconnecting..."),
                Err(e) => {
                    error!("Connection failed: {}, retrying in 5 seconds...", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect_with_retry(&self) -> Result<()> {
        let mut grpc_client = TunnelServiceClient::connect(self.server_addr.clone()).await?;
        let (tx, rx) = mpsc::channel(128);
        
        // 注册到 group
        self.register_group(&mut grpc_client).await?;
        
        // 启动心跳机制
        self.start_heartbeat(tx.clone()).await;
        
        // 启动配置同步（通过tunnel连接）
        self.start_config_sync_via_tunnel(tx.clone()).await;
        
        // 创建双向流
        let outbound = tokio_stream::wrappers::ReceiverStream::new(rx);
        let response = grpc_client.proxy(Request::new(outbound)).await?;
        let mut inbound = response.into_inner();
        
        info!("Tunnel connection established successfully");
        
        // 处理来自服务器的消息
        while let Some(message) = inbound.next().await {
            match message {
                Ok(msg) => {
                    self.handle_tunnel_message(msg, &tx).await;
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
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
                        let path = req.path.clone();
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
                    if let Some(sender) = self.pending_requests.lock().unwrap().remove(&msg.request_id) {
                        let _ = sender.send(resp);
                    }
                }
                Some(tunnel_message::Payload::ConfigSyncResponse(resp)) => {
                    let mut rules_engine = self.rules_engine.lock().unwrap();
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
                    rules_engine.update_rules(resp.rules, resp.upstreams);
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
        let path = req.url.split('?').next().unwrap_or("/");

        let rules_engine = self.rules_engine.lock().unwrap();
        let maybe_rule = rules_engine.match_http_rule(host, path);
        match maybe_rule {
            Some(rule) => {
                info!("Processing tunnel request: {} {} host={} matched_rule=host:{} path_prefix:{} upstream:{} proxy_pass:{}", req.method, req.url, host, rule.match_host, rule.match_path_prefix, rule.action_upstream, rule.action_proxy_pass);
                // 处理 action_proxy_pass
                if !rule.action_proxy_pass.is_empty() {
                    return self.forward_to_backend(&req, &rule.action_proxy_pass, trace_id.clone()).await;
                }
                // 处理 action_upstream
                if !rule.action_upstream.is_empty() {
                    if let Some(backend) = rules_engine.pick_backend(&rule.action_upstream) {
                        return self.forward_to_backend(&req, &backend, trace_id.clone()).await;
                    }
                }
            }
            None => {
                info!("Processing tunnel request: {} {} host={} no_match", req.method, req.url, host);
            }
        }
        // 默认转发到本地服务
        let local_url = format!("http://127.0.0.1:3000{}", req.path);
        self.forward_to_backend(&req, &local_url, trace_id).await
    }

    async fn forward_to_backend(&self, req: &HttpRequest, target_url: &str, trace_id: String) -> HttpResponse {
        // 根据target_url判断是否使用HTTPS
        let is_https = if target_url.starts_with("https://") {
            true
        } else if target_url.starts_with("http://") {
            false
        } else {
            // 如果没有协议前缀，根据端口判断
            target_url.contains(":443")
        };
        if is_https {
            self.forward_to_backend_https(req, target_url, trace_id).await
        } else {
            self.forward_to_backend_http(req, target_url, trace_id).await
        }
    }

    async fn forward_to_backend_http(&self, req: &HttpRequest, target_url: &str, trace_id: String) -> HttpResponse {
        let client = &self.http_client;
        let method = match req.method.parse::<Method>() {
            Ok(m) => m,
            Err(_) => return HttpResponse {
                status_code: 400,
                headers: HashMap::new(),
                body: b"Invalid method".to_vec(),
            },
        };
        let url = if target_url.starts_with("http://") {
            format!("{}{}", target_url, req.url.split('?').skip(1).next().map(|q| format!("?{}", q)).unwrap_or_default())
        } else {
            format!("http://{}{}", target_url, req.url)
        };
        if self.trace_enabled {
            debug!("Forwarding HTTP request to: {}", url);
        }
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
                        message = "client received HTTP response from backend"
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
                        message = "HTTP request to backend failed"
                    );
                }
                HttpResponse {
                    status_code: 500,
                    headers: HashMap::new(),
                    body: b"Request failed".to_vec(),
                }
            }
        }
    }

    async fn forward_to_backend_https(&self, req: &HttpRequest, target_url: &str, trace_id: String) -> HttpResponse {
        let client = &self.https_client;
        let method = match req.method.parse::<Method>() {
            Ok(m) => m,
            Err(_) => return HttpResponse {
                status_code: 400,
                headers: HashMap::new(),
                body: b"Invalid method".to_vec(),
            },
        };
        let url = if target_url.starts_with("https://") {
            format!("{}{}", target_url, req.url.split('?').skip(1).next().map(|q| format!("?{}", q)).unwrap_or_default())
        } else {
            format!("https://{}{}", target_url, req.url)
        };
        if self.trace_enabled {
            debug!("Forwarding HTTPS request to: {}", url);
        }
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
                        message = "client received HTTP response from backend"
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
                        message = "HTTPS request to backend failed"
                    );
                }
                HttpResponse {
                    status_code: 500,
                    headers: HashMap::new(),
                    body: b"Request failed".to_vec(),
                }
            }
        }
    }

    // 添加心跳机制
    async fn start_heartbeat(&self, tx: mpsc::Sender<TunnelMessage>) {
        let client_id = self.client_id.clone();
        let trace_enabled = self.trace_enabled;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let heartbeat = TunnelMessage {
                    client_id: client_id.clone(),
                    request_id: Uuid::new_v4().to_string(),
                    direction: Direction::ClientToServer as i32,
                    payload: Some(tunnel_message::Payload::Heartbeat(Heartbeat {
                        timestamp: chrono::Utc::now().timestamp(),
                    })),
                    trace_id: String::new(),
                };
                if tx.send(heartbeat).await.is_err() {
                    error!("Failed to send heartbeat");
                    break;
                }
                if trace_enabled {
                    debug!("Sent heartbeat to server");
                }
            }
        });
    }

    // 通过tunnel连接进行配置同步
    async fn start_config_sync_via_tunnel(&self, tx: mpsc::Sender<TunnelMessage>) {
        let client_id = self.client_id.clone();
        let group_id = self.group_id.clone();
        let trace_enabled = self.trace_enabled;
        tokio::spawn(async move {
            // 立即发送一次配置同步请求
            let _config_sync_msg = TunnelMessage {
                client_id: client_id.clone(),
                request_id: Uuid::new_v4().to_string(),
                direction: Direction::ClientToServer as i32,
                payload: Some(tunnel_message::Payload::ConfigSync(ConfigSyncRequest {
                    client_id: client_id.clone(),
                    group: group_id.clone(),
                    config_version: "".to_string(),
                })),
                trace_id: String::new(),
            };
            // 定期配置同步
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let config_sync_msg = TunnelMessage {
                    client_id: client_id.clone(),
                    request_id: Uuid::new_v4().to_string(),
                    direction: Direction::ClientToServer as i32,
                    payload: Some(tunnel_message::Payload::ConfigSync(ConfigSyncRequest {
                        client_id: client_id.clone(),
                        group: group_id.clone(),
                        config_version: "".to_string(),
                    })),
                    trace_id: String::new(),
                };
                if let Err(e) = tx.send(config_sync_msg).await {
                    error!("Failed to send config sync: {}", e);
                    break;
                }
                if trace_enabled {
                    debug!("Sent config sync request via tunnel");
                }
            }
        });
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
    // 加载 client 配置
    info!("Loaded client config: {:?}", config);

    // 使用配置中的 server_addr/server_port/client_group_id
    let server_addr = format!("http://{}:{}", config.server_addr, config.server_port);
    let client_group_id = config.client_group_id.clone();
    let trace_enabled = config.trace_enabled.unwrap_or(false);

    let client_id = format!("client-{}", Uuid::new_v4());
    let group_id = client_group_id.clone();
    info!("Starting tunnel client {} in group {}", client_id, group_id);

    let (client, _rx) = TunnelClient::new(client_id.clone(), group_id, server_addr.clone(), trace_enabled);

    // 启动本地 HTTP 入口监听
    let http_port = config.http_entry_port.unwrap_or(8003);
    let http_addr = format!("0.0.0.0:{}", http_port);
    let tunnel_tx = client.tx.clone();
    let pending_requests = client.pending_requests.clone();
    let client_id = client_id.clone();
    tokio::spawn(async move {
        let client_id = client_id.clone();
        let make_svc = make_service_fn(move |_| {
            let tunnel_tx = tunnel_tx.clone();
            let pending_requests = pending_requests.clone();
            let client_id = client_id.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let tunnel_tx = tunnel_tx.clone();
                    let pending_requests = pending_requests.clone();
                    let client_id = client_id.clone();
                    async move {
                        // 1. 生成 request_id
                        let request_id = Uuid::new_v4().to_string();
                        // 2. 记录 access log
                        let method = req.method().to_string();
                        let path = req.uri().path().to_string();
                        let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("").to_string();
                        tracing::info!(
                            event = "client_http_entry",
                            method = %method,
                            path = %path,
                            host = %host,
                            request_id = %request_id,
                            message = "Received HTTP request at client entry"
                        );
                        // 3. 调用 forward_via_tunnel 统一处理
                        tunnel_lib::proxy::forward_via_tunnel(
                            req,
                            &client_id,
                            &tunnel_tx,
                            &pending_requests,
                            request_id.clone(),
                            tunnel_lib::tunnel::Direction::ClientToServer,
                        ).await
                    }
                }))
            }
        });
        let server = HyperServer::bind(&http_addr.parse().unwrap()).serve(make_svc);
        info!("Client HTTP entry listening on http://{}", http_addr);
        if let Err(e) = server.await {
            error!("Client HTTP entry server error: {}", e);
        }
    });

    // 启动本地 gRPC 入口监听
    let grpc_port = config.grpc_entry_port.unwrap_or(8004);
    let grpc_addr = format!("0.0.0.0:{}", grpc_port);
    tokio::spawn(async move {
        let make_svc = make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(|_req| async {
                Ok::<_, hyper::Error>(HyperResponse::new(Body::from("Client gRPC entry (stub)")))
            }))
        });
        let server = HyperServer::bind(&grpc_addr.parse().unwrap()).serve(make_svc);
        info!("Client gRPC entry listening on http://{} (stub, not real gRPC)", grpc_addr);
        if let Err(e) = server.await {
            error!("Client gRPC entry server error: {}", e);
        }
    });

    // 启动 tunnel 连接
    client.start().await?;
    Ok(())
} 