use arc_swap::ArcSwap;
use quinn::Connection;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tunnel_lib::{
    begin_inflight, new_inflight_counter, pick_least_inflight, InflightCounter, InflightGuard,
};

pub struct PooledConnection {
    pub conn: Connection,
    pub inflight: InflightCounter,
}

impl PooledConnection {
    pub fn begin_inflight(&self) -> InflightGuard {
        begin_inflight(&self.inflight)
    }

    pub fn inflight(&self) -> usize {
        self.inflight.load(Ordering::Relaxed)
    }
}

pub struct EntryConnPool {
    snapshot: ArcSwap<Vec<Arc<PooledConnection>>>,
    mu: Mutex<Vec<Arc<PooledConnection>>>,
}

impl EntryConnPool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            snapshot: ArcSwap::from_pointee(Vec::new()),
            mu: Mutex::new(Vec::new()),
        })
    }

    pub fn push(&self, conn: Connection) {
        let mut g = self.mu.lock().unwrap();
        if g.iter().any(|c| c.conn.stable_id() == conn.stable_id()) {
            return;
        }
        g.push(Arc::new(PooledConnection {
            conn,
            inflight: new_inflight_counter(),
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
        pick_least_inflight(
            snap.as_slice(),
            |c| c.conn.close_reason().is_none(),
            |c| c.inflight(),
        )
        .cloned()
    }

    pub fn pool_size(&self) -> usize {
        self.snapshot.load().len()
    }
}
