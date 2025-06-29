use crate::config::Upstream;

/// Pick a backend address from the upstream (simple round-robin or first available).
pub fn pick_backend(upstream: &Upstream) -> Option<String> {
    if !upstream.servers.is_empty() {
        Some(upstream.servers[0].address.clone())
    } else {
        None
    }
} 