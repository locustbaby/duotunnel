use anyhow::Result;
use tracing::{debug, info};
use tunnel_lib::proxy::core::{FlowDirection, StreamFlow, UpstreamResolver};
use tunnel_lib::recv_routing_info;

pub async fn handle_tunnel_stream<A: UpstreamResolver>(
    send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    app: A,
) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;
    info!(
        protocol = ?routing_info.protocol,
        "handling egress request from client"
    );
    StreamFlow::new(app)
        .run_stream(send, recv, "0.0.0.0:0".parse().unwrap(), FlowDirection::Egress, Some(routing_info))
        .await?;
    debug!("egress stream completed");
    Ok(())
}
