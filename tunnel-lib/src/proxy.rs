use crate::tunnel::*;
use hyper::{Request as HyperRequest, Body};
use std::collections::HashMap;
use tokio::sync::oneshot;
use std::sync::Mutex;
use tokio::sync::mpsc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;
use std::sync::Arc;

/// 通用 tunnel 代理转发逻辑，client/server 入口 handler 可复用
pub async fn forward_via_tunnel(
    req: HyperRequest<Body>,
    client_id: &str,
    tunnel_tx: &mpsc::Sender<TunnelMessage>,
    pending_requests: Arc<tokio::sync::Mutex<HashMap<String, oneshot::Sender<HttpResponse>>>>,
    request_id: String,
    direction: Direction,
) -> Result<hyper::Response<Body>, hyper::Error> {
    let (parts, body) = req.into_parts();
    let host = parts.headers.get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
    let path = parts.uri.path();
    let trace_id = parts.headers
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let body_bytes = hyper::body::to_bytes(body).await.unwrap_or_default();
    let http_req = HttpRequest {
        method: parts.method.to_string(),
        url: parts.uri.to_string(),
        host: host.to_string(),
        path: path.to_string(),
        query: parts.uri.query().unwrap_or("").to_string(),
        headers: parts.headers.iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect(),
        body: body_bytes.to_vec(),
        original_dst: "".to_string(),
    };
    let tunnel_msg = TunnelMessage {
        client_id: client_id.to_string(),
        request_id: request_id.clone(),
        direction: direction as i32,
        payload: Some(crate::tunnel::tunnel_message::Payload::HttpRequest(http_req)),
        trace_id: trace_id.clone(),
    };
    let (tx, rx) = oneshot::channel();
    {
        let mut pending = pending_requests.lock().await;
        pending.insert(request_id.clone(), tx);
    }
    if let Err(e) = tunnel_tx.send(tunnel_msg).await {
        let mut pending = pending_requests.lock().await;
        pending.remove(&request_id);
        return Ok(hyper::Response::builder().status(502).body(Body::from(format!("Tunnel send error: {}", e))).unwrap());
    }
    match timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(resp)) => {
            let mut builder = hyper::Response::builder().status(resp.status_code as u16);
            for (k, v) in resp.headers {
                builder = builder.header(k, v);
            }
            Ok(builder.body(Body::from(resp.body)).unwrap())
        }
        Ok(Err(_)) => {
            Ok(hyper::Response::builder().status(502).body(Body::from("Tunnel response failed")).unwrap())
        }
        Err(_) => {
            let mut pending = pending_requests.lock().await;
            pending.remove(&request_id);
            Ok(hyper::Response::builder().status(504).body(Body::from("Tunnel request timeout")).unwrap())
        }
    }
}

/// 辅助函数：从 HyperRequest parts 和 body 构建 HttpRequest
pub fn build_http_request_from_parts(req: &hyper::Request<hyper::Body>, body: Vec<u8>) -> crate::tunnel::HttpRequest {
    let parts = req;
    let host = parts.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
    crate::tunnel::HttpRequest {
        method: parts.method().to_string(),
        url: parts.uri().to_string(),
        host: host.to_string(),
        path: parts.uri().path().to_string(),
        query: parts.uri().query().unwrap_or("").to_string(),
        headers: parts.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect(),
        body,
        original_dst: "".to_string(),
    }
}

/// 辅助函数：构建 TunnelMessage
pub fn build_tunnel_message(
    client_id: &str,
    request_id: &str,
    direction: crate::tunnel::Direction,
    http_req: crate::tunnel::HttpRequest,
    trace_id: &str,
) -> crate::tunnel::TunnelMessage {
    crate::tunnel::TunnelMessage {
        client_id: client_id.to_string(),
        request_id: request_id.to_string(),
        direction: direction as i32,
        payload: Some(crate::tunnel::tunnel_message::Payload::HttpRequest(http_req)),
        trace_id: trace_id.to_string(),
    }
} 