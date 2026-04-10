use arc_swap::ArcSwap;
use quinn::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
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

    pub fn push(&self, conn: Connection) {
        let mut g = self.mu.lock().unwrap();
        g.push(conn);
        self.snapshot.store(Arc::new(g.clone()));
    }

    pub fn remove(&self, conn: &Connection) {
        let mut g = self.mu.lock().unwrap();
        g.retain(|c: &Connection| c.stable_id() != conn.stable_id());
        self.snapshot.store(Arc::new(g.clone()));
    }

    pub fn next_conn(&self) -> Option<Connection> {
        let snap = self.snapshot.load();
        let len = snap.len();
        if len == 0 {
            return None;
        }
        let start = self.counter.fetch_add(1, Ordering::Relaxed);
        for i in 0..len {
            let conn = &snap[(start + i) % len];
            if conn.close_reason().is_none() {
                return Some(conn.clone());
            }
        }
        None
    }

    pub fn pool_size(&self) -> usize {
        self.snapshot.load().len()
    }
}
