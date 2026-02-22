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
        let bare = host.split(':').next().unwrap_or(host);

        // Zero-allocation fast path for ASCII hostnames (covers ~100% of real-world hosts).
        // Stack-allocate a 256-byte buffer and make_ascii_lowercase in place — no heap alloc.
        let mut buf = [0u8; 256];
        if bare.len() <= 256 && bare.is_ascii() {
            let n = bare.len();
            buf[..n].copy_from_slice(bare.as_bytes());
            buf[..n].make_ascii_lowercase();
            // SAFETY: input was ASCII (validated above); make_ascii_lowercase preserves UTF-8 validity.
            let lower = unsafe { std::str::from_utf8_unchecked(&buf[..n]) };

            if let Some(entry) = self.exact.get(lower) {
                return Some(entry.value().clone());
            }

            let wildcards = self.wildcards.read();
            for (pattern, value) in wildcards.iter() {
                if pattern.starts_with("*.") {
                    let suffix = &pattern[1..]; // ".example.com"
                    if lower.ends_with(suffix) {
                        return Some(value.clone());
                    }
                }
            }
            None
        } else {
            // Rare fallback: heap allocate for long or non-ASCII hostnames
            let lower = bare.to_lowercase();
            if let Some(entry) = self.exact.get(&lower) {
                return Some(entry.value().clone());
            }
            let wildcards = self.wildcards.read();
            for (pattern, value) in wildcards.iter() {
                if pattern.starts_with("*.") {
                    let suffix = &pattern[1..];
                    if lower.ends_with(suffix) {
                        return Some(value.clone());
                    }
                }
            }
            None
        }
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
        // Case-insensitive prefix check without heap allocation (no to_lowercase())
        if line.len() > 5 && line[..5].eq_ignore_ascii_case("host:") {
            return Some(line[5..].trim().to_string());
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

    // ── remove & size operations ─────────────────────────────────────────────

    #[test]
    fn test_vhost_router_is_empty_and_len() {
        let router: VhostRouter<String> = VhostRouter::new();
        assert!(router.is_empty());
        assert_eq!(router.len(), 0);

        router.add_route("a.com", "x".to_string());
        assert!(!router.is_empty());
        assert_eq!(router.len(), 1);

        router.add_route("*.b.com", "y".to_string());
        assert_eq!(router.len(), 2);
    }

    #[test]
    fn test_vhost_router_remove_exact() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("example.com", "group-a".to_string());
        assert_eq!(router.get("example.com"), Some("group-a".to_string()));

        router.remove("example.com");
        assert_eq!(router.get("example.com"), None);
        assert!(router.is_empty());
    }

    #[test]
    fn test_vhost_router_remove_wildcard() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.example.com", "group-a".to_string());
        assert_eq!(router.get("api.example.com"), Some("group-a".to_string()));

        router.remove("*.example.com");
        assert_eq!(router.get("api.example.com"), None);
        assert!(router.is_empty());
    }

    #[test]
    fn test_vhost_router_remove_nonexistent_is_noop() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("example.com", "group-a".to_string());
        router.remove("other.com"); // must not panic or affect existing route
        assert_eq!(router.get("example.com"), Some("group-a".to_string()));
        assert_eq!(router.len(), 1);
    }

    // ── wildcard edge cases ──────────────────────────────────────────────────

    #[test]
    fn test_wildcard_does_not_match_parent_domain() {
        // *.example.com must NOT match example.com itself
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.example.com", "wildcard".to_string());
        assert_eq!(router.get("example.com"), None);
    }

    #[test]
    fn test_wildcard_does_not_match_sibling_domain() {
        // *.example.com must NOT match notexample.com
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.example.com", "wildcard".to_string());
        assert_eq!(router.get("notexample.com"), None);
    }

    #[test]
    fn test_exact_takes_priority_over_wildcard() {
        // When both *.example.com and api.example.com are registered,
        // exact must win for api.example.com.
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.example.com", "wildcard-group".to_string());
        router.add_route("api.example.com", "exact-group".to_string());

        assert_eq!(router.get("api.example.com"), Some("exact-group".to_string()));
        assert_eq!(router.get("www.example.com"), Some("wildcard-group".to_string()));
    }

    #[test]
    fn test_wildcard_case_insensitive() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.EXAMPLE.COM", "group-a".to_string());
        assert_eq!(router.get("Api.Example.Com"), Some("group-a".to_string()));
    }

    // ── HTTP header parsing edge cases ───────────────────────────────────────

    #[test]
    fn test_extract_host_uppercase_header_name() {
        // Header name matching must be case-insensitive
        let req = b"GET / HTTP/1.1\r\nHOST: example.com\r\n\r\n";
        assert_eq!(extract_host_from_http(req), Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_host_with_port() {
        // Host header may include a port; it should be returned as-is
        let req = b"GET / HTTP/1.1\r\nHost: example.com:8080\r\n\r\n";
        assert_eq!(
            extract_host_from_http(req),
            Some("example.com:8080".to_string())
        );
    }

    #[test]
    fn test_extract_host_missing_returns_none() {
        let req = b"GET / HTTP/1.1\r\nContent-Type: text/plain\r\n\r\n";
        assert_eq!(extract_host_from_http(req), None);
    }

    #[test]
    fn test_extract_host_extra_whitespace() {
        // Spaces after the colon should be trimmed
        let req = b"GET / HTTP/1.1\r\nHost:   example.com  \r\n\r\n";
        assert_eq!(
            extract_host_from_http(req),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_method_path_post() {
        let req = b"POST /submit HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert_eq!(
            extract_method_path_from_http(req),
            Some(("POST".to_string(), "/submit".to_string()))
        );
    }

    #[test]
    fn test_extract_method_path_missing_path_returns_none() {
        // Request line with only one token — no path
        let req = b"GET\r\nHost: example.com\r\n\r\n";
        assert_eq!(extract_method_path_from_http(req), None);
    }

    #[test]
    fn test_extract_method_path_root() {
        let req = b"DELETE / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert_eq!(
            extract_method_path_from_http(req),
            Some(("DELETE".to_string(), "/".to_string()))
        );
    }
}
