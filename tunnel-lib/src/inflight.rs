use crossbeam_utils::CachePadded;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub type InflightCounter = Arc<CachePadded<AtomicUsize>>;

pub fn new_inflight_counter() -> InflightCounter {
    Arc::new(CachePadded::new(AtomicUsize::new(0)))
}

/// RAII guard: decrements an inflight counter on drop.
pub struct InflightGuard(InflightCounter);

impl Drop for InflightGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Increment `counter` and return a guard that decrements on drop.
pub fn begin_inflight(counter: &InflightCounter) -> InflightGuard {
    counter.fetch_add(1, Ordering::Relaxed);
    InflightGuard(counter.clone())
}

/// Pick the element with the smallest inflight count among healthy entries.
/// Returns `None` if the slice is empty or no entry is healthy.
pub fn pick_least_inflight<T, H, I>(items: &[T], is_healthy: H, inflight: I) -> Option<&T>
where
    H: Fn(&T) -> bool,
    I: Fn(&T) -> usize,
{
    items
        .iter()
        .filter(|t| is_healthy(t))
        .min_by_key(|t| inflight(t))
}
