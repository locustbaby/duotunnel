use crate::ingress::app::{IngressClientApp, LocalProxyMap};
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use tracing::debug;
use tunnel_lib::proxy::core::ProxyEngine;
use tunnel_lib::recv_routing_info;
pub async fn handle_work_stream(
    send: SendStream,
    mut recv: RecvStream,
    proxy_map: Arc<LocalProxyMap>,
    tcp_params: tunnel_lib::TcpParams,
) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;
    debug!(
        proxy_name = %routing_info.proxy_name,
        protocol = ?routing_info.protocol,
        host = ?routing_info.host,
        src = format!("{}:{}", routing_info.src_addr, routing_info.src_port),
        "received work stream"
    );
    let ip = routing_info
        .src_addr
        .parse::<std::net::IpAddr>()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
    let client_addr = std::net::SocketAddr::new(ip, routing_info.src_port);
    let app = IngressClientApp::new(proxy_map, tcp_params);
    let engine = ProxyEngine::new(app);
    engine
        .run_stream(send, recv, client_addr, Some(routing_info))
        .await
}
