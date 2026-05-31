use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct DnsEntry {
    pub addrs: Vec<SocketAddr>,
    pub expires_at: Instant,
}

pub struct EgressDnsCache {
    cache: RwLock<HashMap<(String, u16), DnsEntry>>,
    ttl: Duration,
}

impl EgressDnsCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    fn get_stale(&self, key: &(String, u16)) -> Option<SocketAddr> {
        let map = self.cache.read().unwrap();
        map.get(key).and_then(|entry| entry.addrs.first().cloned())
    }

    pub async fn resolve(&self, host: &str, port: u16) -> anyhow::Result<SocketAddr> {
        let key = (host.to_string(), port);
        
        {
            let map = self.cache.read().unwrap();
            if let Some(entry) = map.get(&key) {
                if Instant::now() < entry.expires_at {
                    if let Some(&addr) = entry.addrs.first() {
                        return Ok(addr);
                    }
                }
            }
        }

        match tokio::net::lookup_host((host, port)).await {
            Ok(resolved) => {
                let resolved_vec: Vec<SocketAddr> = resolved.collect();
                if resolved_vec.is_empty() {
                    if let Some(addr) = self.get_stale(&key) {
                        return Ok(addr);
                    }
                    return Err(anyhow::anyhow!("no resolved IP for {}:{}", host, port));
                }
                let addr = resolved_vec[0];
                {
                    let mut map = self.cache.write().unwrap();
                    map.insert(
                        key,
                        DnsEntry {
                            addrs: resolved_vec,
                            expires_at: Instant::now() + self.ttl,
                        },
                    );
                }
                Ok(addr)
            }
            Err(e) => {
                if let Some(addr) = self.get_stale(&key) {
                    tracing::warn!(host = %host, port = port, error = %e, "DNS lookup failed, using stale cached IP");
                    return Ok(addr);
                }
                Err(e.into())
            }
        }
    }
}
