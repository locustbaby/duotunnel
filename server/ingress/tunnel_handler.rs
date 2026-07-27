use anyhow::Result;
use tracing::{debug, warn};
use tunnel_lib::proxy::core::{ProxyEngine, UpstreamResolver};
use tunnel_lib::recv_routing_info_bounded;

const ROUTING_INFO_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub async fn handle_tunnel_stream<A: UpstreamResolver>(
    send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    app: A,
) -> Result<()> {
    let routing_info =
        tokio::time::timeout(ROUTING_INFO_TIMEOUT, recv_routing_info_bounded(&mut recv))
            .await
            .map_err(|_| anyhow::anyhow!("routing info timed out"))??;
    debug!(
        target_host = ?routing_info.host, protocol = ?routing_info.protocol,
        "handling egress request from client"
    );
    let ip = routing_info.src_addr;
    let client_addr = std::net::SocketAddr::new(ip, routing_info.src_port);
    ProxyEngine::new(app)
        .run_stream(send, recv, client_addr, Some(routing_info))
        .await?;
    debug!("egress stream completed");
    Ok(())
}
