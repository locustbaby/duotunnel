use crate::egress::EgressProxy;
use crate::ingress::handlers::udp_datagram::UdpSessionManager;
use crate::ingress::tunnel_handler;
use crate::runtime::metrics;
use crate::ServerState;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, warn};
use tunnel_lib::{
    negotiate_protocol, recv_message_bounded, recv_message_type, send_message, ClientId, Login,
    LoginResp, MessageType, MAX_LOGIN_BYTES, MIN_SUPPORTED_VERSION, PROTOCOL_VERSION,
};

// Per-connection drain window; kept shorter than the app-level 30s
// SHUTDOWN_DRAIN_TIMEOUT so the runtime backstop still fires after
// connections have closed.
const CONN_SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(15);
// Drain window plus margin for handlers still inside the login handshake.
const CONN_TASK_WAIT_TIMEOUT: Duration = Duration::from_secs(20);
// Refusals are driven by the peer, so they must not turn the accept loop into a
// log amplifier; the counter metric carries the exact rate.
const REFUSAL_LOG_INTERVAL: Duration = Duration::from_secs(1);

pub async fn run_quic_server(
    state: Arc<ServerState>,
    ready: Arc<AtomicBool>,
    shutdown: CancellationToken,
) -> Result<()> {
    let addr = state.tunnel_addr();
    let quic_params = state.quic_transport_params();
    let server_config = tunnel_lib::transport::quic::create_server_config_with(&quic_params)?;
    let udp_socket = tunnel_lib::build_udp_socket(addr, &quic_params)?;
    let endpoint = quinn::Endpoint::new(
        quinn::EndpointConfig::default(),
        Some(server_config),
        udp_socket,
        Arc::new(quinn::TokioRuntime),
    )?;
    // Admission budget for connections that have not yet authenticated. It
    // bounds concurrent pre-auth handlers (tasks, login streams, per-connection
    // buffers); authenticated connections release their permit and are governed
    // by the registry instead (D-9: slot table stays out of scope here).
    let unauth_budget = Arc::new(Semaphore::new(state.max_unauthenticated_connections()));
    info!(
        addr = %addr,
        udp_recv_buf_mb = quic_params.udp_recv_buf_bytes / (1024 * 1024),
        udp_send_buf_mb = quic_params.udp_send_buf_bytes / (1024 * 1024),
        max_unauthenticated_connections = state.max_unauthenticated_connections(),
        "QUIC server listening"
    );
    let conn_tasks = TaskTracker::new();
    let mut last_refusal_log: Option<tokio::time::Instant> = None;
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                info!("QUIC server stopping due to shutdown signal");
                break;
            }
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else {
                    break;
                };
                // Force a Retry round-trip before spending any budget on an
                // unvalidated source address. Without this, spoofed Initial
                // packets would each hold a permit for the whole handshake
                // window and could deterministically lock out every real
                // client. Costs honest clients one extra RTT per connect.
                if !incoming.remote_address_validated() {
                    if incoming.may_retry() {
                        if let Err(e) = incoming.retry() {
                            debug!(error = %e, "failed to send retry for unvalidated address");
                        }
                        continue;
                    }
                    debug!(addr = %incoming.remote_address(), "unvalidated address cannot be retried; ignoring");
                    incoming.ignore();
                    continue;
                }
                let Ok(permit) = unauth_budget.clone().try_acquire_owned() else {
                    // Refuse instead of queueing: quinn answers with
                    // CONNECTION_REFUSED before completing the handshake, so an
                    // over-budget flood costs no task, stream, or crypto state.
                    metrics::unauthenticated_connection_refused();
                    let now = tokio::time::Instant::now();
                    if last_refusal_log.is_none_or(|last| now.duration_since(last) >= REFUSAL_LOG_INTERVAL) {
                        last_refusal_log = Some(now);
                        warn!(addr = %incoming.remote_address(), "refusing QUIC connection: unauthenticated connection budget exhausted");
                    }
                    incoming.refuse();
                    continue;
                };
                let state = state.clone();
                let ready = ready.clone();
                let conn_shutdown = shutdown.clone();
                conn_tasks.spawn(async move {
                    metrics::quic_connection_opened();
                    if let Err(e) = handle_quic_connection(state, ready, incoming, permit, conn_shutdown).await {
                        error!(error = % e, "QUIC connection error");
                    }
                    metrics::quic_connection_closed();
                });
            }
        }
    }
    conn_tasks.close();
    if !conn_tasks.is_empty() {
        info!(
            connections = conn_tasks.len(),
            "waiting for QUIC connection tasks to finish"
        );
    }
    // Connection handlers drain in-flight streams and close their own
    // connections; endpoint.close() before that wait would abort them all.
    if tokio::time::timeout(CONN_TASK_WAIT_TIMEOUT, conn_tasks.wait())
        .await
        .is_err()
    {
        warn!(
            remaining = conn_tasks.len(),
            wait_timeout_secs = CONN_TASK_WAIT_TIMEOUT.as_secs(),
            "QUIC connection tasks did not finish in time; forcing endpoint close"
        );
    }
    endpoint.close(0u32.into(), b"server shutting down");
    Ok(())
}
// `unauth_permit` accounts this connection against the pre-auth budget; every
// failure path releases it by returning, and the success path drops it
// explicitly once authentication concludes. The whole pre-auth phase shares one
// deadline so a peer that stalls each step cannot hold a permit for a multiple
// of the configured login timeout.
async fn handle_quic_connection(
    state: Arc<ServerState>,
    ready: Arc<AtomicBool>,
    incoming: quinn::Incoming,
    unauth_permit: OwnedSemaphorePermit,
    shutdown: CancellationToken,
) -> Result<()> {
    let login_timeout = state.login_timeout();
    let pre_auth_deadline = tokio::time::Instant::now() + login_timeout;
    let conn = match tokio::time::timeout_at(pre_auth_deadline, incoming).await {
        Ok(result) => result?,
        Err(_elapsed) => {
            debug!("QUIC handshake did not complete within the login timeout");
            return Ok(());
        }
    };
    let remote_addr = conn.remote_address();
    info!(addr = % remote_addr, "new QUIC connection");
    let (mut send, mut recv) = match tokio::time::timeout_at(pre_auth_deadline, conn.accept_bi())
        .await
    {
        Ok(Ok(streams)) => streams,
        Ok(Err(e)) => return Err(e.into()),
        Err(_elapsed) => {
            warn!(
                addr = % remote_addr,
                "login handshake timed out waiting for login stream"
            );
            conn.close(0u32.into(), b"login timeout");
            return Ok(());
        }
    };
    let msg_type =
        match tokio::time::timeout_at(pre_auth_deadline, recv_message_type(&mut recv)).await {
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
                    &LoginResp::failure_retryable("login timeout"),
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
    let login: Login =
        match tokio::time::timeout_at(
            pre_auth_deadline,
            recv_message_bounded(&mut recv, MAX_LOGIN_BYTES),
        )
        .await
        {
            Ok(Ok(l)) => l,
            Ok(Err(e)) => return Err(e),
            Err(_elapsed) => {
                warn!(
                    addr = % remote_addr, "login handshake timed out waiting for login body"
                );
                if let Err(e) = send_message(
                    &mut send,
                    MessageType::LoginResp,
                    &LoginResp::failure_retryable("login timeout"),
                )
                .await
                {
                    debug!(addr = %remote_addr, error = %e, "send login body timeout response failed");
                }
                return Ok(());
            }
        };
    let Some(negotiated) = negotiate_protocol(login.protocol_version, login.capabilities) else {
        warn!(
            addr = %remote_addr,
            client_version = login.protocol_version,
            supported = format!("{}..={}", MIN_SUPPORTED_VERSION, PROTOCOL_VERSION),
            "rejecting login: incompatible protocol version"
        );
        // The supported range stays out of the response: ALPN already gates
        // generations, so only malformed or hostile peers reach this path and
        // there is no reason to hand them a version fingerprint.
        if let Err(e) = send_message(
            &mut send,
            MessageType::LoginResp,
            &LoginResp::failure("incompatible protocol version"),
        )
        .await
        {
            debug!(addr = %remote_addr, error = %e, "send version-mismatch response failed");
        }
        conn.close(0u32.into(), b"incompatible protocol version");
        return Ok(());
    };
    if !ready.load(Ordering::Acquire) {
        warn!(addr = %remote_addr, "server not ready for login yet");
        if let Err(e) = send_message(
            &mut send,
            MessageType::LoginResp,
            &LoginResp::failure_retryable("server not ready"),
        )
        .await
        {
            debug!(addr = %remote_addr, error = %e, "send not-ready response failed");
        }
        conn.close(0u32.into(), b"server not ready");
        return Ok(());
    }
    let auth_outcome = match tokio::time::timeout_at(
        pre_auth_deadline,
        state.auth_store().authenticate(&login.token),
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(_elapsed) => {
            // An unbounded auth store (SQL lock contention, stalled ctld) would
            // otherwise pin the permit indefinitely.
            warn!(addr = %remote_addr, "authentication timed out");
            metrics::auth_failure("unknown");
            if let Err(e) = send_message(
                &mut send,
                MessageType::LoginResp,
                &LoginResp::failure_retryable("authentication unavailable"),
            )
            .await
            {
                debug!(addr = %remote_addr, error = %e, "send auth timeout response failed");
            }
            return Ok(());
        }
    };
    let auth_result = match auth_outcome {
        Ok(result) => result,
        Err(e) => {
            warn!(addr = % remote_addr, error = % e, "authentication failed");
            metrics::auth_failure("unknown");
            // Never echo internal error details (DB errors, cache state) to an
            // unauthenticated peer; the reason stays in server logs only. The
            // retryable flag carries the one bit the client legitimately needs:
            // a backing-store fault must not be read as a rejected token, or a
            // transient database blip would permanently stop every client.
            let resp = match e {
                tunnel_store::AuthError::Internal(_) => {
                    LoginResp::failure_retryable("authentication unavailable")
                }
                _ => LoginResp::failure("authentication failed"),
            };
            send_message(&mut send, MessageType::LoginResp, &resp).await?;
            return Ok(());
        }
    };
    let client_group = auth_result.client_group;
    drop(unauth_permit);
    info!(
        addr = % remote_addr,
        client_group = % client_group,
        negotiated_version = negotiated.version,
        capabilities = negotiated.capabilities,
        "authenticated"
    );
    metrics::auth_success(&client_group);
    let client_config = state.client_config_for_group(&client_group);
    let conn_id = ClientId::from(uuid::Uuid::new_v4().to_string());
    if let Err(e) = state
        .registry()
        .register(conn_id.clone(), client_group.clone(), conn.clone(), negotiated)
        .await
    {
        warn!(conn_id = %conn_id, error = %e, "failed to register client connection");
        let _ = send_message(
            &mut send,
            MessageType::LoginResp,
            &LoginResp::failure_retryable("registration failed: slot table exhausted"),
        )
        .await;
        conn.close(0u32.into(), b"registration failed");
        return Err(anyhow::anyhow!("registration failed: {}", e));
    }
    if let Err(e) = send_message(
        &mut send,
        MessageType::LoginResp,
        &LoginResp::success(client_config, client_group.clone(), negotiated),
    )
    .await
    {
        state.registry().unregister(&conn_id);
        return Err(e);
    }
    metrics::client_registered(&client_group);
    let mut revocation_rx = state.revocation_tx().subscribe();
    let udp_sessions = UdpSessionManager::new(conn.clone(), state.egress_map());
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                // Stop taking new streams here, but let in-flight proxied
                // streams finish first (public listeners are cancelled on the
                // same token): QUIC CONNECTION_CLOSE aborts every open stream,
                // so the close must come after the drain, not before.
                let drained = tunnel_lib::wait_for_resource_drain(CONN_SHUTDOWN_DRAIN_TIMEOUT).await;
                info!(conn_id = %conn_id, drained, "closing connection: server shutting down");
                conn.close(0u32.into(), b"server shutting down");
                break;
            }
            _ = conn.closed() => {
                info!(conn_id = %conn_id, "connection closed");
                break;
            }
            result = conn.accept_bi() => {
                match result {
                    Ok((send, recv)) => {
                        debug!("accepted reverse stream from client");
                        let state = state.clone();
                        // Deliberately untracked: these tasks end when the
                        // connection closes (their QUIC streams error out),
                        // and per-stream tracking would tax the hot path.
                        tokio::task::spawn(async move {
                            let egress_map = state.egress_map();
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
            datagram_result = conn.read_datagram() => {
                match datagram_result {
                    Ok(payload) => {
                        if let Err(e) = udp_sessions.forward_client_datagram(payload).await {
                            debug!(conn_id = %conn_id, error = %e, "udp datagram forwarding error");
                        }
                    }
                    Err(e) => {
                        debug!(conn_id = %conn_id, error = %e, "read_datagram error");
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
                        match state.auth_store().authenticate(&login.token).await {
                            Ok(_) => {}
                            Err(e) => {
                                warn!(conn_id = %conn_id, error = %e, "token no longer valid after lag; closing connection");
                                conn.close(0u32.into(), b"token revoked");
                                break;
                            }
                        }
                    }
                    Err(RecvError::Closed) => {
                        debug!(conn_id = %conn_id, "revocation channel closed, tearing down connection");
                        break;
                    }
                }
            }
        }
    }
    udp_sessions.shutdown().await;
    state.registry().unregister(&conn_id);
    metrics::client_unregistered(&client_group);
    Ok(())
}
