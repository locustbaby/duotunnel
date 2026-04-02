use std::sync::atomic::{AtomicUsize, Ordering};
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
    pub fn next(&self) -> Option<&String> {
        if self.servers.is_empty() {
            return None;
        }
        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % self.servers.len();
        self.servers.get(idx)
    }
    pub fn first(&self) -> Option<&String> {
        self.servers.first()
    }
}
