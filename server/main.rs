mod config;
mod proxy;
mod registry;
mod rules;
mod utils;
mod tunnel_server;

use crate::config::ServerConfig;
use crate::rules::RulesEngine;
use crate::registry::ManagedClientRegistry;
use crate::tunnel_server::TunnelServer;


use tunnel_lib::tunnel::tunnel_service_server::{TunnelService, TunnelServiceServer};
use tunnel_lib::tunnel::*;
use tunnel_lib::tunnel::Rule as ProtoRule;

use anyhow::Result;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Body, Request as HyperRequest, Response as HyperResponse, Server as HyperServer, Client as HyperClient};
use hyper::service::{make_service_fn, service_fn};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tonic::{transport::Server, Request, Response, Status};
use std::net::SocketAddr;
use tokio::sync::Mutex as TokioMutex;
use tracing::{info, error, debug, warn};
use tracing_subscriber;
use tunnel_lib::http_forward::forward_http_to_backend;
use tunnel_lib::response::{self, error_response, ProxyErrorKind};
use tunnel_lib::proxy::{HttpEntryProxyTarget, HttpTunnelContext, http_entry_handler};
use dashmap::DashMap;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use crate::utils::pick_backend;
use tokio::task::JoinSet;

#[derive(Clone)]
pub struct MyTunnelServer {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    connected_clients: Arc<tokio::sync::Mutex<HashMap<String, mpsc::Sender<TunnelMessage>>>>,
    token_map: Arc<DashMap<String, CancellationToken>>,
    http_client: Arc<HyperClient<hyper::client::HttpConnector>>,
}

impl MyTunnelServer {
    fn new_with_config(config: &crate::config::ServerConfig, http_client: Arc<HyperClient<hyper::client::HttpConnector>>, pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>, token_map: Arc<DashMap<String, CancellationToken>>) -> Self {
        let rules_engine = RulesEngine::new(config.clone());
        Self {
            client_registry: Arc::new(ManagedClientRegistry::new()),
            pending_requests,
            connected_clients: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            token_map,
            rules_engine: Arc::new(rules_engine),
            http_client,
        }
    }
}

impl Default for MyTunnelServer {
    fn default() -> Self {
        let http_client = Arc::new(HyperClient::new());
        let pending_requests = Arc::new(DashMap::new());
        let token_map = Arc::new(DashMap::new());
        Self::new_with_config(&ServerConfig::load("../config/server.toml").unwrap(), http_client, pending_requests, token_map)
    }
}

#[tonic::async_trait]
impl TunnelService for MyTunnelServer {
    type ControlStreamStream = tokio_stream::wrappers::ReceiverStream<Result<TunnelMessage, Status>>;
    type ProxyStream = tokio_stream::wrappers::ReceiverStream<Result<TunnelMessage, Status>>;

    async fn control_stream(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ControlStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(32);
        let client_registry = self.client_registry.clone();
        let rules_engine = self.rules_engine.clone();
        // 这里只处理注册、心跳、配置同步等控制消息
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(tunnel_msg) => {
                        match tunnel_msg.payload {
                            Some(tunnel_lib::tunnel::tunnel_message::Payload::ConnectRequest(connect_req)) => {
                                client_registry.register_client(&connect_req.client_id, "default-group");
                                // 可选：返回注册结果
                            }
                            Some(tunnel_lib::tunnel::tunnel_message::Payload::Heartbeat(_)) => {
                                client_registry.update_heartbeat(&tunnel_msg.client_id);
                            }
                            Some(tunnel_lib::tunnel::tunnel_message::Payload::ConfigSync(config_req)) => {
                                // 配置同步逻辑
                                let (rules, upstreams) = if let Some(group) = rules_engine.get_group(&config_req.group) {
                                    (group.rules.http.clone(), group.upstreams.clone())
                                } else {
                                    (Vec::new(), std::collections::HashMap::new())
                                };
                                // ...组装 ConfigSyncResponse 并通过 tx 发送回 client
                            }
                            _ => {}
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn proxy(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ProxyStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);
        let pending_requests = self.pending_requests.clone();
        let connected_clients = self.connected_clients.clone();
        let rules_engine = self.rules_engine.clone();
        let client_registry = self.client_registry.clone();
        let http_client = self.http_client.clone();
        let token_map = self.token_map.clone();
        tokio::spawn(async move {
            let mut client_id = String::new();
            let mut client_tx: Option<mpsc::Sender<TunnelMessage>> = None;
            let token = CancellationToken::new();
            while let Some(message) = stream.next().await {
                debug!(target: "server::proxy_stream", ?message, "Received tunnel message from client");
                match message {
                    Ok(msg) => {
                        client_id = msg.client_id.clone();
                        if client_tx.is_none() {
                            let (ctx, crx) = mpsc::channel(128);
                            client_tx = Some(ctx.clone());
                            token_map.insert(client_id.clone(), token.clone());
                            connected_clients.lock().await.insert(client_id.clone(), ctx);
                            let tx_clone = tx.clone();
                            tokio::spawn(async move {
                                let mut receiver = crx;
                                while let Some(tunnel_msg) = receiver.recv().await {
                                    if let Err(_) = tx_clone.send(Ok(tunnel_msg)).await {
                                        break;
                                    }
                                }
                            });
                        }
                        match msg.payload {
                            Some(tunnel_message::Payload::HttpResponse(resp)) => {
                                if msg.direction == Direction::ClientToServer as i32 {
                                    if let Some((_, sender)) = pending_requests.remove(&msg.request_id) {
                                        let _ = sender.send(resp);
                                    } else {
                                        warn!("No pending sender for request_id={}", msg.request_id);
                                    }
                                } else {
                                    warn!("Ignore HttpResponse with wrong direction: {}", msg.direction);
                                }
                            }
                            Some(tunnel_message::Payload::ConfigSync(config_req)) => {
                                if msg.direction == Direction::ClientToServer as i32 {
                                    if let Some(group) = rules_engine.get_group(&config_req.group) {
                                        // 获取该 group 的配置
                                        let (rules, upstreams) = (group.rules.http.clone(), group.upstreams.clone());
                                        // 转换为 proto 格式
                                        let proto_rules: Vec<ProtoRule> = rules.into_iter().map(|r| ProtoRule {
                                            rule_id: String::new(),
                                            r#type: String::new(),
                                            match_host: r.match_host.unwrap_or_default(),
                                            match_path_prefix: r.match_path_prefix.unwrap_or_default(),
                                            match_service: r.match_service.unwrap_or_default(),
                                            match_header: std::collections::HashMap::new(),
                                            action_proxy_pass: String::new(),
                                            action_set_host: r.action_set_host.unwrap_or_default(),
                                            action_upstream: r.action_upstream.unwrap_or_default(),
                                            action_ssl: false,
                                        }).collect();
                                        let proto_upstreams: Vec<Upstream> = upstreams.into_iter().map(|(name, u)| Upstream {
                                            name,
                                            servers: u.servers.into_iter().map(|s| UpstreamServer {
                                                address: s.address,
                                                resolve: s.resolve,
                                            }).collect(),
                                            lb_policy: u.lb_policy.unwrap_or_else(|| "round_robin".to_string()),
                                        }).collect();
                                        let config_response = ConfigSyncResponse {
                                            config_version: "v1.0.0".to_string(),
                                            rules: proto_rules,
                                            upstreams: proto_upstreams,
                                        };
                                        // 发送配置同步响应
                                        let response_msg = TunnelMessage {
                                            client_id: msg.client_id.clone(),
                                            request_id: msg.request_id,
                                            direction: Direction::ServerToClient as i32,
                                            payload: Some(tunnel_message::Payload::ConfigSyncResponse(config_response)),
                                            trace_id: String::new(),
                                        };
                                        if let Some(tx) = client_tx.as_ref() {
                                            if let Err(e) = tx.send(response_msg).await {
                                                error!("Failed to send config sync response: {}", e);
                                            }
                                        }
                                    }
                                } else {
                                    warn!("Ignore ConfigSync with wrong direction: {}", msg.direction);
                                }
                            }
                            Some(tunnel_message::Payload::Heartbeat(_)) => {
                                // 心跳包一般双向都可接受，这里可不判断方向
                                client_registry.update_heartbeat(&client_id);
                            }
                            Some(tunnel_message::Payload::HttpRequest(ref req)) => {
                                info!(
                                    event = "server_received_tunnel_http_request",
                                    request_id = %msg.request_id,
                                    client_id = %msg.client_id,
                                    method = %req.method,
                                    url = %req.url,
                                    host = %req.host,
                                    message = "Server received HTTP request from tunnel"
                                );
                                // 查找 forward 规则
                                let host = req.host.as_str();
                                let path = req.url.split('?').next().unwrap_or("/");
                                let mut response = error_response(
                                    ProxyErrorKind::NoMatchRules,
                                    None,
                                    Some(&msg.trace_id),
                                    Some(&msg.request_id),
                                    Some(&msg.client_id),
                                );
                                if let Some(rule) = rules_engine.match_forward_rule(host, path, None) {
                                    if let Some(ref upstream_name) = rule.action_upstream {
                                        if let Some(upstream) = rules_engine.get_upstream(upstream_name) {
                                            if let Some(backend) = pick_backend(upstream) {
                                                let set_host = rule.action_set_host.as_deref().unwrap_or("");
                                                response = forward_http_to_backend(
                                                    req,
                                                    &backend,
                                                    http_client.clone(),
                                                    set_host
                                                ).await;
                                            }
                                        }
                                    }
                                }
                                let response_msg = TunnelMessage {
                                    client_id: msg.client_id.clone(),
                                    request_id: msg.request_id.clone(),
                                    direction: Direction::ServerToClient as i32,
                                    payload: Some(tunnel_message::Payload::HttpResponse(response)),
                                    trace_id: msg.trace_id.clone(),
                                };
                                if let Some(tx) = client_tx.as_ref() {
                                    if let Err(e) = tx.send(response_msg).await {
                                        error!("Failed to send tunnel HTTP response: {}", e);
                                    }
                                }
                            }
                            Some(payload) => {
                                tracing::warn!("Received unknown payload type: {:?}, direction: {}", payload, msg.direction);
                            }
                            None => {}
                        }
                    }
                    Err(_) => {
                        if let Some(token) = token_map.get(client_id.as_str()) {
                            token.cancel();
                        }
                        connected_clients.lock().await.remove(&client_id);
                        token_map.remove(&client_id);
                        break;
                    }
                }
            }
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        info!("Client {} registering to group {}", req.client_id, req.group);
        
        // 注册客户端到指定组
        self.client_registry.register_client(&req.client_id, &req.group);
        
        let response = RegisterResponse {
            success: true,
            message: "Registration successful".to_string(),
            config_version: "v1.0.0".to_string(),
        };
        
        Ok(Response::new(response))
    }

    async fn config_sync(
        &self,
        request: Request<ConfigSyncRequest>,
    ) -> Result<Response<ConfigSyncResponse>, Status> {
        let req = request.into_inner();
        info!("Client {} syncing config for group {}", req.client_id, req.group);
        
        // 获取该 group 的配置
        let (rules, upstreams) = if let Some(group) = self.rules_engine.get_group(&req.group) {
            (group.rules.http.clone(), group.upstreams.clone())
        } else {
            (Vec::new(), std::collections::HashMap::new())
        };
        
        // 转换为 proto 格式
        let proto_rules: Vec<ProtoRule> = rules.into_iter().map(|r| ProtoRule {
            rule_id: String::new(),
            r#type: String::new(),
            match_host: r.match_host.unwrap_or_default(),
            match_path_prefix: r.match_path_prefix.unwrap_or_default(),
            match_service: r.match_service.unwrap_or_default(),
            match_header: std::collections::HashMap::new(),
            action_proxy_pass: String::new(),
            action_set_host: String::new(),
            action_upstream: r.action_upstream.unwrap_or_default(),
            action_ssl: false,
        }).collect();
        
        let proto_upstreams: Vec<Upstream> = upstreams.into_iter().map(|(name, u)| Upstream {
            name,
            servers: u.servers.into_iter().map(|s| UpstreamServer {
                address: s.address,
                resolve: s.resolve,
            }).collect(),
            lb_policy: u.lb_policy.unwrap_or_else(|| "round_robin".to_string()),
        }).collect();
        
        let response = ConfigSyncResponse {
            config_version: "v1.0.0".to_string(),
            rules: proto_rules,
            upstreams: proto_upstreams,
        };
        
        Ok(Response::new(response))
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let req = request.into_inner();
        info!("Heartbeat from client {} in group {}", req.client_id, req.group);
        
        let response = HeartbeatResponse {
            success: true,
            message: "Heartbeat received".to_string(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
        };
        
        Ok(Response::new(response))
    }
}

async fn reverse_proxy_handler(
    req: HyperRequest<Body>,
    client_registry: Arc<ManagedClientRegistry>,
    _pending: Arc<tokio::sync::Mutex<HashMap<String, oneshot::Sender<HttpResponse>>>>,
) -> Result<HyperResponse<Body>, hyper::Error> {
    // 先提取所有需要的信息，避免借用冲突
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers: HashMap<String, String> = req.headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    
    let host = req.headers().get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost")
        .to_string();
        
    let path = uri.path().to_string();
    info!("Reverse proxy request: {} {} (Host: {})", method, path, host);
    
    // 读取 body
    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(bytes) => bytes.to_vec(),
        Err(_) => Vec::new(),
    };
    
    // 根据 host/path 查找对应的 client group
    if let Some(group) = client_registry.find_group_for_request(&host, &path) {
        info!("Found group {} for request", group);
        
        let http_req = HttpRequest {
            method: method.to_string(),
            url: uri.to_string(),
            host: host.clone(),
            path: path.clone(),
            query: uri.query().unwrap_or("").to_string(),
            headers,
            body,
            original_dst: format!("{}:{}", host, 80), // 简化处理
        };
        
        // 创建响应通道
        let (_tx, rx) = oneshot::channel::<HttpResponse>();
        
        // 发送到 client group（这里需要实现具体的消息路由逻辑）
        match client_registry.send_to_group(&group, http_req).await {
            Ok(_) => {
                // 等待响应
                match timeout(Duration::from_secs(30), rx).await {
                    Ok(Ok(response)) => {
                        let mut builder = HyperResponse::builder().status(response.status_code as u16);
                        for (k, v) in response.headers.iter() {
                            builder = builder.header(k, v);
                        }
                        Ok(builder.body(Body::from(response.body)).unwrap())
                    }
                    Ok(Err(_)) => Ok(HyperResponse::builder()
                        .status(500)
                        .body(Body::from("Internal server error"))
                        .unwrap()),
                    Err(_) => Ok(HyperResponse::builder()
                        .status(504)
                        .body(Body::from("Gateway timeout"))
                        .unwrap()),
                }
            }
            Err(_) => Ok(HyperResponse::builder()
                .status(502)
                .body(Body::from("Bad gateway"))
                .unwrap()),
        }
    } else {
        let err_resp = response::resp_404(None, None, Some("server"));
        Ok::<_, hyper::Error>(HyperResponse::builder()
            .status(err_resp.status_code as u16)
            .header("content-type", "application/json")
            .body(Body::from(err_resp.body))
            .unwrap())
    }
}

/// 服务器端 HTTP 入口 ProxyTarget 实现
pub struct ServerHttpEntryTarget {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pub connected_clients: Arc<TokioMutex<HashMap<String, mpsc::Sender<TunnelMessage>>>>,
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    pub token_map: Arc<DashMap<String, CancellationToken>>,
}

#[async_trait]
impl HttpEntryProxyTarget for ServerHttpEntryTarget {
    async fn handle(
        &self,
        req: HyperRequest<Body>,
        ctx: &HttpTunnelContext,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
        let path = req.uri().path();
        if let Some(rule) = self.rules_engine.match_reverse_proxy_rule(host, path, None) {
            if let Some(group) = &rule.action_client_group {
                let healthy_clients = self.client_registry.get_clients_in_group(group);
                if healthy_clients.is_empty() {
                    let err_resp = response::resp_502(None, None, Some("server"));
                    return Ok(HyperResponse::builder()
                        .status(err_resp.status_code as u16)
                        .header("content-type", "application/json")
                        .body(Body::from(err_resp.body))
                        .unwrap());
                }
                let mut found = None;
                let clients = self.connected_clients.lock().await;
                for client_id in &healthy_clients {
                    if let Some(token) = self.token_map.get(client_id.as_str()) {
                        if !token.is_cancelled() {
                            if let Some(sender) = clients.get(client_id.as_str()) {
                                if !sender.is_closed() {
                                    found = Some((client_id, sender));
                                    break;
                                }
                            }
                        }
                    }
                }
                if let Some((client_id, sender)) = found {
                    // access log
                    let trace_id = req.headers()
                        .get("x-trace-id")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                    tracing::info!(
                        event = "access",
                        trace_id = %trace_id,
                        host = %host,
                        path = %path,
                        group = %group,
                        selected_client = %client_id,
                        healthy_clients = ?healthy_clients,
                        message = "access log"
                    );
                    use tunnel_lib::http_forward::forward_http_via_tunnel;
                    use uuid::Uuid;
                    let request_id = Uuid::new_v4().to_string();
                    return forward_http_via_tunnel(
                        req,
                        &client_id,
                        sender,
                        self.pending_requests.clone(),
                        request_id,
                        tunnel_lib::tunnel::Direction::ServerToClient,
                    ).await;
                } else {
                    let err_resp = response::resp_502(None, None, Some("server"));
                    return Ok(HyperResponse::builder()
                        .status(err_resp.status_code as u16)
                        .header("content-type", "application/json")
                        .body(Body::from(err_resp.body))
                        .unwrap());
                }
            }
        }
        let err_resp = response::resp_404(None, None, Some("server"));
        Ok(HyperResponse::builder()
            .status(err_resp.status_code as u16)
            .header("content-type", "application/json")
            .body(Body::from(err_resp.body))
            .unwrap())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::load("../config/server.toml")?;
    config.validate_rules()?;
    let log_level = config.server.log_level.as_str();
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
    info!("Starting tunnel server...");
    
    // 加载配置
    let config = ServerConfig::load("../config/server.toml")?;
    let rules_engine = Arc::new(RulesEngine::new(config.clone()));
    let http_rule_count = config.forward.rules.http.len();
    let grpc_rule_count = config.reverse_proxy.rules.http.len();
    info!("Loaded config with {} HTTP rules, {} gRPC rules", http_rule_count, grpc_rule_count);
    
    let http_client = Arc::new(HyperClient::new());
    let pending_requests = Arc::new(DashMap::new());
    let token_map = Arc::new(DashMap::new());
    let tunnel_server = MyTunnelServer::new_with_config(&config, http_client.clone(), pending_requests.clone(), token_map.clone());
    let rules_engine = tunnel_server.rules_engine.clone();
    let client_registry = tunnel_server.client_registry.clone();
    let connected_clients = tunnel_server.connected_clients.clone();
    let http_client = tunnel_server.http_client.clone();

    // HTTP 入口监听
    let http_port = config.server.http_entry_port;
    let http_addr: SocketAddr = format!("0.0.0.0:{}", http_port).parse()?;
    let target = Arc::new(ServerHttpEntryTarget {
        rules_engine: rules_engine.clone(),
        client_registry: client_registry.clone(),
        connected_clients: connected_clients.clone(),
        pending_requests: pending_requests.clone(),
        token_map: token_map.clone(),
    });
    let (dummy_tx, _dummy_rx) = mpsc::channel(1);
    let pending_requests = Arc::new(DashMap::new());
    let ctx = HttpTunnelContext {
        client_id: "server".to_string(),
        tunnel_tx: Arc::new(dummy_tx),
        pending_requests: pending_requests.clone(),
        direction: tunnel_lib::tunnel::Direction::ServerToClient,
    };
    let mut join_set = JoinSet::new();

    // HTTP 入口监听
    join_set.spawn(async move {
        let target = target.clone();
        let ctx = ctx.clone();
        let make_svc = make_service_fn(move |_| {
            let ctx = ctx.clone();
            let target = target.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let ctx = ctx.clone();
                    let target = target.clone();
                    async move {
                        http_entry_handler::<ServerHttpEntryTarget>(req, &ctx, &*target).await
                    }
                }))
            }
        });
        let server = HyperServer::bind(&http_addr).serve(make_svc);
        info!("HTTP entry listening on http://{} (tunnel-lib handler)", http_addr);
        if let Err(e) = server.await {
            error!("HTTP entry server error: {}", e);
        }
    });

    // gRPC 入口监听
    let grpc_port = config.server.grpc_entry_port;
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;
    let rules_engine_grpc = rules_engine.clone();
    join_set.spawn(async move {
        let rules_engine = rules_engine_grpc.clone();
        let make_svc = make_service_fn(move |_| {
            let rules_engine = rules_engine.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let rules_engine = rules_engine.clone();
                    async move {
                        let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
                        let service = None;
                        if let Some(rule) = rules_engine.match_reverse_proxy_rule(host, "", service) {
                            return Ok::<_, hyper::Error>(HyperResponse::new(Body::from(format!("Reverse proxy to client group: {:?}", rule.action_client_group))));
                        }
                        if let Some(rule) = rules_engine.match_forward_rule(host, "", service) {
                            return Ok::<_, hyper::Error>(HyperResponse::new(Body::from(format!("Forward proxy to upstream: {:?}", rule.action_upstream))));
                        }
                        Ok::<_, hyper::Error>(HyperResponse::new(Body::from("No matching rule")))
                    }
                }))
            }
        });
        let server = HyperServer::bind(&grpc_addr).serve(make_svc);
        info!("gRPC entry listening on http://{} (stub, not real gRPC)", grpc_addr);
        if let Err(e) = server.await {
            error!("gRPC entry server error: {}", e);
        }
    });

    // tunnel 控制端口监听
    let tunnel_port = config.server.tunnel_port;
    let tunnel_addr: SocketAddr = format!("0.0.0.0:{}", tunnel_port).parse()?;
    let svc = TunnelServiceServer::new(tunnel_server);
    info!("gRPC tunnel server listening on {} (tunnel control)", tunnel_addr);
    join_set.spawn(async move {
        if let Err(e) = Server::builder().add_service(svc).serve(tunnel_addr).await {
            error!("Tunnel server error: {}", e);
        }
    });

    // 等待所有任务完成
    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            error!("A background task failed: {:?}", e);
        }
    }
    Ok(())
}

// 典型请求分发流程：
// 1. 收到 HTTP/gRPC 请求（8001/8002）
// 2. 调用 rules_engine 匹配规则
// 3. 命中 proxy_pass/upstream，则反代到目标；否则通过 tunnel 转发到 client group
// 4. 返回响应 