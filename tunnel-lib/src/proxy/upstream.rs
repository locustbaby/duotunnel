use crate::proxy::tcp::UpstreamScheme;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

const MAX_UPSTREAM_HEALTH_ENTRIES: usize = 4096;
const MAX_ACTIVE_HEALTH_PROBES: usize = 128;

fn global_health_probe_capacity() -> Arc<Semaphore> {
    static CAPACITY: OnceLock<Arc<Semaphore>> = OnceLock::new();
    CAPACITY
        .get_or_init(|| Arc::new(Semaphore::new(MAX_ACTIVE_HEALTH_PROBES)))
        .clone()
}

fn process_start_instant() -> Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    *START.get_or_init(Instant::now)
}

pub fn current_monotonic_deciseconds() -> u32 {
    process_start_instant().elapsed().as_millis().div_ceil(100) as u32
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Healthy = 0,
    Ejected = 1,
    HalfOpen = 2,
}

impl HealthState {
    fn from_u8(v: u8) -> Self {
        match v & 0b11 {
            0 => HealthState::Healthy,
            1 => HealthState::Ejected,
            2 => HealthState::HalfOpen,
            _ => HealthState::Healthy,
        }
    }
}

fn pack_state(
    state: HealthState,
    consecutive_errors: u8,
    epoch: u32,
    eject_until_ticks: u32,
) -> u64 {
    let state_bits = (state as u64) & 0b11;
    let err_bits = ((consecutive_errors as u64) & 0xFF) << 2;
    let epoch_bits = ((epoch as u64) & 0x3F_FFFF) << 10;
    let ticks_bits = (eject_until_ticks as u64) << 32;
    state_bits | err_bits | epoch_bits | ticks_bits
}

fn unpack_state(packed: u64) -> (HealthState, u8, u32, u32) {
    let state = HealthState::from_u8((packed & 0b11) as u8);
    let consecutive_errors = ((packed >> 2) & 0xFF) as u8;
    let epoch = ((packed >> 10) & 0x3F_FFFF) as u32;
    let eject_until_ticks = (packed >> 32) as u32;
    (state, consecutive_errors, epoch, eject_until_ticks)
}

pub struct AtomicBackendState {
    packed: AtomicU64,
}

impl Default for AtomicBackendState {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicBackendState {
    pub fn new() -> Self {
        Self {
            packed: AtomicU64::new(pack_state(HealthState::Healthy, 0, 0, 0)),
        }
    }

    /// Fast lock-free state check. Returns `(is_selectable, observed_epoch)`.
    pub fn is_healthy(&self) -> (bool, u32) {
        let now_ticks = current_monotonic_deciseconds();
        let mut current = self.packed.load(Ordering::Acquire);
        loop {
            let (state, errs, epoch, eject_until) = unpack_state(current);
            match state {
                HealthState::Healthy => {
                    return (true, epoch);
                }
                HealthState::Ejected => {
                    if now_ticks >= eject_until {
                        let new_epoch = (epoch + 1) & 0x3F_FFFF;
                        let next = pack_state(HealthState::HalfOpen, errs, new_epoch, 0);
                        match self.packed.compare_exchange_weak(
                            current,
                            next,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        ) {
                            Ok(_) => return (true, new_epoch),
                            Err(actual) => current = actual,
                        }
                    } else {
                        return (false, epoch);
                    }
                }
                HealthState::HalfOpen => {
                    return (true, epoch);
                }
            }
        }
    }

    /// Epoch-guarded success outcome reporting.
    pub fn mark_success(&self, observed_epoch: u32) {
        let mut current = self.packed.load(Ordering::Acquire);
        loop {
            let (state, errs, epoch, _eject_until) = unpack_state(current);
            if epoch != observed_epoch {
                return;
            }
            if state == HealthState::Healthy && errs == 0 {
                return;
            }
            let new_epoch = (epoch + 1) & 0x3F_FFFF;
            let next = pack_state(HealthState::Healthy, 0, new_epoch, 0);
            match self.packed.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }

    /// Epoch-guarded failure outcome reporting. Returns `Some(probe_token)` if an ejection occurred.
    pub fn mark_failure(&self, observed_epoch: u32) -> Option<u64> {
        let now_ticks = current_monotonic_deciseconds();
        let mut current = self.packed.load(Ordering::Acquire);
        loop {
            let (state, errs, epoch, eject_until) = unpack_state(current);
            if epoch != observed_epoch && state == HealthState::Ejected {
                return None;
            }
            let new_errs = errs.saturating_add(1);
            let new_epoch = (epoch + 1) & 0x3F_FFFF;
            let (next_state, new_until) = if new_errs >= 3 || state == HealthState::HalfOpen {
                (HealthState::Ejected, now_ticks + 100)
            } else {
                (state, eject_until)
            };

            let next = pack_state(next_state, new_errs, new_epoch, new_until);
            match self.packed.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    if next_state == HealthState::Ejected && state != HealthState::Ejected {
                        return Some((new_epoch as u64) | ((now_ticks as u64) << 32));
                    }
                    return None;
                }
                Err(actual) => current = actual,
            }
        }
    }

    pub fn force_unhealthy(&self) {
        let now_ticks = current_monotonic_deciseconds();
        let mut current = self.packed.load(Ordering::Acquire);
        loop {
            let (_, _, epoch, _) = unpack_state(current);
            let new_epoch = (epoch + 1) & 0x3F_FFFF;
            let next = pack_state(HealthState::Ejected, 3, new_epoch, now_ticks + 100);
            match self.packed.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }
}

impl std::fmt::Debug for AtomicBackendState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (state, errs, epoch, eject_until) = unpack_state(self.packed.load(Ordering::Relaxed));
        f.debug_struct("AtomicBackendState")
            .field("state", &state)
            .field("consecutive_errors", &errs)
            .field("epoch", &epoch)
            .field("eject_until_ticks", &eject_until)
            .finish()
    }
}

impl std::fmt::Debug for BackendEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendEntry")
            .field("raw_address", &self.raw_address)
            .field("connect_addr", &self.connect_addr)
            .field("config_fingerprint", &self.config_fingerprint)
            .field("state", &self.state)
            .finish()
    }
}

pub struct BackendEntry {
    pub raw_address: Arc<str>,
    pub connect_addr: String,
    pub config_fingerprint: u64,
    pub state: AtomicBackendState,
}

#[derive(Debug, Clone)]
pub struct BackendRef {
    pub entry: Arc<BackendEntry>,
    pub observed_epoch: u32,
}

impl BackendRef {
    pub fn address(&self) -> &str {
        &self.entry.raw_address
    }

    pub fn mark_success(&self) {
        self.entry.state.mark_success(self.observed_epoch);
    }

    pub fn mark_failure(&self) -> Option<u64> {
        self.entry.state.mark_failure(self.observed_epoch)
    }
}

#[derive(Default)]
pub struct UpstreamHealthRegistry {
    entries: Arc<RwLock<HashMap<String, Arc<BackendEntry>>>>,
}

impl UpstreamHealthRegistry {
    pub fn get_or_create_entry(
        &self,
        namespace: &str,
        server: &str,
        config_fingerprint: u64,
    ) -> Arc<BackendEntry> {
        let key = backend_health_key(namespace, server);
        {
            let map = self.entries.read();
            if let Some(entry) = map.get(&key) {
                if entry.config_fingerprint == config_fingerprint {
                    return entry.clone();
                }
            }
        }
        let mut map = self.entries.write();
        if let Some(entry) = map.get(&key) {
            if entry.config_fingerprint == config_fingerprint {
                return entry.clone();
            }
        }
        let (_, connect_addr_str, _) = UpstreamScheme::from_address(server);
        let entry = Arc::new(BackendEntry {
            raw_address: Arc::from(server),
            connect_addr: connect_addr_str,
            config_fingerprint,
            state: AtomicBackendState::new(),
        });
        if map.len() >= MAX_UPSTREAM_HEALTH_ENTRIES {
            map.retain(|_, e| {
                let (state, errs, _, _) = unpack_state(e.state.packed.load(Ordering::Relaxed));
                state != HealthState::Healthy || errs > 0
            });
            if map.len() >= MAX_UPSTREAM_HEALTH_ENTRIES {
                if let Some(oldest) = map.keys().next().cloned() {
                    map.remove(&oldest);
                }
            }
        }
        map.insert(key, entry.clone());
        entry
    }

    pub fn mark_unhealthy(&self, namespace: &str, server: &str) {
        let entry = self.get_or_create_entry(namespace, server, 0);
        let (is_selectable, observed_epoch) = entry.state.is_healthy();
        let _ = is_selectable;
        if let Some(probe_token) = entry.state.mark_failure(observed_epoch) {
            self.spawn_probe_by_token(entry, probe_token);
        }
    }

    pub fn mark_healthy(&self, namespace: &str, server: &str) {
        let entry = self.get_or_create_entry(namespace, server, 0);
        let (_, observed_epoch) = entry.state.is_healthy();
        entry.state.mark_success(observed_epoch);
    }

    pub fn is_healthy(&self, namespace: &str, server: &str) -> bool {
        let entry = self.get_or_create_entry(namespace, server, 0);
        let (selectable, _) = entry.state.is_healthy();
        selectable
    }

    pub fn spawn_probe_by_token(&self, entry: Arc<BackendEntry>, probe_token: u64) {
        let probe_capacity = match global_health_probe_capacity().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return,
        };
        let connect_addr_str = entry.connect_addr.clone();
        let probe_epoch = (probe_token & 0x3F_FFFF) as u32;
        tokio::spawn(async move {
            let _probe_capacity = probe_capacity;
            let probe_timeout = Duration::from_secs(3);
            let probe_jitter = Duration::from_millis(probe_token % 500);
            for _ in 0..5 {
                tokio::time::sleep(Duration::from_secs(2) + probe_jitter).await;
                let (_, current_epoch) = entry.state.is_healthy();
                if current_epoch != probe_epoch {
                    return;
                }
                if probe_upstream(&connect_addr_str, probe_timeout).await {
                    entry.state.mark_success(probe_epoch);
                    tracing::info!(server = %entry.raw_address, "active health probe succeeded, backend restored");
                    return;
                }
            }
        });
    }
}

pub struct UpstreamGroup {
    pub servers: Vec<String>,
    pub entries: Vec<Arc<BackendEntry>>,
    counter: AtomicUsize,
    health_namespace: Arc<str>,
    health: Arc<UpstreamHealthRegistry>,
}

impl UpstreamGroup {
    pub fn new(servers: Vec<String>) -> Self {
        Self::with_scoped_health_registry(
            servers,
            Arc::<str>::from(""),
            Arc::new(UpstreamHealthRegistry::default()),
        )
    }

    pub fn with_health_registry(servers: Vec<String>, health: Arc<UpstreamHealthRegistry>) -> Self {
        Self::with_scoped_health_registry(servers, Arc::<str>::from(""), health)
    }

    pub fn with_scoped_health_registry(
        servers: Vec<String>,
        health_namespace: Arc<str>,
        health: Arc<UpstreamHealthRegistry>,
    ) -> Self {
        let entries = servers
            .iter()
            .map(|s| health.get_or_create_entry(&health_namespace, s, 0))
            .collect();
        Self {
            servers,
            entries,
            counter: AtomicUsize::new(0),
            health_namespace,
            health,
        }
    }

    pub fn next_healthy_ref(&self) -> Option<BackendRef> {
        if self.entries.is_empty() {
            return None;
        }
        let len = self.entries.len();
        let raw = self.counter.fetch_add(1, Ordering::Relaxed);
        for i in 0..len {
            let idx = if len.is_power_of_two() {
                (raw + i) & (len - 1)
            } else {
                (raw + i) % len
            };
            if let Some(entry) = self.entries.get(idx) {
                let (healthy, observed_epoch) = entry.state.is_healthy();
                if healthy {
                    return Some(BackendRef {
                        entry: entry.clone(),
                        observed_epoch,
                    });
                }
            }
        }
        // Bounded Panic Routing / Fail-Open fallback
        let idx = if len.is_power_of_two() {
            raw & (len - 1)
        } else {
            raw % len
        };
        if let Some(entry) = self.entries.get(idx) {
            let (_, observed_epoch) = entry.state.is_healthy();
            Some(BackendRef {
                entry: entry.clone(),
                observed_epoch,
            })
        } else {
            None
        }
    }

    pub fn mark_unhealthy(&self, server: &str) {
        self.health.mark_unhealthy(&self.health_namespace, server);
    }

    pub fn mark_healthy(&self, server: &str) {
        self.health.mark_healthy(&self.health_namespace, server);
    }

    pub fn is_healthy(&self, server: &str) -> bool {
        self.health.is_healthy(&self.health_namespace, server)
    }

    pub fn next_healthy(&self) -> Option<&String> {
        if self.servers.is_empty() {
            return None;
        }
        let len = self.servers.len();
        let raw = self.counter.fetch_add(1, Ordering::Relaxed);
        for i in 0..len {
            let idx = if len.is_power_of_two() {
                (raw + i) & (len - 1)
            } else {
                (raw + i) % len
            };
            if let Some(server) = self.servers.get(idx) {
                if self.is_healthy(server) {
                    return Some(server);
                }
            }
        }
        self.next()
    }

    pub fn next(&self) -> Option<&String> {
        if self.servers.is_empty() {
            return None;
        }
        let raw = self.counter.fetch_add(1, Ordering::Relaxed);
        let len = self.servers.len();
        let idx = if len.is_power_of_two() {
            raw & (len - 1)
        } else {
            raw % len
        };
        self.servers.get(idx)
    }

    pub fn first(&self) -> Option<&String> {
        self.servers.first()
    }
}

fn backend_health_key(namespace: &str, server: &str) -> String {
    format!("{}:{namespace}{server}", namespace.len())
}

async fn probe_upstream(connect_addr: &str, timeout: Duration) -> bool {
    let Ok(mut addresses) = tokio::net::lookup_host(connect_addr).await else {
        return false;
    };
    let Some(address) = addresses.next() else {
        return false;
    };
    tokio::time::timeout(timeout, tokio::net::TcpStream::connect(address))
        .await
        .is_ok_and(|result| result.is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    trait GroupExt {
        fn force_unhealthy(&self, addr: &str);
    }
    impl GroupExt for UpstreamGroup {
        fn force_unhealthy(&self, addr: &str) {
            if let Some(entry) = self.entries.iter().find(|e| e.raw_address.as_ref() == addr) {
                entry.state.force_unhealthy();
            } else {
                self.health.mark_unhealthy(&self.health_namespace, addr);
            }
        }
    }

    #[tokio::test]
    async fn mark_unhealthy_and_healthy_updates_health_state() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string()]);

        group.force_unhealthy("127.0.0.1:1");
        assert!(!group.is_healthy("127.0.0.1:1"));

        group.mark_healthy("127.0.0.1:1");
        assert!(group.is_healthy("127.0.0.1:1"));
    }

    #[tokio::test]
    async fn next_healthy_skips_unhealthy_server() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string(), "127.0.0.1:2".to_string()]);

        group.force_unhealthy("127.0.0.1:1");

        for _ in 0..4 {
            assert_eq!(
                group.next_healthy().map(String::as_str),
                Some("127.0.0.1:2")
            );
        }
    }

    #[tokio::test]
    async fn next_healthy_returns_a_server_when_all_servers_are_ejected_panic_routing() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string(), "127.0.0.1:2".to_string()]);
        group.force_unhealthy("127.0.0.1:1");
        group.force_unhealthy("127.0.0.1:2");

        assert!(group.next_healthy().is_some());
    }

    #[test]
    fn stale_epoch_success_does_not_clear_new_ejection() {
        let state = AtomicBackendState::new();
        let (healthy, epoch1) = state.is_healthy();
        assert!(healthy);

        state.force_unhealthy();
        let (healthy_after, _epoch2) = state.is_healthy();
        assert!(!healthy_after);

        // Stale success from epoch1 should NOT restore health!
        state.mark_success(epoch1);
        let (healthy_final, _) = state.is_healthy();
        assert!(!healthy_final);
    }
}
