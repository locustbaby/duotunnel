use crossbeam_utils::CachePadded;
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

thread_local! {
    static ROTATING_INDEX: Cell<usize> = const { Cell::new(0) };
}

pub type InflightCounter = Arc<CachePadded<AtomicUsize>>;

pub fn new_inflight_counter() -> InflightCounter {
    Arc::new(CachePadded::new(AtomicUsize::new(0)))
}

pub struct InflightGuard(InflightCounter);

impl Drop for InflightGuard {
    fn drop(&mut self) {
        let mut current = self.0.load(Ordering::Relaxed);
        loop {
            if current == 0 {
                break;
            }
            match self.0.compare_exchange_weak(
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
    counter.fetch_add(1, Ordering::Relaxed);
    InflightGuard(counter.clone())
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
