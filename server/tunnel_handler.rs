use anyhow::Result;
use tracing::{info, debug};
use tunnel_lib::{recv_routing_info, RoutingInfo};
use tunnel_lib::proxy::core::{ProxyEngine, Context, Protocol};
use std::sync::Arc;
use crate::config::ServerConfigFile;
use crate::egress::{ServerEgressApp, ServerEgressMap};

pub async fn handle_tunnel_stream(
    send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    config: Arc<ServerConfigFile>,
    egress_map: Arc<ServerEgressMap>,
) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;
    
    info!(
        target_host = ?routing_info.host,
        protocol = %routing_info.protocol,
        "handling egress request from client"
    );

    let protocol = match routing_info.protocol.as_str() {
        "websocket" => Protocol::WebSocket,
        "h1" => Protocol::H1,
        "h2" => Protocol::H2,
        _ => Protocol::Unknown,
    };

    let client_addr = format!("{}:{}", routing_info.src_addr, routing_info.src_port)
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

    let app = ServerEgressApp::new(egress_map);
    let engine = ProxyEngine::new(app);

    engine.run_stream(send, recv, client_addr, Some(routing_info)).await?;

    debug!("egress stream completed");

    Ok(())
}
