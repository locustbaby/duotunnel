use anyhow::Result;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn, debug};

use tunnel_lib::proxy;

use crate::{ServerState, metrics};

pub async fn run_tcp_listener(
    state: Arc<ServerState>,
    port: u16,
    proxy_name: String,
    group_id: String,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!(addr = %addr, proxy = %proxy_name, group = %group_id, "TCP listener started");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        state.tcp_params.apply(&stream)?;
        debug!(peer_addr = %peer_addr, "new TCP connection");

        let permit = match state.tcp_semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                warn!(peer_addr = %peer_addr, "TCP connection rejected: max connections reached");
                metrics::connection_rejected("tcp");
                continue;
            }
        };

        let state = state.clone();
        let proxy_name = proxy_name.clone();
        let group_id = group_id.clone();

        tokio::spawn(async move {
            let _permit = permit;
            metrics::tcp_connection_opened();
            let result = handle_tcp_connection(state, stream, proxy_name, group_id).await;
            if let Err(e) = &result {
                debug!(error = %e, "TCP connection error");
                metrics::request_completed("tcp", "error");
            } else {
                metrics::request_completed("tcp", "success");
            }
            metrics::tcp_connection_closed();
        });
    }
}

async fn handle_tcp_connection(
    state: Arc<ServerState>,
    stream: TcpStream,
    proxy_name: String,
    group_id: String,
) -> Result<()> {
    use tunnel_lib::detect_protocol_and_host;

    let peer_addr = stream.peer_addr()?;

    let mut buf = vec![0u8; state.proxy_buffer_params.peek_buf_size];
    let n = stream.peek(&mut buf).await?;
    let (protocol, host) = detect_protocol_and_host(&buf[..n]);

    debug!(protocol = %protocol, host = ?host, "detected protocol on tcp listener");

    let client_conn = state.registry.select_client_for_group(&group_id)
        .ok_or_else(|| anyhow::anyhow!("no client for group: {}", group_id))?;

    proxy::forward_to_client(
        &client_conn,
        tunnel_lib::RoutingInfo {
            proxy_name,
            src_addr: peer_addr.ip().to_string(),
            src_port: peer_addr.port(),
            protocol: protocol.to_string(),
            host,
        },
        stream,
    ).await
}
