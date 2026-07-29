use anyhow::{Context, Result};
use bytes::Bytes;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex as AsyncMutex, OwnedSemaphorePermit, Semaphore};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UdpRegId(u64);

const SESSION_CONNECTING: u8 = 0;
const SESSION_DRAINING: u8 = 1;
const SESSION_CONNECTED: u8 = 2;
const SESSION_FAILED: u8 = 3;

// The map entry survives the entire connect/drain transition; phase and gate
// together prevent a stale establisher from overwriting a newer session.
struct SessionState {
    phase: AtomicU8,
    pending_packets: Mutex<std::collections::VecDeque<Vec<u8>>>,
    session: Mutex<Option<Arc<UdpSession>>>,
    send_gate: AsyncMutex<()>,
}

struct SessionEntry {
    reg_id: UdpRegId,
    state: Arc<SessionState>,
    created_at: std::time::Instant,
    _connection_capacity: OwnedSemaphorePermit,
    _global_capacity: OwnedSemaphorePermit,
}

struct UdpSession {
    socket: Arc<UdpSocket>,
    last_activity: Arc<AtomicU64>,
    cancel_token: CancellationToken,
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
    sessions: &DashMap<UdpSessionKey, Arc<SessionEntry>>,
    key: &UdpSessionKey,
    expected_reg_id: UdpRegId,
) -> Option<Arc<SessionEntry>> {
    let removed = sessions
        .remove_if(key, |_, entry| entry.reg_id == expected_reg_id)
        .map(|(_, entry)| entry);
    if let Some(entry) = &removed {
        entry.state.phase.store(SESSION_FAILED, Ordering::Release);
    }
    removed
}

fn remove_session_if_idle(
    sessions: &DashMap<UdpSessionKey, Arc<SessionEntry>>,
    key: &UdpSessionKey,
    now: u64,
) -> Option<Arc<SessionEntry>> {
    let removed = sessions
        .remove_if(key, |_, entry| {
            let phase = entry.state.phase.load(Ordering::Acquire);
            if phase == SESSION_CONNECTED {
                entry
                    .state
                    .session
                    .lock()
                    .as_ref()
                    .map(|s| s.last_activity.load(Ordering::Relaxed))
                    .is_some_and(|last| now.saturating_sub(last) > UDP_SESSION_IDLE_TIMEOUT_SECS)
            } else {
                phase == SESSION_CONNECTING && entry.created_at.elapsed().as_secs() > 10
            }
        })
        .map(|(_, entry)| entry);
    if let Some(entry) = &removed {
        entry.state.phase.store(SESSION_FAILED, Ordering::Release);
    }
    removed
}

fn cancel_session_entry(entry: &SessionEntry) {
    if let Some(session) = entry.state.session.lock().as_ref() {
        session.cancel_token.cancel();
    }
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

struct CreateSessionReq {
    key: UdpSessionKey,
    reg_id: UdpRegId,
    entry: Arc<SessionEntry>,
}

#[derive(Clone)]
pub struct UdpSessionManager {
    sessions: Arc<DashMap<UdpSessionKey, Arc<SessionEntry>>>,
    conn: quinn::Connection,
    state: Arc<crate::ServerState>,
    // Parent of every per-session token: one cancel stops all reply pumps.
    root_cancel: CancellationToken,
    tasks: TaskTracker,
    session_capacity: Arc<Semaphore>,
    global_session_capacity: Arc<Semaphore>,
    worker_abort_handles: Arc<Mutex<Vec<tokio::task::AbortHandle>>>,
    create_tx: mpsc::Sender<CreateSessionReq>,
    next_udp_reg_id: Arc<std::sync::atomic::AtomicU64>,
}

impl UdpSessionManager {
    pub fn new(conn: quinn::Connection, state: Arc<crate::ServerState>) -> Self {
        let sessions = Arc::new(DashMap::<UdpSessionKey, Arc<SessionEntry>>::new());
        let root_cancel = CancellationToken::new();
        let tasks = TaskTracker::new();
        let (create_tx, mut create_rx) = mpsc::channel::<CreateSessionReq>(256);
        let next_udp_reg_id = Arc::new(std::sync::atomic::AtomicU64::new(1));
        let worker_abort_handles = Arc::new(Mutex::new(Vec::with_capacity(UDP_DATAGRAM_WORKERS)));
        let session_capacity = Arc::new(Semaphore::new(MAX_UDP_SESSIONS_PER_CONNECTION));
        let global_session_capacity = global_udp_session_capacity();

        let manager_tmp = Self {
            sessions: sessions.clone(),
            conn: conn.clone(),
            state: state.clone(),
            root_cancel: root_cancel.clone(),
            tasks: tasks.clone(),
            session_capacity,
            global_session_capacity,
            worker_abort_handles,
            create_tx,
            next_udp_reg_id,
        };

        let establisher_manager = manager_tmp.clone();
        let cancel_establisher = root_cancel.clone();
        tasks.spawn(async move {
            let _tracked = tunnel_lib::track_resource(tunnel_lib::TrackedResource::UdpTask);
            loop {
                tokio::select! {
                    _ = cancel_establisher.cancelled() => break,
                    req = create_rx.recv() => {
                        let Some(req) = req else { break; };
                        let manager = establisher_manager.clone();
                        let tasks = manager.tasks.clone();
                        tasks.spawn(async move {
                            if let Err(e) = manager.establish_session(req).await {
                                tracing::debug!("UDP session establish failed: {:?}", e);
                            }
                        });
                    }
                }
            }
        });

        manager_tmp
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
                            let should_evict = {
                                let entry = entry.value();
                                let phase = entry.state.phase.load(Ordering::Acquire);
                                if phase == SESSION_CONNECTED {
                                    entry
                                        .state
                                        .session
                                        .lock()
                                        .as_ref()
                                        .map(|s| s.last_activity.load(Ordering::Relaxed))
                                        .is_some_and(|last| now.saturating_sub(last) > UDP_SESSION_IDLE_TIMEOUT_SECS)
                                } else {
                                    phase == SESSION_CONNECTING && entry.created_at.elapsed().as_secs() > 10
                                }
                            };
                            if should_evict {
                                evicted.push(entry.key().clone());
                            }
                        }
                        for key in evicted {
                            if let Some(entry) = remove_session_if_idle(&sessions, &key, now) {
                                debug!(proxy_name = %key.proxy_name, client_addr = %key.client_addr, "evicting idle udp session");
                                cancel_session_entry(&entry);
                            }
                        }
                    }
                }
            }
            for entry in sessions.iter() {
                cancel_session_entry(entry.value());
            }
            sessions.clear();
        });
    }

    // Bounded wait: pumps exit on cancellation at their next poll, so the
    // timeout only guards against a pump wedged in a blocking state.
    pub async fn shutdown(&self) {
        self.root_cancel.cancel();
        for entry in self.sessions.iter() {
            entry.state.phase.store(SESSION_FAILED, Ordering::Release);
            cancel_session_entry(entry.value());
        }
        self.sessions.clear();
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
                let value = entry.value();
                let phase = value.state.phase.load(Ordering::Acquire);
                if phase == SESSION_CONNECTED {
                    value
                        .state
                        .session
                        .lock()
                        .as_ref()
                        .map(|s| s.last_activity.load(Ordering::Relaxed))
                        .is_some_and(|last| {
                            now.saturating_sub(last) > UDP_SESSION_IDLE_TIMEOUT_SECS
                        })
                } else {
                    phase == SESSION_CONNECTING && value.created_at.elapsed().as_secs() > 10
                }
            })
            .map(|entry| entry.key().clone());
        key.and_then(|key| {
            remove_session_if_idle(&self.sessions, &key, now).map(|entry| {
                cancel_session_entry(&entry);
                true
            })
        })
        .unwrap_or(false)
    }

    async fn forward_client_envelope(&self, envelope: UdpDatagramEnvelope) -> Result<()> {
        let key = &envelope.session;
        loop {
            let entry = self.sessions.get(key);
            match entry {
                Some(entry_ref) => {
                    let entry = entry_ref.value().clone();
                    drop(entry_ref);
                    return self.forward_session_entry(entry, envelope.payload).await;
                }
                None => {
                    use dashmap::mapref::entry::Entry;
                    match self.sessions.entry(key.clone()) {
                        Entry::Vacant(vacant) => {
                            let capacity = match self.session_capacity.clone().try_acquire_owned() {
                                Ok(permit) => permit,
                                Err(_) if self.evict_one_idle_session() => {
                                    self.session_capacity.clone().try_acquire_owned().map_err(
                                        |_| anyhow::anyhow!("UDP session capacity exhausted"),
                                    )?
                                }
                                Err(_) => {
                                    return Err(anyhow::anyhow!("UDP session capacity exhausted"))
                                }
                            };
                            let global_capacity = self
                                .global_session_capacity
                                .clone()
                                .try_acquire_owned()
                                .map_err(|_| {
                                    anyhow::anyhow!("global UDP session capacity exhausted")
                                })?;

                            let reg_id =
                                UdpRegId(self.next_udp_reg_id.fetch_add(1, Ordering::Relaxed));
                            let entry = Arc::new(SessionEntry {
                                reg_id,
                                state: Arc::new(SessionState {
                                    phase: AtomicU8::new(SESSION_CONNECTING),
                                    pending_packets: Mutex::new(std::collections::VecDeque::from(
                                        vec![envelope.payload.clone()],
                                    )),
                                    session: Mutex::new(None),
                                    send_gate: AsyncMutex::new(()),
                                }),
                                created_at: std::time::Instant::now(),
                                _connection_capacity: capacity,
                                _global_capacity: global_capacity,
                            });
                            vacant.insert(entry.clone());

                            let req = CreateSessionReq {
                                key: key.clone(),
                                reg_id,
                                entry,
                            };
                            if self.create_tx.try_send(req).is_err() {
                                remove_session_if_same(&self.sessions, key, reg_id);
                                return Err(anyhow::anyhow!("UDP session creation queue full"));
                            }
                            return Ok(());
                        }
                        Entry::Occupied(_) => {
                            continue;
                        }
                    }
                }
            }
        }
    }

    async fn forward_session_entry(
        &self,
        entry: Arc<SessionEntry>,
        payload: Vec<u8>,
    ) -> Result<()> {
        loop {
            match entry.state.phase.load(Ordering::Acquire) {
                SESSION_CONNECTED => {
                    let session = entry
                        .state
                        .session
                        .lock()
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("UDP session became unavailable"))?;
                    let _gate = entry.state.send_gate.lock().await;
                    if entry.state.phase.load(Ordering::Acquire) != SESSION_CONNECTED {
                        continue;
                    }
                    self.send_session_datagram(&session, &payload).await?;
                    return Ok(());
                }
                SESSION_CONNECTING | SESSION_DRAINING => {
                    let mut pending = entry.state.pending_packets.lock();
                    if entry.state.phase.load(Ordering::Acquire) == SESSION_CONNECTED {
                        drop(pending);
                        continue;
                    }
                    if entry.state.phase.load(Ordering::Acquire) == SESSION_FAILED {
                        return Err(anyhow::anyhow!("UDP session is shutting down"));
                    }
                    if pending.len() >= 10 {
                        pending.pop_front();
                        crate::runtime::metrics::udp_datagram_dropped("pending_overflow");
                        debug!(
                            reg_id = entry.reg_id.0,
                            "dropping oldest UDP pending packet"
                        );
                    }
                    pending.push_back(payload);
                    return Ok(());
                }
                _ => return Err(anyhow::anyhow!("UDP session is unavailable")),
            }
        }
    }

    async fn send_session_datagram(&self, session: &Arc<UdpSession>, payload: &[u8]) -> Result<()> {
        let result = tokio::select! {
            _ = self.root_cancel.cancelled() => {
                Err(anyhow::anyhow!("UDP session manager is shutting down"))
            }
            result = tokio::time::timeout(UDP_SESSION_OPERATION_TIMEOUT, session.socket.send(payload)) => {
                result
                    .map_err(|_| anyhow::anyhow!("UDP upstream send timed out"))?
                    .map_err(anyhow::Error::from)
            }
        };
        if result.is_err() {
            crate::runtime::metrics::udp_datagram_dropped("upstream_send");
        }
        result?;
        session
            .last_activity
            .store(current_time_secs(), Ordering::Relaxed);
        Ok(())
    }

    async fn establish_session(&self, req: CreateSessionReq) -> Result<()> {
        let reg_id = req.reg_id;
        let key = req.key.clone();
        let entry = req.entry.clone();
        let res = self.establish_session_inner(req).await;
        if res.is_err() {
            entry.state.phase.store(SESSION_FAILED, Ordering::Release);
            remove_session_if_same(&self.sessions, &key, reg_id);
        }
        res
    }

    async fn establish_session_inner(&self, req: CreateSessionReq) -> Result<()> {
        let key = &req.key;
        let reg_id = req.reg_id;
        let entry = req.entry.clone();

        if self.root_cancel.is_cancelled()
            || entry.state.phase.load(Ordering::Acquire) != SESSION_CONNECTING
        {
            return Err(anyhow::anyhow!("UDP session manager is shutting down"));
        }

        let generation = self
            .state
            .admit_runtime_generation()
            .context("server is not admitting new UDP sessions")?;

        let egress_map = generation.routing().egress_map();
        let upstream_addr = tokio::select! {
            _ = self.root_cancel.cancelled() => {
                return Err(anyhow::anyhow!("UDP session manager is shutting down"));
            }
            result = tokio::time::timeout(
                UDP_SESSION_OPERATION_TIMEOUT,
                egress_map.resolve_udp_target(&key.proxy_name),
            ) => result
                .map_err(|_| anyhow::anyhow!("UDP upstream resolution timed out"))?
                .with_context(|| format!(
                    "runtime generation {} has no UDP target for {}",
                    generation.sequence(), key.proxy_name
                ))?,
        };
        let socket = tokio::select! {
            _ = self.root_cancel.cancelled() => {
                return Err(anyhow::anyhow!("UDP session manager is shutting down"));
            }
            result = tokio::time::timeout(UDP_SESSION_OPERATION_TIMEOUT, UdpSocket::bind("0.0.0.0:0")) => {
                Arc::new(result.map_err(|_| anyhow::anyhow!("UDP socket bind timed out"))??)
            }
        };
        tokio::select! {
            _ = self.root_cancel.cancelled() => {
                return Err(anyhow::anyhow!("UDP session manager is shutting down"));
            }
            result = tokio::time::timeout(UDP_SESSION_OPERATION_TIMEOUT, socket.connect(upstream_addr)) => {
                result.map_err(|_| anyhow::anyhow!("UDP upstream connect timed out"))??;
            }
        }

        if self.root_cancel.is_cancelled()
            || entry
                .state
                .phase
                .compare_exchange(
                    SESSION_CONNECTING,
                    SESSION_DRAINING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_err()
        {
            return Err(anyhow::anyhow!("UDP session was evicted or cancelled"));
        }

        let cancel_token = self.root_cancel.child_token();
        let last_activity = Arc::new(AtomicU64::new(current_time_secs()));

        let session = Arc::new(UdpSession {
            socket,
            last_activity: last_activity.clone(),
            cancel_token: cancel_token.clone(),
        });

        {
            let mut slot = entry.state.session.lock();
            *slot = Some(session.clone());
        }
        let _gate = entry.state.send_gate.lock().await;
        loop {
            if self.root_cancel.is_cancelled() {
                return Err(anyhow::anyhow!("UDP session manager is shutting down"));
            }
            let packets = {
                let mut pending = entry.state.pending_packets.lock();
                std::mem::take(&mut *pending)
            };
            if packets.is_empty() {
                entry
                    .state
                    .phase
                    .store(SESSION_CONNECTED, Ordering::Release);
                break;
            }
            for payload in packets {
                self.send_session_datagram(&session, &payload)
                    .await
                    .context("draining UDP pending packet")?;
            }
        }

        let sessions = self.sessions.clone();
        let conn = self.conn.clone();
        let session_key = req.key.clone();
        let reply_socket = session.socket.clone();
        let cancel_clone = cancel_token.clone();
        self.tasks.spawn(async move {
            let _tracked = tunnel_lib::track_resource(tunnel_lib::TrackedResource::UdpTask);
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
            if let Some(entry) = remove_session_if_same(&sessions, &session_key, reg_id) {
                cancel_session_entry(&entry);
            }
        });
        Ok(())
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

    async fn test_entry(last_activity: u64, reg_id: UdpRegId) -> Arc<SessionEntry> {
        let connection_capacity = Arc::new(Semaphore::new(1)).try_acquire_owned().unwrap();
        let global_capacity = Arc::new(Semaphore::new(1)).try_acquire_owned().unwrap();
        let session = Arc::new(UdpSession {
            socket: Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap()),
            last_activity: Arc::new(AtomicU64::new(last_activity)),
            cancel_token: CancellationToken::new(),
        });
        let state = Arc::new(SessionState {
            phase: AtomicU8::new(SESSION_CONNECTED),
            pending_packets: Mutex::new(std::collections::VecDeque::new()),
            session: Mutex::new(Some(session)),
            send_gate: AsyncMutex::new(()),
        });
        Arc::new(SessionEntry {
            reg_id,
            state,
            created_at: std::time::Instant::now(),
            _connection_capacity: connection_capacity,
            _global_capacity: global_capacity,
        })
    }

    fn test_key() -> UdpSessionKey {
        UdpSessionKey {
            proxy_name: "dns".into(),
            client_addr: "127.0.0.1".parse().unwrap(),
            client_port: 53000,
        }
    }

    #[tokio::test]
    async fn old_reply_pump_cannot_remove_recreated_session() {
        let sessions = DashMap::new();
        let key = test_key();
        let old_reg_id = UdpRegId(1);
        let replacement_reg_id = UdpRegId(2);
        let replacement = test_entry(2, replacement_reg_id).await;
        let replacement_session = replacement.state.session.lock().clone().unwrap();
        sessions.insert(key.clone(), replacement);

        assert!(remove_session_if_same(&sessions, &key, old_reg_id).is_none());

        {
            let val = sessions.get(&key).unwrap();
            let session = val.value().state.session.lock().clone().unwrap();
            assert!(Arc::ptr_eq(&session, &replacement_session));
        }

        assert!(remove_session_if_same(&sessions, &key, replacement_reg_id).is_some());
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn idle_eviction_rechecks_activity_before_remove() {
        let sessions = DashMap::new();
        let key = test_key();
        let reg_id = UdpRegId(1);
        let entry = test_entry(1, reg_id).await;
        let session = entry.state.session.lock().clone().unwrap();
        sessions.insert(key.clone(), entry);
        session.last_activity.store(100, Ordering::Relaxed);

        assert!(remove_session_if_idle(&sessions, &key, 110).is_none());
        assert!(sessions.contains_key(&key));
    }
}
