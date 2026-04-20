use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use tunnel_lib::plugin::Resolver;

const DNS_CACHE_TTL: Duration = Duration::from_secs(30);
const MAX_CACHE_ENTRIES: usize = 1024;

struct CachedAddrs {
    addrs: Vec<SocketAddr>,
    cached_at: Instant,
}

/// DNS resolver with a lazy 30-second cache.
///
/// Wraps `tokio::net::lookup_host`. IP literals bypass the cache. On a miss
/// or expiry all resolved addresses are stored so callers can do happy-eyeballs
/// or retry across A/AAAA records. The cache is bounded at
/// `MAX_CACHE_ENTRIES`; past the bound, a miss triggers eviction of every
/// entry older than `DNS_CACHE_TTL` before the insert.
pub struct CachedResolver {
    cache: DashMap<String, CachedAddrs>,
}

impl CachedResolver {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    fn evict_expired(&self) {
        let now = Instant::now();
        self.cache
            .retain(|_, v| now.duration_since(v.cached_at) < DNS_CACHE_TTL);
    }

    fn evict_one_oldest(&self) {
        let oldest_key = self
            .cache
            .iter()
            .min_by_key(|entry| entry.value().cached_at)
            .map(|entry| entry.key().clone());
        if let Some(key) = oldest_key {
            self.cache.remove(&key);
        }
    }
}

impl Default for CachedResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Resolver for CachedResolver {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>> {
        if let Ok(addr) = host.parse::<std::net::IpAddr>() {
            return Ok(vec![SocketAddr::new(addr, port)]);
        }

        let cache_key = format!("{}:{}", host, port);
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.cached_at.elapsed() < DNS_CACHE_TTL {
                return Ok(cached.addrs.clone());
            }
        }

        let addrs: Vec<SocketAddr> = tokio::net::lookup_host(cache_key.as_str())
            .await
            .map_err(|e| anyhow!("failed to resolve {}: {}", cache_key, e))?
            .collect();
        if addrs.is_empty() {
            return Err(anyhow!("no resolved IP for {}", cache_key));
        }

        // A cache hit with expired data falls through here and re-inserts,
        // so no extra `contains_key` guard is needed — we only over-evict in
        // the rare case where the cache is actually full.
        if self.cache.len() >= MAX_CACHE_ENTRIES {
            self.evict_expired();
            if self.cache.len() >= MAX_CACHE_ENTRIES {
                self.evict_one_oldest();
            }
        }
        self.cache.insert(
            cache_key,
            CachedAddrs {
                addrs: addrs.clone(),
                cached_at: Instant::now(),
            },
        );
        Ok(addrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ip_literal_resolves_without_cache() {
        let resolver = CachedResolver::new();
        let out = resolver.resolve("127.0.0.1", 8080).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].port(), 8080);
        assert!(resolver.cache.is_empty());
    }
}
