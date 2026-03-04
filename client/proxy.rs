use crate::app::{ClientApp, LocalProxyMap};
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use tracing::info;
use tunnel_lib::proxy::core::ProxyEngine;
use tunnel_lib::recv_routing_info;

pub async fn handle_work_stream(
    send: SendStream,
    mut recv: RecvStream,
    proxy_map: Arc<LocalProxyMap>,
    tcp_params: tunnel_lib::TcpParams,
) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;

    info!(
        proxy_name = %routing_info.proxy_name,
        protocol = %routing_info.protocol,
        host = ?routing_info.host,
        src = format!("{}:{}", routing_info.src_addr, routing_info.src_port),
        "received work stream"
    );

    if routing_info.protocol == "h2" {
        let host = routing_info
            .host
            .clone()
            .unwrap_or_else(|| routing_info.proxy_name.clone());

        let upstream_addr = proxy_map
            .get_local_address(&routing_info.proxy_name, Some(&host), false)
            .ok_or_else(|| anyhow::anyhow!("no upstream for H2 request to {}", host))?;

        info!(
            "H2 protocol detected, forwarding to upstream: {}",
            upstream_addr
        );

        let normalized_addr = upstream_addr
            .replace("wss://", "https://")
            .replace("ws://", "http://");
        let normalized_addr = if normalized_addr.contains("://") {
            normalized_addr
        } else {
            format!("http://{}", normalized_addr)
        };
        let upstream_uri: hyper::Uri = normalized_addr
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid upstream URI '{}': {}", normalized_addr, e))?;

        let scheme = upstream_uri
            .scheme_str()
            .unwrap_or("http")
            .to_string();
        let authority = upstream_uri
            .authority()
            .map(|a| a.as_str().to_string())
            .unwrap_or(upstream_addr);

        let stream = tunnel_lib::QuinnStream { send, recv };
        tunnel_lib::proxy::h2::serve_h2_forward(
            stream,
            proxy_map.https_client.clone(),
            scheme,
            authority,
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
