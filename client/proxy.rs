use hyper::{Body, Request as HyperRequest, Response as HyperResponse};
use tokio::sync::{mpsc, oneshot};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;
use tracing::info;
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse};

pub async fn handle_http_entry(
    req: HyperRequest<Body>,
    client_id: String,
    tunnel_tx: mpsc::Sender<TunnelMessage>,
    pending_requests: Arc<tokio::sync::Mutex<HashMap<String, oneshot::Sender<HttpResponse>>>>,
) -> Result<HyperResponse<Body>, hyper::Error> {
    let request_id = Uuid::new_v4().to_string();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("").to_string();
    info!(
        event = "client_http_entry",
        method = %method,
        path = %path,
        host = %host,
        request_id = %request_id,
        message = "Received HTTP request at client entry"
    );
    info!(
        event = "client_forward_via_tunnel",
        request_id = %request_id,
        message = "Forwarding HTTP request to server via tunnel"
    );
    tunnel_lib::proxy::forward_via_tunnel(
        req,
        &client_id,
        &tunnel_tx,
        pending_requests.clone(),
        request_id.clone(),
        tunnel_lib::tunnel::Direction::ClientToServer,
    ).await
} 