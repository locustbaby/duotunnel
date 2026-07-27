use crate::health::ClientHealth;
use anyhow::{anyhow, ensure, Result};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use quinn::Connection;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Notify};
use tracing::info;
use tunnel_lib::{
    inflight_load, new_inflight_table, pick_from_preferred_shards, pick_p2c_inflight_owned,
    stable_shard_index, ConnectionHandle, NegotiatedProtocol, VhostRouter,
};

const MAX_CLIENT_CONNECTION_STATES: usize = 262_144;

pub struct PooledConnection {
    pub handle: Arc<ConnectionHandle>,
    activity: Arc<SessionActivity>,
    // Unread until the first capability-gated feature lands; stored now so
    // gating can happen at the connection-selection site without re-plumbing.
    #[allow(dead_code)]
    pub negotiated: NegotiatedProtocol,
}

impl PooledConnection {
    pub(crate) fn mark_business_completed(&self) {
        self.activity.mark_business_completed();
    }
}

#[derive(Default)]
pub(crate) struct SessionActivity {
    business_completed: AtomicBool,
    completed_notify: Notify,
}

impl SessionActivity {
    pub(crate) fn mark_business_completed(&self) {
        if !self.business_completed.load(Ordering::Relaxed)
            && self
                .business_completed
                .compare_exchange(false, true, Ordering::Release, Ordering::Relaxed)
                .is_ok()
        {
            self.completed_notify.notify_waiters();
        }
    }

    pub(crate) fn business_completed(&self) -> bool {
        self.business_completed.load(Ordering::Acquire)
    }

    pub(crate) async fn wait_for_business_completion(&self) {
        loop {
            let notified = self.completed_notify.notified();
            if self.business_completed() {
                return;
            }
            notified.await;
        }
    }
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
        activity: Arc<SessionActivity>,
        reply: oneshot::Sender<Result<PoolCommit>>,
    },
    Remove {
        stable_id: usize,
        reply: oneshot::Sender<Result<PoolCommit>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PoolCommit {
    pub(crate) changed: bool,
    pub(crate) active_tunnels: usize,
}

struct ActorAliveGuard {
    health: Arc<ClientHealth>,
    handles: HashMap<usize, Arc<ConnectionHandle>>,
    by_id: Arc<DashMap<usize, Arc<PooledConnection>>>,
}

impl Drop for ActorAliveGuard {
    fn drop(&mut self) {
        for handle in self.handles.values() {
            handle.retire();
        }
        self.by_id.clear();
        self.health.set_active_tunnels(0);
        self.health.set_pool_actor_alive(false);
    }
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
    by_id: Arc<DashMap<usize, Arc<PooledConnection>>>,
    health: Arc<ClientHealth>,
    egress_rules: ArcSwap<AllowedHostIndex>,
    #[cfg(test)]
    actor_abort: tokio::task::AbortHandle,
}

impl EntryConnPool {
    pub fn new(
        max_concurrent_streams: u32,
        max_pending_streams: usize,
        connections: u32,
        shard_count: usize,
        health: Arc<ClientHealth>,
    ) -> Result<Arc<Self>> {
        let capacity = checked_connection_capacity(connections)?;
        let inflight_table = new_inflight_table(capacity);
        let shard_count = shard_count.max(1);
        let (tx, mut rx) = mpsc::channel(1024);
        let snapshots = (0..shard_count)
            .map(|_| Arc::new(ArcSwap::from_pointee(Vec::new())))
            .collect::<Vec<_>>();
        let snapshots_for_actor = snapshots.clone();
        let by_id = Arc::new(DashMap::new());
        let by_id_for_actor = by_id.clone();
        let health_for_actor = health.clone();

        let actor = tokio::spawn(async move {
            health_for_actor.set_pool_actor_alive(true);
            let mut alive_guard = ActorAliveGuard {
                health: health_for_actor.clone(),
                handles: HashMap::new(),
                by_id: by_id_for_actor.clone(),
            };
            let mut shards = (0..shard_count)
                .map(|_| PoolShard::new())
                .collect::<Vec<_>>();
            let mut active_tunnels = 0usize;

            while let Some(msg) = rx.recv().await {
                match msg {
                    PoolMsg::Push {
                        conn,
                        shard_id,
                        negotiated,
                        activity,
                        reply,
                    } => {
                        let stable_id = conn.stable_id();
                        let shard_id = shard_id % shard_count;
                        let shard = &mut shards[shard_id];
                        if shard.ids.contains_key(&stable_id) {
                            let _ = reply.send(Ok(PoolCommit {
                                changed: false,
                                active_tunnels,
                            }));
                            continue;
                        }
                        let Some(connection_state) = inflight_table.allocate() else {
                            let _ =
                                reply.send(Err(anyhow!("client connection state table exhausted")));
                            continue;
                        };
                        let index = shard.conns.len();
                        shard.ids.insert(stable_id, index);
                        let handle = ConnectionHandle::spawn(
                            conn,
                            connection_state,
                            shard_id,
                            max_concurrent_streams,
                            max_pending_streams,
                        );
                        alive_guard.handles.insert(stable_id, handle.clone());
                        let pooled = Arc::new(PooledConnection {
                            handle,
                            activity,
                            negotiated,
                        });
                        by_id_for_actor.insert(stable_id, pooled.clone());
                        shard.conns.push(pooled);
                        snapshots_for_actor[shard_id].store(shard.snapshot());
                        active_tunnels += 1;
                        health_for_actor.set_active_tunnels(active_tunnels);
                        let facts = health_for_actor.snapshot();
                        info!(
                            active_tunnels = facts.active_tunnels,
                            desired_tunnels = facts.desired_tunnels,
                            degraded = facts.degraded,
                            "connection pool tunnel committed"
                        );
                        let _ = reply.send(Ok(PoolCommit {
                            changed: true,
                            active_tunnels,
                        }));
                    }
                    PoolMsg::Remove { stable_id, reply } => {
                        let mut changed = false;
                        for (shard_id, shard) in shards.iter_mut().enumerate() {
                            if let Some(index) = shard.ids.remove(&stable_id) {
                                let existing = &shard.conns[index];
                                existing.handle.retire();
                                alive_guard.handles.remove(&stable_id);
                                by_id_for_actor.remove(&stable_id);
                                shard.conns.swap_remove(index);
                                if index < shard.conns.len() {
                                    let swapped_id = shard.conns[index].handle.stable_id();
                                    shard.ids.insert(swapped_id, index);
                                }
                                snapshots_for_actor[shard_id].store(shard.snapshot());
                                active_tunnels = active_tunnels.saturating_sub(1);
                                health_for_actor.set_active_tunnels(active_tunnels);
                                let facts = health_for_actor.snapshot();
                                info!(
                                    active_tunnels = facts.active_tunnels,
                                    desired_tunnels = facts.desired_tunnels,
                                    degraded = facts.degraded,
                                    "connection pool tunnel removed"
                                );
                                changed = true;
                                break;
                            }
                        }
                        let _ = reply.send(Ok(PoolCommit {
                            changed,
                            active_tunnels,
                        }));
                    }
                }
            }
        });
        #[cfg(test)]
        let actor_abort = actor.abort_handle();
        drop(actor);

        Ok(Arc::new(Self {
            tx,
            shard_count,
            shards: snapshots,
            by_id,
            health,
            egress_rules: ArcSwap::from_pointee(AllowedHostIndex::default()),
            #[cfg(test)]
            actor_abort,
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

    pub async fn push(
        &self,
        conn: Connection,
        negotiated: NegotiatedProtocol,
        activity: Arc<SessionActivity>,
    ) -> Result<PoolCommit> {
        let shard_id = stable_shard_index(&conn.stable_id(), self.shard_count);
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(PoolMsg::Push {
                conn,
                shard_id,
                negotiated,
                activity,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow!("connection pool actor is unavailable"))?;
        reply_rx
            .await
            .map_err(|_| anyhow!("connection pool actor stopped before push acknowledgement"))?
    }

    pub async fn remove(&self, conn: &Connection) -> Result<PoolCommit> {
        self.remove_stable_id(conn.stable_id()).await
    }

    pub async fn remove_stable_id(&self, stable_id: usize) -> Result<PoolCommit> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(PoolMsg::Remove {
                stable_id,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow!("connection pool actor is unavailable"))?;
        reply_rx
            .await
            .map_err(|_| anyhow!("connection pool actor stopped before remove acknowledgement"))?
    }

    pub fn next_conn_for_shard_excluding(
        &self,
        preferred_shard: usize,
        excluded: &HashSet<usize>,
    ) -> Option<Arc<PooledConnection>> {
        pick_from_preferred_shards(&self.shards, preferred_shard, |shard| {
            let snapshot = shard.load();
            pick_p2c_inflight_owned(
                snapshot.as_slice(),
                |c| {
                    c.handle.connection_state().is_selectable()
                        && c.handle.close_reason().is_none()
                        && !excluded.contains(&c.handle.stable_id())
                },
                |c| inflight_load(c.handle.connection_state(), Ordering::Relaxed),
            )
        })
    }

    pub fn connection_by_stable_id(&self, stable_id: usize) -> Option<Arc<PooledConnection>> {
        let connection = self.by_id.get(&stable_id)?.clone();
        (connection.handle.connection_state().is_selectable()
            && connection.handle.close_reason().is_none())
        .then_some(connection)
    }

    pub fn pool_size(&self) -> usize {
        self.health.snapshot().active_tunnels
    }

    #[cfg(test)]
    fn actor_alive(&self) -> bool {
        self.health.snapshot().pool_actor_alive
    }

    #[cfg(test)]
    fn abort_actor(&self) {
        self.actor_abort.abort();
    }
}

fn checked_connection_capacity(connections: u32) -> Result<usize> {
    let connections = usize::try_from(connections)
        .map_err(|_| anyhow!("connection count does not fit this platform"))?;
    let capacity = connections.max(1);
    ensure!(
        capacity <= MAX_CLIENT_CONNECTION_STATES,
        "client connection state capacity {} exceeds hard limit {}",
        capacity,
        MAX_CLIENT_CONNECTION_STATES
    );
    Ok(capacity)
}

#[cfg(test)]
mod tests {
    use super::{checked_connection_capacity, AllowedHostIndex, EntryConnPool};
    use crate::health::ClientHealth;
    use std::sync::Arc;
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
    fn connection_capacity_rejects_values_above_hard_limit() {
        let error =
            checked_connection_capacity(u32::MAX).expect_err("oversized capacity must fail");
        assert!(error.to_string().contains("exceeds hard limit"));
    }

    #[test]
    fn connection_capacity_tracks_configured_tunnel_count() {
        assert_eq!(checked_connection_capacity(4).unwrap(), 4);
        assert_eq!(checked_connection_capacity(0).unwrap(), 1);
    }

    #[tokio::test]
    async fn actor_failure_is_returned_to_callers_and_clears_health() {
        let health = Arc::new(ClientHealth::new(false, 1, 1));
        let pool = EntryConnPool::new(100, 25, 1, 1, health.clone()).unwrap();
        tokio::task::yield_now().await;
        assert!(pool.actor_alive());

        pool.abort_actor();
        for _ in 0..10 {
            if !pool.actor_alive() {
                break;
            }
            tokio::task::yield_now().await;
        }

        assert!(!pool.actor_alive());
        assert!(pool.remove_stable_id(123).await.is_err());
        assert!(!health.snapshot().ready);
    }
}
