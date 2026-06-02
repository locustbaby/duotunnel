use arc_swap::ArcSwap;
use quinn::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tunnel_lib::{
    inflight_load, new_inflight_table, pick_p2c_inflight, InflightSlotId, InflightTable,
};

pub struct PooledConnection {
    pub conn: Connection,
    pub inflight_table: Arc<InflightTable>,
    pub slot_id: InflightSlotId,
}

struct PoolState {
    conns: Vec<Arc<PooledConnection>>,
    ids: HashMap<usize, usize>,
}

pub struct EntryConnPool {
    snapshot: ArcSwap<Vec<Arc<PooledConnection>>>,
    mu: Mutex<PoolState>,
    inflight_table: Arc<InflightTable>,
}

impl EntryConnPool {
    pub fn new(max_concurrent_streams: u32, connections: u32) -> Arc<Self> {
        let capacity = ((max_concurrent_streams as usize) * (connections as usize) * 2).max(1024);
        Arc::new(Self {
            snapshot: ArcSwap::from_pointee(Vec::new()),
            mu: Mutex::new(PoolState {
                conns: Vec::new(),
                ids: HashMap::new(),
            }),
            inflight_table: new_inflight_table(capacity),
        })
    }

    pub fn push(&self, conn: Connection) {
        let mut g = self.mu.lock().unwrap();
        let stable_id = conn.stable_id();
        if g.ids.contains_key(&stable_id) {
            return;
        }
        let Some(slot_id) = self.inflight_table.alloc_slot() else {
            tracing::error!(conn_id = stable_id, "Inflight slot table exhausted");
            return;
        };
        let index = g.conns.len();
        g.ids.insert(stable_id, index);
        g.conns.push(Arc::new(PooledConnection {
            conn,
            inflight_table: self.inflight_table.clone(),
            slot_id,
        }));
        self.snapshot.store(Arc::new(g.conns.clone()));
    }

    pub fn remove(&self, conn: &Connection) {
        let mut g = self.mu.lock().unwrap();
        let stable_id = conn.stable_id();
        if let Some(index) = g.ids.remove(&stable_id) {
            let existing = &g.conns[index];
            self.inflight_table.free_slot(existing.slot_id);
            g.conns.swap_remove(index);
            if index < g.conns.len() {
                let swapped_id = g.conns[index].conn.stable_id();
                g.ids.insert(swapped_id, index);
            }
            self.snapshot.store(Arc::new(g.conns.clone()));
        }
    }

    pub fn next_conn_excluding(&self, excluded: &[usize]) -> Option<Arc<PooledConnection>> {
        let snap = self.snapshot.load();
        pick_p2c_inflight(
            snap.as_slice(),
            32,
            3,
            |c| c.conn.close_reason().is_none() && !excluded.contains(&c.conn.stable_id()),
            |c| {
                inflight_load(
                    &c.inflight_table,
                    c.slot_id,
                    std::sync::atomic::Ordering::Relaxed,
                )
            },
        )
        .cloned()
    }

    pub fn pool_size(&self) -> usize {
        self.snapshot.load().len()
    }
}
