use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);
const DNS_INFLIGHT_WAIT_TIMEOUT: Duration = Duration::from_secs(6);
const DNS_FAILURE_STALE_BACKOFF: Duration = Duration::from_secs(10);
const DNS_NEGATIVE_CACHE_TTL: Duration = Duration::from_secs(2);
const DEFAULT_MAX_DNS_CACHE_ENTRIES: usize = 4096;

pub struct DnsEntry {
    pub addrs: Vec<SocketAddr>,
    pub expires_at: Instant,
    next_index: AtomicUsize,
    negative_error: Option<String>,
}

impl Clone for DnsEntry {
    fn clone(&self) -> Self {
        Self {
            addrs: self.addrs.clone(),
            expires_at: self.expires_at,
            next_index: AtomicUsize::new(self.next_index.load(Ordering::Relaxed)),
            negative_error: self.negative_error.clone(),
        }
    }
}

impl DnsEntry {
    fn new(addrs: Vec<SocketAddr>, expires_at: Instant) -> Self {
        Self {
            addrs,
            expires_at,
            next_index: AtomicUsize::new(0),
            negative_error: None,
        }
    }

    fn new_negative(error: String, expires_at: Instant) -> Self {
        Self {
            addrs: Vec::new(),
            expires_at,
            next_index: AtomicUsize::new(0),
            negative_error: Some(error),
        }
    }

    fn next_addr(&self) -> Option<SocketAddr> {
        let len = self.addrs.len();
        if len == 0 {
            return None;
        }
        let idx = self.next_index.fetch_add(1, Ordering::Relaxed);
        self.addrs.get(idx % len).copied()
    }
}

type DnsKey = (String, u16);
type InflightTx = broadcast::Sender<Result<Vec<SocketAddr>, String>>;

pub struct EgressDnsCache {
    cache: Arc<DashMap<DnsKey, DnsEntry>>,
    inflight: Arc<DashMap<DnsKey, InflightTx>>,
    ttl: Duration,
    max_entries: usize,
    insertion_lock: Arc<parking_lot::Mutex<()>>,
}

impl EgressDnsCache {
    pub fn new(ttl: Duration) -> Self {
        Self::with_capacity(ttl, DEFAULT_MAX_DNS_CACHE_ENTRIES)
    }

    pub fn with_capacity(ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            inflight: Arc::new(DashMap::new()),
            ttl,
            max_entries: max_entries.max(1),
            insertion_lock: Arc::new(parking_lot::Mutex::new(())),
        }
    }

    fn insert_bounded(
        cache: &DashMap<DnsKey, DnsEntry>,
        insertion_lock: &parking_lot::Mutex<()>,
        max_entries: usize,
        key: DnsKey,
        entry: DnsEntry,
    ) {
        let _guard = insertion_lock.lock();
        if cache.len() >= max_entries && !cache.contains_key(&key) {
            // One scan at capacity (only on a cache miss, which already paid for a DNS
            // lookup): drop expired entries, then evict a negative entry before any
            // positive one so a flood of failing names cannot push out hot hosts.
            let now = Instant::now();
            let mut expired = Vec::new();
            let mut negative_victim: Option<(Instant, DnsKey)> = None;
            let mut positive_victim: Option<(Instant, DnsKey)> = None;
            for item in cache.iter() {
                let expires_at = item.value().expires_at;
                if expires_at <= now {
                    expired.push(item.key().clone());
                    continue;
                }
                let slot = if item.value().negative_error.is_some() {
                    &mut negative_victim
                } else {
                    &mut positive_victim
                };
                if slot.as_ref().is_none_or(|(oldest, _)| expires_at < *oldest) {
                    *slot = Some((expires_at, item.key().clone()));
                }
            }
            for key in &expired {
                cache.remove(key);
            }
            if cache.len() >= max_entries {
                if let Some((_, victim)) = negative_victim.or(positive_victim) {
                    cache.remove(&victim);
                }
            }
        }
        cache.insert(key, entry);
    }

    async fn wait_for_inflight(
        &self,
        mut rx: broadcast::Receiver<Result<Vec<SocketAddr>, String>>,
        key: &DnsKey,
        host: &str,
        port: u16,
    ) -> anyhow::Result<SocketAddr> {
        match tokio::time::timeout(DNS_INFLIGHT_WAIT_TIMEOUT, rx.recv()).await {
            Ok(Ok(Ok(resolved_vec))) => {
                if let Some(addr) = self.get_stale(key) {
                    return Ok(addr);
                }
                if let Some(&addr) = resolved_vec.first() {
                    return Ok(addr);
                }
                Err(anyhow::anyhow!("no resolved IP for {}:{}", host, port))
            }
            Ok(Ok(Err(err_msg))) => {
                if let Some(addr) = self.get_stale(key) {
                    tracing::warn!(
                        host = %host, port = port, error = %err_msg,
                        "DNS lookup failed, using stale cached IP"
                    );
                    return Ok(addr);
                }
                Err(anyhow::anyhow!("DNS lookup failed: {}", err_msg))
            }
            Ok(Err(recv_err)) => {
                if let Some(addr) = self.get_stale(key) {
                    tracing::warn!(
                        host = %host, port = port, error = %recv_err,
                        "In-flight DNS lookup channel failed, using stale cached IP"
                    );
                    return Ok(addr);
                }
                Err(anyhow::anyhow!(
                    "DNS lookup channel failed for {}:{}: {}",
                    host,
                    port,
                    recv_err
                ))
            }
            Err(_) => {
                if let Some(addr) = self.get_stale(key) {
                    tracing::warn!(
                        host = %host, port = port,
                        "In-flight DNS lookup timed out, using stale cached IP"
                    );
                    return Ok(addr);
                }
                Err(anyhow::anyhow!(
                    "DNS lookup did not complete for {}:{}",
                    host,
                    port
                ))
            }
        }
    }

    fn get_stale(&self, key: &(String, u16)) -> Option<SocketAddr> {
        self.cache.get(key).and_then(|entry| entry.next_addr())
    }

    pub async fn resolve(&self, host: &str, port: u16) -> anyhow::Result<SocketAddr> {
        let key = (host.to_string(), port);

        if let Some(entry) = self.cache.get(&key) {
            if Instant::now() < entry.expires_at {
                if let Some(addr) = entry.next_addr() {
                    return Ok(addr);
                }
                if let Some(ref err_msg) = entry.negative_error {
                    return Err(anyhow::anyhow!("DNS lookup failed: {}", err_msg));
                }
            }
        }

        loop {
            let rx = self.inflight.get(&key).map(|sender| sender.subscribe());

            if let Some(rx) = rx {
                return self.wait_for_inflight(rx, &key, host, port).await;
            }

            let (tx, _rx) = broadcast::channel(1);
            use dashmap::mapref::entry::Entry;
            match self.inflight.entry(key.clone()) {
                Entry::Occupied(_) => {
                    continue;
                }
                Entry::Vacant(ve) => {
                    ve.insert(tx.clone());
                }
            }

            let key_clone = key.clone();
            let rx = tx.subscribe();
            let host_clone = host.to_string();
            let cache = self.cache.clone();
            let inflight = self.inflight.clone();
            let ttl = self.ttl;
            let max_entries = self.max_entries;
            let insertion_lock = self.insertion_lock.clone();

            tokio::spawn(async move {
                let lookup_res = tokio::time::timeout(
                    DNS_LOOKUP_TIMEOUT,
                    tokio::net::lookup_host((host_clone.clone(), port)),
                )
                .await;

                let result = match lookup_res {
                    Ok(Ok(resolved)) => {
                        let resolved_vec: Vec<SocketAddr> = resolved.collect();
                        if resolved_vec.is_empty() {
                            Err("lookup returned empty address list".to_string())
                        } else {
                            Ok(resolved_vec)
                        }
                    }
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(_) => Err("DNS resolution timed out".to_string()),
                };

                match &result {
                    Ok(resolved_vec) => {
                        Self::insert_bounded(
                            &cache,
                            &insertion_lock,
                            max_entries,
                            key_clone.clone(),
                            DnsEntry::new(resolved_vec.clone(), Instant::now() + ttl),
                        );
                    }
                    Err(err_msg) => {
                        let mut updated_stale = false;
                        if let Some(mut entry) = cache.get_mut(&key_clone) {
                            if !entry.addrs.is_empty() {
                                entry.expires_at = Instant::now() + DNS_FAILURE_STALE_BACKOFF;
                                updated_stale = true;
                            }
                        }
                        if !updated_stale {
                            Self::insert_bounded(
                                &cache,
                                &insertion_lock,
                                max_entries,
                                key_clone.clone(),
                                DnsEntry::new_negative(
                                    err_msg.clone(),
                                    Instant::now() + DNS_NEGATIVE_CACHE_TTL,
                                ),
                            );
                        }
                    }
                }

                let _ = tx.send(result);
                inflight.remove(&key_clone);
            });

            return self.wait_for_inflight(rx, &key, host, port).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_dns_cache_expiry() {
        let cache = EgressDnsCache::new(Duration::from_millis(50));
        let addr = cache.resolve("localhost", 80).await.unwrap();
        assert!(addr.ip().is_loopback());

        // Wait for it to expire
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Resolve again
        let addr2 = cache.resolve("localhost", 80).await.unwrap();
        assert_eq!(addr.port(), addr2.port());
    }

    #[tokio::test]
    async fn test_dns_single_flight() {
        let cache = Arc::new(EgressDnsCache::new(Duration::from_secs(10)));
        let mut tasks = Vec::new();
        for _ in 0..10 {
            let cache_clone = cache.clone();
            tasks.push(tokio::spawn(async move {
                cache_clone.resolve("localhost", 80).await
            }));
        }

        for task in tasks {
            let res = task.await.unwrap();
            assert!(res.is_ok());
            assert!(res.unwrap().ip().is_loopback());
        }
    }

    #[test]
    fn test_dns_entry_round_robin() {
        let entry = DnsEntry::new(
            vec![
                "127.0.0.1:80".parse().unwrap(),
                "127.0.0.2:80".parse().unwrap(),
            ],
            Instant::now() + Duration::from_secs(10),
        );

        assert_eq!(entry.next_addr().unwrap().ip().to_string(), "127.0.0.1");
        assert_eq!(entry.next_addr().unwrap().ip().to_string(), "127.0.0.2");
        assert_eq!(entry.next_addr().unwrap().ip().to_string(), "127.0.0.1");
    }

    #[tokio::test]
    async fn test_dns_negative_cache_and_capacity_bound() {
        let cache = EgressDnsCache::with_capacity(Duration::from_secs(10), 2);
        EgressDnsCache::insert_bounded(
            &cache.cache,
            &cache.insertion_lock,
            cache.max_entries,
            ("nonexistent1.invalid".to_string(), 80),
            DnsEntry::new_negative(
                "nxdomain".to_string(),
                Instant::now() + Duration::from_secs(5),
            ),
        );
        let err = cache
            .resolve("nonexistent1.invalid", 80)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("nxdomain"));

        EgressDnsCache::insert_bounded(
            &cache.cache,
            &cache.insertion_lock,
            cache.max_entries,
            ("nonexistent2.invalid".to_string(), 80),
            DnsEntry::new_negative(
                "nxdomain".to_string(),
                Instant::now() + Duration::from_secs(5),
            ),
        );
        EgressDnsCache::insert_bounded(
            &cache.cache,
            &cache.insertion_lock,
            cache.max_entries,
            ("nonexistent3.invalid".to_string(), 80),
            DnsEntry::new_negative(
                "nxdomain".to_string(),
                Instant::now() + Duration::from_secs(5),
            ),
        );
        assert!(cache.cache.len() <= 2);
    }
    #[test]
    fn eviction_prefers_negative_then_earliest_expiring_entries() {
        let cache = EgressDnsCache::with_capacity(Duration::from_secs(60), 3);
        let insert = |host: &str, entry: DnsEntry| {
            EgressDnsCache::insert_bounded(
                &cache.cache,
                &cache.insertion_lock,
                cache.max_entries,
                (host.to_string(), 80),
                entry,
            );
        };
        let addr: SocketAddr = "10.0.0.1:80".parse().unwrap();
        let now = Instant::now();
        insert(
            "hot-long.example",
            DnsEntry::new(vec![addr], now + Duration::from_secs(60)),
        );
        insert(
            "hot-short.example",
            DnsEntry::new(vec![addr], now + Duration::from_secs(30)),
        );
        for index in 0..10 {
            insert(
                &format!("missing-{index}.invalid"),
                DnsEntry::new_negative("nxdomain".into(), now + Duration::from_secs(2)),
            );
        }
        assert_eq!(cache.cache.len(), 3);
        assert!(cache
            .cache
            .contains_key(&("hot-long.example".to_string(), 80)));
        assert!(cache
            .cache
            .contains_key(&("hot-short.example".to_string(), 80)));

        insert(
            "hot-new.example",
            DnsEntry::new(vec![addr], now + Duration::from_secs(60)),
        );
        insert(
            "hot-newer.example",
            DnsEntry::new(vec![addr], now + Duration::from_secs(60)),
        );
        assert!(cache
            .cache
            .contains_key(&("hot-long.example".to_string(), 80)));
        assert!(!cache
            .cache
            .contains_key(&("hot-short.example".to_string(), 80)));
    }

    #[test]
    fn concurrent_inserts_respect_capacity() {
        let cache = EgressDnsCache::with_capacity(Duration::from_secs(60), 8);
        std::thread::scope(|scope| {
            for worker in 0..16 {
                let cache = &cache;
                scope.spawn(move || {
                    for index in 0..100 {
                        EgressDnsCache::insert_bounded(
                            &cache.cache,
                            &cache.insertion_lock,
                            cache.max_entries,
                            (format!("{worker}-{index}.invalid"), 80),
                            DnsEntry::new_negative(
                                "missing".into(),
                                Instant::now() + Duration::from_secs(60),
                            ),
                        );
                        assert!(cache.cache.len() <= cache.max_entries);
                    }
                });
            }
        });
    }
}
