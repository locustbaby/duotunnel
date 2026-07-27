use anyhow::anyhow;
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{
    recv_message, recv_message_type, send_message, Login, LoginResp, MessageType,
    NegotiatedProtocol, MIN_SUPPORTED_VERSION, PROTOCOL_VERSION, SUPPORTED_CAPABILITIES,
};

use crate::bootstrap::config::ClientConfigFile;
use crate::egress::udp_listener::{forward_incoming_datagram, UdpListenerRegistry};
use crate::ingress::app::LocalProxyMap;
use crate::ingress::handler::handle_work_stream;
use crate::plugins;
use crate::tunnel::conn_pool::{EntryConnPool, SessionActivity};
use crate::tunnel::supervisor::ConnectError;

// Kept shorter than the app-level 30s drain backstop in runtime::app so both
// waits stay bounded even when the local drain times out.
const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug)]
pub(crate) enum RunClientOutcome {
    Shutdown,
    SessionEnded(SessionReport),
}

#[derive(Debug)]
pub(crate) struct SessionReport {
    pub(crate) lifetime: Duration,
    pub(crate) completed_business: bool,
    pub(crate) error: ConnectError,
}

pub(crate) async fn run_client(
    config: &ClientConfigFile,
    endpoint: &quinn::Endpoint,
    shutdown: CancellationToken,
    entry_pool: Arc<EntryConnPool>,
    udp_registry: Arc<UdpListenerRegistry>,
) -> std::result::Result<RunClientOutcome, ConnectError> {
    // Nothing is in flight until the connection joins the pool, so racing this
    // phase against shutdown is both safe and necessary: connect walks every
    // resolved address and login spans several timeouts, so a sequential await
    // would keep the process alive for minutes after a stop signal.
    let (conn, resp, negotiated) = tokio::select! {
        biased;
        _ = shutdown.cancelled() => {
            info!("shutdown signal received before tunnel session was established");
            return Ok(RunClientOutcome::Shutdown);
        }
        established = establish_session(config, endpoint) => established?,
    };
    entry_pool.set_egress_rules(resp.config.egress_rules.clone());
    let lb = Arc::new(plugins::lb_round_robin::RoundRobinLb::new());
    let resolver = Arc::new(plugins::resolver_cached::CachedResolver::new());
    let proxy_map = Arc::new(
        LocalProxyMap::from_config(
            &resp.config,
            &tunnel_lib::HttpClientParams::from(&config.http_pool),
            lb,
            resolver,
        )
        .map_err(ConnectError::fatal)?,
    );
    let tcp_params = tunnel_lib::TcpParams::from(&config.tcp);

    let activity = Arc::new(SessionActivity::default());
    let commit = entry_pool
        .push(conn.clone(), negotiated, activity.clone())
        .await
        .map_err(ConnectError::fatal)?;
    info!(
        active_tunnels = commit.active_tunnels,
        changed = commit.changed,
        "tunnel committed to connection pool"
    );
    let session_started = Instant::now();
    let mut completed_business = false;
    let session_error = loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                info!("shutdown signal received, stopping tunnel stream accept");
                break None;
            }
            _ = activity.wait_for_business_completion(), if !completed_business => {
                completed_business = true;
                info!("tunnel session completed business traffic");
            }
            reason = conn.closed() => {
                warn!(reason = ?reason, "Connection closed by server");
                break Some(ConnectError::transient(anyhow!("connection closed by server: {:?}", reason)));
            }
            stream_result = conn.accept_bi() => {
                match stream_result {
                    Ok((send, recv)) => {
                        debug!("Accepted work stream from server");
                        let proxy_map = proxy_map.clone();
                        let tcp_params = tcp_params.clone();
                        let activity = activity.clone();
                        // Deliberately untracked: work-stream tasks end when
                        // the connection closes (their QUIC streams error
                        // out), and per-stream tracking would tax the hot
                        // path.
                        crate::runtime::spawn_task(async move {
                            match handle_work_stream(send, recv, proxy_map, tcp_params).await {
                                Ok(()) => {
                                    activity.mark_business_completed();
                                }
                                Err(e) => {
                                    debug!(error = %e, "work stream error");
                                }
                            }
                        });
                    }
                    Err(e) => {
                        warn!(error = %e, "Connection error");
                        break Some(ConnectError::transient(anyhow!("accept_bi failed: {}", e)));
                    }
                }
            }
            datagram_result = conn.read_datagram() => {
                match datagram_result {
                    Ok(payload) => {
                        if let Err(e) = forward_incoming_datagram(&udp_registry, payload).await {
                            debug!(error = %e, "udp datagram forward error");
                        } else if !completed_business {
                            activity.mark_business_completed();
                            completed_business = true;
                            info!("tunnel session completed business traffic");
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "datagram read error");
                        break Some(ConnectError::transient(anyhow!("read_datagram failed: {}", e)));
                    }
                }
            }
        }
    };
    completed_business |= activity.business_completed();
    let lifetime = session_started.elapsed();
    let commit = entry_pool
        .remove(&conn)
        .await
        .map_err(ConnectError::fatal)?;
    info!(
        active_tunnels = commit.active_tunnels,
        changed = commit.changed,
        session_lifetime_ms = lifetime.as_millis(),
        completed_business,
        "tunnel removed from connection pool"
    );
    if shutdown.is_cancelled() {
        // Entry listeners stop accepting on the same token; drain local
        // in-flight relays before closing, because CONNECTION_CLOSE aborts
        // every open stream — the close must come after the drain.
        let drained = tunnel_lib::wait_for_resource_drain(SHUTDOWN_DRAIN_TIMEOUT).await;
        info!(drained, "closing tunnel connection: client shutting down");
        conn.close(0u32.into(), b"client shutting down");
        return Ok(RunClientOutcome::Shutdown);
    }
    tokio::select! {
        _ = shutdown.cancelled() => {}
        _ = tokio::time::sleep(Duration::from_millis(config.reconnect.grace_ms)) => {}
    }
    match session_error {
        Some(error) => Ok(RunClientOutcome::SessionEnded(SessionReport {
            lifetime,
            completed_business,
            error,
        })),
        None => Ok(RunClientOutcome::Shutdown),
    }
}

/// Connects and completes the login handshake. Split out of `run_client` so the
/// caller can race the whole phase against shutdown: no stream is in flight
/// yet, so dropping this future loses nothing that needs draining.
async fn establish_session(
    config: &ClientConfigFile,
    endpoint: &quinn::Endpoint,
) -> std::result::Result<(quinn::Connection, LoginResp, NegotiatedProtocol), ConnectError> {
    let conn = connect_to_server(config, endpoint).await?;
    info!("Connected to server");
    let login_timeout = Duration::from_millis(config.reconnect.login_timeout_ms);
    let (mut send, mut recv) = tunnel_lib::timeout(login_timeout, conn.open_bi())
        .await
        .map_err(|_| ConnectError::transient(anyhow!("open_bi timed out")))?
        .map_err(|e| ConnectError::transient(anyhow!("failed to open login stream: {}", e)))?;
    let login = Login {
        token: config.auth_token.clone(),
        protocol_version: PROTOCOL_VERSION,
        capabilities: SUPPORTED_CAPABILITIES,
    };
    tunnel_lib::timeout(
        login_timeout,
        send_message(&mut send, MessageType::Login, &login),
    )
    .await
    .map_err(|_| ConnectError::transient(anyhow!("sending login timed out")))?
    .map_err(|e| ConnectError::transient(anyhow!("failed to send login: {}", e)))?;
    debug!("Login message sent");
    let msg_type = tunnel_lib::timeout(login_timeout, recv_message_type(&mut recv))
        .await
        .map_err(|_| ConnectError::transient(anyhow!("waiting login response timed out")))?
        .map_err(|e| {
            ConnectError::transient(anyhow!("failed to read login response type: {}", e))
        })?;
    if msg_type != MessageType::LoginResp {
        return Err(ConnectError::fatal(anyhow!(
            "protocol mismatch: expected LoginResp, got {:?}",
            msg_type
        )));
    }
    let resp: LoginResp = tunnel_lib::timeout(login_timeout, recv_message(&mut recv))
        .await
        .map_err(|_| ConnectError::transient(anyhow!("reading LoginResp payload timed out")))?
        .map_err(|e| ConnectError::transient(anyhow!("failed to decode LoginResp: {}", e)))?;
    if !resp.success {
        return Err(crate::tunnel::supervisor::classify_login_failure(
            resp.retryable,
            resp.error.as_deref(),
        ));
    }
    // The server negotiates min(its max, our reported version), so anything
    // outside our own supported range means a broken or hostile server —
    // retrying cannot help.
    if resp.negotiated_version < MIN_SUPPORTED_VERSION || resp.negotiated_version > PROTOCOL_VERSION
    {
        return Err(ConnectError::fatal(anyhow!(
            "protocol version negotiation failed: server negotiated v{}, client supports v{}..=v{}",
            resp.negotiated_version,
            MIN_SUPPORTED_VERSION,
            PROTOCOL_VERSION
        )));
    }
    let negotiated = NegotiatedProtocol {
        version: resp.negotiated_version,
        // Re-mask: never enable a capability the server echoes back but this
        // build did not advertise.
        capabilities: resp.capabilities & SUPPORTED_CAPABILITIES,
    };
    info!(
        client_group = %resp.client_group,
        upstreams = resp.config.upstreams.len(),
        negotiated_version = negotiated.version,
        capabilities = negotiated.capabilities,
        "Login successful, config received"
    );
    Ok((conn, resp, negotiated))
}

async fn connect_to_server(
    config: &ClientConfigFile,
    endpoint: &quinn::Endpoint,
) -> std::result::Result<quinn::Connection, ConnectError> {
    let addrs = resolve_server_addresses(config).await?;
    let connect_timeout = Duration::from_millis(config.reconnect.connect_timeout_ms);
    let sni = config.tls_server_name().to_string();
    let mut errors = Vec::new();
    for addr in addrs {
        info!(server_addr = %addr, sni = %sni, "Connecting to server");
        let connecting = endpoint
            .connect(addr, &sni)
            .map_err(|e| ConnectError::transient(anyhow!("connect setup failed: {}", e)))?;
        match tunnel_lib::timeout(connect_timeout, connecting).await {
            Ok(Ok(conn)) => return Ok(conn),
            Ok(Err(e)) => {
                errors.push(format!("{}: {}", addr, e));
            }
            Err(_) => {
                errors.push(format!("{}: connect timeout", addr));
            }
        }
    }
    Err(ConnectError::transient(anyhow!(
        "all connection attempts failed ({})",
        errors.join("; ")
    )))
}

async fn resolve_server_addresses(
    config: &ClientConfigFile,
) -> std::result::Result<Vec<SocketAddr>, ConnectError> {
    if let Ok(ip) = config.server_addr.parse::<IpAddr>() {
        return Ok(vec![SocketAddr::new(ip, config.server_port)]);
    }
    let resolve_timeout = Duration::from_millis(config.reconnect.resolve_timeout_ms);
    let host = config.server_addr.clone();
    let lookup = tokio::net::lookup_host((host.as_str(), config.server_port));
    let resolved = tunnel_lib::timeout(resolve_timeout, lookup)
        .await
        .map_err(|_| {
            ConnectError::transient(anyhow!(
                "DNS resolve timed out for {}:{}",
                host,
                config.server_port
            ))
        })?
        .map_err(|e| {
            ConnectError::transient(anyhow!(
                "DNS resolve failed for {}:{}: {}",
                host,
                config.server_port,
                e
            ))
        })?;
    let mut seen = HashSet::new();
    let mut addrs = Vec::new();
    for addr in resolved {
        if seen.insert(addr) {
            addrs.push(addr);
        }
    }
    if addrs.is_empty() {
        return Err(ConnectError::transient(anyhow!(
            "no resolved addresses for {}:{}",
            host,
            config.server_port
        )));
    }
    Ok(addrs)
}
