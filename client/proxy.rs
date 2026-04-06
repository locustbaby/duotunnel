use crate::app::{ClientApp, LocalProxyMap};
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use tracing::info;
use tunnel_lib::proxy::core::ProxyEngine;
use tunnel_lib::recv_routing_info;
#[cfg_attr(feature = "profiling", tracing::instrument(skip_all))]
pub async fn handle_work_stream(
    send: SendStream,
    mut recv: RecvStream,
    proxy_map: Arc<LocalProxyMap>,
    tcp_params: tunnel_lib::TcpParams,
) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;
    info!(
        proxy_name = % routing_info.proxy_name, protocol = % routing_info.protocol, host
        = ? routing_info.host, src = format!("{}:{}", routing_info.src_addr, routing_info
        .src_port), "received work stream"
    );
    if routing_info.protocol == "h2" {
        // Round-robin select the backend, then look up its pre-parsed (scheme, authority).
        let upstream_addr = proxy_map
            .get_local_address(&routing_info.proxy_name)
            .ok_or_else(|| {
                anyhow::anyhow!("no upstream for proxy_name: {}", routing_info.proxy_name)
            })?;
        let (scheme, authority) = proxy_map
            .get_h2_addr_for_server(&upstream_addr)
            .ok_or_else(|| anyhow::anyhow!("no H2 addr cached for upstream: {}", upstream_addr))?;
        info!(
            "H2 protocol detected, forwarding to upstream: {}",
            authority
        );
        let stream = tunnel_lib::QuinnStream { send, recv };
        tunnel_lib::proxy::h2::serve_h2_forward(
            stream,
            proxy_map.https_client.clone(),
            proxy_map.h2c_client.clone(),
            scheme.to_string(),
            authority.to_string(),
        )
        .await?;
    } else {
        let app = ClientApp::new(proxy_map, tcp_params);
        let engine = ProxyEngine::new(app);
        let client_addr = format!("{}:{}", routing_info.src_addr, routing_info.src_port)
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
        engine
            .run_stream(send, recv, client_addr, Some(routing_info))
            .await?;
    }
    Ok(())
}
