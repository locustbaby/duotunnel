use crate::config::build_client_config_for_group;
use crate::egress::EgressProxy;
use crate::handlers::SEMAPHORE_WAIT_MS;
use crate::{metrics, tunnel_handler, ServerState};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use tunnel_lib::{
    config_hash, recv_message, recv_message_type, send_message, Login, LoginResp, MessageType,
    Ping, Pong,
};

/// Interval between application-layer pings sent by server to client.
const PING_INTERVAL: Duration = Duration::from_secs(60);
/// How long to wait for a Pong before closing the connection.
const PONG_TIMEOUT: Duration = Duration::from_secs(15);
pub async fn run_quic_server_on_endpoint(
    endpoint: quinn::Endpoint,
    state: Arc<ServerState>,
    cancel: CancellationToken,
) -> Result<()> {
    loop {
        let incoming = tokio::select! {
            _ = cancel.cancelled() => {
                endpoint.close(0u32.into(), b"server shutdown");
                break;
            }
            inc = endpoint.accept() => match inc {
                Some(inc) => inc,
                None => break,
            },
        };
        let state = state.clone();
        let permit = match tokio::time::timeout(
            Duration::from_millis(SEMAPHORE_WAIT_MS),
            state.quic_semaphore.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => permit,
            Ok(Err(_semaphore_closed)) => {
                break;
            }
            Err(_elapsed) => {
                warn!("QUIC connection rejected: max connections reached after wait");
                metrics::connection_rejected("quic");
                continue;
            }
        };
        tokio::spawn(async move {
            let _permit = permit;
            metrics::quic_connection_opened();
            if let Err(e) = handle_quic_connection(state, incoming).await {
                error!(error = % e, "QUIC connection error");
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
    info!(addr = % remote_addr, "new QUIC connection");
    let (mut send, mut recv) = conn.accept_bi().await?;
    let login_timeout = Duration::from_secs(state.config.server.login_timeout_secs);
    let msg_type = match tokio::time::timeout(login_timeout, recv_message_type(&mut recv)).await {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => return Err(e),
        Err(_elapsed) => {
            warn!(
                addr = % remote_addr,
                "login handshake timed out waiting for message type"
            );
            if let Err(e) = send_message(
                &mut send,
                MessageType::LoginResp,
                &LoginResp::failure("login timeout"),
            )
            .await
            {
                debug!(addr = %remote_addr, error = %e, "send login timeout response failed");
            }
            return Ok(());
        }
    };
    if msg_type != MessageType::Login {
        warn!(addr = % remote_addr, msg_type = ? msg_type, "expected Login message");
        if let Err(e) = send_message(
            &mut send,
            MessageType::LoginResp,
            &LoginResp::failure(format!("unexpected message type: {:?}", msg_type)),
        )
        .await
        {
            debug!(addr = %remote_addr, error = %e, "send unexpected-msg-type response failed");
        }
        return Ok(());
    }
    let login: Login = match tokio::time::timeout(login_timeout, recv_message(&mut recv)).await {
        Ok(Ok(l)) => l,
        Ok(Err(e)) => return Err(e),
        Err(_elapsed) => {
            warn!(
                addr = % remote_addr, "login handshake timed out waiting for login body"
            );
            if let Err(e) = send_message(
                &mut send,
                MessageType::LoginResp,
                &LoginResp::failure("login timeout"),
            )
            .await
            {
                debug!(addr = %remote_addr, error = %e, "send login body timeout response failed");
            }
            return Ok(());
        }
    };
    let auth_result = match state.auth_store.authenticate(&login.token).await {
        Ok(result) => result,
        Err(e) => {
            warn!(addr = % remote_addr, error = % e, "authentication failed");
            metrics::auth_failure("unknown");
            send_message(
                &mut send,
                MessageType::LoginResp,
                &LoginResp::failure(e.to_string()),
            )
            .await?;
            return Ok(());
        }
    };
    let client_group = auth_result.client_group;
    info!(addr = % remote_addr, client_group = % client_group, "authenticated");
    metrics::auth_success(&client_group);
    let client_config = {
        let routing = state.routing.load();
        build_client_config_for_group(&routing.tunnel_management, &client_group).unwrap_or_default()
    };
    let conn_id = uuid::Uuid::new_v4().to_string();
    send_message(
        &mut send,
        MessageType::LoginResp,
        &LoginResp::success(client_config, client_group.clone()),
    )
    .await?;

    state.registry.register(
        conn_id.clone(),
        client_group.clone(),
        conn.clone(),
    );
    metrics::client_registered(&client_group);

    // Spawn the control stream task: handles Ping/Pong and config sync.
    {
        let conn_clone = conn.clone();
        let conn_id_clone = conn_id.clone();
        let client_group_clone = client_group.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) =
                run_control_stream(conn_clone, state_clone, &conn_id_clone, &client_group_clone)
                    .await
            {
                debug!(
                    conn_id = %conn_id_clone,
                    error = %e,
                    "control stream ended"
                );
            }
        });
    }

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
                        let state = state.clone();
                        tokio::spawn(async move {
                            let egress_map = state.routing.load().egress_map.clone();
                            if let Err(e) = tunnel_handler::handle_tunnel_stream(send, recv, EgressProxy(egress_map)).await {
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
                    Ok(revoked_name) if revoked_name == client_group => {
                        warn!(conn_id = %conn_id, client_group = %client_group, "closing connection: token revoked");
                        conn.close(0u32.into(), b"token revoked");
                        break;
                    }
                    Ok(_) => {}
                    Err(RecvError::Lagged(n)) => {
                        warn!(conn_id = %conn_id, skipped = n, "revocation channel lagged; re-validating token");
                        match state.auth_store.authenticate(&login.token).await {
                            Ok(_) => {}
                            Err(e) => {
                                warn!(conn_id = %conn_id, error = %e, "token no longer valid after lag; closing connection");
                                conn.close(0u32.into(), b"token revoked");
                                break;
                            }
                        }
                    }
                    Err(RecvError::Closed) => {}
                }
            }
        }
    }
    state.registry.unregister(&conn_id);
    metrics::client_unregistered(&client_group);
    Ok(())
}

/// Manages the server→client control stream for a single connection.
///
/// Protocol (Ping/Pong carries config hash — no separate ConfigCheck/ConfigAck):
///   Every PING_INTERVAL the server sends:
///     Ping { seq, timestamp_ms, config_hash }   ← server's current hash for this group
///   Client replies:
///     Pong { seq, config_hash }                 ← client's currently-applied hash
///   If hashes differ → server sends ConfigPush { full config }
///   No Pong within PONG_TIMEOUT → server closes the connection
async fn run_control_stream(
    conn: quinn::Connection,
    state: Arc<ServerState>,
    conn_id: &str,
    client_group: &str,
) -> Result<()> {
    // Open a bidirectional stream: server writes, client replies.
    let (mut ctrl_send, mut ctrl_recv) = conn.open_bi().await?;

    // Send an initial Ping immediately so the client's accept_bi() unblocks.
    // QUIC only notifies the peer about a new stream when data arrives; without
    // this first send the client would wait until the 60-second timer fires.
    let server_config_hash = {
        let routing = state.routing.load();
        build_client_config_for_group(&routing.tunnel_management, client_group)
            .map(|c| config_hash(&c))
            .unwrap_or(0)
    };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let initial_ping = Ping { seq: 0, timestamp_ms: ts, config_hash: server_config_hash };
    send_message(&mut ctrl_send, MessageType::Ping, &initial_ping).await?;
    debug!(conn_id = %conn_id, server_config_hash, "sent initial Ping on control stream");

    let mut ping_ticker = tokio::time::interval(PING_INTERVAL);
    ping_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    ping_ticker.tick().await; // consume the immediate first tick

    let mut seq: u64 = 1; // start at 1 since initial ping used seq=0
    let mut pending_pong: Option<u64> = Some(0); // waiting for pong to initial ping

    loop {
        tokio::select! {
            // Incoming Pong from client.
            recv_result = recv_message_type(&mut ctrl_recv) => {
                match recv_result {
                    Ok(MessageType::Pong) => {
                        let pong: Pong = match recv_message(&mut ctrl_recv).await {
                            Ok(p) => p,
                            Err(e) => {
                                debug!(conn_id = %conn_id, error = %e, "failed to decode Pong");
                                break;
                            }
                        };
                        if pending_pong != Some(pong.seq) {
                            // stale or duplicate pong — ignore
                            continue;
                        }
                        pending_pong = None;
                        ping_ticker.reset_after(PING_INTERVAL);

                        // Compare config hashes; push full config if client is stale.
                        let server_hash = {
                            let routing = state.routing.load();
                            build_client_config_for_group(&routing.tunnel_management, client_group)
                                .map(|c| config_hash(&c))
                                .unwrap_or(0)
                        };
                        if pong.config_hash != server_hash {
                            let config = {
                                let routing = state.routing.load();
                                build_client_config_for_group(&routing.tunnel_management, client_group)
                                    .unwrap_or_default()
                            };
                            if let Err(e) = send_message(
                                &mut ctrl_send,
                                MessageType::ConfigPush,
                                &config,
                            )
                            .await
                            {
                                debug!(conn_id = %conn_id, error = %e, "failed to send ConfigPush");
                                break;
                            }
                            info!(
                                conn_id = %conn_id,
                                client_group = %client_group,
                                server_hash,
                                client_hash = pong.config_hash,
                                "sent ConfigPush (hash mismatch)"
                            );
                        } else {
                            debug!(conn_id = %conn_id, seq = pong.seq, "pong received, config up-to-date");
                        }
                    }
                    Ok(other) => {
                        debug!(conn_id = %conn_id, msg_type = ?other, "unexpected msg on control recv");
                    }
                    Err(e) => {
                        debug!(conn_id = %conn_id, error = %e, "control recv error");
                        break;
                    }
                }
            }
            // Periodic Ping — also embeds the server's current config hash.
            _ = ping_ticker.tick() => {
                if pending_pong.is_some() {
                    warn!(conn_id = %conn_id, "pong timeout, closing connection");
                    conn.close(0u32.into(), b"pong timeout");
                    break;
                }
                let server_config_hash = {
                    let routing = state.routing.load();
                    build_client_config_for_group(&routing.tunnel_management, client_group)
                        .map(|c| config_hash(&c))
                        .unwrap_or(0)
                };
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let ping = Ping { seq, timestamp_ms: ts, config_hash: server_config_hash };
                if let Err(e) = send_message(&mut ctrl_send, MessageType::Ping, &ping).await {
                    debug!(conn_id = %conn_id, error = %e, "failed to send Ping");
                    break;
                }
                debug!(conn_id = %conn_id, seq, server_config_hash, "sent Ping");
                pending_pong = Some(seq);
                seq = seq.wrapping_add(1);
                ping_ticker.reset_after(PONG_TIMEOUT);
            }
        }
    }
    Ok(())
}
