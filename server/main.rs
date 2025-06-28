mod config;
mod proxy;
mod registry;
mod rules;

use crate::config::ServerConfig;
use crate::rules::RulesEngine;
use crate::registry::ManagedClientRegistry;


use tunnel_lib::tunnel::tunnel_service_server::{TunnelService, TunnelServiceServer};
use tunnel_lib::tunnel::*;
use tunnel_lib::tunnel::Rule as ProtoRule;

use anyhow::Result;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Body, Request as HyperRequest, Response as HyperResponse, Server as HyperServer, Method, Client as HyperClient};
use hyper::service::{make_service_fn, service_fn};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
use std::net::SocketAddr;
use crate::proxy::{ProxyHandler};
use tokio::sync::Mutex as TokioMutex;
use tracing::{info, error, debug, warn};
use tracing_subscriber;
use tunnel_lib::http_forward::forward_http_to_backend;

#[derive(Clone)]
pub struct MyTunnelServer {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pending_reverse_requests: Arc<tokio::sync::Mutex<HashMap<String, oneshot::Sender<HttpResponse>>>>,
    connected_clients: Arc<tokio::sync::Mutex<HashMap<String, mpsc::Sender<TunnelMessage>>>>,
    http_client: Arc<HyperClient<hyper::client::HttpConnector>>,
}

impl MyTunnelServer {
    fn new_with_config(config: &crate::config::ServerConfig, http_client: Arc<HyperClient<hyper::client::HttpConnector>>) -> Self {
        let rules_engine = RulesEngine::new(config.clone());
        Self {
            client_registry: Arc::new(ManagedClientRegistry::new()),
            pending_reverse_requests: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            connected_clients: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            rules_engine: Arc::new(rules_engine),
            http_client,
        }
    }
}

impl Default for MyTunnelServer {
    fn default() -> Self {
        let http_client = Arc::new(HyperClient::new());
        Self::new_with_config(&ServerConfig::load("../config/server.toml").unwrap(), http_client)
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
        // clone 需要的 Arc 字段，避免 &self 被 move 进 tokio::spawn
        let pending_reverse_requests = self.pending_reverse_requests.clone();
        let connected_clients = self.connected_clients.clone();
        let rules_engine = self.rules_engine.clone();
        let client_registry = self.client_registry.clone();
        let http_client = self.http_client.clone();
        tokio::spawn(async move {
            let mut client_id = String::new();
            let mut client_tx: Option<mpsc::Sender<TunnelMessage>> = None;
            while let Some(message) = stream.next().await {
                debug!(target: "server::proxy_stream", ?message, "Received tunnel message from client");
                match message {
                    Ok(msg) => {
                        client_id = msg.client_id.clone();
                        // 存储客户端连接（只在第一次收到消息时）
                        if client_tx.is_none() {
                            let (ctx, crx) = mpsc::channel(128);
                            client_tx = Some(ctx.clone());
                            connected_clients.lock().await.insert(client_id.clone(), ctx);
                            // 启动客户端消息分发器
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
                        // 处理消息
                        match msg.payload {
                            Some(tunnel_message::Payload::HttpResponse(resp)) => {
                                if msg.direction == Direction::ClientToServer as i32 {
                                    if let Some(sender) = pending_reverse_requests.lock().await.remove(&msg.request_id) {
                                        let _ = sender.send(resp);
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
                                let mut response = HttpResponse {
                                    status_code: 404,
                                    headers: HashMap::new(),
                                    body: b"Not Found".to_vec(),
                                };
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
                    Err(_) => break,
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
        Ok(HyperResponse::builder()
            .status(404)
            .body(Body::from("No route found"))
            .unwrap())
    }
}

fn pick_backend(upstream: &crate::config::Upstream) -> Option<String> {
    if !upstream.servers.is_empty() {
        Some(upstream.servers[0].address.clone())
    } else {
        None
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
    let tunnel_server = MyTunnelServer::new_with_config(&config, http_client.clone());
    let rules_engine = tunnel_server.rules_engine.clone();
    let client_registry = tunnel_server.client_registry.clone();
    let pending_reverse_requests = tunnel_server.pending_reverse_requests.clone();
    let connected_clients = tunnel_server.connected_clients.clone();
    let http_client = tunnel_server.http_client.clone();

    let proxy_handler = Arc::new(ProxyHandler::with_dependencies(
        client_registry.clone(),
        pending_reverse_requests.clone(),
        connected_clients.clone(),
        config.server.trace_enabled.unwrap_or(true),
    ));

    // HTTP 入口监听
    let http_port = config.server.http_entry_port;
    let http_addr: SocketAddr = format!("0.0.0.0:{}", http_port).parse()?;
    let rules_engine_http = rules_engine.clone();
    let proxy_handler_http = proxy_handler.clone();
    tokio::spawn({
        let rules_engine = rules_engine_http.clone();
        let proxy_handler = proxy_handler_http.clone();
        async move {
            let make_svc = make_service_fn(move |_| {
                let rules_engine = rules_engine.clone();
                let proxy_handler = proxy_handler.clone();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        let rules_engine = rules_engine.clone();
                        let proxy_handler = proxy_handler.clone();
                        async move {
                            debug!("HTTP request received: {} {} (Host: {})", 
                                   req.method(), req.uri().path(), 
                                   req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or(""));
                            let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
                            let path = req.uri().path();
                            // 1. reverse_proxy 优先
                            if let Some(rule) = rules_engine.match_reverse_proxy_rule(host, path, None) {
                                debug!("Matched reverse_proxy rule: {:?}", rule);
                                if let Some(group) = &rule.action_client_group {
                                    debug!("Forwarding to client group: {}", group);
                                    return proxy_handler.forward_via_tunnel(req, group).await;
                                }
                            }
                            // 2. forward 规则
                            if let Some(rule) = rules_engine.match_forward_rule(host, path, None) {
                                if let Some(upstream_name) = &rule.action_upstream {
                                    return proxy_handler.handle_proxy_pass(req, upstream_name).await;
                                }
                            }
                            Ok::<_, hyper::Error>(HyperResponse::new(Body::from("No matching rule")))
                        }
                    }))
                }
            });
            let server = HyperServer::bind(&http_addr).serve(make_svc);
            info!("HTTP entry listening on http://{}", http_addr);
            if let Err(e) = server.await {
                error!("HTTP entry server error: {}", e);
            }
        }
    });

    // gRPC 入口监听
    let grpc_port = config.server.grpc_entry_port;
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;
    let rules_engine_grpc = rules_engine.clone();
    tokio::spawn({
        let rules_engine = rules_engine_grpc.clone();
        async move {
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
        }
    });

    // tunnel 控制端口监听
    let tunnel_port = config.server.tunnel_port;
    let tunnel_addr: SocketAddr = format!("0.0.0.0:{}", tunnel_port).parse()?;
    let svc = TunnelServiceServer::new(tunnel_server);
    info!("gRPC tunnel server listening on {} (tunnel control)", tunnel_addr);
    Server::builder()
        .add_service(svc)
        .serve(tunnel_addr)
        .await?;

    Ok(())
}

// 典型请求分发流程：
// 1. 收到 HTTP/gRPC 请求（8001/8002）
// 2. 调用 rules_engine 匹配规则
// 3. 命中 proxy_pass/upstream，则反代到目标；否则通过 tunnel 转发到 client group
// 4. 返回响应 