use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Notify;

pub struct ResourceMetrics {
    pub accepted_connections_active: AtomicU64,
    pub stream_pending_queue_depth: AtomicU64,
    pub slowpath_waiting_tasks: AtomicU64,
    reverse_streams_active: AtomicU64,
    http_requests_active: AtomicU64,
    udp_tasks_active: AtomicU64,
}

impl ResourceMetrics {
    pub const fn new() -> Self {
        Self {
            accepted_connections_active: AtomicU64::new(0),
            stream_pending_queue_depth: AtomicU64::new(0),
            slowpath_waiting_tasks: AtomicU64::new(0),
            reverse_streams_active: AtomicU64::new(0),
            http_requests_active: AtomicU64::new(0),
            udp_tasks_active: AtomicU64::new(0),
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

    pub fn reverse_streams(&self) -> u64 {
        self.reverse_streams_active.load(Ordering::Relaxed)
    }

    pub fn http_requests(&self) -> u64 {
        self.http_requests_active.load(Ordering::Relaxed)
    }

    pub fn udp_tasks(&self) -> u64 {
        self.udp_tasks_active.load(Ordering::Relaxed)
    }

    fn is_drained(&self) -> bool {
        self.active_connections() == 0
            && self.pending_streams() == 0
            && self.reverse_streams() == 0
            && self.http_requests() == 0
            && self.udp_tasks() == 0
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

pub static METRICS: ResourceMetrics = ResourceMetrics::new();
static CONNECTION_ACTIVITY: Notify = Notify::const_new();

#[derive(Clone, Copy)]
pub enum TrackedResource {
    ReverseStream,
    HttpRequest,
    UdpTask,
}

pub struct ResourceGuard {
    resource: TrackedResource,
}

pub fn track_resource(resource: TrackedResource) -> ResourceGuard {
    let counter = match resource {
        TrackedResource::ReverseStream => &METRICS.reverse_streams_active,
        TrackedResource::HttpRequest => &METRICS.http_requests_active,
        TrackedResource::UdpTask => &METRICS.udp_tasks_active,
    };
    counter.fetch_add(1, Ordering::Relaxed);
    ResourceGuard { resource }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        let counter = match self.resource {
            TrackedResource::ReverseStream => &METRICS.reverse_streams_active,
            TrackedResource::HttpRequest => &METRICS.http_requests_active,
            TrackedResource::UdpTask => &METRICS.udp_tasks_active,
        };
        counter.fetch_sub(1, Ordering::Relaxed);
        CONNECTION_ACTIVITY.notify_waiters();
    }
}

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
        if METRICS.is_drained() {
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
