use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn, debug, error};

use tunnel_lib::{
    MessageType, Login, LoginResp, ClientConfig,
    send_message, recv_message, recv_message_type,
};

use crate::{ServerState, tunnel_handler, metrics};

pub async fn run_quic_server(state: Arc<ServerState>) -> Result<()> {
    let addr = format!("0.0.0.0:{}", state.config.server.tunnel_port);

    let (certs, key) = tunnel_lib::infra::pki::generate_self_signed_cert()?;

    let mut crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    crypto.alpn_protocols = vec![b"tunnel-quic".to_vec()];

    let server_config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(crypto)?
    ));

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

async fn handle_quic_connection(
    state: Arc<ServerState>,
    incoming: quinn::Incoming,
) -> Result<()> {
    let conn = incoming.await?;
    let remote_addr = conn.remote_address();
    info!(addr = %remote_addr, "new QUIC connection");

    let (mut send, mut recv) = conn.accept_bi().await?;

    let msg_type = recv_message_type(&mut recv).await?;
    if msg_type != MessageType::Login {
        warn!(msg_type = ?msg_type, "expected Login message");
        return Ok(());
    }

    let login: Login = recv_message(&mut recv).await?;
    let group_id = login.group_id.clone().unwrap_or_else(|| "default".to_string());

    info!(
        client_id = %login.client_id,
        group_id = %group_id,
        "client login attempt"
    );

    // Validate authentication token
    if !state.config.validate_token(&group_id, &login.token) {
        warn!(
            client_id = %login.client_id,
            group_id = %group_id,
            "authentication failed: invalid token"
        );
        metrics::auth_failure(&group_id);
        let resp = LoginResp {
            success: false,
            error: Some("Invalid authentication token".to_string()),
            config: ClientConfig::default(),
        };
        send_message(&mut send, MessageType::LoginResp, &resp).await?;
        return Ok(());
    }

    metrics::auth_success(&group_id);

    // Check for duplicate ClientID and handle conflict
    if let Some(existing_conn) = state.registry.get_client_connection(&login.client_id) {
        warn!(
            client_id = %login.client_id,
            "duplicate client ID detected, closing old connection"
        );
        metrics::duplicate_client_closed();
        existing_conn.close(0u32.into(), b"duplicate client");
        state.registry.unregister(&login.client_id);
    }

    let client_config = state.config.to_client_config(&group_id)
        .unwrap_or_else(|| ClientConfig::default());

    let resp = LoginResp {
        success: true,
        error: None,
        config: client_config,
    };
    send_message(&mut send, MessageType::LoginResp, &resp).await?;

    info!(
        client_id = %login.client_id,
        group_id = %group_id,
        "client authenticated and registered"
    );

    state.registry.register(login.client_id.clone(), group_id.clone(), conn.clone());
    metrics::client_registered(&group_id);

    loop {
        tokio::select! {
            _ = conn.closed() => {
                info!(client_id = %login.client_id, "connection closed");
                break;
            }
            result = conn.accept_bi() => {
                match result {
                    Ok((send, recv)) => {
                        debug!("accepted reverse stream from client");
                        let config = state.config.clone(); // O(1) Arc clone — no deep copy
                        let egress_map = state.egress_map.clone();
                        tokio::spawn(async move {
                            if let Err(e) = tunnel_handler::handle_tunnel_stream(send, recv, config, egress_map).await {
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

    state.registry.unregister(&login.client_id);
    metrics::client_unregistered(&group_id);

    Ok(())
}
