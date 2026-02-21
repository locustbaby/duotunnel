pub struct UpstreamAddr {
    pub is_https: bool,
    pub host: String,
    pub port: u16,
    pub connect_addr: String,
}

/// Parse an upstream address into its components.
///
/// Handles all of:
/// - `http://host:port` and `https://host:port` (explicit scheme)
/// - `host:port` (no scheme; port 443 → HTTPS, otherwise HTTP)
/// - `[::1]:8080` (IPv6 bracket notation)
/// - Rejects false positives: `host:4430` must NOT be treated as HTTPS
pub fn parse_upstream(addr: &str) -> UpstreamAddr {
    // Strip explicit scheme if present
    let (scheme_https, clean_addr) = if addr.starts_with("https://") {
        (Some(true), addr.trim_start_matches("https://"))
    } else if addr.starts_with("http://") {
        (Some(false), addr.trim_start_matches("http://"))
    } else {
        (None, addr)
    };

    let host = extract_host(clean_addr).to_string();
    let port = extract_port(clean_addr);

    let is_https = scheme_https.unwrap_or_else(|| port == Some(443));

    let effective_port = port.unwrap_or(if is_https { 443 } else { 80 });

    let connect_addr = if port.is_some() {
        clean_addr.to_string()
    } else if is_https {
        format!("{}:443", clean_addr)
    } else {
        format!("{}:80", clean_addr)
    };

    UpstreamAddr { is_https, host, port: effective_port, connect_addr }
}

/// Extract the host portion from an address, handling IPv6 bracket notation.
///
/// - `[::1]:8080`      → `::1`
/// - `[::1]`           → `::1`
/// - `example.com:80`  → `example.com`
/// - `example.com`     → `example.com`
fn extract_host(addr: &str) -> &str {
    if addr.starts_with('[') {
        // IPv6: find the closing bracket
        addr[1..].split(']').next().unwrap_or(addr)
    } else {
        addr.split(':').next().unwrap_or(addr)
    }
}

/// Extract the port number from an address, handling IPv6 bracket notation.
///
/// - `[::1]:8080`      → Some(8080)
/// - `[::1]`           → None
/// - `example.com:443` → Some(443)
/// - `example.com`     → None
fn extract_port(addr: &str) -> Option<u16> {
    if addr.starts_with('[') {
        // IPv6: port comes after `]:`
        let after_bracket = addr.split(']').nth(1)?;
        after_bracket.strip_prefix(':')?.parse().ok()
    } else {
        // Plain host:port — only one colon expected
        let mut parts = addr.splitn(2, ':');
        parts.next(); // skip host
        parts.next()?.parse().ok()
    }
}

pub fn normalize_host(host: &str) -> String {
    if host.starts_with('[') {
        // IPv6 in brackets: strip brackets
        host[1..].split(']').next().unwrap_or(host).to_lowercase()
    } else {
        host.split(':').next().unwrap_or(host).to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_http() {
        let a = parse_upstream("example.com:8080");
        assert_eq!(a.host, "example.com");
        assert_eq!(a.port, 8080);
        assert!(!a.is_https);
        assert_eq!(a.connect_addr, "example.com:8080");
    }

    #[test]
    fn test_port_443_is_https() {
        let a = parse_upstream("example.com:443");
        assert!(a.is_https);
        assert_eq!(a.port, 443);
    }

    #[test]
    fn test_port_4430_is_not_https() {
        let a = parse_upstream("example.com:4430");
        assert!(!a.is_https, "port 4430 must not be treated as HTTPS");
        assert_eq!(a.port, 4430);
    }

    #[test]
    fn test_explicit_https_scheme() {
        let a = parse_upstream("https://example.com");
        assert!(a.is_https);
        assert_eq!(a.port, 443);
        assert_eq!(a.host, "example.com");
        assert_eq!(a.connect_addr, "example.com:443");
    }

    #[test]
    fn test_explicit_https_with_port() {
        let a = parse_upstream("https://example.com:8443");
        assert!(a.is_https);
        assert_eq!(a.port, 8443);
        assert_eq!(a.connect_addr, "example.com:8443");
    }

    #[test]
    fn test_explicit_http_scheme() {
        let a = parse_upstream("http://example.com:8080");
        assert!(!a.is_https);
        assert_eq!(a.port, 8080);
        assert_eq!(a.host, "example.com");
    }

    #[test]
    fn test_ipv6_with_port() {
        let a = parse_upstream("[::1]:8080");
        assert_eq!(a.host, "::1");
        assert_eq!(a.port, 8080);
        assert!(!a.is_https);
        assert_eq!(a.connect_addr, "[::1]:8080");
    }

    #[test]
    fn test_ipv6_port_443() {
        let a = parse_upstream("[::1]:443");
        assert_eq!(a.host, "::1");
        assert_eq!(a.port, 443);
        assert!(a.is_https);
    }

    #[test]
    fn test_no_port_defaults_to_80() {
        let a = parse_upstream("example.com");
        assert_eq!(a.port, 80);
        assert!(!a.is_https);
        assert_eq!(a.connect_addr, "example.com:80");
    }

    #[test]
    fn test_normalize_host_ipv6() {
        assert_eq!(normalize_host("[::1]:8080"), "::1");
        assert_eq!(normalize_host("[::1]"), "::1");
    }

    #[test]
    fn test_normalize_host_plain() {
        assert_eq!(normalize_host("Example.COM:8080"), "example.com");
    }
}
