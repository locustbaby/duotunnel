use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, debug};

pub async fn start_tcp_listener<F, Fut>(
    port: u16,
    handler: F,
    protocol_name: &str,
) -> Result<()>
where
    F: Fn(TcpStream) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("{} listener started on 0.0.0.0:{}", protocol_name, port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        debug!("Accepted {} connection from {}", protocol_name, peer_addr);

        let handler = handler.clone();
        let protocol_name = protocol_name.to_string();

        tokio::spawn(async move {
            if let Err(e) = handler(socket).await {
                debug!("{} connection error: {}", protocol_name, e);
            }
        });
    }
}

pub async fn peek_bytes(stream: &TcpStream, buf: &mut [u8]) -> std::io::Result<usize> {
    stream.peek(buf).await
}

/// Lock-free virtual host router with O(1) exact match and wildcard support.
///
/// Uses DashMap for concurrent exact-match lookups (most common case),
/// and a small RwLock-protected Vec for wildcard patterns.
pub struct VhostRouter<T: Clone + Send + Sync> {
    /// Exact hostname matches (lock-free)
    exact: DashMap<String, T>,
    /// Wildcard patterns like "*.example.com" (read-optimized)
    wildcards: RwLock<Vec<(String, T)>>,
}

impl<T: Clone + Send + Sync> VhostRouter<T> {
    pub fn new() -> Self {
        Self {
            exact: DashMap::new(),
            wildcards: RwLock::new(Vec::new()),
        }
    }

    pub fn add_route(&self, host: &str, value: T) {
        let host_lower = host.to_lowercase();
        if host_lower.starts_with("*.") {
            let mut wildcards = self.wildcards.write();
            wildcards.push((host_lower, value));
        } else {
            self.exact.insert(host_lower, value);
        }
    }

    pub fn get(&self, host: &str) -> Option<T> {
        let host = host.split(':').next().unwrap_or(host).to_lowercase();

        // Fast path: exact match (lock-free)
        if let Some(entry) = self.exact.get(&host) {
            return Some(entry.value().clone());
        }

        // Slow path: wildcard matching (read lock only)
        let wildcards = self.wildcards.read();
        for (pattern, value) in wildcards.iter() {
            if pattern.starts_with("*.") {
                let suffix = &pattern[1..]; // ".example.com"
                if host.ends_with(suffix) {
                    return Some(value.clone());
                }
            }
        }

        None
    }

    pub fn remove(&self, host: &str) {
        let host_lower = host.to_lowercase();
        if host_lower.starts_with("*.") {
            let mut wildcards = self.wildcards.write();
            wildcards.retain(|(p, _)| p != &host_lower);
        } else {
            self.exact.remove(&host_lower);
        }
    }

    pub fn len(&self) -> usize {
        self.exact.len() + self.wildcards.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.exact.is_empty() && self.wildcards.read().is_empty()
    }
}

impl<T: Clone + Send + Sync> Default for VhostRouter<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PortRouter<T: Clone + Send + Sync> {
    routes: HashMap<u16, T>,
}

impl<T: Clone + Send + Sync> PortRouter<T> {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, port: u16, value: T) {
        self.routes.insert(port, value);
    }

    pub fn get(&self, port: u16) -> Option<&T> {
        self.routes.get(&port)
    }

    pub fn remove(&mut self, port: u16) {
        self.routes.remove(&port);
    }
}

impl<T: Clone + Send + Sync> Default for PortRouter<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn extract_host_from_http(data: &[u8]) -> Option<String> {
    let data_str = std::str::from_utf8(data).ok()?;

    for line in data_str.lines() {
        if line.to_lowercase().starts_with("host:") {
            let host = line[5..].trim();
            return Some(host.to_string());
        }
    }
    None
}

pub fn extract_method_path_from_http(data: &[u8]) -> Option<(String, String)> {
    let data_str = std::str::from_utf8(data).ok()?;
    let first_line = data_str.lines().next()?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// Shared VhostRouter - now lock-free for most operations
pub type SharedVhostRouter<T> = Arc<VhostRouter<T>>;

pub fn new_shared_vhost_router<T: Clone + Send + Sync>() -> SharedVhostRouter<T> {
    Arc::new(VhostRouter::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vhost_router_exact_match() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("example.com", "group-a".to_string());

        assert_eq!(router.get("example.com"), Some("group-a".to_string()));
        assert_eq!(router.get("Example.COM"), Some("group-a".to_string()));
        assert_eq!(router.get("other.com"), None);
    }

    #[test]
    fn test_vhost_router_wildcard() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.example.com", "group-a".to_string());

        assert_eq!(router.get("api.example.com"), Some("group-a".to_string()));
        assert_eq!(router.get("www.example.com"), Some("group-a".to_string()));
        assert_eq!(router.get("example.com"), None);
    }

    #[test]
    fn test_vhost_router_with_port() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("example.com", "group-a".to_string());

        assert_eq!(router.get("example.com:8080"), Some("group-a".to_string()));
    }

    #[test]
    fn test_extract_host() {
        let req = b"GET / HTTP/1.1\r\nHost: example.com\r\nContent-Type: text/html\r\n\r\n";
        assert_eq!(extract_host_from_http(req), Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_method_path() {
        let req = b"GET /api/users HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert_eq!(
            extract_method_path_from_http(req),
            Some(("GET".to_string(), "/api/users".to_string()))
        );
    }
}
