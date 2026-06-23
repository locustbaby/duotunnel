use quinn::Connection;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tunnel_lib::{
    inflight_load, new_inflight_table, pick_from_preferred_shards, pick_p2c_inflight_owned,
    stable_shard_index, ConnectionHandle,
};

pub struct PooledConnection {
    pub handle: Arc<ConnectionHandle>,
}

struct PoolShard {
    conns: Vec<Arc<PooledConnection>>,
    ids: HashMap<usize, usize>,
}

impl PoolShard {
    fn new() -> Self {
        Self {
            conns: Vec::new(),
            ids: HashMap::new(),
        }
    }

    fn len(&self) -> usize {
        self.conns.len()
    }
}

enum PoolMsg {
    Push {
        conn: Connection,
        shard_id: usize,
    },
    Remove(usize),
    NextConn {
        preferred_shard: usize,
        excluded: Vec<usize>,
        reply: oneshot::Sender<Option<Arc<PooledConnection>>>,
    },
    PoolSize(oneshot::Sender<usize>),
}

pub struct EntryConnPool {
    tx: mpsc::Sender<PoolMsg>,
    shard_count: usize,
    next_push_shard: AtomicUsize,
    egress_rules: RwLock<Vec<tunnel_lib::EgressVhostRuleDef>>,
}

impl EntryConnPool {
    pub fn new(max_concurrent_streams: u32, connections: u32, shard_count: usize) -> Arc<Self> {
        let capacity = ((max_concurrent_streams as usize) * (connections as usize) * 2).max(1024);
        let inflight_table = new_inflight_table(capacity);
        let shard_count = shard_count.max(1);
        let (tx, mut rx) = mpsc::channel(1024);

        tokio::spawn(async move {
            let mut shards = (0..shard_count)
                .map(|_| PoolShard::new())
                .collect::<Vec<_>>();

            while let Some(msg) = rx.recv().await {
                match msg {
                    PoolMsg::Push { conn, shard_id } => {
                        let stable_id = conn.stable_id();
                        let shard = &mut shards[shard_id % shard_count];
                        if shard.ids.contains_key(&stable_id) {
                            continue;
                        }
                        let Some(slot_id) = inflight_table.alloc_slot() else {
                            continue;
                        };
                        let index = shard.conns.len();
                        shard.ids.insert(stable_id, index);
                        let handle = ConnectionHandle::spawn(
                            conn,
                            inflight_table.clone(),
                            slot_id,
                            shard_id % shard_count,
                            max_concurrent_streams,
                        );
                        shard.conns.push(Arc::new(PooledConnection { handle }));
                    }
                    PoolMsg::Remove(stable_id) => {
                        for shard in &mut shards {
                            if let Some(index) = shard.ids.remove(&stable_id) {
                                let existing = &shard.conns[index];
                                inflight_table.free_slot(existing.handle.slot_id());
                                shard.conns.swap_remove(index);
                                if index < shard.conns.len() {
                                    let swapped_id = shard.conns[index].handle.stable_id();
                                    shard.ids.insert(swapped_id, index);
                                }
                                break;
                            }
                        }
                    }
                    PoolMsg::NextConn {
                        preferred_shard,
                        excluded,
                        reply,
                    } => {
                        let chosen = pick_from_preferred_shards(
                            shards.as_slice(),
                            preferred_shard,
                            |shard| {
                                pick_p2c_inflight_owned(
                                    shard.conns.as_slice(),
                                    |c| {
                                        c.handle.close_reason().is_none()
                                            && !excluded.contains(&c.handle.stable_id())
                                    },
                                    |c| {
                                        inflight_load(
                                            c.handle.inflight_table(),
                                            c.handle.slot_id(),
                                            Ordering::Relaxed,
                                        )
                                    },
                                )
                            },
                        );
                        let _ = reply.send(chosen);
                    }
                    PoolMsg::PoolSize(reply) => {
                        let _ = reply.send(shards.iter().map(PoolShard::len).sum());
                    }
                }
            }
        });

        Arc::new(Self {
            tx,
            shard_count,
            next_push_shard: AtomicUsize::new(0),
            egress_rules: RwLock::new(Vec::new()),
        })
    }

    pub fn shard_for_hash<T: Hash>(&self, value: &T) -> usize {
        stable_shard_index(value, self.shard_count)
    }

    pub fn set_egress_rules(&self, rules: Vec<tunnel_lib::EgressVhostRuleDef>) {
        *self.egress_rules.write() = rules;
    }

    pub fn egress_rules(&self) -> Vec<tunnel_lib::EgressVhostRuleDef> {
        self.egress_rules.read().clone()
    }

    pub async fn push(&self, conn: Connection) {
        let shard_id = self.next_push_shard.fetch_add(1, Ordering::Relaxed) % self.shard_count;
        let _ = self.tx.send(PoolMsg::Push { conn, shard_id }).await;
    }

    pub async fn remove(&self, conn: &Connection) {
        self.remove_stable_id(conn.stable_id()).await;
    }

    pub async fn remove_stable_id(&self, stable_id: usize) {
        let _ = self.tx.send(PoolMsg::Remove(stable_id)).await;
    }

    pub async fn next_conn_for_shard_excluding(
        &self,
        preferred_shard: usize,
        excluded: Vec<usize>,
    ) -> Option<Arc<PooledConnection>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .tx
            .send(PoolMsg::NextConn {
                preferred_shard: preferred_shard % self.shard_count,
                excluded,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
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
