use crossbeam_utils::CachePadded;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use tokio::sync::Notify;

thread_local! {
    static ROTATING_INDEX: Cell<usize> = const { Cell::new(0) };
}

use crossbeam_queue::ArrayQueue;
use std::sync::OnceLock;

fn global_connection_state_pool() -> &'static ArrayQueue<Arc<ConnectionState>> {
    static POOL: OnceLock<ArrayQueue<Arc<ConnectionState>>> = OnceLock::new();
    POOL.get_or_init(|| ArrayQueue::new(2048))
}

struct InflightCounters {
    pending_opens: AtomicUsize,
    active_streams: AtomicUsize,
    notify: Arc<Notify>,
}

pub struct ConnectionState {
    id: AtomicU64,
    counters: CachePadded<InflightCounters>,
    admission: AtomicUsize,
    registered: AtomicBool,
    owner: parking_lot::Mutex<Weak<InflightTable>>,
}

const RETIRED_BIT: usize = 1usize << (usize::BITS - 1);
const ADMISSION_COUNT_MASK: usize = RETIRED_BIT - 1;

pub struct InflightTable {
    capacity: usize,
    registered: AtomicUsize,
    next_id: AtomicU64,
}

pub struct InflightGuard {
    state: Arc<ConnectionState>,
    phase: InflightPhase,
}

enum InflightPhase {
    PendingOpen,
    ActiveStream,
}

impl InflightTable {
    pub fn new(size: usize) -> Self {
        Self {
            capacity: size.max(1),
            registered: AtomicUsize::new(0),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn allocate(self: &Arc<Self>) -> Option<Arc<ConnectionState>> {
        let mut registered = self.registered.load(Ordering::Acquire);
        loop {
            if registered >= self.capacity {
                return None;
            }
            match self.registered.compare_exchange_weak(
                registered,
                registered + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual) => registered = actual,
            }
        }

        let new_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let pool = global_connection_state_pool();
        while let Some(state) = pool.pop() {
            if Arc::strong_count(&state) == 1 {
                state.id.store(new_id, Ordering::Relaxed);
                state.counters.pending_opens.store(0, Ordering::Relaxed);
                state.counters.active_streams.store(0, Ordering::Relaxed);
                state.admission.store(0, Ordering::Relaxed);
                state.registered.store(true, Ordering::Relaxed);
                *state.owner.lock() = Arc::downgrade(self);
                return Some(state);
            }
        }

        Some(Arc::new(ConnectionState {
            id: AtomicU64::new(new_id),
            counters: CachePadded::new(InflightCounters {
                pending_opens: AtomicUsize::new(0),
                active_streams: AtomicUsize::new(0),
                notify: Arc::new(Notify::new()),
            }),
            admission: AtomicUsize::new(0),
            registered: AtomicBool::new(true),
            owner: parking_lot::Mutex::new(Arc::downgrade(self)),
        }))
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn registered_connections(&self) -> usize {
        self.registered.load(Ordering::Acquire)
    }
}

impl ConnectionState {
    pub fn id(&self) -> u64 {
        self.id.load(Ordering::Relaxed)
    }

    pub fn is_selectable(&self) -> bool {
        self.admission.load(Ordering::Acquire) & RETIRED_BIT == 0
    }

    pub fn retire(&self) -> bool {
        self.admission.fetch_or(RETIRED_BIT, Ordering::AcqRel);
        self.release_registration()
    }

    fn try_acquire_admission(&self) -> bool {
        let mut admission = self.admission.load(Ordering::Acquire);
        loop {
            if admission & RETIRED_BIT != 0
                || admission & ADMISSION_COUNT_MASK == ADMISSION_COUNT_MASK
            {
                return false;
            }
            match self.admission.compare_exchange_weak(
                admission,
                admission + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(actual) => admission = actual,
            }
        }
    }

    fn release_registration(&self) -> bool {
        if self.registered.swap(false, Ordering::AcqRel) {
            if let Some(owner) = self.owner.lock().upgrade() {
                let previous = owner.registered.fetch_sub(1, Ordering::AcqRel);
                debug_assert!(previous > 0, "inflight registration count underflow");
            }
            true
        } else {
            false
        }
    }
}

impl Drop for ConnectionState {
    fn drop(&mut self) {
        self.release_registration();
    }
}

pub fn new_inflight_table(size: usize) -> Arc<InflightTable> {
    Arc::new(InflightTable::new(size))
}

pub fn begin_inflight(state: &Arc<ConnectionState>) -> Option<InflightGuard> {
    if !state.try_acquire_admission() {
        return None;
    }
    state.counters.pending_opens.fetch_add(1, Ordering::Relaxed);
    Some(InflightGuard {
        state: state.clone(),
        phase: InflightPhase::PendingOpen,
    })
}

impl InflightGuard {
    pub fn promote(mut self) -> Self {
        let previous = self
            .state
            .counters
            .pending_opens
            .fetch_sub(1, Ordering::Relaxed);
        debug_assert!(previous > 0, "pending inflight count underflow");
        self.state
            .counters
            .active_streams
            .fetch_add(1, Ordering::Relaxed);
        self.phase = InflightPhase::ActiveStream;
        self
    }
}

impl Drop for InflightGuard {
    fn drop(&mut self) {
        let counter = match self.phase {
            InflightPhase::PendingOpen => &self.state.counters.pending_opens,
            InflightPhase::ActiveStream => &self.state.counters.active_streams,
        };
        let previous = counter.fetch_sub(1, Ordering::Relaxed);
        debug_assert!(previous > 0, "inflight count underflow");
        let admission = self.state.admission.fetch_sub(1, Ordering::Release);
        debug_assert!(admission & ADMISSION_COUNT_MASK > 0);
        self.state.counters.notify.notify_one();
    }
}

pub fn inflight_load(state: &ConnectionState, ordering: Ordering) -> usize {
    state.counters.pending_opens.load(ordering) + state.counters.active_streams.load(ordering)
}

pub fn inflight_notify(state: &ConnectionState) -> Arc<Notify> {
    state.counters.notify.clone()
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

/// Uses the Power of Two Choices (P2C) algorithm for large lists.
/// For lists smaller than or equal to `threshold`, delegates to `pick_least_inflight` (O(N) scan).
/// For larger lists, it randomly picks two items and returns the healthier one with lower inflight.
/// If neither item is healthy, it will retry up to `max_retries` times.
/// If all retries fail to find a healthy item, it will fallback to the O(N) scan `pick_least_inflight`.
pub fn pick_p2c_inflight<T, H, I>(
    items: &[T],
    threshold: usize,
    max_retries: usize,
    is_healthy: H,
    inflight: I,
) -> Option<&T>
where
    H: Fn(&T) -> bool,
    I: Fn(&T) -> usize,
{
    let len = items.len();
    if len <= threshold || len < 2 {
        return pick_least_inflight(items, is_healthy, inflight);
    }

    // Attempt P2C bounded by `max_retries`
    for _ in 0..=max_retries {
        let idx1 = fastrand::usize(..len);
        let idx2 = {
            let r = fastrand::usize(..len - 1);
            if r >= idx1 {
                r + 1
            } else {
                r
            }
        };

        let c1 = &items[idx1];
        let c2 = &items[idx2];
        let c1_healthy = is_healthy(c1);
        let c2_healthy = is_healthy(c2);

        match (c1_healthy, c2_healthy) {
            (true, true) => {
                if inflight(c1) <= inflight(c2) {
                    return Some(c1);
                } else {
                    return Some(c2);
                }
            }
            (true, false) => return Some(c1),
            (false, true) => return Some(c2),
            (false, false) => continue, // both unhealthy, try again
        }
    }

    // All random P2C picks were unhealthy, fallback to O(N) scan
    pick_least_inflight(items, is_healthy, inflight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_connection_guard_cannot_touch_replacement() {
        let table = new_inflight_table(1);
        let old = table.allocate().unwrap();
        let old_guard = begin_inflight(&old).unwrap().promote();

        assert!(old.retire());
        let replacement = table.allocate().unwrap();
        let replacement_guard = begin_inflight(&replacement).unwrap().promote();
        assert_ne!(old.id(), replacement.id());
        assert_eq!(inflight_load(&replacement, Ordering::Relaxed), 1);

        drop(old_guard);

        assert_eq!(inflight_load(&old, Ordering::Relaxed), 0);
        assert_eq!(inflight_load(&replacement, Ordering::Relaxed), 1);
        drop(replacement_guard);
    }

    #[test]
    fn concurrent_old_guard_drops_do_not_modify_new_connection() {
        let table = new_inflight_table(1);
        let old = table.allocate().unwrap();
        let guards: Vec<_> = (0..64)
            .map(|_| begin_inflight(&old).unwrap().promote())
            .collect();
        old.retire();

        let replacement = table.allocate().unwrap();
        let replacement_guard = begin_inflight(&replacement).unwrap().promote();
        let threads: Vec<_> = guards
            .into_iter()
            .map(|guard| std::thread::spawn(move || drop(guard)))
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }

        assert_eq!(inflight_load(&old, Ordering::Relaxed), 0);
        assert_eq!(inflight_load(&replacement, Ordering::Relaxed), 1);
        drop(replacement_guard);
    }

    #[test]
    fn retirement_is_idempotent_and_releases_registration_capacity_once() {
        let table = new_inflight_table(1);
        let state = table.allocate().unwrap();
        assert_eq!(table.capacity(), 1);
        assert_eq!(table.registered_connections(), 1);
        assert!(table.allocate().is_none());

        assert!(state.retire());
        assert!(!state.retire());
        assert!(begin_inflight(&state).is_none());
        assert_eq!(table.registered_connections(), 0);
        let replacement = table.allocate().unwrap();
        assert_eq!(table.registered_connections(), 1);
        drop(replacement);
    }
}
