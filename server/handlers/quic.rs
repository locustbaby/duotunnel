use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use tunnel_lib::{
    recv_message, recv_message_type, send_message, ClientConfig, Login, LoginResp, MessageType,
};

use crate::config::build_client_config_for_group;
use crate::{metrics, tunnel_handler, ServerState};

pub async fn run_quic_server(state: Arc<ServerState>) -> Result<()> {
    let addr = format!("0.0.0.0:{}", state.config.server.tunnel_port);

    let quic_params = tunnel_lib::QuicTransportParams::from(&state.config.server.quic);
    let server_config = tunnel_lib::transport::quic::create_server_config_with(&quic_params)?;
    let endpoint = quinn::Endpoint::server(server_config, addr.parse()?)?;

    info!(addr = %addr, "QUIC server listening");

    while let Some(incoming) = endpoint.accept().await {
        let state = state.clone();
        let permit = match state.quic_semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                warn!("QUIC connection rejected: max connections reached");
                metrics::connection_rejected("quic");
                continue;
            }
        };
        tokio::spawn(async move {
            let _permit = permit;
            metrics::quic_connection_opened();
            if let Err(e) = handle_quic_connection(state, incoming).await {
                error!(error = %e, "QUIC connection error");
            }
            metrics::quic_connection_closed();
        });
    }

    Ok(())
}

async fn handle_quic_connection(state: Arc<ServerState>, incoming: quinn::Incoming) -> Result<()> {
    let conn = incoming.await?;
    let remote_addr = conn.remote_address();
    info!(addr = %remote_addr, "new QUIC connection");

    let (mut send, mut recv) = conn.accept_bi().await?;

    let msg_type = recv_message_type(&mut recv).await?;
    if msg_type != MessageType::Login {
        warn!(addr = %remote_addr, msg_type = ?msg_type, "expected Login message");
        return Ok(());
    }

    let login: Login = recv_message(&mut recv).await?;

    let auth_result = match state.auth_store.authenticate(&login.token).await {
        Ok(result) => result,
        Err(e) => {
            warn!(addr = %remote_addr, error = %e, "authentication failed");
            metrics::auth_failure("unknown");
            let resp = LoginResp {
                success: false,
                error: Some(e.to_string()),
                config: ClientConfig::default(),
                client_name: String::new(),
            };
            send_message(&mut send, MessageType::LoginResp, &resp).await?;
            return Ok(());
        }
    };

    let client_name = auth_result.client_name;
    info!(addr = %remote_addr, client_name = %client_name, "authenticated");
    metrics::auth_success(&client_name);

    let client_config = {
        let routing = state.routing.load();
        build_client_config_for_group(&routing.tunnel_management, &client_name)
            .unwrap_or_default()
    };

    let conn_id = format!(
        "{}-{}",
        client_name,
        conn.stable_id()
    );

    let resp = LoginResp {
        success: true,
        error: None,
        config: client_config,
        client_name: client_name.clone(),
    };
    send_message(&mut send, MessageType::LoginResp, &resp).await?;

    if let Some(old_conn) = state.registry.replace_or_register(
        conn_id.clone(),
        client_name.clone(),
        conn.clone(),
    ) {
        warn!(conn_id = %conn_id, "duplicate conn_id, closing old connection");
        metrics::duplicate_client_closed();
        old_conn.close(0u32.into(), b"duplicate client");
    }
    metrics::client_registered(&client_name);

    loop {
        tokio::select! {
            _ = conn.closed() => {
                info!(conn_id = %conn_id, "connection closed");
                break;
            }
            result = conn.accept_bi() => {
                match result {
                    Ok((send, recv)) => {
                        debug!("accepted reverse stream from client");
                        let egress_map = state.routing.load().egress_map.clone();
                        tokio::spawn(async move {
                            if let Err(e) = tunnel_handler::handle_tunnel_stream(send, recv, egress_map).await {
                                debug!(error = %e, "egress stream error");
                            }
                        });
                    }
                    Err(e) => {
                        debug!(error = %e, "accept_bi error");
                        break;
                    }
                }
            }
        }
    }

    state.registry.unregister(&conn_id);
    metrics::client_unregistered(&client_name);

    Ok(())
}
