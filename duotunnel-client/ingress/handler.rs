use crate::ingress::app::{IngressClientApp, LocalProxyMap};
use anyhow::Result;
use duotunnel_lib::plugin::{LoadBalancer, Resolver};
use duotunnel_lib::proxy::core::ProxyEngine;
use duotunnel_lib::recv_routing_info;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use tracing::debug;

pub async fn handle_work_stream<L: LoadBalancer, R: Resolver>(
    send: SendStream,
    mut recv: RecvStream,
    proxy_map: Arc<LocalProxyMap<L, R>>,
    tcp_params: duotunnel_lib::TcpParams,
    buffer_params: duotunnel_lib::ProxyBufferParams,
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
    let app =
        IngressClientApp::new_with_buffer_params(proxy_map, tcp_params, buffer_params.clone());
    let engine = ProxyEngine::new_with_buffer_params(app, buffer_params);
    engine
        .run_stream(send, recv, client_addr, Some(routing_info))
        .await
}
