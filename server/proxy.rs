use crate::rules::RulesEngine;
use std::sync::Arc;
use hyper::{Body, Request as HyperRequest, Response as HyperResponse, Server as HyperServer};
use hyper::service::{make_service_fn, service_fn};
use std::collections::HashMap;

use tunnel_lib::tunnel::HttpRequest;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::registry::ManagedClientRegistry;
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse, Direction};
use tokio::sync::{oneshot, mpsc};
use uuid::Uuid;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, info_span, Instrument};
use tunnel_lib::http_forward::{build_http_tunnel_message, forward_http_to_backend, set_host_header};
use tunnel_lib::response::{error_response, ProxyErrorKind};

// Global round-robin index for each upstream
lazy_static! {
    static ref UPSTREAM_INDEX: Mutex<std::collections::HashMap<String, usize>> = Mutex::new(std::collections::HashMap::new());
}

fn pick_backend(upstream: &crate::config::Upstream) -> Option<String> {
    let len = upstream.servers.len();
    if len == 0 {
        return None;
    }
    let mut idx_map = UPSTREAM_INDEX.lock().unwrap();
    let idx = idx_map.entry(format!("upstream-{}", upstream.servers[0].address)).or_insert(0);
    let server = &upstream.servers[*idx % len];
    *idx = (*idx + 1) % len;
    Some(server.address.clone())
}

pub async fn start_http_entry(rules_engine: Arc<RulesEngine>, proxy_handler: Arc<ProxyHandler>) {
    let make_svc = make_service_fn(move |_| {
        let rules_engine = rules_engine.clone();
        let proxy_handler = proxy_handler.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: HyperRequest<Body>| {
                handle_http_request(req, rules_engine.clone(), proxy_handler.clone())
            }))
        }
    });
    let addr = ([0, 0, 0, 0], 8001).into();
    let server = HyperServer::bind(&addr).serve(make_svc);
    println!("HTTP entry listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("HTTP entry server error: {}", e);
    }
}

pub struct ProxyHandler {
    client_registry: Arc<ManagedClientRegistry>,
    pending_reverse_requests: Arc<tokio::sync::Mutex<HashMap<String, oneshot::Sender<HttpResponse>>>>,
    connected_clients: Arc<tokio::sync::Mutex<HashMap<String, mpsc::Sender<TunnelMessage>>>>,
    pub trace_enabled: bool,
}

impl ProxyHandler {
    pub fn new(trace_enabled: bool) -> Self {
        Self {
            client_registry: Arc::new(ManagedClientRegistry::new()),
            pending_reverse_requests: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            connected_clients: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            trace_enabled,
        }
    }

    pub fn with_dependencies(
        client_registry: Arc<ManagedClientRegistry>,
        pending_reverse_requests: Arc<tokio::sync::Mutex<HashMap<String, oneshot::Sender<HttpResponse>>>>,
        connected_clients: Arc<tokio::sync::Mutex<HashMap<String, mpsc::Sender<TunnelMessage>>>>,
        trace_enabled: bool,
    ) -> Self {
        Self {
            client_registry,
            pending_reverse_requests,
            connected_clients,
            trace_enabled,
        }
    }

    // 直接代理到后端服务器
    pub async fn handle_proxy_pass(
        &self,
        req: HyperRequest<Body>,
        backend_url: &str,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        use hyper::Client;
        
        // 创建客户端
        let client = Client::new();
        
        // 构造目标 URL
        let target_url = if backend_url.starts_with("http") {
            format!("{}{}", backend_url, req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(""))
        } else {
            format!("http://{}{}", backend_url, req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(""))
        };
        
        // 分解请求
        let (parts, body) = req.into_parts();
        let body_bytes = hyper::body::to_bytes(body).await.unwrap_or_default();
        
        let mut proxy_req = hyper::Request::builder()
            .method(parts.method)
            .uri(&target_url);
        
        // 复制头部（除了 host）
        for (key, value) in parts.headers.iter() {
            if key != "host" {
                proxy_req = proxy_req.header(key, value);
            }
        }
        
        let proxy_req = proxy_req.body(Body::from(body_bytes)).unwrap();
        
        println!("Proxying request to: {}", target_url);
        
        // 发送请求
        match client.request(proxy_req).await {
            Ok(response) => {
                println!("Proxy response status: {}", response.status());
                Ok(response)
            }
            Err(e) => {
                tracing::error!("Proxy request failed: {}", e);
                let err_resp = tunnel_lib::response::resp_502(
                    None, // trace_id
                    None, // request_id
                    Some("server"),
                );
                Ok(HyperResponse::builder()
                    .status(err_resp.status_code as u16)
                    .header("content-type", "application/json")
                    .body(Body::from(err_resp.body))
                    .unwrap())
            }
        }
    }

    // 实际的 tunnel 转发实现
    pub async fn forward_via_tunnel(
        &self,
        req: HyperRequest<Body>,
        target_group: &str,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("localhost");
        let path = req.uri().path();
        let trace_id = req.headers()
            .get("x-trace-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let (target_client, healthy_clients, all_clients) = {
            let healthy_clients = self.client_registry.get_clients_in_group(target_group);
            let all_clients = self.client_registry.get_all_clients();
            let target_client = healthy_clients.get(0).cloned();
            (target_client, healthy_clients, all_clients)
        };
        match target_client {
            Some(client) => {
                tracing::info!(
                    event = "access",
                    trace_id = %trace_id,
                    host = %host,
                    path = %path,
                    group = %target_group,
                    selected_client = %client,
                    healthy_clients = ?healthy_clients,
                    all_clients = ?all_clients,
                    message = "access log"
                );
                let client_tx = self.connected_clients.lock().await.get(&client).cloned();
                if client_tx.is_none() {
                    return Ok(HyperResponse::builder()
                        .status(502)
                        .body(Body::from("Client not connected"))
                        .unwrap());
                }
                let client_tx = client_tx.unwrap();
                // clone headers and uri before consuming req
                let method = req.method().clone();
                let uri = req.uri().clone();
                let headers = req.headers().clone();
                let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
                let request_id = Uuid::new_v4().to_string();
                // 构造 HttpRequest
                let http_req = tunnel_lib::tunnel::HttpRequest {
                    method: method.to_string(),
                    url: uri.to_string(),
                    host: headers.get("host").and_then(|h| h.to_str().ok()).unwrap_or("").to_string(),
                    path: uri.path().to_string(),
                    query: uri.query().unwrap_or("").to_string(),
                    headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect(),
                    body: body_bytes.to_vec(),
                    original_dst: "".to_string(),
                };
                let tunnel_msg = tunnel_lib::http_forward::build_http_tunnel_message(
                    &client,
                    &request_id,
                    tunnel_lib::tunnel::Direction::ServerToClient,
                    http_req,
                    &trace_id,
                );
                let (tx, rx) = oneshot::channel();
                self.pending_reverse_requests.lock().await.insert(request_id.clone(), tx);
                if let Err(e) = client_tx.send(tunnel_msg).await {
                    self.pending_reverse_requests.lock().await.remove(&request_id);
                    return Ok(HyperResponse::builder()
                        .status(502)
                        .body(Body::from(format!("Failed to send tunnel message: {}", e)))
                        .unwrap());
                }
                match timeout(Duration::from_secs(30), rx).await {
                    Ok(Ok(http_resp)) => {
                        let mut response_builder = HyperResponse::builder().status(http_resp.status_code as u16);
                        for (key, value) in http_resp.headers {
                            response_builder = response_builder.header(key, value);
                        }
                        Ok(response_builder.body(Body::from(http_resp.body)).unwrap())
                    }
                    Ok(Err(_)) => {
                        Ok(HyperResponse::builder().status(502).body(Body::from("Tunnel response failed")).unwrap())
                    }
                    Err(_) => {
                        self.pending_reverse_requests.lock().await.remove(&request_id);
                        Ok(HyperResponse::builder().status(504).body(Body::from("Tunnel request timeout")).unwrap())
                    }
                }
            },
            None => {
                if self.trace_enabled {
                    tracing::error!(
                        event = "select_client_error",
                        trace_id = %trace_id,
                        host = %host,
                        path = %path,
                        group = %target_group,
                        message = "No healthy clients found in group"
                    );
                }
                return Ok(HyperResponse::builder()
                    .status(502)
                    .body(Body::from("No healthy clients found in group"))
                    .unwrap());
            }
        }
    }
}

// 更新 HTTP 处理逻辑，区分 reverse proxy 和 client group 转发
pub async fn handle_http_request(
    req: HyperRequest<Body>,
    rules_engine: Arc<RulesEngine>,
    proxy_handler: Arc<ProxyHandler>,
) -> Result<HyperResponse<Body>, hyper::Error> {
    let host = req.headers().get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");
    let path = req.uri().path();

    // 1. 尝试匹配规则，匹配则发到对应 client group
    if let Some(rule) = rules_engine.match_reverse_proxy_rule(host, path, None) {
        if let Some(group_name) = rule.extract_client_group() {
            println!("Forwarding to client group: {}", group_name);
            return proxy_handler.forward_via_tunnel(req, &group_name).await;
        }
    }

    // 2. 未匹配规则，404
    println!("No matching rule for host: {}, path: {}", host, path);
    Ok(HyperResponse::builder()
        .status(404)
        .body(Body::from("No matching rule"))
        .unwrap())
}

// 从 host 提取目标 group（简单实现）
fn extract_group_from_host(host: &str) -> String {
    if host.contains("group-a") || host == "a.com" {
        "group-a".to_string()
    } else if host.contains("group-b") || host == "b.com" {
        "group-b".to_string()
    } else {
        "".to_string()
    }
}

// Start gRPC entry on 8002, match rules and dispatch
pub async fn start_grpc_entry(rules_engine: Arc<RulesEngine>, proxy_handler: Arc<ProxyHandler>) {
    // For demonstration, use hyper to listen on 8002, but in real use, tonic or tower would be used for gRPC
    // Here we just show the structure and leave a TODO for real gRPC proxy logic
    let make_svc = make_service_fn(move |_| {
        let rules_engine = rules_engine.clone();
        let proxy_handler = proxy_handler.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: HyperRequest<Body>| {
                handle_grpc_request(req, rules_engine.clone(), proxy_handler.clone())
            }))
        }
    });
    let addr = ([0, 0, 0, 0], 8002).into();
    let server = HyperServer::bind(&addr).serve(make_svc);
    println!("gRPC entry listening on http://{} (stub, not real gRPC)", addr);
    if let Err(e) = server.await {
        eprintln!("gRPC entry server error: {}", e);
    }
}

// 更新 gRPC 处理逻辑，同样区分 reverse proxy 和 client group 转发
pub async fn handle_grpc_request(
    req: HyperRequest<Body>,
    rules_engine: Arc<RulesEngine>,
    proxy_handler: Arc<ProxyHandler>,
) -> Result<HyperResponse<Body>, hyper::Error> {
    let host = req.headers().get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");
    
    // 从 gRPC 请求中提取 service（简化实现，实际需要解析 gRPC 协议）
    let service = req.headers().get("grpc-service")
        .and_then(|s| s.to_str().ok())
        .unwrap_or("");
    
    let headers: HashMap<String, String> = req.headers().iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // 1. 尝试匹配 gRPC 规则
    if let Some(rule) = rules_engine.match_reverse_proxy_rule(host, service, Some(service)) {
        
        // 2. 处理 reverse proxy 规则（使用本地 upstream）
        if rule.is_reverse_proxy_rule() {
            if let Some(ref upstream_name) = rule.action_upstream {
                if let Some(upstream) = rules_engine.get_upstream(upstream_name) {
                    if let Some(backend) = pick_backend(upstream) {
                        println!("gRPC reverse proxy to upstream '{}': {}", upstream_name, backend);
                        return proxy_handler.handle_proxy_pass(req, &backend).await;
                    } else {
        return Ok(HyperResponse::builder()
            .status(502)
                            .body(Body::from("No available backends in gRPC upstream"))
            .unwrap());
                    }
    } else {
        return Ok({
            let err_resp = error_response(
                ProxyErrorKind::NoUpstream,
                None,
                None,
                None,
                None,
            );
            let mut builder = HyperResponse::builder()
                .status(err_resp.status_code as u16)
                .header("content-type", "application/json");
            for (k, v) in err_resp.headers.iter() {
                builder = builder.header(k, v);
            }
            builder.body(Body::from(err_resp.body)).unwrap()
        });
    }
            }
        }
        
        // 3. 处理 client group 转发规则
        if rule.is_client_group_rule() {
            if let Some(group_name) = rule.extract_client_group() {
                println!("gRPC forwarding to client group: {}", group_name);
                return proxy_handler.forward_via_tunnel(req, &group_name).await;
            }
        }
    }

    // 5. 未匹配规则，尝试基于 host 的默认 group 转发
    let target_group = extract_group_from_host(host);
    if !target_group.is_empty() {
        println!("gRPC default group forwarding to: {}", target_group);
        return proxy_handler.forward_via_tunnel(req, &target_group).await;
    }

    // 6. 默认 404
    println!("No matching gRPC rule for host: {}, service: {}", host, service);
    Ok(HyperResponse::builder()
        .status(404)
        .body(Body::from("No matching gRPC rule or group"))
        .unwrap())
}

// TODO: start_grpc_entry 同理监听 8002，按 gRPC 规则分发 