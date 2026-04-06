use anyhow::Result;
use tracing::{debug, info, instrument, warn};
use tunnel_lib::proxy::core::{Protocol, ProxyApp, ProxyEngine};
use tunnel_lib::recv_routing_info;

#[instrument(skip_all)]
pub async fn handle_tunnel_stream<A: ProxyApp>(
    send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    app: A,
) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;
    info!(
        target_host = ?routing_info.host, protocol = %routing_info.protocol,
        "handling egress request from client"
    );
    let _protocol = match routing_info.protocol.as_str() {
        "websocket" => Protocol::WebSocket,
        "h1" => Protocol::H1,
        "h2" => Protocol::H2,
        _ => Protocol::Unknown,
    };
    let client_addr = match format!("{}:{}", routing_info.src_addr, routing_info.src_port).parse() {
        Ok(addr) => addr,
        Err(e) => {
            warn!(src_addr = %routing_info.src_addr, src_port = routing_info.src_port, error = %e, "failed to parse client addr");
            return Err(anyhow::anyhow!("invalid client addr: {}", e));
        }
    };
    ProxyEngine::new(app)
        .run_stream(send, recv, client_addr, Some(routing_info))
        .await?;
    debug!("egress stream completed");
    Ok(())
}
