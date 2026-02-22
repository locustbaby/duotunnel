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
    } else if addr.starts_with("wss://") {
        (Some(true), addr.trim_start_matches("wss://"))
    } else if addr.starts_with("http://") {
        (Some(false), addr.trim_start_matches("http://"))
    } else if addr.starts_with("ws://") {
        (Some(false), addr.trim_start_matches("ws://"))
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
    if let Some(rest) = addr.strip_prefix('[') {
        // IPv6: find the closing bracket
        rest.split(']').next().unwrap_or(addr)
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
    if let Some(rest) = host.strip_prefix('[') {
        // IPv6 in brackets: strip brackets
        rest.split(']').next().unwrap_or(host).to_lowercase()
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

    // ── connect_addr completeness ────────────────────────────────────────────

    #[test]
    fn test_connect_addr_no_port_https() {
        // https:// scheme with no port → connect_addr must include :443
        let a = parse_upstream("https://example.com");
        assert_eq!(a.connect_addr, "example.com:443");
    }

    #[test]
    fn test_connect_addr_no_port_http() {
        // http:// scheme with no port → connect_addr must include :80
        let a = parse_upstream("http://example.com");
        assert_eq!(a.connect_addr, "example.com:80");
        assert!(!a.is_https);
        assert_eq!(a.port, 80);
    }

    #[test]
    fn test_connect_addr_explicit_port_preserved() {
        // Explicit port must be preserved verbatim in connect_addr
        let a = parse_upstream("https://example.com:8443");
        assert_eq!(a.connect_addr, "example.com:8443");
        assert_eq!(a.port, 8443);
    }

    // ── ws:// and wss:// schemes ─────────────────────────────────────────────

    #[test]
    fn test_wss_scheme_treated_as_https() {
        // wss:// is a WebSocket over TLS — should be parsed as HTTPS
        let a = parse_upstream("wss://echo.websocket.org");
        assert!(a.is_https, "wss:// must be treated as HTTPS");
        assert_eq!(a.host, "echo.websocket.org");
        assert_eq!(a.port, 443);
    }

    #[test]
    fn test_ws_scheme_treated_as_http() {
        // ws:// is plain WebSocket — should be parsed as HTTP
        let a = parse_upstream("ws://127.0.0.1:8765");
        assert!(!a.is_https, "ws:// must not be treated as HTTPS");
        assert_eq!(a.host, "127.0.0.1");
        assert_eq!(a.port, 8765);
    }

    // ── http:// scheme with port 443 — scheme wins ───────────────────────────

    #[test]
    fn test_http_scheme_overrides_port_443() {
        // Explicit http:// must NOT become HTTPS even if port is 443
        let a = parse_upstream("http://example.com:443");
        assert!(!a.is_https, "explicit http:// must override port-443 heuristic");
        assert_eq!(a.port, 443);
    }

    // ── plain no-port host defaults ──────────────────────────────────────────

    #[test]
    fn test_bare_host_defaults_to_http_port_80() {
        let a = parse_upstream("backend.internal");
        assert!(!a.is_https);
        assert_eq!(a.port, 80);
        assert_eq!(a.host, "backend.internal");
        assert_eq!(a.connect_addr, "backend.internal:80");
    }

    // ── IPv6 no-port default ─────────────────────────────────────────────────

    #[test]
    fn test_ipv6_no_port_defaults_to_80() {
        let a = parse_upstream("[::1]");
        assert_eq!(a.host, "::1");
        assert_eq!(a.port, 80);
        assert!(!a.is_https);
    }

    // ── malformed / non-standard port strings ────────────────────────────────

    #[test]
    fn test_invalid_port_string_falls_back_to_default() {
        // "abc" cannot be parsed as u16; extract_port returns None → default port 80
        let a = parse_upstream("example.com:abc");
        assert_eq!(a.port, 80, "invalid port string should fall back to port 80");
        assert!(!a.is_https);
    }

    #[test]
    fn test_port_overflow_falls_back_to_default() {
        // 99999 overflows u16; extract_port returns None → default port 80
        let a = parse_upstream("example.com:99999");
        assert_eq!(a.port, 80, "port > 65535 should fall back to port 80");
    }

    // ── normalize_host edge cases ────────────────────────────────────────────

    #[test]
    fn test_normalize_host_no_port() {
        // No port suffix — should return the host lowercased
        assert_eq!(normalize_host("Example.COM"), "example.com");
    }

    #[test]
    fn test_normalize_host_ipv6_no_port() {
        assert_eq!(normalize_host("[::1]"), "::1");
    }

    #[test]
    fn test_normalize_host_mixed_case_ipv6() {
        assert_eq!(normalize_host("[FE80::1]:443"), "fe80::1");
    }
}
