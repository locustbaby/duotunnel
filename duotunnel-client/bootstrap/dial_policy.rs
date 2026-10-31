use anyhow::{anyhow, Result};
use std::net::{IpAddr, Ipv6Addr};
use std::sync::Arc;

/// Client-side policy for server-pushed dial targets, enforced in two stages:
/// the target name/port when the upstream map is built, and every resolved IP at
/// dial time. Name rules trust what the name resolves to (minus the optional
/// loopback/link-local guard); IP and CIDR rules constrain the actual address.
#[derive(Debug, Default)]
pub struct DialPolicy {
    rules: Vec<DialRule>,
    deny_loopback_and_link_local: bool,
}

#[derive(Debug)]
struct DialRule {
    host: HostRule,
    port: Option<u16>,
}

#[derive(Debug)]
enum HostRule {
    Any,
    Exact(String),
    Suffix(String),
    Cidr { net: IpAddr, prefix: u8 },
}

impl DialRule {
    fn port_matches(&self, port: Option<u16>) -> bool {
        match (self.port, port) {
            (Some(required), Some(port)) => required == port,
            _ => true,
        }
    }
}

impl DialPolicy {
    pub fn new(patterns: &[String], deny_loopback_and_link_local: bool) -> Result<Self> {
        let rules = patterns
            .iter()
            .map(|pattern| parse_rule(pattern))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            rules,
            deny_loopback_and_link_local,
        })
    }

    pub fn is_unrestricted(&self) -> bool {
        self.rules.is_empty() && !self.deny_loopback_and_link_local
    }

    /// Name stage. A hostname is admitted when a name rule matches it, or when an
    /// IP/CIDR rule could match its resolved address (checked by `allows_resolved`).
    pub fn allows_target(&self, addr: &str) -> bool {
        let parsed = duotunnel_lib::transport::addr::parse_upstream(addr);
        let host = parsed.host.trim_matches(|c| c == '[' || c == ']');
        let port = Some(parsed.port);
        if let Ok(ip) = host.parse::<IpAddr>() {
            return self.allows_ip(None, ip, port);
        }
        if self.rules.is_empty() {
            return true;
        }
        self.rules.iter().any(|rule| {
            rule.port_matches(port)
                && match &rule.host {
                    HostRule::Cidr { .. } => true,
                    host_rule => name_matches(host_rule, host),
                }
        })
    }

    /// Resolution stage. `port` is `None` when the resolver does not know it (hyper's
    /// DNS hook); the name stage has already enforced port constraints in that case.
    pub fn allows_resolved(&self, host: &str, ip: IpAddr, port: Option<u16>) -> bool {
        self.allows_ip(Some(host), ip, port)
    }

    fn allows_ip(&self, host: Option<&str>, ip: IpAddr, port: Option<u16>) -> bool {
        let ip = canonical_ip(ip);
        let cidr_allowed = self.rules.iter().any(|rule| {
            rule.port_matches(port)
                && matches!(rule.host, HostRule::Cidr { net, prefix } if cidr_contains(net, prefix, ip))
        });
        if cidr_allowed {
            return true;
        }
        let name_allowed = self.rules.is_empty()
            || self.rules.iter().any(|rule| {
                rule.port_matches(port)
                    && match (&rule.host, host) {
                        (HostRule::Any, _) => true,
                        (HostRule::Cidr { .. }, _) | (_, None) => false,
                        (host_rule, Some(host)) => name_matches(host_rule, host),
                    }
            });
        name_allowed && !(self.deny_loopback_and_link_local && is_loopback_or_link_local(ip))
    }

    pub fn resolved_addr_filter(self: &Arc<Self>) -> Option<duotunnel_lib::ResolvedAddrFilter> {
        if self.is_unrestricted() {
            return None;
        }
        let policy = Arc::clone(self);
        Some(Arc::new(move |host: &str, ip: IpAddr| {
            policy.allows_resolved(host, ip, None)
        }))
    }
}

fn name_matches(rule: &HostRule, host: &str) -> bool {
    match rule {
        HostRule::Any => true,
        HostRule::Exact(name) => host.eq_ignore_ascii_case(name),
        HostRule::Suffix(suffix) => {
            host.len() > suffix.len()
                && host.as_bytes()[host.len() - suffix.len() - 1] == b'.'
                && host[host.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
        }
        HostRule::Cidr { .. } => false,
    }
}

fn parse_rule(raw: &str) -> Result<DialRule> {
    let pattern = raw.trim();
    if pattern.is_empty() {
        return Err(anyhow!("allowed_dial_targets entries must be nonempty"));
    }
    let (host, port) = split_pattern(pattern).ok_or_else(|| {
        anyhow!("invalid allowed_dial_targets entry {raw:?}: bad host/port syntax")
    })?;
    let host = if host == "*" {
        HostRule::Any
    } else if let Some(suffix) = host.strip_prefix("*.") {
        if suffix.is_empty() || suffix.contains('*') {
            return Err(anyhow!("invalid allowed_dial_targets wildcard {raw:?}"));
        }
        HostRule::Suffix(suffix.to_owned())
    } else if let Some((net, prefix)) = host.split_once('/') {
        let net: IpAddr = net
            .parse()
            .map_err(|_| anyhow!("invalid allowed_dial_targets CIDR {raw:?}"))?;
        let net = canonical_ip(net);
        let max = if net.is_ipv4() { 32 } else { 128 };
        let prefix: u8 = prefix
            .parse()
            .ok()
            .filter(|p| *p <= max)
            .ok_or_else(|| anyhow!("invalid allowed_dial_targets CIDR prefix {raw:?}"))?;
        HostRule::Cidr { net, prefix }
    } else if let Ok(ip) = host.parse::<IpAddr>() {
        let ip = canonical_ip(ip);
        let prefix = if ip.is_ipv4() { 32 } else { 128 };
        HostRule::Cidr { net: ip, prefix }
    } else if host.contains('*') {
        return Err(anyhow!(
            "invalid allowed_dial_targets entry {raw:?}: only a leading '*.' wildcard is supported"
        ));
    } else {
        HostRule::Exact(host.to_owned())
    };
    Ok(DialRule { host, port })
}

/// Returns `None` for malformed syntax so validation fails closed instead of
/// silently treating a typo as a hostname.
fn split_pattern(pattern: &str) -> Option<(&str, Option<u16>)> {
    if let Some(rest) = pattern.strip_prefix('[') {
        let (host, tail) = rest.split_once(']')?;
        return match tail {
            "" | ":*" => Some((host, None)),
            _ => {
                let port = tail.strip_prefix(':')?.parse::<u16>().ok()?;
                Some((host, Some(port)))
            }
        };
    }
    if pattern.matches(':').count() == 1 {
        let (host, port) = pattern.rsplit_once(':')?;
        if port == "*" {
            return Some((host, None));
        }
        return Some((host, Some(port.parse::<u16>().ok()?)));
    }
    Some((pattern, None))
}

fn canonical_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(v6)),
        v4 => v4,
    }
}

fn cidr_contains(net: IpAddr, prefix: u8, ip: IpAddr) -> bool {
    match (net, ip) {
        (IpAddr::V4(net), IpAddr::V4(ip)) => {
            let mask = u32::MAX.checked_shl(32 - u32::from(prefix)).unwrap_or(0);
            u32::from(net) & mask == u32::from(ip) & mask
        }
        (IpAddr::V6(net), IpAddr::V6(ip)) => {
            let mask = u128::MAX.checked_shl(128 - u32::from(prefix)).unwrap_or(0);
            u128::from(net) & mask == u128::from(ip) & mask
        }
        _ => false,
    }
}

const AWS_IPV6_METADATA: Ipv6Addr = Ipv6Addr::new(0xfd00, 0xec2, 0, 0, 0, 0, 0, 0x254);

fn is_loopback_or_link_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_link_local() || v4.is_unspecified(),
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xffc0) == 0xfe80
                || v6 == AWS_IPV6_METADATA
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(patterns: &[&str], deny: bool) -> DialPolicy {
        let patterns: Vec<String> = patterns.iter().map(|p| p.to_string()).collect();
        DialPolicy::new(&patterns, deny).unwrap()
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn empty_policy_allows_everything() {
        let p = policy(&[], false);
        assert!(p.is_unrestricted());
        assert!(p.allows_target("http://169.254.169.254:80"));
        assert!(p.allows_resolved("x", ip("169.254.169.254"), Some(80)));
    }

    #[test]
    fn name_stage_matches_hosts_ips_and_ports() {
        let p = policy(
            &["127.0.0.1", "localhost:8080", "[::1]:9000", "*.svc.local"],
            false,
        );
        assert!(p.allows_target("http://127.0.0.1:3000"));
        assert!(p.allows_target("127.0.0.1:22"));
        assert!(p.allows_target("http://localhost:8080"));
        assert!(p.allows_target("[::1]:9000"));
        assert!(!p.allows_target("[::1]:9001"));
        assert!(p.allows_target("https://api.svc.local:443"));
        assert!(!p.allows_target("http://169.254.169.254:80"));
        // With an IP rule present, other hostnames are admitted and judged by resolved IP.
        assert!(p.allows_resolved("localhost", ip("127.0.0.1"), Some(8081)));
        assert!(!p.allows_resolved("localhost", ip("::1"), Some(8081)));

        let names_only = policy(&["localhost:8080", "*.svc.local"], false);
        assert!(!names_only.allows_target("http://localhost:8081"));
        assert!(!names_only.allows_target("https://svc.local:443"));
        assert!(!names_only.allows_target("https://other.example:443"));
    }

    #[test]
    fn cidr_rules_constrain_resolved_addresses_of_hostnames() {
        let p = policy(&["10.0.0.0/8:8080"], false);
        assert!(p.allows_target("api.internal:8080"));
        assert!(!p.allows_target("api.internal:9090"));
        assert!(p.allows_resolved("api.internal", ip("10.1.2.3"), Some(8080)));
        assert!(!p.allows_resolved("api.internal", ip("169.254.169.254"), Some(8080)));
        assert!(!p.allows_resolved("api.internal", ip("10.1.2.3"), Some(9090)));
        assert!(p.allows_target("10.9.9.9:8080"));
        assert!(!p.allows_target("11.0.0.1:8080"));
    }

    #[test]
    fn name_rules_trust_resolution_unless_guard_enabled() {
        let open = policy(&["*.svc.local"], false);
        assert!(open.allows_resolved("evil.svc.local", ip("169.254.169.254"), None));
        let guarded = policy(&["*.svc.local"], true);
        assert!(!guarded.allows_resolved("evil.svc.local", ip("169.254.169.254"), None));
        assert!(!guarded.allows_resolved("evil.svc.local", ip("::ffff:127.0.0.1"), None));
        assert!(!guarded.allows_resolved("evil.svc.local", ip("fe80::1"), None));
        assert!(guarded.allows_resolved("api.svc.local", ip("10.0.0.5"), None));
        assert!(!guarded.allows_resolved("other.example", ip("10.0.0.5"), None));
    }

    #[test]
    fn explicit_ip_rule_overrides_guard() {
        let p = policy(&["127.0.0.1", "*"], true);
        assert!(p.allows_target("127.0.0.1:80"));
        assert!(!p.allows_target("[::1]:80"));
        assert!(!p.allows_resolved("anything", ip("169.254.169.254"), None));
        assert!(p.allows_resolved("anything", ip("8.8.8.8"), None));
    }

    #[test]
    fn guard_without_allowlist_blocks_only_special_addresses() {
        let p = policy(&[], true);
        assert!(!p.is_unrestricted());
        assert!(p.allows_target("api.example:443"));
        assert!(!p.allows_target("169.254.169.254:80"));
        assert!(!p.allows_resolved("api.example", ip("127.0.0.1"), None));
        assert!(p.allows_resolved("api.example", ip("93.184.216.34"), None));
    }

    #[test]
    fn invalid_patterns_fail_closed() {
        for pattern in [
            "[::1]:65536",
            "[::1]:abc",
            "[::1]:",
            "[::1]junk",
            "",
            "10.0.0.0/33",
            "host:99999",
            "a*b.example",
        ] {
            assert!(
                DialPolicy::new(&[pattern.to_string()], false).is_err(),
                "{pattern:?} should be rejected"
            );
        }
    }
}
