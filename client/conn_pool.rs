use arc_swap::ArcSwap;
use quinn::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// A lock-free, live-updating pool of QUIC connections shared across tasks.
///
/// Writers (supervisor slots) call `push` / `remove` to register connections.
/// Readers (entry listener) call `next_conn` which is a single atomic load + increment — no locks.
///
/// Internally uses ArcSwap (RCU): reads pay one SeqCst load; writes rebuild the Vec under a
/// std::sync::Mutex and swap atomically, so readers always see a consistent snapshot.
pub struct EntryConnPool {
    snapshot: ArcSwap<Vec<Connection>>,
    mu: Mutex<Vec<Connection>>,
    counter: AtomicUsize,
}

impl EntryConnPool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            snapshot: ArcSwap::from_pointee(Vec::new()),
            mu: Mutex::new(Vec::new()),
            counter: AtomicUsize::new(0),
        })
    }

    /// Register a new connection. Called by each supervisor slot after login succeeds.
    pub fn push(&self, conn: Connection) {
        let mut g = self.mu.lock().unwrap();
        g.push(conn);
        self.snapshot.store(Arc::new(g.clone()));
    }

    /// Remove a connection when a slot disconnects.
    pub fn remove(&self, conn: &Connection) {
        let mut g = self.mu.lock().unwrap();
        g.retain(|c: &Connection| c.stable_id() != conn.stable_id());
        self.snapshot.store(Arc::new(g.clone()));
    }

    /// Pick the next live connection in round-robin order.
    /// Returns `None` only if no connections are registered yet.
    pub fn next_conn(&self) -> Option<Connection> {
        let snap = self.snapshot.load();
        let len = snap.len();
        if len == 0 {
            return None;
        }
        let start = self.counter.fetch_add(1, Ordering::Relaxed);
        // Walk up to `len` slots; skip connections that are already closing.
        for i in 0..len {
            let conn = &snap[(start + i) % len];
            if conn.close_reason().is_none() {
                return Some(conn.clone());
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.snapshot.load().is_empty()
    }

    /// Number of connections currently in the pool. Used to bound retry loops.
    pub fn pool_size(&self) -> usize {
        self.snapshot.load().len()
    }
}
