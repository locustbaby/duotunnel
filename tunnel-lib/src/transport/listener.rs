use crate::models::id::{GroupId, ProxyName};
use anyhow::Result;
use dashmap::DashMap;
use parking_lot::RwLock;
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};
pub const DEFAULT_ACCEPT_WORKERS: usize = 4;

pub fn build_reuseport_listener(addr: SocketAddr) -> Result<TcpListener> {
    let domain = if addr.is_ipv6() {
        Domain::IPV6
    } else {
        Domain::IPV4
    };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    if let Err(e) = socket.set_reuse_port(true) {
        warn!("SO_REUSEPORT unavailable ({}), continuing without it", e);
    }
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(4096)?;
    let std_listener: std::net::TcpListener = socket.into();
    Ok(TcpListener::from_std(std_listener)?)
}
pub async fn start_tcp_listener<F, Fut>(port: u16, handler: F, protocol_name: &str) -> Result<()>
where
    F: Fn(TcpStream) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = build_reuseport_listener(addr)?;
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
/// Typed result of a `VhostRouter` lookup: replaces the anonymous `(Arc<str>, Arc<str>)` tuple
/// to give callers named fields instead of positional `.0` / `.1` access.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RouteTarget {
    pub group_id: GroupId,
    pub proxy_name: ProxyName,
}

pub fn canonicalize_egress_host(host: &str) -> Result<String> {
    let host = host.trim();
    if host.is_empty() {
        anyhow::bail!("host must not be empty");
    }
    if host.contains("://") || host.contains('/') || host.contains('?') || host.contains('#') {
        anyhow::bail!("host must be a bare host or authority, not a URL");
    }

    let (wildcard, authority) = if let Some(rest) = host.strip_prefix("*.") {
        if rest.is_empty() {
            anyhow::bail!("wildcard host must include a suffix");
        }
        (true, rest)
    } else {
        (false, host)
    };

    let canonical = canonicalize_authority_host(authority)?;
    if wildcard {
        Ok(format!("*.{canonical}"))
    } else {
        Ok(canonical)
    }
}

fn canonicalize_authority_host(authority: &str) -> Result<String> {
    if let Some(rest) = authority.strip_prefix('[') {
        let end = rest
            .find(']')
            .ok_or_else(|| anyhow::anyhow!("bracketed IPv6 host is missing closing bracket"))?;
        let host = &rest[..end];
        let suffix = &rest[end + 1..];
        if !suffix.is_empty() {
            let port = suffix
                .strip_prefix(':')
                .ok_or_else(|| anyhow::anyhow!("unexpected characters after bracketed host"))?;
            validate_port(port)?;
        }
        if host.is_empty() {
            anyhow::bail!("host must not be empty");
        }
        return Ok(host.to_lowercase());
    }

    let colon_count = authority.as_bytes().iter().filter(|b| **b == b':').count();
    let host = if colon_count == 1 {
        let (host, port) = authority
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("invalid authority host"))?;
        validate_port(port)?;
        host
    } else {
        authority
    };

    if host.is_empty() {
        anyhow::bail!("host must not be empty");
    }
    Ok(host.to_lowercase())
}

fn validate_port(port: &str) -> Result<()> {
    if port.is_empty() || port.parse::<u16>().is_err() {
        anyhow::bail!("port must be a valid u16");
    }
    Ok(())
}

pub struct VhostRouter<T: Clone + Send + Sync> {
    exact: DashMap<String, T>,
    wildcards: RwLock<Vec<(String, T)>>,
    has_wildcards: AtomicBool,
}
impl<T: Clone + Send + Sync> VhostRouter<T> {
    pub fn new() -> Self {
        Self {
            exact: DashMap::new(),
            wildcards: RwLock::new(Vec::new()),
            has_wildcards: AtomicBool::new(false),
        }
    }
    pub fn add_route(&self, host: &str, value: T) {
        let host_key = canonicalize_egress_host(host).unwrap_or_else(|_| host.to_lowercase());
        if host_key.starts_with("*.") {
            let mut wildcards = self.wildcards.write();
            wildcards.push((host_key, value));
            self.has_wildcards.store(true, Ordering::Relaxed);
        } else {
            self.exact.insert(host_key, value);
        }
    }
    pub fn get(&self, host: &str) -> Option<T> {
        let lower = canonicalize_egress_host(host).ok()?;
        let mut buf = [0u8; 256];
        if lower.len() <= 256 && lower.is_ascii() {
            let n = lower.len();
            buf[..n].copy_from_slice(lower.as_bytes());
            // SAFETY: lower is ASCII-validated above.
            let lower = unsafe { std::str::from_utf8_unchecked(&buf[..n]) };
            if let Some(entry) = self.exact.get(lower) {
                return Some(entry.value().clone());
            }
            if self.has_wildcards.load(Ordering::Relaxed) {
                let wildcards = self.wildcards.read();
                for (pattern, value) in wildcards.iter() {
                    if pattern.starts_with("*.") {
                        let suffix = &pattern[1..];
                        if lower.ends_with(suffix) {
                            return Some(value.clone());
                        }
                    }
                }
            }
            None
        } else {
            if let Some(entry) = self.exact.get(&lower) {
                return Some(entry.value().clone());
            }
            if self.has_wildcards.load(Ordering::Relaxed) {
                let wildcards = self.wildcards.read();
                for (pattern, value) in wildcards.iter() {
                    if pattern.starts_with("*.") {
                        let suffix = &pattern[1..];
                        if lower.ends_with(suffix) {
                            return Some(value.clone());
                        }
                    }
                }
            }
            None
        }
    }
    pub fn remove(&self, host: &str) {
        let host_key = canonicalize_egress_host(host).unwrap_or_else(|_| host.to_lowercase());
        if host_key.starts_with("*.") {
            let mut wildcards = self.wildcards.write();
            wildcards.retain(|(p, _)| p != &host_key);
            if wildcards.is_empty() {
                self.has_wildcards.store(false, Ordering::Relaxed);
            }
        } else {
            self.exact.remove(&host_key);
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
// HTTP parsing helpers live in protocol::http_utils; re-exported here for
// backwards compatibility with existing callers.
pub use crate::protocol::http_utils::{extract_host_from_http, extract_method_path_from_http};
pub type SharedVhostRouter<T> = Arc<VhostRouter<T>>;
pub fn new_shared_vhost_router<T: Clone + Send + Sync>() -> SharedVhostRouter<T> {
    Arc::new(VhostRouter::new())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonicalize_egress_host_strips_ports_and_lowercases_domains() {
        assert_eq!(
            canonicalize_egress_host("Example.COM:443").unwrap(),
            "example.com"
        );
        assert_eq!(canonicalize_egress_host("[::1]:443").unwrap(), "::1");
        assert_eq!(
            canonicalize_egress_host("*.Example.COM:443").unwrap(),
            "*.example.com"
        );
    }
    #[test]
    fn canonicalize_egress_host_rejects_urls() {
        assert!(canonicalize_egress_host("https://example.com").is_err());
    }
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
        router.add_route("example.com:443", "group-a".to_string());
        assert_eq!(router.get("example.com:8080"), Some("group-a".to_string()));
    }
    #[test]
    fn test_vhost_router_bracketed_ipv6_with_port() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("[::1]:443", "group-a".to_string());
        assert_eq!(router.get("[::1]:8443"), Some("group-a".to_string()));
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
        router.remove("other.com");
        assert_eq!(router.get("example.com"), Some("group-a".to_string()));
        assert_eq!(router.len(), 1);
    }
    #[test]
    fn test_wildcard_does_not_match_parent_domain() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.example.com", "wildcard".to_string());
        assert_eq!(router.get("example.com"), None);
    }
    #[test]
    fn test_wildcard_does_not_match_sibling_domain() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.example.com", "wildcard".to_string());
        assert_eq!(router.get("notexample.com"), None);
    }
    #[test]
    fn test_exact_takes_priority_over_wildcard() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.example.com", "wildcard-group".to_string());
        router.add_route("api.example.com", "exact-group".to_string());
        assert_eq!(
            router.get("api.example.com"),
            Some("exact-group".to_string())
        );
        assert_eq!(
            router.get("www.example.com"),
            Some("wildcard-group".to_string())
        );
    }
    #[test]
    fn test_wildcard_case_insensitive() {
        let router: VhostRouter<String> = VhostRouter::new();
        router.add_route("*.EXAMPLE.COM", "group-a".to_string());
        assert_eq!(router.get("Api.Example.Com"), Some("group-a".to_string()));
    }
    #[test]
    fn test_extract_host_uppercase_header_name() {
        let req = b"GET / HTTP/1.1\r\nHOST: example.com\r\n\r\n";
        assert_eq!(extract_host_from_http(req), Some("example.com".to_string()));
    }
    #[test]
    fn test_extract_host_with_port() {
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
        let req = b"GET / HTTP/1.1\r\nHost:   example.com  \r\n\r\n";
        assert_eq!(extract_host_from_http(req), Some("example.com".to_string()));
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
