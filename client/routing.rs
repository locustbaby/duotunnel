use anyhow::{Result, Context};
use tunnel_lib::proto::tunnel::{Rule, Upstream};

/// Match rule by type and host
pub fn match_rule_by_type_and_host<'a>(rules: &'a [Rule], rule_type: &str, host: &str) -> Result<Option<&'a Rule>> {
    // Strip port from host for matching (Host header can include port like "hostname:8001")
    let host_without_port = host.split(':').next().unwrap_or(host).trim();
    
    for rule in rules {
        // Match type
        if rule.r#type != rule_type {
            continue;
        }
        
        // Match host (compare hostname without port)
        let rule_host_without_port = rule.match_host.split(':').next().unwrap_or(&rule.match_host).trim();
        if !rule.match_host.is_empty() && rule_host_without_port.eq_ignore_ascii_case(host_without_port) {
            return Ok(Some(rule));
        }
    }
    
    Ok(None)
}

/// Resolve upstream name to actual address (preserving protocol)
/// Returns (full_address_with_protocol, is_ssl)
pub fn resolve_upstream(
    action_proxy_pass: &str,
    upstreams: &[Upstream],
) -> Result<(String, bool)> {
    let address = if let Some(upstream) = upstreams.iter().find(|u| u.name == action_proxy_pass) {
        if upstream.servers.is_empty() {
            anyhow::bail!("Upstream '{}' has no servers", action_proxy_pass);
        }
        
        // Simple round-robin: just pick the first server
        let server = &upstream.servers[0];
        server.address.clone()
    } else {
        // It's a direct address
        action_proxy_pass.to_string()
    };
    
    // Determine if SSL is needed based on address scheme
    let is_ssl = address.trim().starts_with("https://");
    
    // Return full address with protocol preserved
    Ok((address.trim().to_string(), is_ssl))
}

/// Parse target address into hostname and port
/// Returns (hostname, port) with default ports based on protocol
pub fn parse_target_addr(addr: &str, is_ssl: bool) -> Result<(String, u16)> {
    let addr = addr.trim();
    
    // Remove protocol prefix if present
    let addr = if let Some(pos) = addr.find("://") {
        &addr[pos + 3..]
    } else {
        addr
    };
    
    // Remove path if present
    let addr = if let Some(pos) = addr.find('/') {
        &addr[..pos]
    } else {
        addr
    };
    
    // Parse hostname and port
    // Use rfind(':') to handle IPv6 addresses like [::1]:8080
    if let Some(pos) = addr.rfind(':') {
        // Check if it's an IPv6 address in brackets
        if addr.starts_with('[') {
            // IPv6 with port: [::1]:8080
            let bracket_end = addr.find(']').ok_or_else(|| anyhow::anyhow!("Invalid IPv6 address: {}", addr))?;
            let hostname = addr[1..bracket_end].to_string();
            let port = addr[bracket_end + 2..].parse::<u16>()
                .context(format!("Invalid port in address: {}", addr))?;
            Ok((hostname, port))
        } else {
            // Regular hostname:port or IPv4:port
            let hostname = addr[..pos].to_string();
            let port = addr[pos + 1..].parse::<u16>()
                .context(format!("Invalid port in address: {}", addr))?;
            Ok((hostname, port))
        }
    } else {
        // No port specified, use default based on protocol
        let default_port = if is_ssl {
            443  // HTTPS default port
        } else {
            80   // HTTP default port
        };
        Ok((addr.to_string(), default_port))
    }
}

