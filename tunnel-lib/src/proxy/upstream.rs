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

pub struct UpstreamGroup {
    pub servers: Vec<String>,
    counter: AtomicUsize,
    health_namespace: Arc<str>,
    health: Arc<UpstreamHealthRegistry>,
}

#[derive(Default)]
pub struct UpstreamHealthRegistry {
    health_epoch: AtomicU64,
    unhealthy: Arc<RwLock<HashMap<String, HealthEjection>>>,
}

#[derive(Clone, Copy)]
struct HealthEjection {
    until: Instant,
    epoch: u64,
    probe_running: bool,
    probe_id: u64,
}

impl UpstreamHealthRegistry {
    pub fn mark_unhealthy(&self, namespace: &str, server: &str) {
        let key = backend_health_key(namespace, server);
        let epoch = self.health_epoch.fetch_add(1, Ordering::Relaxed) + 1;
        let mut map = self.unhealthy.write();
        if map.len() >= MAX_UPSTREAM_HEALTH_ENTRIES && !map.contains_key(&key) {
            let now = Instant::now();
            map.retain(|_, entry| entry.until > now);
            if map.len() >= MAX_UPSTREAM_HEALTH_ENTRIES {
                if let Some(oldest) = map
                    .iter()
                    .min_by_key(|(_, entry)| entry.until)
                    .map(|(existing_key, _)| existing_key.clone())
                {
                    map.remove(&oldest);
                }
            }
        }
        let existing_probe = map
            .get(&key)
            .filter(|entry| entry.probe_running)
            .map(|entry| entry.probe_id);
        let probe_id = existing_probe.unwrap_or(epoch);
        map.insert(
            key.clone(),
            HealthEjection {
                until: Instant::now() + Duration::from_secs(10),
                epoch,
                probe_running: true,
                probe_id,
            },
        );
        if existing_probe.is_some() {
            return;
        }
        drop(map);

        let probe_capacity = match global_health_probe_capacity().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                if let Some(entry) = self.unhealthy.write().get_mut(&key) {
                    if entry.probe_id == probe_id {
                        entry.probe_running = false;
                    }
                }
                return;
            }
        };
        let server_clone = server.to_string();
        let key_clone = key;
        let unhealthy_clone = self.unhealthy.clone();
        tokio::spawn(async move {
            let _probe_capacity = probe_capacity;
            let (_, connect_addr_str, _) = UpstreamScheme::from_address(&server_clone);
            let probe_timeout = Duration::from_secs(3);
            let probe_jitter = Duration::from_millis(probe_id % 500);
            for _ in 0..5 {
                tokio::time::sleep(Duration::from_secs(2) + probe_jitter).await;
                let Some(probe_epoch) = unhealthy_clone
                    .read()
                    .get(&key_clone)
                    .map(|entry| entry.epoch)
                else {
                    return;
                };
                if probe_upstream(&connect_addr_str, probe_timeout).await {
                    let mut map = unhealthy_clone.write();
                    if map
                        .get(&key_clone)
                        .is_some_and(|entry| entry.epoch == probe_epoch)
                    {
                        map.remove(&key_clone);
                        tracing::info!(server = %server_clone, "active health probe succeeded, backend restored to pool early");
                        return;
                    }
                }
            }
            let mut map = unhealthy_clone.write();
            if let Some(entry) = map.get_mut(&key_clone) {
                if entry.probe_id == probe_id {
                    entry.probe_running = false;
                }
            }
        });
    }

    pub fn mark_healthy(&self, namespace: &str, server: &str) {
        let mut map = self.unhealthy.write();
        map.remove(&backend_health_key(namespace, server));
    }

    fn is_healthy(&self, namespace: &str, server: &str) -> bool {
        let key = backend_health_key(namespace, server);
        let mut map = self.unhealthy.write();
        match map.get(&key) {
            Some(entry) if Instant::now() < entry.until => false,
            Some(_) => {
                map.remove(&key);
                true
            }
            None => true,
        }
    }
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
        let now = Instant::now();
        health
            .unhealthy
            .write()
            .retain(|_, entry| entry.until > now);
        Self {
            servers,
            counter: AtomicUsize::new(0),
            health_namespace,
            health,
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
        None
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

    #[tokio::test]
    async fn mark_unhealthy_and_healthy_updates_health_state() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string()]);

        group.mark_unhealthy("127.0.0.1:1");
        assert!(!group.is_healthy("127.0.0.1:1"));

        group.mark_healthy("127.0.0.1:1");
        assert!(group.is_healthy("127.0.0.1:1"));
    }

    #[tokio::test]
    async fn next_healthy_skips_unhealthy_server() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string(), "127.0.0.1:2".to_string()]);

        group.mark_unhealthy("127.0.0.1:1");

        for _ in 0..4 {
            assert_eq!(
                group.next_healthy().map(String::as_str),
                Some("127.0.0.1:2")
            );
        }
    }

    #[tokio::test]
    async fn next_healthy_returns_none_when_all_servers_are_ejected() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string(), "127.0.0.1:2".to_string()]);
        group.mark_unhealthy("127.0.0.1:1");
        group.mark_unhealthy("127.0.0.1:2");

        assert!(group.next_healthy().is_none());
    }

    #[test]
    fn expired_unhealthy_entry_is_selectable() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string()]);
        group.health.unhealthy.write().insert(
            backend_health_key("", "127.0.0.1:1"),
            HealthEjection {
                until: Instant::now() - Duration::from_secs(1),
                epoch: 1,
                probe_running: false,
                probe_id: 1,
            },
        );

        assert!(group.is_healthy("127.0.0.1:1"));
        assert!(!group
            .health
            .unhealthy
            .read()
            .contains_key(&backend_health_key("", "127.0.0.1:1")));
        assert_eq!(
            group.next_healthy().map(String::as_str),
            Some("127.0.0.1:1")
        );
    }

    #[tokio::test]
    async fn repeated_failure_refreshes_the_ejection_epoch() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string()]);
        group.mark_unhealthy("127.0.0.1:1");
        let first = *group
            .health
            .unhealthy
            .read()
            .get(&backend_health_key("", "127.0.0.1:1"))
            .unwrap();

        group.mark_unhealthy("127.0.0.1:1");
        let refreshed = *group
            .health
            .unhealthy
            .read()
            .get(&backend_health_key("", "127.0.0.1:1"))
            .unwrap();

        assert!(refreshed.epoch > first.epoch);
        assert!(refreshed.until >= first.until);
        assert!(refreshed.probe_running);
        assert_eq!(refreshed.probe_id, first.probe_id);
    }

    #[tokio::test]
    async fn recreated_ejection_gets_a_new_probe_owner() {
        let group = UpstreamGroup::new(vec!["127.0.0.1:1".to_string()]);
        let key = backend_health_key("", "127.0.0.1:1");
        group.mark_unhealthy("127.0.0.1:1");
        let first_probe_id = group.health.unhealthy.read().get(&key).unwrap().probe_id;

        group.mark_healthy("127.0.0.1:1");
        group.mark_unhealthy("127.0.0.1:1");
        let replacement_probe_id = group.health.unhealthy.read().get(&key).unwrap().probe_id;

        assert_ne!(replacement_probe_id, first_probe_id);
    }

    #[tokio::test]
    async fn shared_registry_preserves_ejection_across_group_generations() {
        let health = Arc::new(UpstreamHealthRegistry::default());
        let first = UpstreamGroup::with_scoped_health_registry(
            vec!["127.0.0.1:1".to_string()],
            Arc::<str>::from("group-a"),
            health.clone(),
        );
        first.mark_unhealthy("127.0.0.1:1");

        let replacement = UpstreamGroup::with_scoped_health_registry(
            vec!["127.0.0.1:1".to_string()],
            Arc::<str>::from("group-a"),
            health,
        );

        assert!(replacement.next_healthy().is_none());
    }

    #[tokio::test]
    async fn shared_registry_isolates_groups_with_the_same_address() {
        let health = Arc::new(UpstreamHealthRegistry::default());
        let first = UpstreamGroup::with_scoped_health_registry(
            vec!["127.0.0.1:1".to_string()],
            Arc::<str>::from("group-a"),
            health.clone(),
        );
        let second = UpstreamGroup::with_scoped_health_registry(
            vec!["127.0.0.1:1".to_string()],
            Arc::<str>::from("group-b"),
            health,
        );

        first.mark_unhealthy("127.0.0.1:1");

        assert_eq!(
            second.next_healthy().map(String::as_str),
            Some("127.0.0.1:1")
        );
    }
}
