use crossbeam_utils::CachePadded;
use parking_lot::Mutex;
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

thread_local! {
    static ROTATING_INDEX: Cell<usize> = const { Cell::new(0) };
}

pub struct InflightSlot {
    pending_opens: AtomicUsize,
    active_streams: AtomicUsize,
    notify: Arc<Notify>,
}

pub type InflightSlotId = usize;

pub struct InflightTable {
    slots: Vec<CachePadded<InflightSlot>>,
    free: Mutex<Vec<InflightSlotId>>,
}

pub struct InflightGuard {
    table: Arc<InflightTable>,
    slot_id: InflightSlotId,
    phase: InflightPhase,
}

enum InflightPhase {
    PendingOpen,
    ActiveStream,
}

impl InflightTable {
    pub fn new(size: usize) -> Self {
        let len = size.max(1);
        let mut free = Vec::with_capacity(len);
        let mut slots = Vec::with_capacity(len);
        for slot_id in 0..len {
            free.push(slot_id);
            slots.push(CachePadded::new(InflightSlot {
                pending_opens: AtomicUsize::new(0),
                active_streams: AtomicUsize::new(0),
                notify: Arc::new(Notify::new()),
            }));
        }
        free.reverse();
        Self {
            slots,
            free: Mutex::new(free),
        }
    }

    pub fn alloc_slot(&self) -> Option<InflightSlotId> {
        self.free.lock().pop()
    }

    pub fn free_slot(&self, slot_id: InflightSlotId) {
        if slot_id < self.slots.len() {
            self.free.lock().push(slot_id);
        }
    }

    fn slot(&self, slot_id: InflightSlotId) -> &InflightSlot {
        &self.slots[slot_id]
    }
}

pub fn new_inflight_table(size: usize) -> Arc<InflightTable> {
    Arc::new(InflightTable::new(size))
}

pub fn begin_inflight(table: &Arc<InflightTable>, slot_id: InflightSlotId) -> InflightGuard {
    table
        .slot(slot_id)
        .pending_opens
        .fetch_add(1, Ordering::Relaxed);
    InflightGuard {
        table: table.clone(),
        slot_id,
        phase: InflightPhase::PendingOpen,
    }
}

impl InflightGuard {
    pub fn promote(mut self) -> Self {
        let slot = self.table.slot(self.slot_id);
        slot.pending_opens.fetch_sub(1, Ordering::Relaxed);
        slot.active_streams.fetch_add(1, Ordering::Relaxed);
        self.phase = InflightPhase::ActiveStream;
        self
    }
}

impl Drop for InflightGuard {
    fn drop(&mut self) {
        let slot = self.table.slot(self.slot_id);
        let counter = match self.phase {
            InflightPhase::PendingOpen => &slot.pending_opens,
            InflightPhase::ActiveStream => &slot.active_streams,
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
                Ok(_) => {
                    slot.notify.notify_one();
                    break;
                }
                Err(actual) => current = actual,
            }
        }
    }
}

pub fn inflight_load(
    table: &Arc<InflightTable>,
    slot_id: InflightSlotId,
    ordering: Ordering,
) -> usize {
    let slot = table.slot(slot_id);
    slot.pending_opens.load(ordering) + slot.active_streams.load(ordering)
}

pub fn inflight_notify(table: &Arc<InflightTable>, slot_id: InflightSlotId) -> Arc<Notify> {
    table.slot(slot_id).notify.clone()
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
