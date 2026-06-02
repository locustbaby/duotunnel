use std::sync::atomic::{AtomicU64, Ordering};

pub struct ResourceMetrics {
    pub accepted_connections_active: AtomicU64,
    pub stream_pending_queue_depth: AtomicU64,
    pub slowpath_waiting_tasks: AtomicU64,
}

impl ResourceMetrics {
    pub const fn new() -> Self {
        Self {
            accepted_connections_active: AtomicU64::new(0),
            stream_pending_queue_depth: AtomicU64::new(0),
            slowpath_waiting_tasks: AtomicU64::new(0),
        }
    }

    pub fn active_connections(&self) -> u64 {
        self.accepted_connections_active.load(Ordering::Relaxed)
    }

    pub fn pending_streams(&self) -> u64 {
        self.stream_pending_queue_depth.load(Ordering::Relaxed)
    }

    pub fn waiting_tasks(&self) -> u64 {
        self.slowpath_waiting_tasks.load(Ordering::Relaxed)
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

pub static METRICS: ResourceMetrics = ResourceMetrics::new();

pub struct ConnActiveGuard;

impl Drop for ConnActiveGuard {
    fn drop(&mut self) {
        METRICS
            .accepted_connections_active
            .fetch_sub(1, Ordering::Relaxed);
    }
}
