use anyhow::{Context, Result};
use bytes::Bytes;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, warn};
use tunnel_lib::{
    decode_udp_datagram_envelope, encode_udp_datagram_envelope, UdpDatagramEnvelope, UdpSessionKey,
    MAX_DATAGRAM_BYTES,
};

const UDP_SESSION_IDLE_TIMEOUT_SECS: u64 = 30;
const UDP_DATAGRAM_WORKERS: usize = 4;
const UDP_DATAGRAM_QUEUE_PER_WORKER: usize = 256;
const MAX_UDP_SESSIONS_PER_CONNECTION: usize = 1024;
const MAX_UDP_SESSIONS_GLOBAL: usize = 16_384;
const MAX_UDP_QUEUED_ENVELOPES_GLOBAL: usize = 16_384;
const UDP_SESSION_OPERATION_TIMEOUT: Duration = Duration::from_secs(3);

fn global_udp_session_capacity() -> Arc<Semaphore> {
    static CAPACITY: OnceLock<Arc<Semaphore>> = OnceLock::new();
    CAPACITY
        .get_or_init(|| Arc::new(Semaphore::new(MAX_UDP_SESSIONS_GLOBAL)))
        .clone()
}

fn global_udp_queue_capacity() -> Arc<Semaphore> {
    static CAPACITY: OnceLock<Arc<Semaphore>> = OnceLock::new();
    CAPACITY
        .get_or_init(|| Arc::new(Semaphore::new(MAX_UDP_QUEUED_ENVELOPES_GLOBAL)))
        .clone()
}

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
    _connection_capacity: OwnedSemaphorePermit,
    _global_capacity: OwnedSemaphorePermit,
}

pub struct UdpDatagramDispatcher {
    manager: UdpSessionManager,
    queues: OnceLock<Vec<mpsc::Sender<QueuedUdpEnvelope>>>,
    global_queue_capacity: Arc<Semaphore>,
}

struct QueuedUdpEnvelope {
    envelope: UdpDatagramEnvelope,
    _global_capacity: OwnedSemaphorePermit,
}

fn remove_session_if_same(
    sessions: &DashMap<UdpSessionKey, Arc<UdpSession>>,
    key: &UdpSessionKey,
    expected: &Arc<UdpSession>,
) -> Option<Arc<UdpSession>> {
    sessions
        .remove_if(key, |_, current| Arc::ptr_eq(current, expected))
        .map(|(_, session)| session)
}

fn remove_session_if_idle(
    sessions: &DashMap<UdpSessionKey, Arc<UdpSession>>,
    key: &UdpSessionKey,
    now: u64,
) -> Option<Arc<UdpSession>> {
    sessions
        .remove_if(key, |_, session| {
            now.saturating_sub(session.last_activity.load(Ordering::Relaxed))
                > UDP_SESSION_IDLE_TIMEOUT_SECS
        })
        .map(|(_, session)| session)
}

impl UdpDatagramDispatcher {
    pub fn try_enqueue(&self, payload: Bytes) -> Result<bool> {
        let envelope = decode_udp_datagram_envelope(payload.as_ref())?;
        let global_capacity = match self.global_queue_capacity.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return Ok(false),
        };
        let queues = self
            .queues
            .get_or_init(|| self.manager.start_datagram_workers());
        let worker = stable_session_worker(&envelope.session, queues.len());
        Ok(queues[worker]
            .try_send(QueuedUdpEnvelope {
                envelope,
                _global_capacity: global_capacity,
            })
            .is_ok())
    }
}

fn stable_session_worker(session: &UdpSessionKey, workers: usize) -> usize {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    session.hash(&mut hasher);
    (hasher.finish() as usize) % workers.max(1)
}

#[derive(Clone)]
pub struct UdpSessionManager {
    sessions: Arc<DashMap<UdpSessionKey, Arc<UdpSession>>>,
    conn: quinn::Connection,
    state: Arc<crate::ServerState>,
    // Parent of every per-session token: one cancel stops all reply pumps.
    root_cancel: CancellationToken,
    tasks: TaskTracker,
    session_capacity: Arc<Semaphore>,
    global_session_capacity: Arc<Semaphore>,
    worker_abort_handles: Arc<Mutex<Vec<tokio::task::AbortHandle>>>,
}

impl UdpSessionManager {
    pub fn new(conn: quinn::Connection, state: Arc<crate::ServerState>) -> Self {
        let sessions = Arc::new(DashMap::<UdpSessionKey, Arc<UdpSession>>::new());
        let root_cancel = CancellationToken::new();
        let tasks = TaskTracker::new();

        Self {
            sessions,
            conn,
            state,
            root_cancel,
            tasks,
            session_capacity: Arc::new(Semaphore::new(MAX_UDP_SESSIONS_PER_CONNECTION)),
            global_session_capacity: global_udp_session_capacity(),
            worker_abort_handles: Arc::new(Mutex::new(Vec::with_capacity(UDP_DATAGRAM_WORKERS))),
        }
    }

    pub fn spawn_datagram_workers(&self) -> UdpDatagramDispatcher {
        UdpDatagramDispatcher {
            manager: self.clone(),
            queues: OnceLock::new(),
            global_queue_capacity: global_udp_queue_capacity(),
        }
    }

    fn start_datagram_workers(&self) -> Vec<mpsc::Sender<QueuedUdpEnvelope>> {
        self.spawn_eviction_loop();
        let mut queues = Vec::with_capacity(UDP_DATAGRAM_WORKERS);
        for _ in 0..UDP_DATAGRAM_WORKERS {
            let (tx, mut rx) = mpsc::channel::<QueuedUdpEnvelope>(UDP_DATAGRAM_QUEUE_PER_WORKER);
            let manager = self.clone();
            let cancel = self.root_cancel.clone();
            let handle = self.tasks.spawn(async move {
                let _tracked = tunnel_lib::track_resource(tunnel_lib::TrackedResource::UdpTask);
                loop {
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        envelope = rx.recv() => {
                            let Some(envelope) = envelope else {
                                break;
                            };
                            tokio::select! {
                                _ = cancel.cancelled() => break,
                                result = manager.forward_client_envelope(envelope.envelope) => {
                                    if let Err(error) = result {
                                        debug!(
                                            error = %error,
                                            "udp datagram worker dropped packet"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            });
            self.worker_abort_handles.lock().push(handle.abort_handle());
            queues.push(tx);
        }
        queues
    }

    fn spawn_eviction_loop(&self) {
        let conn = self.conn.clone();
        let sessions = self.sessions.clone();
        let cancel = self.root_cancel.clone();
        self.tasks.spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        debug!("udp session eviction loop stopping: manager shut down");
                        break;
                    }
                    _ = conn.closed() => {
                        debug!("udp session eviction loop stopping: connection closed");
                        break;
                    }
                    _ = interval.tick() => {
                        let now = current_time_secs();
                        let mut evicted = Vec::new();
                        for entry in sessions.iter() {
                            let last = entry.value().last_activity.load(Ordering::Relaxed);
                            if now.saturating_sub(last) > UDP_SESSION_IDLE_TIMEOUT_SECS {
                                evicted.push(entry.key().clone());
                            }
                        }
                        for key in evicted {
                            if let Some(session) = remove_session_if_idle(&sessions, &key, now) {
                                debug!(proxy_name = %key.proxy_name, client_addr = %key.client_addr, "evicting idle udp session");
                                session.cancel_token.cancel();
                            }
                        }
                    }
                }
            }
            for entry in sessions.iter() {
                entry.value().cancel_token.cancel();
            }
            sessions.clear();
        });
    }

    // Bounded wait: pumps exit on cancellation at their next poll, so the
    // timeout only guards against a pump wedged in a blocking state.
    pub async fn shutdown(&self) {
        self.root_cancel.cancel();
        self.tasks.close();
        if tokio::time::timeout(Duration::from_secs(2), self.tasks.wait())
            .await
            .is_err()
        {
            warn!("udp session tasks did not stop within shutdown timeout; aborting workers");
            for handle in self.worker_abort_handles.lock().iter() {
                handle.abort();
            }
            if tokio::time::timeout(Duration::from_secs(1), self.tasks.wait())
                .await
                .is_err()
            {
                warn!("udp session tasks did not finalize after worker abort");
            }
        }
    }

    fn evict_one_idle_session(&self) -> bool {
        let now = current_time_secs();
        let key = self
            .sessions
            .iter()
            .find(|entry| {
                now.saturating_sub(entry.value().last_activity.load(Ordering::Relaxed))
                    > UDP_SESSION_IDLE_TIMEOUT_SECS
            })
            .map(|entry| entry.key().clone());
        key.and_then(|key| {
            remove_session_if_idle(&self.sessions, &key, now).map(|session| {
                session.cancel_token.cancel();
                true
            })
        })
        .unwrap_or(false)
    }

    async fn forward_client_envelope(&self, envelope: UdpDatagramEnvelope) -> Result<()> {
        let session = self.get_or_create_session(&envelope.session).await?;
        session
            .last_activity
            .store(current_time_secs(), Ordering::Relaxed);
        tokio::time::timeout(
            UDP_SESSION_OPERATION_TIMEOUT,
            session.socket.send(&envelope.payload),
        )
        .await
        .map_err(|_| anyhow::anyhow!("UDP upstream send timed out"))??;
        Ok(())
    }

    async fn get_or_create_session(&self, key: &UdpSessionKey) -> Result<Arc<UdpSession>> {
        if let Some(session) = self.sessions.get(key) {
            return Ok(session.clone());
        }
        let generation = self
            .state
            .admit_runtime_generation()
            .context("server is not admitting new UDP sessions")?;
        let capacity = match self.session_capacity.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) if self.evict_one_idle_session() => self
                .session_capacity
                .clone()
                .try_acquire_owned()
                .map_err(|_| anyhow::anyhow!("UDP session capacity exhausted"))?,
            Err(_) => return Err(anyhow::anyhow!("UDP session capacity exhausted")),
        };
        let global_capacity = self
            .global_session_capacity
            .clone()
            .try_acquire_owned()
            .map_err(|_| anyhow::anyhow!("global UDP session capacity exhausted"))?;

        let upstream_addr = tokio::time::timeout(
            UDP_SESSION_OPERATION_TIMEOUT,
            generation
                .routing()
                .egress_map()
                .resolve_udp_target(&key.proxy_name),
        )
        .await
        .map_err(|_| anyhow::anyhow!("UDP upstream resolution timed out"))?
        .with_context(|| {
            format!(
                "runtime generation {} has no UDP target for {}",
                generation.sequence(),
                key.proxy_name
            )
        })?;
        let socket = Arc::new(
            tokio::time::timeout(UDP_SESSION_OPERATION_TIMEOUT, UdpSocket::bind("0.0.0.0:0"))
                .await
                .map_err(|_| anyhow::anyhow!("UDP socket bind timed out"))??,
        );
        tokio::time::timeout(UDP_SESSION_OPERATION_TIMEOUT, socket.connect(upstream_addr))
            .await
            .map_err(|_| anyhow::anyhow!("UDP upstream connect timed out"))??;

        let cancel_token = self.root_cancel.child_token();
        let last_activity = Arc::new(AtomicU64::new(current_time_secs()));

        use dashmap::mapref::entry::Entry;
        let session = match self.sessions.entry(key.clone()) {
            Entry::Vacant(vacant) => {
                let session = Arc::new(UdpSession {
                    socket,
                    last_activity: last_activity.clone(),
                    cancel_token: cancel_token.clone(),
                    _connection_capacity: capacity,
                    _global_capacity: global_capacity,
                });
                let value = vacant.insert(session).clone();

                let sessions = self.sessions.clone();
                let conn = self.conn.clone();
                let session_key = key.clone();
                let reply_socket = value.socket.clone();
                let owned_session = value.clone();
                let cancel_clone = cancel_token.clone();
                self.tasks.spawn(async move {
                    let _tracked =
                        tunnel_lib::track_resource(tunnel_lib::TrackedResource::UdpTask);
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
                    remove_session_if_same(&sessions, &session_key, &owned_session);
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_session(last_activity: u64) -> Arc<UdpSession> {
        let connection_capacity = Arc::new(Semaphore::new(1)).try_acquire_owned().unwrap();
        let global_capacity = Arc::new(Semaphore::new(1)).try_acquire_owned().unwrap();
        Arc::new(UdpSession {
            socket: Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap()),
            last_activity: Arc::new(AtomicU64::new(last_activity)),
            cancel_token: CancellationToken::new(),
            _connection_capacity: connection_capacity,
            _global_capacity: global_capacity,
        })
    }

    fn test_key() -> UdpSessionKey {
        UdpSessionKey {
            proxy_name: "dns".into(),
            client_addr: "127.0.0.1".to_string(),
            client_port: 53000,
        }
    }

    #[tokio::test]
    async fn old_reply_pump_cannot_remove_recreated_session() {
        let sessions = DashMap::new();
        let key = test_key();
        let old = test_session(1).await;
        let replacement = test_session(2).await;
        sessions.insert(key.clone(), replacement.clone());

        assert!(remove_session_if_same(&sessions, &key, &old).is_none());
        assert!(Arc::ptr_eq(
            sessions.get(&key).unwrap().value(),
            &replacement
        ));
    }

    #[tokio::test]
    async fn idle_eviction_rechecks_activity_before_remove() {
        let sessions = DashMap::new();
        let key = test_key();
        let session = test_session(1).await;
        sessions.insert(key.clone(), session.clone());
        session.last_activity.store(100, Ordering::Relaxed);

        assert!(remove_session_if_idle(&sessions, &key, 110).is_none());
        assert!(sessions.contains_key(&key));
    }
}
