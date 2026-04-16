use arc_swap::ArcSwap;
use crossbeam_utils::CachePadded;
use quinn::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct PooledConnection {
    pub conn: Connection,
    pub inflight: Arc<CachePadded<AtomicUsize>>,
}

pub struct InflightGuard(Arc<CachePadded<AtomicUsize>>);

impl Drop for InflightGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

impl PooledConnection {
    pub fn begin_inflight(&self) -> InflightGuard {
        self.inflight.fetch_add(1, Ordering::Relaxed);
        InflightGuard(self.inflight.clone())
    }

    pub fn inflight(&self) -> usize {
        self.inflight.load(Ordering::Relaxed)
    }
}

pub struct EntryConnPool {
    snapshot: ArcSwap<Vec<Arc<PooledConnection>>>,
    mu: Mutex<Vec<Arc<PooledConnection>>>,
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
        g.push(Arc::new(PooledConnection {
            conn,
            inflight: Arc::new(CachePadded::new(AtomicUsize::new(0))),
        }));
        self.snapshot.store(Arc::new(g.clone()));
    }

    pub fn remove(&self, conn: &Connection) {
        let mut g = self.mu.lock().unwrap();
        g.retain(|c| c.conn.stable_id() != conn.stable_id());
        self.snapshot.store(Arc::new(g.clone()));
    }

    pub fn next_conn(&self) -> Option<Arc<PooledConnection>> {
        let snap = self.snapshot.load();
        let len = snap.len();
        if len == 0 {
            return None;
        }
        let start = self.counter.fetch_add(1, Ordering::Relaxed);
        for i in 0..len {
            let c = &snap[(start + i) % len];
            if c.conn.close_reason().is_none() {
                return Some(c.clone());
            }
        }
        None
    }

    pub fn pool_size(&self) -> usize {
        self.snapshot.load().len()
    }
}
