use crate::tunnel::*;
use hyper::{Request as HyperRequest, Body, Client, Method};
use hyper::client::HttpConnector;
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
pub fn http_request_from_hyper(req: &hyper::Request<hyper::Body>, body: Vec<u8>) -> crate::tunnel::HttpRequest {
    let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
    crate::tunnel::HttpRequest {
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
pub async fn forward_http_to_backend(
    req: &crate::tunnel::HttpRequest,
    backend_url: &str,
    http_client: Arc<Client<HttpConnector, Body>>,
    override_host: &str,
) -> crate::tunnel::HttpResponse {
    let client = &*http_client;
    let method = match req.method.parse::<Method>() {
        Ok(m) => m,
        Err(_) => return crate::tunnel::HttpResponse {
            status_code: 400,
            headers: HashMap::new(),
            body: b"Invalid method".to_vec(),
        },
    };
    // Compose URL using url crate
    let url = if let Ok(parsed) = Url::parse(&req.url) {
        parsed.to_string()
    } else {
        let base_url = if !req.host.is_empty() {
            format!("http://{}", req.host)
        } else if backend_url.starts_with("http://") || backend_url.starts_with("https://") {
            backend_url.to_string()
        } else if backend_url.ends_with(":443") {
            format!("https://{}", backend_url)
        } else {
            format!("http://{}", backend_url)
        };
        match Url::parse(&base_url).and_then(|base| base.join(&req.url)) {
            Ok(url) => url.to_string(),
            Err(e) => {
                error!("Failed to join base_url and req.url: {}", e);
                format!("{}{}", base_url, req.url)
            }
        }
    };
    // Build headers, set Host
    let mut headers = req.headers.clone();
    let backend_host = match Url::parse(&url) {
        Ok(url) => url.host_str().unwrap_or("").to_string(),
        Err(_) => String::new(),
    };
    let set_host = if !override_host.is_empty() {
        override_host
    } else {
        backend_host.as_str()
    };
    set_host_header(&mut headers, set_host);
    let mut builder = HyperRequest::builder().method(method).uri(&url);
    for (k, v) in headers.iter() {
        builder = builder.header(k, v);
    }
    let hyper_req = match builder.body(Body::from(req.body.clone())) {
        Ok(req) => req,
        Err(_) => return crate::tunnel::HttpResponse {
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
            crate::tunnel::HttpResponse {
                status_code: status,
                headers,
                body: body_bytes,
            }
        }
        Err(_e) => {
            resp_500(
                Some("Request failed"),
                None, // trace_id
                None, // request_id
                None, // identity
            )
        }
    }
} 