use arc_swap::ArcSwap;
use quinn::Connection;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tunnel_lib::{new_inflight_counter, pick_least_inflight, InflightCounter};

pub struct PooledConnection {
    pub conn: Connection,
    pub inflight: InflightCounter,
}

struct PoolState {
    conns: Vec<Arc<PooledConnection>>,
    ids: HashSet<usize>,
}

pub struct EntryConnPool {
    snapshot: ArcSwap<Vec<Arc<PooledConnection>>>,
    mu: Mutex<PoolState>,
}

impl EntryConnPool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            snapshot: ArcSwap::from_pointee(Vec::new()),
            mu: Mutex::new(PoolState {
                conns: Vec::new(),
                ids: HashSet::new(),
            }),
        })
    }

    pub fn push(&self, conn: Connection) {
        let mut g = self.mu.lock().unwrap();
        let stable_id = conn.stable_id();
        if !g.ids.insert(stable_id) {
            return;
        }
        g.conns.push(Arc::new(PooledConnection {
            conn,
            inflight: new_inflight_counter(),
        }));
        self.snapshot.store(Arc::new(g.conns.clone()));
    }

    pub fn remove(&self, conn: &Connection) {
        let mut g = self.mu.lock().unwrap();
        let stable_id = conn.stable_id();
        if g.ids.remove(&stable_id) {
            g.conns.retain(|c| c.conn.stable_id() != stable_id);
            self.snapshot.store(Arc::new(g.conns.clone()));
        }
    }

    pub fn next_conn_excluding(&self, excluded: &[usize]) -> Option<Arc<PooledConnection>> {
        let snap = self.snapshot.load();
        pick_least_inflight(
            snap.as_slice(),
            |c| {
                c.conn.close_reason().is_none()
                    && !excluded.contains(&c.conn.stable_id())
            },
            |c| c.inflight.load(Ordering::Relaxed),
        )
        .cloned()
    }

    pub fn pool_size(&self) -> usize {
        self.snapshot.load().len()
    }
}
