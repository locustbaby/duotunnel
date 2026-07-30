use anyhow::Result;
use duotunnel_lib::proxy::core::{ProxyEngine, UpstreamResolver};
use duotunnel_lib::recv_routing_info_bounded;
use tracing::debug;

const ROUTING_INFO_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub async fn handle_tunnel_stream<A: UpstreamResolver>(
    send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    app: A,
    peek_buf_size: usize,
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
    ProxyEngine::new_with_peek_buf_size(app, peek_buf_size)
        .run_stream(send, recv, client_addr, Some(routing_info))
        .await?;
    debug!("egress stream completed");
    Ok(())
}
