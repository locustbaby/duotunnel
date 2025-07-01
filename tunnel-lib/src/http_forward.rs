use crate::tunnel::*;
use hyper::{Request as HyperRequest, Body, Client, Method};
use hyper::client::HttpConnector;
use hyper::client::connect::Connect;
use std::collections::HashMap;
use tokio::sync::oneshot;
use tokio::sync::mpsc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;
use std::sync::Arc;
use url::Url;
use tracing::error;
use crate::response::{ProxyErrorKind, error_response, resp_404, resp_502, resp_500};
use dashmap::DashMap;

/// Forward an HTTP request via the tunnel, used by both client and server HTTP handlers.
/// Handles request serialization, tunnel message sending, and response mapping.
pub async fn forward_http_via_tunnel(
    http_req: HyperRequest<Body>,
    client_id: &str,
    tunnel_sender: &mpsc::Sender<TunnelMessage>,
    pending_map: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    request_id: String,
    direction: Direction,
    stream_id: String,
) -> Result<hyper::Response<Body>, hyper::Error> {
    let (parts, body) = http_req.into_parts();
    let host = parts.headers.get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
    let trace_id = parts.headers
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let body_bytes = hyper::body::to_bytes(body).await.unwrap_or_default();
    let http_request = HttpRequest {
        stream_id: stream_id.clone(),
        method: parts.method.to_string(),
        url: parts.uri.to_string(),
        host: host.to_string(),
        path: parts.uri.path().to_string(),
        query: parts.uri.query().unwrap_or("").to_string(),
        headers: parts.headers.iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect(),
        body: body_bytes.to_vec(),
        original_dst: "".to_string(),
    };
    let tunnel_msg = build_http_tunnel_message(
        client_id,
        &request_id,
        direction,
        http_request,
        &trace_id,
    );
    let (tx, rx) = oneshot::channel();
    pending_map.insert(request_id.clone(), tx);
    if let Err(e) = tunnel_sender.send(tunnel_msg).await {
        pending_map.remove(&request_id);
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
            pending_map.remove(&request_id);
            let err_resp = error_response(
                ProxyErrorKind::Timeout,
                None,
                Some(&trace_id),
                Some(&request_id),
                Some(client_id),
            );
            let mut builder = hyper::Response::builder().status(err_resp.status_code as u16);
            for (k, v) in err_resp.headers.iter() {
                builder = builder.header(k, v);
            }
            Ok(builder.body(Body::from(err_resp.body)).unwrap())
        }
    }
}

/// Build an HttpRequest from a HyperRequest and body bytes.
pub fn http_request_from_hyper(req: &hyper::Request<hyper::Body>, body: Vec<u8>, stream_id: String) -> crate::tunnel::HttpRequest {
    let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
    crate::tunnel::HttpRequest {
        stream_id,
        method: req.method().to_string(),
        url: req.uri().to_string(),
        host: host.to_string(),
        path: req.uri().path().to_string(),
        query: req.uri().query().unwrap_or("").to_string(),
        headers: req.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect(),
        body,
        original_dst: "".to_string(),
    }
}

/// Build a TunnelMessage for HTTP forwarding.
pub fn build_http_tunnel_message(
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

/// Set or override the Host header in a header map.
pub fn set_host_header(headers: &mut HashMap<String, String>, set_host: &str) {
    if !set_host.is_empty() {
        headers.insert("host".to_string(), set_host.to_string());
    }
}

/// Forward an HTTP/HTTPS request to a backend, setting Host as needed.
pub async fn forward_http_to_backend<C>(
    req: &crate::tunnel::HttpRequest,
    backend_url: &str,
    https_client: Arc<Client<C, Body>>,
    override_host: &str,
) -> crate::tunnel::HttpResponse
where
    C: Connect + Clone + Send + Sync + 'static,
{
    let client = &*https_client;
    let method = match req.method.parse::<hyper::Method>() {
        Ok(m) => m,
        Err(_) => return crate::tunnel::HttpResponse {
            status_code: 400,
            headers: std::collections::HashMap::new(),
            body: b"Invalid method".to_vec(),
        },
    };
    let url = match Url::parse(backend_url).and_then(|base| base.join(&req.url)) {
        Ok(url) => url.to_string(),
        Err(e) => {
            tracing::error!("Failed to join backend_url and req.url: {}", e);
            format!("{}{}", backend_url, req.url)
        }
    };
    let mut builder = hyper::Request::builder()
        .method(method)
        .uri(&url);
    for (k, v) in req.headers.iter() {
        builder = builder.header(k, v);
    }
    let hyper_req = match builder.body(Body::from(req.body.clone())) {
        Ok(req) => req,
        Err(_) => return crate::tunnel::HttpResponse {
            status_code: 400,
            headers: std::collections::HashMap::new(),
            body: b"Invalid request".to_vec(),
        },
    };
    match client.request(hyper_req).await {
        Ok(resp) => {
            let status = resp.status().as_u16() as i32;
            let headers: std::collections::HashMap<String, String> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let body_bytes = match hyper::body::to_bytes(resp.into_body()).await {
                Ok(bytes) => bytes.to_vec(),
                Err(_) => b"Failed to read response body".to_vec(),
            };
            crate::tunnel::HttpResponse {
                status_code: status,
                headers,
                body: body_bytes,
            }
        }
        Err(e) => {
            tracing::error!("HTTP(S) request to backend failed: {}", e);
            crate::tunnel::HttpResponse {
                status_code: 502,
                headers: std::collections::HashMap::new(),
                body: b"Bad Gateway".to_vec(),
            }
        }
    }
} 