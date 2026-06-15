use crate::egress::ServerEgressMap;
use anyhow::Result;
use bytes::Bytes;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use tunnel_lib::{
    decode_udp_datagram_envelope, encode_udp_datagram_envelope, UdpDatagramEnvelope, UdpSessionKey,
    MAX_DATAGRAM_BYTES,
};

const UDP_SESSION_IDLE_TIMEOUT_SECS: u64 = 30;

fn current_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

struct UdpSession {
    socket: Arc<UdpSocket>,
    last_activity: Arc<AtomicU64>,
    cancel_token: CancellationToken,
}

#[derive(Clone)]
pub struct UdpSessionManager {
    sessions: Arc<DashMap<UdpSessionKey, Arc<UdpSession>>>,
    conn: quinn::Connection,
    egress_map: Arc<ServerEgressMap>,
}

impl UdpSessionManager {
    pub fn new(conn: quinn::Connection, egress_map: Arc<ServerEgressMap>) -> Self {
        let sessions = Arc::new(DashMap::<UdpSessionKey, Arc<UdpSession>>::new());
        let conn_clone = conn.clone();
        let sessions_clone = sessions.clone();

        // Evict idle sessions periodically to prevent resource leakage on inactive paths.
        // Evicted sessions will have their cancellation tokens triggered to break the reply pumps.
        crate::runtime::spawn_task(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                tokio::select! {
                    _ = conn_clone.closed() => {
                        debug!("udp session eviction loop stopping: connection closed");
                        break;
                    }
                    _ = interval.tick() => {
                        let now = current_time_secs();
                        let mut evicted = Vec::new();
                        for entry in sessions_clone.iter() {
                            let last = entry.value().last_activity.load(Ordering::Relaxed);
                            if now.saturating_sub(last) > UDP_SESSION_IDLE_TIMEOUT_SECS {
                                evicted.push(entry.key().clone());
                            }
                        }
                        for key in evicted {
                            debug!(proxy_name = %key.proxy_name, client_addr = %key.client_addr, "evicting idle udp session");
                            if let Some((_, session)) = sessions_clone.remove(&key) {
                                session.cancel_token.cancel();
                            }
                        }
                    }
                }
            }
        });

        Self {
            sessions,
            conn,
            egress_map,
        }
    }

    pub async fn forward_client_datagram(&self, payload: Bytes) -> Result<()> {
        let envelope = decode_udp_datagram_envelope(payload.as_ref())?;
        let session = self.get_or_create_session(&envelope.session).await?;
        session
            .last_activity
            .store(current_time_secs(), Ordering::Relaxed);
        session.socket.send(&envelope.payload).await?;
        Ok(())
    }

    async fn get_or_create_session(&self, key: &UdpSessionKey) -> Result<Arc<UdpSession>> {
        if let Some(session) = self.sessions.get(key) {
            return Ok(session.clone());
        }

        let upstream_addr = self.egress_map.resolve_udp_target(&key.proxy_name).await?;
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        socket.connect(upstream_addr).await?;

        let cancel_token = CancellationToken::new();
        let last_activity = Arc::new(AtomicU64::new(current_time_secs()));

        use dashmap::mapref::entry::Entry;
        let session = match self.sessions.entry(key.clone()) {
            Entry::Vacant(vacant) => {
                let session = Arc::new(UdpSession {
                    socket,
                    last_activity: last_activity.clone(),
                    cancel_token: cancel_token.clone(),
                });
                let value = vacant.insert(session).clone();

                let sessions = self.sessions.clone();
                let conn = self.conn.clone();
                let session_key = key.clone();
                let reply_socket = value.socket.clone();
                let cancel_clone = cancel_token.clone();
                crate::runtime::spawn_task(async move {
                    if let Err(e) = pump_udp_replies(
                        conn,
                        reply_socket,
                        session_key.clone(),
                        last_activity,
                        cancel_clone,
                    )
                    .await
                    {
                        debug!(proxy_name = %session_key.proxy_name, error = %e, "udp reply pump stopped");
                    }
                    sessions.remove(&session_key);
                });
                value
            }
            Entry::Occupied(occupied) => occupied.get().clone(),
        };

        Ok(session)
    }
}

async fn pump_udp_replies(
    conn: quinn::Connection,
    socket: Arc<UdpSocket>,
    session: UdpSessionKey,
    last_activity: Arc<AtomicU64>,
    cancel_token: CancellationToken,
) -> Result<()> {
    let mut buf = [0u8; MAX_DATAGRAM_BYTES];
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                break;
            }
            recv_res = socket.recv(&mut buf) => {
                let n = recv_res?;
                last_activity.store(current_time_secs(), Ordering::Relaxed);
                let envelope = UdpDatagramEnvelope {
                    session: session.clone(),
                    payload: buf[..n].to_vec(),
                };
                let encoded = match encode_udp_datagram_envelope(&envelope) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        warn!(proxy_name = %session.proxy_name, error = %e, "dropping oversized upstream udp datagram");
                        continue;
                    }
                };
                conn.send_datagram(Bytes::copy_from_slice(encoded.as_slice()))?;
            }
        }
    }
    Ok(())
}
