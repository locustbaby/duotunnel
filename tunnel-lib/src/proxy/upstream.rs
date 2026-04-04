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
        let raw = self.counter.fetch_add(1, Ordering::Relaxed);
        let len = self.servers.len();
        let idx = if len.is_power_of_two() {
            raw & (len - 1)
        } else {
            raw % len
        };
        self.servers.get(idx)
    }
    pub fn first(&self) -> Option<&String> {
        self.servers.first()
    }
}
