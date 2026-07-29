use crate::proxy::tcp::UpstreamScheme;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, Weak};
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
    use std::sync::atomic::AtomicU32;
    static SPAWN_ONCE: std::sync::Once = std::sync::Once::new();
    static CACHED_DECISECONDS: AtomicU32 = AtomicU32::new(0);

    SPAWN_ONCE.call_once(|| {
        let _ = std::thread::Builder::new()
            .name("time-updater".to_string())
            .spawn(|| {
                let start = process_start_instant();
                loop {
                    let deciseconds = start.elapsed().as_millis().div_ceil(100);
                    CACHED_DECISECONDS.store(deciseconds as u32, Ordering::Release);
                    std::thread::sleep(Duration::from_millis(50));
                }
            });
    });

    let val = CACHED_DECISECONDS.load(Ordering::Acquire);
    if val == 0 {
        process_start_instant().elapsed().as_millis().div_ceil(100) as u32
    } else {
        val
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Healthy = 0,
    Ejected = 1,
}

impl HealthState {
    fn from_u8(v: u8) -> Self {
        match v & 1 {
            0 => HealthState::Healthy,
            1 => HealthState::Ejected,
            _ => HealthState::Healthy,
        }
    }
}

const EPOCH_MASK: u64 = u32::MAX as u64;
const EPOCH_SHIFT: u32 = 7;
const EJECT_TICKS_MASK: u32 = 0x00FF_FFFF;
const EJECT_TICKS_SHIFT: u32 = 39;
const EJECT_TICKS_HALF_RANGE: u32 = 0x0080_0000;
const _: () = assert!(EPOCH_SHIFT + 32 <= EJECT_TICKS_SHIFT);
const _: () = assert!(EJECT_TICKS_SHIFT + 24 <= 64);

fn pack_state(
    state: HealthState,
    consecutive_errors: u8,
    epoch: u64,
    eject_until_ticks: u32,
) -> u64 {
    let state_bits = (state as u64) & 1;
    let err_bits = ((consecutive_errors as u64) & 0x3F) << 1;
    let epoch_bits = (epoch & EPOCH_MASK) << EPOCH_SHIFT;
    let eject_bits = ((eject_until_ticks & EJECT_TICKS_MASK) as u64) << EJECT_TICKS_SHIFT;
    state_bits | err_bits | epoch_bits | eject_bits
}

fn unpack_state(packed: u64) -> (HealthState, u8, u64, u32) {
    let state = HealthState::from_u8((packed & 1) as u8);
    let consecutive_errors = ((packed >> 1) & 0x3F) as u8;
    let epoch = (packed >> EPOCH_SHIFT) & EPOCH_MASK;
    let eject_until_ticks = ((packed >> EJECT_TICKS_SHIFT) as u32) & EJECT_TICKS_MASK;
    (state, consecutive_errors, epoch, eject_until_ticks)
}

fn current_health_ticks() -> u32 {
    current_monotonic_deciseconds() & EJECT_TICKS_MASK
}

fn tick_deadline_reached(now: u32, deadline: u32) -> bool {
    (now.wrapping_sub(deadline) & EJECT_TICKS_MASK) < EJECT_TICKS_HALF_RANGE
}

#[inline]
fn tick_deadline(now: u32, delta: u32) -> u32 {
    now.wrapping_add(delta) & EJECT_TICKS_MASK
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthSelection {
    Healthy(u64),
    Ejected,
    ProbeLease(u64),
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

    pub fn is_healthy(&self) -> HealthSelection {
        self.is_healthy_at(current_health_ticks())
    }

    fn is_healthy_at(&self, now_ticks: u32) -> HealthSelection {
        let mut current = self.packed.load(Ordering::Acquire);
        loop {
            let (state, errs, epoch, eject_until) = unpack_state(current);
            match state {
                HealthState::Healthy => {
                    return HealthSelection::Healthy(epoch);
                }
                HealthState::Ejected => {
                    if tick_deadline_reached(now_ticks, eject_until) {
                        let new_epoch = (epoch + 1) & EPOCH_MASK;
                        let leased_until = tick_deadline(now_ticks, 50);
                        let next = pack_state(HealthState::Ejected, errs, new_epoch, leased_until);
                        match self.packed.compare_exchange_weak(
                            current,
                            next,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        ) {
                            Ok(_) => return HealthSelection::ProbeLease(new_epoch),
                            Err(actual) => current = actual,
                        }
                    } else {
                        return HealthSelection::Ejected;
                    }
                }
            }
        }
    }

    /// Epoch-guarded success outcome reporting.
    pub fn mark_success(&self, observed_epoch: u64) {
        let mut current = self.packed.load(Ordering::Acquire);
        loop {
            let (state, errs, epoch, _) = unpack_state(current);
            if epoch != observed_epoch {
                return;
            }
            if state == HealthState::Healthy && errs == 0 {
                return;
            }
            let new_epoch = (epoch + 1) & EPOCH_MASK;
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
    pub fn mark_failure(&self, observed_epoch: u64) -> Option<u64> {
        let now_ticks = current_health_ticks();
        let mut current = self.packed.load(Ordering::Acquire);
        loop {
            let (state, errs, epoch, _) = unpack_state(current);
            if epoch != observed_epoch {
                return None;
            }
            let new_errs = std::cmp::min(errs.saturating_add(1), 63);
            let new_epoch = (epoch + 1) & EPOCH_MASK;
            let (next_state, extend_ejection) = if new_errs >= 3 || state == HealthState::Ejected {
                (HealthState::Ejected, true)
            } else {
                (state, false)
            };

            let next = pack_state(
                next_state,
                new_errs,
                new_epoch,
                if extend_ejection {
                    tick_deadline(now_ticks, 100)
                } else {
                    0
                },
            );
            match self.packed.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    if next_state == HealthState::Ejected {
                        return Some(new_epoch);
                    }
                    return None;
                }
                Err(actual) => current = actual,
            }
        }
    }

    pub fn force_unhealthy(&self) {
        let now_ticks = current_health_ticks();
        let mut current = self.packed.load(Ordering::Acquire);
        loop {
            let (_, _, epoch, _) = unpack_state(current);
            let new_epoch = (epoch + 1) & EPOCH_MASK;
            let next = pack_state(
                HealthState::Ejected,
                3,
                new_epoch,
                tick_deadline(now_ticks, 100),
            );
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

    pub fn current_epoch(&self) -> u64 {
        let (_, _, epoch, _) = unpack_state(self.packed.load(Ordering::Relaxed));
        epoch
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
    pub observed_epoch: u64,
    pub is_panic_fallback: bool,
    pub registry: Option<Weak<UpstreamHealthRegistry>>,
}

impl BackendRef {
    pub fn address(&self) -> &str {
        &self.entry.raw_address
    }

    pub fn mark_success(&self) {
        if self.is_panic_fallback {
            return;
        }
        self.entry.state.mark_success(self.observed_epoch);
    }

    pub fn mark_failure(&self) -> Option<u64> {
        if self.is_panic_fallback {
            return None;
        }
        self.entry.state.mark_failure(self.observed_epoch)
    }

    pub fn record_failure(&self) {
        if let Some(probe_token) = self.mark_failure() {
            if let Some(reg) = self.registry.as_ref().and_then(Weak::upgrade) {
                reg.spawn_probe_by_token(self.entry.clone(), probe_token);
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct UpstreamHealthRegistry {
    entries: Arc<RwLock<HashMap<String, std::sync::Weak<BackendEntry>>>>,
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
            if let Some(weak_entry) = map.get(&key) {
                if let Some(entry) = weak_entry.upgrade() {
                    if config_fingerprint == 0 || entry.config_fingerprint == config_fingerprint {
                        return entry;
                    }
                }
            }
        }
        let mut map = self.entries.write();
        if let Some(weak_entry) = map.get(&key) {
            if let Some(entry) = weak_entry.upgrade() {
                if config_fingerprint == 0 || entry.config_fingerprint == config_fingerprint {
                    return entry;
                }
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
            if let Some(oldest) = map.keys().next().cloned() {
                map.remove(&oldest);
            }
        }
        map.insert(key, Arc::downgrade(&entry));
        entry
    }

    pub fn mark_unhealthy(&self, namespace: &str, server: &str) {
        let entry = self.get_or_create_entry(namespace, server, 0);
        match entry.state.is_healthy() {
            HealthSelection::Healthy(observed_epoch)
            | HealthSelection::ProbeLease(observed_epoch) => {
                if let Some(probe_token) = entry.state.mark_failure(observed_epoch) {
                    self.spawn_probe_by_token(entry, probe_token);
                }
            }
            HealthSelection::Ejected => {}
        }
    }

    pub fn mark_healthy(&self, namespace: &str, server: &str) {
        let entry = self.get_or_create_entry(namespace, server, 0);
        match entry.state.is_healthy() {
            HealthSelection::Healthy(observed_epoch)
            | HealthSelection::ProbeLease(observed_epoch) => {
                entry.state.mark_success(observed_epoch);
            }
            HealthSelection::Ejected => {}
        }
    }

    pub fn is_healthy(&self, namespace: &str, server: &str) -> bool {
        let entry = self.get_or_create_entry(namespace, server, 0);
        matches!(
            entry.state.is_healthy(),
            HealthSelection::Healthy(_) | HealthSelection::ProbeLease(_)
        )
    }

    pub fn spawn_probe_by_token(&self, entry: Arc<BackendEntry>, probe_token: u64) {
        let probe_capacity = match global_health_probe_capacity().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return,
        };
        let connect_addr_str = entry.connect_addr.clone();
        tokio::spawn(async move {
            let _probe_capacity = probe_capacity;
            let probe_timeout = Duration::from_secs(3);
            let probe_jitter = Duration::from_millis(probe_token % 500);
            let mut probe_token = probe_token;
            for _ in 0..5 {
                tokio::time::sleep(Duration::from_secs(2) + probe_jitter).await;
                let current_epoch = entry.state.current_epoch();
                if current_epoch != probe_token {
                    return;
                }

                let expected_lease_epoch = (probe_token + 1) & EPOCH_MASK;
                match entry.state.is_healthy() {
                    HealthSelection::ProbeLease(leased_epoch)
                        if leased_epoch == expected_lease_epoch =>
                    {
                        if probe_upstream(&connect_addr_str, probe_timeout).await {
                            entry.state.mark_success(leased_epoch);
                            tracing::info!(server = %entry.raw_address, "active health probe succeeded, backend restored");
                            return;
                        } else {
                            entry.state.mark_failure(leased_epoch);
                            probe_token = entry.state.current_epoch();
                        }
                    }
                    HealthSelection::ProbeLease(_) => return,
                    HealthSelection::Healthy(_) => {
                        return;
                    }
                    HealthSelection::Ejected => {
                        continue;
                    }
                }
            }
        });
    }
}

pub struct UpstreamGroup {
    pub servers: Vec<String>,
    pub entries: Vec<Arc<BackendEntry>>,
    counter: AtomicUsize,
    health: Arc<UpstreamHealthRegistry>,
    has_ejections: std::sync::atomic::AtomicBool,
}
impl UpstreamGroup {
    pub fn calculate_fingerprint(servers: &[String]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        servers.hash(&mut hasher);
        hasher.finish()
    }

    pub fn new(servers: Vec<String>) -> Self {
        let fingerprint = Self::calculate_fingerprint(&servers);
        Self::with_scoped_health_registry(
            servers,
            Arc::<str>::from(""),
            Arc::new(UpstreamHealthRegistry::default()),
            fingerprint,
        )
    }

    pub fn with_health_registry(servers: Vec<String>, health: Arc<UpstreamHealthRegistry>) -> Self {
        let fingerprint = Self::calculate_fingerprint(&servers);
        Self::with_scoped_health_registry(servers, Arc::<str>::from(""), health, fingerprint)
    }

    pub fn with_scoped_health_registry(
        servers: Vec<String>,
        health_namespace: Arc<str>,
        health: Arc<UpstreamHealthRegistry>,
        config_fingerprint: u64,
    ) -> Self {
        let entries: Vec<Arc<BackendEntry>> = servers
            .iter()
            .map(|s| health.get_or_create_entry(&health_namespace, s, config_fingerprint))
            .collect();
        let has_ejections = entries
            .iter()
            .any(|e| matches!(e.state.is_healthy(), HealthSelection::Ejected));
        Self {
            servers,
            entries,
            counter: AtomicUsize::new(0),
            health,
            has_ejections: std::sync::atomic::AtomicBool::new(has_ejections),
        }
    }

    pub fn next_healthy_ref(&self) -> Option<BackendRef> {
        if self.entries.is_empty() {
            return None;
        }
        let len = self.entries.len();
        let raw = self.counter.fetch_add(1, Ordering::Relaxed);
        if !self.has_ejections.load(Ordering::Relaxed) {
            let idx = if len.is_power_of_two() {
                raw & (len - 1)
            } else {
                raw % len
            };
            if let Some(entry) = self.entries.get(idx) {
                return Some(BackendRef {
                    entry: entry.clone(),
                    observed_epoch: entry.state.current_epoch(),
                    is_panic_fallback: false,
                    registry: Some(Arc::downgrade(&self.health)),
                });
            }
        }
        let now_ticks = current_health_ticks();
        for i in 0..len {
            let offset = raw.wrapping_add(i);
            let idx = if len.is_power_of_two() {
                offset & (len - 1)
            } else {
                offset % len
            };
            if let Some(entry) = self.entries.get(idx) {
                match entry.state.is_healthy_at(now_ticks) {
                    HealthSelection::Healthy(observed_epoch)
                    | HealthSelection::ProbeLease(observed_epoch) => {
                        return Some(BackendRef {
                            entry: entry.clone(),
                            observed_epoch,
                            is_panic_fallback: false,
                            registry: Some(Arc::downgrade(&self.health)),
                        });
                    }
                    HealthSelection::Ejected => {}
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
            let (is_panic_fallback, observed_epoch) = match entry.state.is_healthy_at(now_ticks) {
                HealthSelection::Healthy(e) | HealthSelection::ProbeLease(e) => (false, e),
                HealthSelection::Ejected => (true, entry.state.current_epoch()),
            };
            Some(BackendRef {
                entry: entry.clone(),
                observed_epoch,
                is_panic_fallback,
                registry: Some(Arc::downgrade(&self.health)),
            })
        } else {
            None
        }
    }

    pub fn sync_ejection_flag(&self) {
        let has_ejected = self
            .entries
            .iter()
            .any(|e| matches!(e.state.is_healthy(), HealthSelection::Ejected));
        self.has_ejections
            .store(has_ejected, std::sync::atomic::Ordering::Release);
    }

    pub fn mark_unhealthy(&self, server: &str) {
        if let Some(entry) = self
            .entries
            .iter()
            .find(|e| e.raw_address.as_ref() == server)
        {
            match entry.state.is_healthy() {
                HealthSelection::Healthy(observed_epoch)
                | HealthSelection::ProbeLease(observed_epoch) => {
                    if let Some(probe_token) = entry.state.mark_failure(observed_epoch) {
                        self.has_ejections
                            .store(true, std::sync::atomic::Ordering::Release);
                        self.health.spawn_probe_by_token(entry.clone(), probe_token);
                    }
                }
                HealthSelection::Ejected => {}
            }
        }
    }

    pub fn mark_healthy(&self, server: &str) {
        if let Some(entry) = self
            .entries
            .iter()
            .find(|e| e.raw_address.as_ref() == server)
        {
            match entry.state.is_healthy() {
                HealthSelection::Healthy(observed_epoch)
                | HealthSelection::ProbeLease(observed_epoch) => {
                    entry.state.mark_success(observed_epoch);
                    self.sync_ejection_flag();
                }
                HealthSelection::Ejected => {}
            }
        }
    }

    pub fn is_healthy(&self, server: &str) -> bool {
        if let Some(entry) = self
            .entries
            .iter()
            .find(|e| e.raw_address.as_ref() == server)
        {
            matches!(
                entry.state.is_healthy(),
                HealthSelection::Healthy(_) | HealthSelection::ProbeLease(_)
            )
        } else {
            false
        }
    }

    pub fn next_healthy(&self) -> Option<&String> {
        if self.servers.is_empty() {
            return None;
        }
        let len = self.servers.len();
        let raw = self.counter.fetch_add(1, Ordering::Relaxed);
        let now_ticks = current_health_ticks();
        for i in 0..len {
            let offset = raw.wrapping_add(i);
            let idx = if len.is_power_of_two() {
                offset & (len - 1)
            } else {
                offset % len
            };
            if let Some(entry) = self.entries.get(idx) {
                if matches!(
                    entry.state.is_healthy_at(now_ticks),
                    HealthSelection::Healthy(_) | HealthSelection::ProbeLease(_)
                ) {
                    return self.servers.get(idx);
                }
            }
        }
        self.servers.get(if len.is_power_of_two() {
            raw & (len - 1)
        } else {
            raw % len
        })
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
    format!("{}:{namespace}:{server}", namespace.len())
}

async fn probe_upstream(connect_addr: &str, timeout: Duration) -> bool {
    tokio::time::timeout(timeout, async {
        let Ok(addresses) = tokio::net::lookup_host(connect_addr).await else {
            return false;
        };
        for address in addresses.take(8) {
            if tokio::net::TcpStream::connect(address).await.is_ok() {
                return true;
            }
        }
        false
    })
    .await
    .unwrap_or(false)
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
                self.sync_ejection_flag();
            }
        }
    }

    #[tokio::test]
    async fn mark_unhealthy_and_healthy_updates_health_state() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string()]);

        group.force_unhealthy("127.0.0.1:1");
        assert!(!group.is_healthy("127.0.0.1:1"));

        if let Some(entry) = group
            .entries
            .iter()
            .find(|e| e.raw_address.as_ref() == "127.0.0.1:1")
        {
            let epoch = entry.state.current_epoch();
            entry.state.packed.store(
                pack_state(HealthState::Ejected, 3, epoch, 0),
                Ordering::Relaxed,
            );
        }

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
        let epoch1 = match state.is_healthy() {
            HealthSelection::Healthy(e) => e,
            _ => panic!("Expected healthy"),
        };

        state.force_unhealthy();
        assert!(matches!(state.is_healthy(), HealthSelection::Ejected));

        // Stale success from epoch1 should NOT restore health!
        state.mark_success(epoch1);
        assert!(matches!(state.is_healthy(), HealthSelection::Ejected));
    }

    #[test]
    fn stale_failure_does_not_increment_errors_or_eject() {
        let state = AtomicBackendState::new();
        let epoch1 = match state.is_healthy() {
            HealthSelection::Healthy(e) => e,
            _ => panic!("Expected healthy"),
        };
        state.force_unhealthy();

        let (_, errs, current_epoch, _) = unpack_state(state.packed.load(Ordering::Relaxed));
        assert_eq!(errs, 3);
        assert_ne!(current_epoch, epoch1);

        state.mark_failure(epoch1);
        let (_, errs_after, _, _) = unpack_state(state.packed.load(Ordering::Relaxed));
        assert_eq!(errs_after, 3);
    }

    #[tokio::test]
    async fn single_flight_probe_lease() {
        let state = Arc::new(AtomicBackendState::new());
        state.force_unhealthy();
        let epoch = state.current_epoch();
        state.packed.store(
            pack_state(HealthState::Ejected, 3, epoch, 0),
            Ordering::Relaxed,
        );

        let mut handles = vec![];
        for _ in 0..20 {
            let state_clone = state.clone();
            handles.push(tokio::spawn(async move { state_clone.is_healthy() }));
        }

        let mut leases = 0;
        let mut ejecteds = 0;
        for h in handles {
            match h.await.unwrap() {
                HealthSelection::ProbeLease(_) => leases += 1,
                HealthSelection::Ejected => ejecteds += 1,
                HealthSelection::Healthy(_) => panic!(),
            }
        }
        assert_eq!(leases, 1);
        assert_eq!(ejecteds, 19);
    }

    #[tokio::test]
    async fn panic_fallback_success_does_not_clear_ejection() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string()]);
        group.force_unhealthy("127.0.0.1:1");

        let backend_ref = group.next_healthy_ref().unwrap();
        assert!(backend_ref.is_panic_fallback);

        backend_ref.mark_success();
        assert!(!group.is_healthy("127.0.0.1:1"));
    }

    #[tokio::test]
    async fn background_probe_restores_health() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let addr_str = addr.to_string();

        let registry = UpstreamHealthRegistry::default();
        let entry = registry.get_or_create_entry("ns", &addr_str, 0);

        let (healthy_init, _) = match entry.state.is_healthy() {
            HealthSelection::Healthy(e) => (true, e),
            _ => panic!(),
        };
        assert!(healthy_init);

        entry
            .state
            .packed
            .store(pack_state(HealthState::Healthy, 2, 0, 0), Ordering::SeqCst);
        let probe_token = entry.state.mark_failure(0).unwrap();
        assert_eq!(probe_token, 1);

        let (_, errors, epoch, _) = unpack_state(entry.state.packed.load(Ordering::Acquire));
        entry.state.packed.store(
            pack_state(HealthState::Ejected, errors, epoch, 0),
            Ordering::Release,
        );

        registry.spawn_probe_by_token(entry.clone(), 1);

        let accept_handle = tokio::spawn(async move {
            let _ = listener.accept().await;
        });

        tokio::time::sleep(Duration::from_millis(3500)).await;

        tokio::time::timeout(Duration::from_secs(1), accept_handle)
            .await
            .expect("active health probe did not connect")
            .expect("active health probe task panicked");

        assert!(matches!(
            entry.state.is_healthy(),
            HealthSelection::Healthy(_)
        ));
    }
}
