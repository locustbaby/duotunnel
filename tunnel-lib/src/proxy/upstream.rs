use crate::proxy::tcp::UpstreamScheme;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct UpstreamGroup {
    pub servers: Vec<String>,
    counter: AtomicUsize,
    unhealthy: Arc<RwLock<HashMap<String, Instant>>>,
}

impl UpstreamGroup {
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            counter: AtomicUsize::new(0),
            unhealthy: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn mark_unhealthy(&self, server: &str) {
        let mut map = self.unhealthy.write().unwrap();
        if map.contains_key(server) {
            return;
        }
        map.insert(server.to_string(), Instant::now() + Duration::from_secs(10));

        let server_clone = server.to_string();
        let unhealthy_clone = self.unhealthy.clone();
        tokio::spawn(async move {
            let (_, connect_addr_str, _) = UpstreamScheme::from_address(&server_clone);
            let probe_timeout = Duration::from_secs(3);
            for _ in 0..5 {
                tokio::time::sleep(Duration::from_secs(2)).await;
                if let Ok(addr) = connect_addr_str.parse::<std::net::SocketAddr>() {
                    if tokio::time::timeout(probe_timeout, tokio::net::TcpStream::connect(addr))
                        .await
                        .is_ok_and(|r| r.is_ok())
                    {
                        let mut map = unhealthy_clone.write().unwrap();
                        map.remove(&server_clone);
                        tracing::info!(server = %server_clone, "active health probe succeeded, backend restored to pool early");
                        return;
                    }
                } else {
                    let mut parts = connect_addr_str.rsplitn(2, ':');
                    if let (Some(port_str), Some(host_str)) = (parts.next(), parts.next()) {
                        if let Ok(port) = port_str.parse::<u16>() {
                            if let Ok(resolved) =
                                tokio::net::lookup_host(format!("{}:{}", host_str, port)).await
                            {
                                if let Some(addr) = resolved.collect::<Vec<_>>().first() {
                                    if tokio::time::timeout(
                                        probe_timeout,
                                        tokio::net::TcpStream::connect(addr),
                                    )
                                    .await
                                    .is_ok_and(|r| r.is_ok())
                                    {
                                        let mut map = unhealthy_clone.write().unwrap();
                                        map.remove(&server_clone);
                                        tracing::info!(server = %server_clone, "active health probe succeeded, backend restored to pool early");
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn mark_healthy(&self, server: &str) {
        let mut map = self.unhealthy.write().unwrap();
        map.remove(server);
    }

    pub fn is_healthy(&self, server: &str) -> bool {
        let map = self.unhealthy.read().unwrap();
        if let Some(expires_at) = map.get(server) {
            Instant::now() >= *expires_at
        } else {
            true
        }
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
