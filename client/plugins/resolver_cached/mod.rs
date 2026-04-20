use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use tunnel_lib::plugin::Resolver;

const DNS_CACHE_TTL: Duration = Duration::from_secs(30);

struct CachedAddr {
    addr: SocketAddr,
    cached_at: Instant,
}

/// DNS resolver with a lazy 30-second cache.
///
/// Wraps `tokio::net::lookup_host`. An IP literal bypasses the cache. On a
/// miss or expiry, the first resolved address is stored.
pub struct CachedResolver {
    cache: DashMap<String, CachedAddr>,
}

impl CachedResolver {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
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
        // IP literals bypass the cache entirely.
        if let Ok(addr) = host.parse::<std::net::IpAddr>() {
            return Ok(vec![SocketAddr::new(addr, port)]);
        }

        let cache_key = format!("{}:{}", host, port);
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.cached_at.elapsed() < DNS_CACHE_TTL {
                return Ok(vec![cached.addr]);
            }
        }

        let addr = {
            let mut addrs = tokio::net::lookup_host(cache_key.as_str())
                .await
                .map_err(|e| anyhow!("failed to resolve {}: {}", cache_key, e))?;
            addrs
                .next()
                .ok_or_else(|| anyhow!("no resolved IP for {}", cache_key))?
        };
        self.cache.insert(
            cache_key,
            CachedAddr {
                addr,
                cached_at: Instant::now(),
            },
        );
        Ok(vec![addr])
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
        // No cache entry recorded for IP literals.
        assert!(resolver.cache.is_empty());
    }
}
