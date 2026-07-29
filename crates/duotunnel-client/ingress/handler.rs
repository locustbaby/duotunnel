use crate::ingress::app::{IngressClientApp, LocalProxyMap};
use anyhow::Result;
use duotunnel_core::plugin::{LoadBalancer, Resolver};
use duotunnel_core::proxy::core::ProxyEngine;
use duotunnel_core::recv_routing_info;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use tracing::debug;

pub async fn handle_work_stream<L: LoadBalancer, R: Resolver>(
    send: SendStream,
    mut recv: RecvStream,
    proxy_map: Arc<LocalProxyMap<L, R>>,
    tcp_params: duotunnel_core::TcpParams,
) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;
    debug!(
        proxy_name = %routing_info.proxy_name,
        protocol = ?routing_info.protocol,
        host = ?routing_info.host,
        src = format!("{}:{}", routing_info.src_addr, routing_info.src_port),
        "received work stream"
    );
    let client_addr = std::net::SocketAddr::new(routing_info.src_addr, routing_info.src_port);
    let app = IngressClientApp::new(proxy_map, tcp_params);
    let engine = ProxyEngine::new(app);
    engine
        .run_stream(send, recv, client_addr, Some(routing_info))
        .await
}
