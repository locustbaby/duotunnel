use anyhow::Result;
use tracing::{debug, warn};
use tunnel_lib::proxy::core::{ProxyEngine, UpstreamResolver};
use tunnel_lib::recv_routing_info;

pub async fn handle_tunnel_stream<A: UpstreamResolver>(
    send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    app: A,
) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;
    debug!(
        target_host = ?routing_info.host, protocol = ?routing_info.protocol,
        "handling egress request from client"
    );
    let ip = match routing_info.src_addr.parse::<std::net::IpAddr>() {
        Ok(ip) => ip,
        Err(e) => {
            warn!(src_addr = %routing_info.src_addr, src_port = routing_info.src_port, error = %e, "failed to parse client IP");
            return Err(anyhow::anyhow!("invalid client IP: {}", e));
        }
    };
    let client_addr = std::net::SocketAddr::new(ip, routing_info.src_port);
    ProxyEngine::new(app)
        .run_stream(send, recv, client_addr, Some(routing_info))
        .await?;
    debug!("egress stream completed");
    Ok(())
}
