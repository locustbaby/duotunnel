use anyhow::{anyhow, ensure, Result};
use arc_swap::ArcSwap;
use quinn::Connection;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tunnel_lib::{
    inflight_load, new_inflight_table, pick_from_preferred_shards, pick_p2c_inflight_owned,
    stable_shard_index, ConnectionHandle, NegotiatedProtocol, VhostRouter,
};

const MIN_CLIENT_INFLIGHT_SLOTS: usize = 1024;
const MAX_CLIENT_INFLIGHT_SLOTS: usize = 262_144;

pub struct PooledConnection {
    pub handle: Arc<ConnectionHandle>,
    // Unread until the first capability-gated feature lands; stored now so
    // gating can happen at the connection-selection site without re-plumbing.
    #[allow(dead_code)]
    pub negotiated: NegotiatedProtocol,
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

    fn snapshot(&self) -> Arc<Vec<Arc<PooledConnection>>> {
        Arc::new(self.conns.clone())
    }
}

enum PoolMsg {
    Push {
        conn: Connection,
        shard_id: usize,
        negotiated: NegotiatedProtocol,
        reply: oneshot::Sender<()>,
    },
    Remove {
        stable_id: usize,
        reply: oneshot::Sender<()>,
    },
}

#[derive(Default)]
struct AllowedHostIndex {
    hosts: VhostRouter<()>,
}

impl AllowedHostIndex {
    fn build(rules: Vec<tunnel_lib::EgressVhostRuleDef>) -> Self {
        let hosts = VhostRouter::new();
        for rule in rules {
            hosts.add_route(&rule.match_host, ());
        }
        Self { hosts }
    }

    fn is_rejected_host(&self, host: &str) -> bool {
        self.hosts.get(host).is_none()
    }
}

pub struct EntryConnPool {
    tx: mpsc::Sender<PoolMsg>,
    shard_count: usize,
    shards: Vec<Arc<ArcSwap<Vec<Arc<PooledConnection>>>>>,
    total_size: Arc<AtomicUsize>,
    egress_rules: ArcSwap<AllowedHostIndex>,
}

impl EntryConnPool {
    pub fn new(
        max_concurrent_streams: u32,
        max_pending_streams: usize,
        connections: u32,
        shard_count: usize,
    ) -> Result<Arc<Self>> {
        let capacity = checked_inflight_capacity(max_concurrent_streams, connections)?;
        let inflight_table = new_inflight_table(capacity);
        let shard_count = shard_count.max(1);
        let (tx, mut rx) = mpsc::channel(1024);
        let snapshots = (0..shard_count)
            .map(|_| Arc::new(ArcSwap::from_pointee(Vec::new())))
            .collect::<Vec<_>>();
        let snapshots_for_actor = snapshots.clone();
        let total_size = Arc::new(AtomicUsize::new(0));
        let total_size_for_actor = total_size.clone();

        tokio::spawn(async move {
            let mut shards = (0..shard_count)
                .map(|_| PoolShard::new())
                .collect::<Vec<_>>();

            while let Some(msg) = rx.recv().await {
                match msg {
                    PoolMsg::Push {
                        conn,
                        shard_id,
                        negotiated,
                        reply,
                    } => {
                        let stable_id = conn.stable_id();
                        let shard_id = shard_id % shard_count;
                        let shard = &mut shards[shard_id];
                        if shard.ids.contains_key(&stable_id) {
                            let _ = reply.send(());
                            continue;
                        }
                        let Some(slot_id) = inflight_table.alloc_slot() else {
                            let _ = reply.send(());
                            continue;
                        };
                        let index = shard.conns.len();
                        shard.ids.insert(stable_id, index);
                        let handle = ConnectionHandle::spawn(
                            conn,
                            inflight_table.clone(),
                            slot_id,
                            shard_id,
                            max_concurrent_streams,
                            max_pending_streams,
                        );
                        shard
                            .conns
                            .push(Arc::new(PooledConnection { handle, negotiated }));
                        snapshots_for_actor[shard_id].store(shard.snapshot());
                        total_size_for_actor.fetch_add(1, Ordering::Release);
                        let _ = reply.send(());
                    }
                    PoolMsg::Remove { stable_id, reply } => {
                        for (shard_id, shard) in shards.iter_mut().enumerate() {
                            if let Some(index) = shard.ids.remove(&stable_id) {
                                let existing = &shard.conns[index];
                                inflight_table.free_slot(existing.handle.slot_id());
                                shard.conns.swap_remove(index);
                                if index < shard.conns.len() {
                                    let swapped_id = shard.conns[index].handle.stable_id();
                                    shard.ids.insert(swapped_id, index);
                                }
                                snapshots_for_actor[shard_id].store(shard.snapshot());
                                total_size_for_actor.fetch_sub(1, Ordering::Release);
                                break;
                            }
                        }
                        let _ = reply.send(());
                    }
                }
            }
        });

        Ok(Arc::new(Self {
            tx,
            shard_count,
            shards: snapshots,
            total_size,
            egress_rules: ArcSwap::from_pointee(AllowedHostIndex::default()),
        }))
    }

    pub fn shard_for_hash<T: Hash>(&self, value: &T) -> usize {
        stable_shard_index(value, self.shard_count)
    }

    pub fn set_egress_rules(&self, rules: Vec<tunnel_lib::EgressVhostRuleDef>) {
        self.egress_rules
            .store(Arc::new(AllowedHostIndex::build(rules)));
    }

    pub fn is_rejected_host(&self, host: &str) -> bool {
        self.egress_rules.load().is_rejected_host(host)
    }

    pub async fn push(&self, conn: Connection, negotiated: NegotiatedProtocol) {
        let shard_id = stable_shard_index(&conn.stable_id(), self.shard_count);
        let (reply_tx, reply_rx) = oneshot::channel();
        let _ = self
            .tx
            .send(PoolMsg::Push {
                conn,
                shard_id,
                negotiated,
                reply: reply_tx,
            })
            .await;
        let _ = reply_rx.await;
    }

    pub async fn remove(&self, conn: &Connection) {
        self.remove_stable_id(conn.stable_id()).await;
    }

    pub async fn remove_stable_id(&self, stable_id: usize) {
        let (reply_tx, reply_rx) = oneshot::channel();
        let _ = self
            .tx
            .send(PoolMsg::Remove {
                stable_id,
                reply: reply_tx,
            })
            .await;
        let _ = reply_rx.await;
    }

    pub fn next_conn_for_shard_excluding(
        &self,
        preferred_shard: usize,
        excluded: &[usize],
    ) -> Option<Arc<PooledConnection>> {
        pick_from_preferred_shards(&self.shards, preferred_shard, |shard| {
            let snapshot = shard.load();
            pick_p2c_inflight_owned(
                snapshot.as_slice(),
                |c| c.handle.close_reason().is_none() && !excluded.contains(&c.handle.stable_id()),
                |c| {
                    inflight_load(
                        c.handle.inflight_table(),
                        c.handle.slot_id(),
                        Ordering::Relaxed,
                    )
                },
            )
        })
    }

    pub fn pool_size(&self) -> usize {
        self.total_size.load(Ordering::Acquire)
    }
}

fn checked_inflight_capacity(max_concurrent_streams: u32, connections: u32) -> Result<usize> {
    let streams = usize::try_from(max_concurrent_streams)
        .map_err(|_| anyhow!("max_concurrent_streams does not fit this platform"))?;
    let connections = usize::try_from(connections)
        .map_err(|_| anyhow!("connection count does not fit this platform"))?;
    let capacity = streams
        .checked_mul(connections)
        .and_then(|value| value.checked_mul(2))
        .ok_or_else(|| {
            anyhow!(
                "client inflight capacity overflow: max_concurrent_streams={} connections={}",
                max_concurrent_streams,
                connections
            )
        })?
        .max(MIN_CLIENT_INFLIGHT_SLOTS);
    ensure!(
        capacity <= MAX_CLIENT_INFLIGHT_SLOTS,
        "client inflight capacity {} exceeds hard limit {} (max_concurrent_streams={}, connections={})",
        capacity,
        MAX_CLIENT_INFLIGHT_SLOTS,
        max_concurrent_streams,
        connections
    );
    Ok(capacity)
}

#[cfg(test)]
mod tests {
    use super::{checked_inflight_capacity, AllowedHostIndex, MIN_CLIENT_INFLIGHT_SLOTS};
    use tunnel_lib::EgressVhostRuleDef;

    fn rule(host: &str, action: &str) -> EgressVhostRuleDef {
        EgressVhostRuleDef {
            match_host: host.to_string(),
            action_upstream: action.to_string(),
        }
    }

    #[test]
    fn allowed_index_rejects_hosts_without_matching_rule() {
        let index = AllowedHostIndex::build(vec![rule("example.com", "upstream")]);

        assert!(!index.is_rejected_host("example.com"));
        assert!(index.is_rejected_host("unknown.example.com"));
    }

    #[test]
    fn allowed_index_canonicalizes_case_and_ports() {
        let index = AllowedHostIndex::build(vec![rule("Blocked.COM:443", "upstream")]);

        assert!(!index.is_rejected_host("Blocked.COM:8443"));
        assert!(!index.is_rejected_host("blocked.com:8443"));
    }

    #[test]
    fn allowed_index_supports_wildcards_without_matching_parent() {
        let index = AllowedHostIndex::build(vec![rule("*.example.com", "upstream")]);

        assert!(!index.is_rejected_host("api.example.com"));
        assert!(index.is_rejected_host("example.com"));
    }

    #[test]
    fn inflight_capacity_rejects_arithmetic_overflow() {
        let error = checked_inflight_capacity(u32::MAX, u32::MAX)
            .expect_err("overflowing inflight capacity must fail");
        assert!(error.to_string().contains("inflight capacity"));
    }

    #[test]
    fn inflight_capacity_rejects_values_above_hard_limit() {
        let error = checked_inflight_capacity(1000, 132)
            .expect_err("oversized inflight capacity must fail");
        assert!(error.to_string().contains("exceeds hard limit"));
    }

    #[test]
    fn inflight_capacity_keeps_minimum_headroom() {
        assert_eq!(
            checked_inflight_capacity(1, 1).expect("small capacity should be valid"),
            MIN_CLIENT_INFLIGHT_SLOTS
        );
    }
}
