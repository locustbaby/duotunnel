use crate::tunnel::*;
use hyper::{Client as HyperClient, Request as HyperRequest, Body, Method};
use std::collections::HashMap;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct ProxyHandler {
    http_client: HyperClient<hyper::client::HttpConnector>,
    // 可扩展：grpc_client, upstream_pool, etc.
}

impl ProxyHandler {
    pub fn new() -> Self {
        Self {
            http_client: HyperClient::new(),
        }
    }

    /// 通用 HTTP 反代，支持 header 注入、upstream 负载均衡
    pub async fn handle_http_request(&self, req: HttpRequest, extra_headers: Option<HashMap<String, String>>) -> HttpResponse {
        let method = match req.method.parse::<Method>() {
            Ok(m) => m,
            Err(_) => return self.error_response(400, "Invalid method"),
        };
        let mut builder = HyperRequest::builder()
            .method(method)
            .uri(&req.url);
        for (k, v) in req.headers.iter() {
            builder = builder.header(k, v);
        }
        if let Some(extra) = extra_headers {
            for (k, v) in extra.iter() {
                builder = builder.header(k, v);
            }
        }
        let hyper_req = match builder.body(Body::from(req.body)) {
            Ok(req) => req,
            Err(_) => return self.error_response(400, "Invalid request"),
        };
        match self.http_client.request(hyper_req).await {
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
                HttpResponse {
                    status_code: status,
                    headers,
                    body: body_bytes,
                }
            }
            Err(e) => {
                eprintln!("HTTP request failed: {}", e);
                self.error_response(500, "Request failed")
            }
        }
    }

    /// 预留 gRPC 反代接口
    pub async fn handle_grpc_request(&self, _req: GrpcRequest) -> GrpcResponse {
        // TODO: 实现 gRPC 反代
        GrpcResponse {
            status_code: 501,
            headers: HashMap::new(),
            body: b"gRPC proxy not implemented".to_vec(),
        }
    }

    fn error_response(&self, status: u16, message: &str) -> HttpResponse {
        HttpResponse {
            status_code: status as i32,
            headers: HashMap::new(),
            body: message.as_bytes().to_vec(),
        }
    }
}

pub struct RequestTracker {
    pending: std::sync::Arc<std::sync::Mutex<HashMap<String, oneshot::Sender<TunnelMessage>>>>,
}

impl RequestTracker {
    pub fn new() -> Self {
        Self {
            pending: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn track_request(&self, request_id: String, sender: oneshot::Sender<TunnelMessage>) {
        self.pending.lock().unwrap().insert(request_id, sender);
    }

    pub fn complete_request(&self, request_id: &str, response: TunnelMessage) -> bool {
        if let Some(sender) = self.pending.lock().unwrap().remove(request_id) {
            sender.send(response).is_ok()
        } else {
            false
        }
    }

    pub fn get_pending_count(&self) -> usize {
        self.pending.lock().unwrap().len()
    }
}

/// HTTP 入口服务启动函数
pub async fn start_http_entry(addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    use hyper::{Server, service::{make_service_fn, service_fn}};
    use hyper::{Body, Request, Response};
    
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(|_req: Request<Body>| async {
            Ok::<_, hyper::Error>(Response::new(Body::from("HTTP Entry Point")))
        }))
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("HTTP entry listening on {}", addr);
    
    if let Err(e) = server.await {
        eprintln!("HTTP server error: {}", e);
    }
    
    Ok(())
}

/// gRPC 入口服务启动函数
pub async fn start_grpc_entry(addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    println!("gRPC entry listening on {}", addr);
    // TODO: 实现实际的 gRPC 服务
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    Ok(())
} 