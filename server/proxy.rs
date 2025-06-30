use std::sync::Arc;
use hyper::{Body, Request as HyperRequest, Response as HyperResponse};
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use crate::rules::RulesEngine;
use crate::registry::ManagedClientRegistry;
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse};
use tunnel_lib::proxy::{HttpEntryProxyTarget, HttpTunnelContext};
use tunnel_lib::response;
use async_trait::async_trait;
use uuid;

/// 服务器端 HTTP 入口 ProxyTarget 实现
pub struct ServerHttpEntryTarget {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pub connected_streams: Arc<DashMap<(String, String), mpsc::Sender<TunnelMessage>>>,
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    pub token_map: Arc<DashMap<String, CancellationToken>>,
}

#[async_trait]
impl HttpEntryProxyTarget for ServerHttpEntryTarget {
    async fn handle(
        &self,
        req: HyperRequest<Body>,
        _ctx: &HttpTunnelContext,
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
                for client_id in &healthy_clients {
                    if let Some(token) = self.token_map.get(client_id.as_str()) {
                        if !token.is_cancelled() {
                            for entry in self.connected_streams.iter() {
                                let (cid, stream_id) = entry.key();
                                if cid == client_id && !entry.is_closed() {
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
                                        selected_stream = %stream_id,
                                        healthy_clients = ?healthy_clients,
                                        message = "access log"
                                    );
                                    use tunnel_lib::http_forward::forward_http_via_tunnel;
                                    use uuid::Uuid;
                                    let request_id = Uuid::new_v4().to_string();
                                    return forward_http_via_tunnel(
                                        req,
                                        &client_id,
                                        entry.value(),
                                        self.pending_requests.clone(),
                                        request_id,
                                        tunnel_lib::tunnel::Direction::ServerToClient,
                                        stream_id.clone(),
                                    ).await;
                                }
                            }
                        }
                    }
                }
                let err_resp = response::resp_502(None, None, Some("server"));
                return Ok(HyperResponse::builder()
                    .status(err_resp.status_code as u16)
                    .header("content-type", "application/json")
                    .body(Body::from(err_resp.body))
                    .unwrap());
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