use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Notify;

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
static CONNECTION_ACTIVITY: Notify = Notify::const_new();

pub struct ConnActiveGuard;

impl Drop for ConnActiveGuard {
    fn drop(&mut self) {
        let previous = METRICS
            .accepted_connections_active
            .fetch_sub(1, Ordering::Relaxed);
        if previous <= 1 {
            CONNECTION_ACTIVITY.notify_waiters();
        }
    }
}

pub async fn wait_for_resource_drain(timeout: Duration) -> bool {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if METRICS.active_connections() == 0 && METRICS.pending_streams() == 0 {
            return true;
        }
        let now = tokio::time::Instant::now();
        if now >= deadline {
            return false;
        }
        let wait = (deadline - now).min(Duration::from_millis(50));
        let notified = CONNECTION_ACTIVITY.notified();
        tokio::select! {
            _ = notified => {}
            _ = tokio::time::sleep(wait) => {}
        }
    }
}
