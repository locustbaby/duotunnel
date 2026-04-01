use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use tunnel_lib::{
    recv_message, recv_message_type, send_message, Login, LoginResp, MessageType,
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

    let login_timeout = Duration::from_secs(state.config.server.login_timeout_secs);

    let msg_type = match tokio::time::timeout(login_timeout, recv_message_type(&mut recv)).await {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => return Err(e.into()),
        Err(_elapsed) => {
            warn!(addr = %remote_addr, "login handshake timed out waiting for message type");
            let _ = send_message(&mut send, MessageType::LoginResp, &LoginResp::failure("login timeout")).await;
            return Ok(());
        }
    };

    if msg_type != MessageType::Login {
        warn!(addr = %remote_addr, msg_type = ?msg_type, "expected Login message");
        let _ = send_message(&mut send, MessageType::LoginResp, &LoginResp::failure(format!("unexpected message type: {:?}", msg_type))).await;
        return Ok(());
    }

    let login: Login = match tokio::time::timeout(login_timeout, recv_message(&mut recv)).await {
        Ok(Ok(l)) => l,
        Ok(Err(e)) => return Err(e.into()),
        Err(_elapsed) => {
            warn!(addr = %remote_addr, "login handshake timed out waiting for login body");
            let _ = send_message(&mut send, MessageType::LoginResp, &LoginResp::failure("login timeout")).await;
            return Ok(());
        }
    };

    let auth_result = match state.auth_store.authenticate(&login.token).await {
        Ok(result) => result,
        Err(e) => {
            warn!(addr = %remote_addr, error = %e, "authentication failed");
            metrics::auth_failure("unknown");
            send_message(&mut send, MessageType::LoginResp, &LoginResp::failure(e.to_string())).await?;
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

    let conn_id = uuid::Uuid::new_v4().to_string();

    send_message(&mut send, MessageType::LoginResp, &LoginResp::success(client_config, client_name.clone())).await?;

    state.registry.register(conn_id.clone(), client_name.clone(), conn.clone());
    metrics::client_registered(&client_name);

    let mut revocation_rx = state.revocation_tx.subscribe();

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
            recv_result = revocation_rx.recv() => {
                use tokio::sync::broadcast::error::RecvError;
                match recv_result {
                    Ok(revoked_name) if revoked_name == client_name => {
                        warn!(conn_id = %conn_id, client_name = %client_name,
                              "closing connection: token revoked");
                        conn.close(0u32.into(), b"token revoked");
                        break;
                    }
                    Ok(_) => {
                        // Different client revoked — not our concern.
                    }
                    Err(RecvError::Lagged(n)) => {
                        // Missed n revocation messages; re-validate token against DB.
                        warn!(conn_id = %conn_id, skipped = n, "revocation channel lagged; re-validating token");
                        if state.auth_store.authenticate(&login.token).await.is_err() {
                            warn!(conn_id = %conn_id, "token no longer valid after lag; closing connection");
                            conn.close(0u32.into(), b"token revoked");
                            break;
                        }
                    }
                    Err(RecvError::Closed) => {
                        // Server is shutting down; let conn.closed() handle exit.
                    }
                }
            }
        }
    }

    state.registry.unregister(&conn_id);
    metrics::client_unregistered(&client_name);

    Ok(())
}
