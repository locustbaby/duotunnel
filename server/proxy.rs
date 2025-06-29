use crate::rules::RulesEngine;
use std::sync::Arc;
use hyper::{Body, Request as HyperRequest, Response as HyperResponse, Server as HyperServer};
use hyper::service::{make_service_fn, service_fn};
use std::collections::HashMap;

use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::registry::ManagedClientRegistry;
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse};
use tokio::sync::{oneshot, mpsc};
use uuid::Uuid;
use std::time::Duration;
use tokio::time::timeout;
use tunnel_lib::response::{error_response, ProxyErrorKind};
use dashmap::DashMap;
use crate::utils::pick_backend;

// Global round-robin index for each upstream
lazy_static! {
    static ref UPSTREAM_INDEX: Mutex<std::collections::HashMap<String, usize>> = Mutex::new(std::collections::HashMap::new());
}

pub struct ProxyHandler {
    client_registry: Arc<ManagedClientRegistry>,
    pending_reverse_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    connected_clients: Arc<tokio::sync::Mutex<HashMap<String, mpsc::Sender<TunnelMessage>>>>,
    pub trace_enabled: bool,
}

impl ProxyHandler {
    pub fn new(trace_enabled: bool) -> Self {
        Self {
            client_registry: Arc::new(ManagedClientRegistry::new()),
            pending_reverse_requests: Arc::new(DashMap::new()),
            connected_clients: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            trace_enabled,
        }
    }

    pub fn with_dependencies(
        client_registry: Arc<ManagedClientRegistry>,
        pending_reverse_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
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
                self.pending_reverse_requests.insert(request_id.clone(), tx);
                if let Err(e) = client_tx.send(tunnel_msg).await {
                    self.pending_reverse_requests.remove(&request_id);
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
                        self.pending_reverse_requests.remove(&request_id);
                        let err_resp = error_response(
                            ProxyErrorKind::Timeout,
                            None,
                            Some(&trace_id),
                            Some(&request_id),
                            Some(&client),
                        );
                        let mut builder = HyperResponse::builder().status(err_resp.status_code as u16);
                        for (k, v) in err_resp.headers.iter() {
                            builder = builder.header(k, v);
                        }
                        Ok(builder.body(Body::from(err_resp.body)).unwrap())
                    }
                    Err(_) => {
                        self.pending_reverse_requests.remove(&request_id);
                        let err_resp = error_response(
                            ProxyErrorKind::Timeout,
                            None,
                            Some(&trace_id),
                            Some(&request_id),
                            Some(&client),
                        );
                        let mut builder = HyperResponse::builder().status(err_resp.status_code as u16);
                        for (k, v) in err_resp.headers.iter() {
                            builder = builder.header(k, v);
                        }
                        Ok(builder.body(Body::from(err_resp.body)).unwrap())
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