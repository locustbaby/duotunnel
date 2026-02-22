use std::sync::atomic::{AtomicUsize, Ordering};

/// Upstream group with round-robin load balancing.
///
/// Used by both `server/egress.rs` (ServerEgressMap) and `client/app.rs` (LocalProxyMap)
/// to distribute requests across a pool of server addresses.
pub struct UpstreamGroup {
    pub servers: Vec<String>,
    counter: AtomicUsize,
}

impl UpstreamGroup {
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            counter: AtomicUsize::new(0),
        }
    }

    /// Return the next server address using round-robin selection.
    pub fn next(&self) -> Option<&String> {
        if self.servers.is_empty() {
            return None;
        }
        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % self.servers.len();
        self.servers.get(idx)
    }

    /// Return the first server address without advancing the counter.
    pub fn first(&self) -> Option<&String> {
        self.servers.first()
    }
}
