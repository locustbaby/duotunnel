use hyper::{Request as HyperRequest, Body, Response as HyperResponse};
use tokio::sync::{mpsc, oneshot};
use std::sync::Arc;
use dashmap::DashMap;
use crate::tunnel::{TunnelMessage, HttpResponse, Direction};
use async_trait::async_trait;

/// Aggregated context for HTTP tunnel entry handling.
#[derive(Clone)]
pub struct HttpTunnelContext {
    pub client_id: String,
    pub tunnel_tx: Arc<mpsc::Sender<TunnelMessage>>,
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    pub direction: Direction,
    // 可扩展: trace_id, timeout, user_info, etc.
}

#[async_trait]
pub trait HttpEntryProxyTarget: Send + Sync + 'static {
    async fn handle(
        &self,
        req: HyperRequest<Body>,
        ctx: &HttpTunnelContext,
    ) -> Result<HyperResponse<Body>, hyper::Error>;
}

/// Generic HTTP entry handler, delegating to the provided ProxyTarget implementation.
pub async fn http_entry_handler<T: HttpEntryProxyTarget>(
    req: HyperRequest<Body>,
    ctx: &HttpTunnelContext,
    target: &T,
) -> Result<HyperResponse<Body>, hyper::Error> {
    target.handle(req, ctx).await
}