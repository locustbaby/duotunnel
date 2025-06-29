use hyper::{Body, Request as HyperRequest, Response as HyperResponse};
use tokio::sync::{mpsc, oneshot, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use tracing::info;
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse};
use tunnel_lib::http_forward::{forward_http_via_tunnel, set_host_header};
use tunnel_lib::proxy::{HttpEntryProxyTarget, HttpTunnelContext};
use tunnel_lib::response::{self, ProxyErrorKind, error_response};
use async_trait::async_trait;

/// Client 端 HTTP 入口 ProxyTarget 实现
pub struct ClientHttpEntryTarget {
    pub tunnel_tx: Arc<RwLock<Option<mpsc::Sender<TunnelMessage>>>>,
    pub pending_requests: Arc<RwLock<Option<Arc<DashMap<String, oneshot::Sender<HttpResponse>>>>>>,
    pub client_id: Arc<RwLock<Option<String>>>,
}

#[async_trait]
impl HttpEntryProxyTarget for ClientHttpEntryTarget {
    async fn handle(
        &self,
        req: HyperRequest<Body>,
        _ctx: &HttpTunnelContext,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        // 动态获取当前活跃 channel
        let tunnel_tx = self.tunnel_tx.read().await.clone();
        let pending_requests_opt = self.pending_requests.read().await.clone();
        let client_id = self.client_id.read().await.clone().unwrap_or_else(|| "unknown".to_string());
        if let Some(tunnel_tx) = tunnel_tx {
            if let Some(pending_requests) = pending_requests_opt {
                // 正常转发
                let request_id = Uuid::new_v4().to_string();
                return forward_http_via_tunnel(
                    req,
                    &client_id,
                    &tunnel_tx,
                    pending_requests,
                    request_id,
                    tunnel_lib::tunnel::Direction::ClientToServer,
                ).await;
            } else {
                // pending_requests 尚未初始化，优雅返回 502
                let resp = error_response(
                    ProxyErrorKind::NoUpstream,
                    Some("Tunnel not connected, please retry (pending_requests)"),
                    None,
                    None,
                    Some(client_id.as_str()),
                );
                return Ok(HyperResponse::builder()
                    .status(502)
                    .header("content-type", "application/json")
                    .body(Body::from(resp.body))
                    .unwrap());
            }
        } else {
            // 无可用 channel，优雅返回 502
            let resp = error_response(
                ProxyErrorKind::NoUpstream,
                Some("Tunnel not connected, please retry"),
                None,
                None,
                Some(client_id.as_str()),
            );
            return Ok(HyperResponse::builder()
                .status(502)
                .header("content-type", "application/json")
                .body(Body::from(resp.body))
                .unwrap());
        }
    }
} 