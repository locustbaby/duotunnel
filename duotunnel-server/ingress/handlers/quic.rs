use crate::egress::EgressProxy;
use crate::ingress::handlers::udp_datagram::UdpSessionManager;
use crate::ingress::tunnel_handler;
use crate::runtime::metrics;
use crate::ServerState;
use anyhow::Result;
use duotunnel_lib::{
    negotiate_protocol, recv_message_bounded, recv_message_type, send_message, ClientId, Login,
    LoginResp, MessageType, MAX_LOGIN_BYTES, MIN_SUPPORTED_VERSION, PROTOCOL_VERSION,
};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, warn};

// Per-connection drain window; kept shorter than the app-level 30s
// SHUTDOWN_DRAIN_TIMEOUT so the runtime backstop still fires after
// connections have closed.
// Drain window plus margin for handlers still inside the login handshake.
const CONN_TASK_WAIT_TIMEOUT: Duration = Duration::from_secs(20);
const CONN_STREAM_DRAIN_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_REVERSE_STREAMS_PER_CONNECTION: usize = 1000;
const MAX_REVERSE_STREAMS_GLOBAL: usize = 4096;
// Refusals are driven by the peer, so they must not turn the accept loop into a
// log amplifier; the counter metric carries the exact rate.
const REFUSAL_LOG_INTERVAL: Duration = Duration::from_secs(1);

fn global_reverse_stream_capacity() -> Arc<Semaphore> {
    static CAPACITY: OnceLock<Arc<Semaphore>> = OnceLock::new();
    CAPACITY
        .get_or_init(|| Arc::new(Semaphore::new(MAX_REVERSE_STREAMS_GLOBAL)))
        .clone()
}

pub async fn run_quic_server(state: Arc<ServerState>, shutdown: CancellationToken) -> Result<()> {
    let addr = state.tunnel_addr();
    let quic_params = state.quic_transport_params();
    let server_config = duotunnel_lib::transport::quic::create_server_config_with(&quic_params)?;
    let udp_socket = duotunnel_lib::build_udp_socket(addr, &quic_params)?;
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
    state.health().mark_quic_bound(true);
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
                let conn_shutdown = shutdown.clone();
                conn_tasks.spawn(async move {
                    metrics::quic_connection_opened();
                    if let Err(e) = handle_quic_connection(state, incoming, permit, conn_shutdown).await {
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
    state.health().mark_quic_bound(false);
    Ok(())
}
// `unauth_permit` accounts this connection against the pre-auth budget; every
// failure path releases it by returning, and the success path drops it
// explicitly once authentication concludes. The whole pre-auth phase shares one
// deadline so a peer that stalls each step cannot hold a permit for a multiple
// of the configured login timeout.
async fn handle_quic_connection(
    state: Arc<ServerState>,
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
    let (mut send, mut recv) =
        match tokio::time::timeout_at(pre_auth_deadline, conn.accept_bi()).await {
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
    let login: Login = match tokio::time::timeout_at(
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
    if !state.health().admits_security_sensitive_work() {
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
    let security_admission = match tokio::time::timeout_at(
        pre_auth_deadline,
        state.security_apply_gate().read(),
    )
    .await
    {
        Ok(guard) => guard,
        Err(_elapsed) => {
            warn!(addr = %remote_addr, "authentication admission timed out");
            let _ = send_message(
                &mut send,
                MessageType::LoginResp,
                &LoginResp::failure_retryable("authentication unavailable"),
            )
            .await;
            return Ok(());
        }
    };
    let auth_outcome =
        match tokio::time::timeout_at(pre_auth_deadline, state.authenticate_pinned(&login.token))
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
                duotunnel_lib::AuthError::Internal(_) => {
                    LoginResp::failure_retryable("authentication unavailable")
                }
                _ => LoginResp::failure("authentication failed"),
            };
            send_message(&mut send, MessageType::LoginResp, &resp).await?;
            return Ok(());
        }
    };
    let (auth_result, auth_generation) = auth_result;
    let client_group = auth_result.client_group;
    let token_hash = auth_result.token_hash;
    drop(unauth_permit);
    info!(
        addr = % remote_addr,
        client_group = % client_group,
        negotiated_version = negotiated.version,
        capabilities = negotiated.capabilities,
        "authenticated"
    );
    metrics::auth_success(&client_group);
    let client_config = state.client_config_for_generation(&auth_generation, &client_group);
    let conn_id = ClientId::from(uuid::Uuid::new_v4().to_string());
    if let Err(e) = state
        .registry()
        .register(
            conn_id.clone(),
            client_group.clone(),
            conn.clone(),
            negotiated,
            token_hash,
        )
        .await
    {
        warn!(conn_id = %conn_id, error = %e, "failed to register client connection");
        let _ = send_message(
            &mut send,
            MessageType::LoginResp,
            &LoginResp::failure_retryable("registration failed"),
        )
        .await;
        conn.close(0u32.into(), b"registration failed");
        return Err(anyhow::anyhow!("registration failed: {}", e));
    }
    drop(security_admission);
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
    let udp_sessions = UdpSessionManager::new(conn.clone(), state.clone());
    let udp_dispatch = udp_sessions.spawn_datagram_workers();
    let mut reverse_tasks = tokio::task::JoinSet::new();
    let reverse_stream_capacity = Arc::new(Semaphore::new(MAX_REVERSE_STREAMS_PER_CONNECTION));
    let global_reverse_stream_capacity = global_reverse_stream_capacity();
    let mut security_freshness_tick = tokio::time::interval(Duration::from_secs(5));
    security_freshness_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                info!(conn_id = %conn_id, "stopping connection: server shutting down");
                break;
            }
            _ = conn.closed() => {
                info!(conn_id = %conn_id, "connection closed");
                break;
            }
            _ = security_freshness_tick.tick() => {
                if !state.health().admits_security_sensitive_work() {
                    warn!(
                        conn_id = %conn_id,
                        client_group = %client_group,
                        "closing connection: control security freshness expired"
                    );
                    conn.close(0u32.into(), b"security freshness expired");
                    break;
                }
            }
            result = conn.accept_bi() => {
                match result {
                    Ok((mut send, mut recv)) => {
                        let Some(generation) = state.admit_runtime_generation() else {
                            metrics::connection_rejected_not_ready("quic_reverse");
                            let _ = send.reset(0u32.into());
                            let _ = recv.stop(0u32.into());
                            continue;
                        };
                        let Ok(global_permit) = global_reverse_stream_capacity
                            .clone()
                            .try_acquire_owned()
                        else {
                            metrics::reverse_stream_rejected("global_capacity");
                            let _ = send.reset(0u32.into());
                            let _ = recv.stop(0u32.into());
                            continue;
                        };
                        let Ok(connection_permit) = reverse_stream_capacity
                            .clone()
                            .try_acquire_owned()
                        else {
                            metrics::reverse_stream_rejected("connection_capacity");
                            let _ = send.reset(0u32.into());
                            let _ = recv.stop(0u32.into());
                            continue;
                        };
                        debug!("accepted reverse stream from client");
                        let egress_map = generation.routing().egress_map();
                        reverse_tasks.spawn(async move {
                            let _global_permit = global_permit;
                            let _connection_permit = connection_permit;
                            let _tracked = duotunnel_lib::track_resource(
                                duotunnel_lib::TrackedResource::ReverseStream,
                            );
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
            completed = reverse_tasks.join_next(), if !reverse_tasks.is_empty() => {
                if let Some(Err(error)) = completed {
                    debug!(conn_id = %conn_id, error = %error, "reverse stream task failed");
                }
            }
            datagram_result = conn.read_datagram() => {
                match datagram_result {
                    Ok(payload) => {
                        match udp_dispatch.try_enqueue(payload) {
                            Ok(true) => {}
                            Ok(false) => {
                                metrics::udp_datagram_dropped("queue_full");
                            }
                            Err(e) => {
                                metrics::udp_datagram_dropped("decode");
                                debug!(conn_id = %conn_id, error = %e, "invalid udp datagram");
                            }
                        }
                    }
                    Err(e) => {
                        debug!(conn_id = %conn_id, error = %e, "read_datagram error");
                        break;
                    }
                }
            }
        }
    }
    state.registry().unregister(&conn_id);
    udp_sessions.shutdown().await;
    let reverse_drained = tokio::time::timeout(CONN_STREAM_DRAIN_TIMEOUT, async {
        while reverse_tasks.join_next().await.is_some() {}
    })
    .await
    .is_ok();
    if !reverse_drained {
        warn!(
            conn_id = %conn_id,
            timeout_secs = CONN_STREAM_DRAIN_TIMEOUT.as_secs(),
            "reverse stream drain timed out; aborting remaining tasks"
        );
        reverse_tasks.abort_all();
        while reverse_tasks.join_next().await.is_some() {}
    }
    conn.close(0u32.into(), b"server shutting down");
    metrics::client_unregistered(&client_group);
    Ok(())
}
