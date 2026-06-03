use quinn::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tunnel_lib::{
    inflight_load, new_inflight_table, pick_p2c_inflight, InflightSlotId, InflightTable,
};

pub struct PooledConnection {
    pub conn: Connection,
    pub inflight_table: Arc<InflightTable>,
    pub slot_id: InflightSlotId,
}

enum PoolMsg {
    Push(Connection),
    Remove(usize),
    NextConn {
        excluded: Vec<usize>,
        reply: oneshot::Sender<Option<Arc<PooledConnection>>>,
    },
    PoolSize(oneshot::Sender<usize>),
}

pub struct EntryConnPool {
    tx: mpsc::Sender<PoolMsg>,
}

impl EntryConnPool {
    pub fn new(max_concurrent_streams: u32, connections: u32) -> Arc<Self> {
        let capacity = ((max_concurrent_streams as usize) * (connections as usize) * 2).max(1024);
        let inflight_table = new_inflight_table(capacity);
        let (tx, mut rx) = mpsc::channel(1024);

        tokio::spawn(async move {
            let mut conns: Vec<Arc<PooledConnection>> = Vec::new();
            let mut ids: HashMap<usize, usize> = HashMap::new();

            while let Some(msg) = rx.recv().await {
                match msg {
                    PoolMsg::Push(conn) => {
                        let stable_id = conn.stable_id();
                        if ids.contains_key(&stable_id) {
                            continue;
                        }
                        let Some(slot_id) = inflight_table.alloc_slot() else {
                            continue;
                        };
                        let index = conns.len();
                        ids.insert(stable_id, index);
                        conns.push(Arc::new(PooledConnection {
                            conn,
                            inflight_table: inflight_table.clone(),
                            slot_id,
                        }));
                    }
                    PoolMsg::Remove(stable_id) => {
                        if let Some(index) = ids.remove(&stable_id) {
                            let existing = &conns[index];
                            inflight_table.free_slot(existing.slot_id);
                            conns.swap_remove(index);
                            if index < conns.len() {
                                let swapped_id = conns[index].conn.stable_id();
                                ids.insert(swapped_id, index);
                            }
                        }
                    }
                    PoolMsg::NextConn { excluded, reply } => {
                        let chosen = pick_p2c_inflight(
                            conns.as_slice(),
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
                        ).cloned();
                        let _ = reply.send(chosen);
                    }
                    PoolMsg::PoolSize(reply) => {
                        let _ = reply.send(conns.len());
                    }
                }
            }
        });

        Arc::new(Self { tx })
    }

    pub async fn push(&self, conn: Connection) {
        let _ = self.tx.send(PoolMsg::Push(conn)).await;
    }

    pub async fn remove(&self, conn: &Connection) {
        let _ = self.tx.send(PoolMsg::Remove(conn.stable_id())).await;
    }

    pub async fn next_conn_excluding(&self, excluded: Vec<usize>) -> Option<Arc<PooledConnection>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self.tx.send(PoolMsg::NextConn { excluded, reply: reply_tx }).await.is_err() {
            return None;
        }
        reply_rx.await.ok().flatten()
    }

    pub async fn pool_size(&self) -> usize {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self.tx.send(PoolMsg::PoolSize(reply_tx)).await.is_err() {
            return 0;
        }
        reply_rx.await.unwrap_or(0)
    }
}
