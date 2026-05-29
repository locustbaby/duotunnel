use crossbeam_utils::CachePadded;
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

thread_local! {
    static ROTATING_INDEX: Cell<usize> = const { Cell::new(0) };
}

pub struct InflightCounts {
    pending_opens: AtomicUsize,
    active_streams: AtomicUsize,
}

pub type InflightCounter = Arc<CachePadded<InflightCounts>>;

pub fn new_inflight_counter() -> InflightCounter {
    Arc::new(CachePadded::new(InflightCounts {
        pending_opens: AtomicUsize::new(0),
        active_streams: AtomicUsize::new(0),
    }))
}

enum InflightPhase {
    PendingOpen,
    ActiveStream,
}

pub struct InflightGuard {
    counter: InflightCounter,
    phase: InflightPhase,
}

impl Drop for InflightGuard {
    fn drop(&mut self) {
        let counter = match self.phase {
            InflightPhase::PendingOpen => &self.counter.pending_opens,
            InflightPhase::ActiveStream => &self.counter.active_streams,
        };
        let mut current = counter.load(Ordering::Relaxed);
        loop {
            if current == 0 {
                break;
            }
            match counter.compare_exchange_weak(
                current,
                current - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }
}

pub fn begin_inflight(counter: &InflightCounter) -> InflightGuard {
    counter.pending_opens.fetch_add(1, Ordering::Relaxed);
    InflightGuard {
        counter: counter.clone(),
        phase: InflightPhase::PendingOpen,
    }
}

impl InflightGuard {
    pub fn promote(mut self) -> Self {
        self.counter.pending_opens.fetch_sub(1, Ordering::Relaxed);
        self.counter.active_streams.fetch_add(1, Ordering::Relaxed);
        self.phase = InflightPhase::ActiveStream;
        self
    }
}

pub fn inflight_load(counter: &InflightCounter, ordering: Ordering) -> usize {
    counter.pending_opens.load(ordering) + counter.active_streams.load(ordering)
}

pub fn pick_least_inflight<T, H, I>(items: &[T], is_healthy: H, inflight: I) -> Option<&T>
where
    H: Fn(&T) -> bool,
    I: Fn(&T) -> usize,
{
    if items.is_empty() {
        return None;
    }
    let rotating = ROTATING_INDEX.with(|cell| {
        let val = cell.get();
        cell.set(val.wrapping_add(1));
        val
    });
    let mut best_item: Option<&T> = None;
    let mut best_val = usize::MAX;
    let len = items.len();
    for i in 0..len {
        let idx = (rotating + i) % len;
        let item = &items[idx];
        if is_healthy(item) {
            let val = inflight(item);
            if val < best_val {
                best_val = val;
                best_item = Some(item);
            }
        }
    }
    best_item
}
