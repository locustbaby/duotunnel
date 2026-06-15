use dashmap::DashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct DnsEntry {
    pub addrs: Vec<SocketAddr>,
    pub expires_at: Instant,
}

pub struct EgressDnsCache {
    cache: DashMap<(String, u16), DnsEntry>,
    inflight: DashMap<(String, u16), broadcast::Sender<Result<Vec<SocketAddr>, String>>>,
    ttl: Duration,
}

impl EgressDnsCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: DashMap::new(),
            inflight: DashMap::new(),
            ttl,
        }
    }

    fn get_stale(&self, key: &(String, u16)) -> Option<SocketAddr> {
        self.cache
            .get(key)
            .and_then(|entry| entry.addrs.first().cloned())
    }

    pub async fn resolve(&self, host: &str, port: u16) -> anyhow::Result<SocketAddr> {
        let key = (host.to_string(), port);

        // Fast path: check cache
        if let Some(entry) = self.cache.get(&key) {
            if Instant::now() < entry.expires_at {
                if let Some(&addr) = entry.addrs.first() {
                    return Ok(addr);
                }
            }
        }

        loop {
            let rx = {
                if let Some(sender) = self.inflight.get(&key) {
                    Some(sender.subscribe())
                } else {
                    None
                }
            };

            if let Some(mut rx) = rx {
                // Wait for the in-flight resolution with a timeout
                match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
                    Ok(Ok(Ok(resolved_vec))) => {
                        if let Some(&addr) = resolved_vec.first() {
                            return Ok(addr);
                        }
                        if let Some(addr) = self.get_stale(&key) {
                            return Ok(addr);
                        }
                        return Err(anyhow::anyhow!("no resolved IP for {}:{}", host, port));
                    }
                    Ok(Ok(Err(err_msg))) => {
                        if let Some(addr) = self.get_stale(&key) {
                            tracing::warn!(
                                host = %host, port = port, error = %err_msg,
                                "In-flight DNS lookup failed, using stale cached IP"
                            );
                            return Ok(addr);
                        }
                        return Err(anyhow::anyhow!("DNS lookup failed: {}", err_msg));
                    }
                    Ok(Err(_)) | Err(_) => {
                        // Channel lagged/closed or resolution timed out, retry the loop
                        continue;
                    }
                }
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

            let host_clone = host.to_string();
            let key_clone = key.clone();

            let lookup_res = tokio::time::timeout(
                Duration::from_secs(5),
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

            let _ = tx.send(result.clone());
            self.inflight.remove(&key);

            match result {
                Ok(resolved_vec) => {
                    let addr = resolved_vec[0];
                    self.cache.insert(
                        key_clone,
                        DnsEntry {
                            addrs: resolved_vec,
                            expires_at: Instant::now() + self.ttl,
                        },
                    );
                    return Ok(addr);
                }
                Err(err_msg) => {
                    if let Some(addr) = self.get_stale(&key_clone) {
                        tracing::warn!(
                            host = %host_clone, port = port, error = %err_msg,
                            "DNS lookup failed, using stale cached IP"
                        );
                        return Ok(addr);
                    }
                    return Err(anyhow::anyhow!(
                        "DNS lookup failed for {}:{}: {}",
                        host_clone,
                        port,
                        err_msg
                    ));
                }
            }
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
}
