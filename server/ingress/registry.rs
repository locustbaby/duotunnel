use arc_swap::ArcSwap;
use dashmap::DashMap;
use quinn::Connection;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::info;
use tunnel_lib::{
    inflight_load, new_inflight_table, pick_from_preferred_shards, pick_p2c_inflight_owned,
    stable_shard_index, ClientId, ConnectionHandle, ErrorKind, GroupId, NegotiatedProtocol,
    ProxyError,
};

struct ClientInfo {
    group_id: GroupId,
    shard_id: usize,
}

struct RegisteredConn {
    handle: Arc<ConnectionHandle>,
    negotiated: NegotiatedProtocol,
}

#[derive(Clone)]
pub struct SelectedConnection {
    pub conn_id: ClientId,
    pub handle: Arc<ConnectionHandle>,
    // Unread until the first capability-gated feature lands; stored now so
    // gating can happen at the connection-selection site without re-plumbing.
    #[allow(dead_code)]
    pub negotiated: NegotiatedProtocol,
}

pub struct ClientGroup {
    shards: Vec<ArcSwap<Vec<Arc<SelectedConnection>>>>,
}

impl ClientGroup {
    pub fn new(shard_count: usize) -> Self {
        Self {
            shards: (0..shard_count)
                .map(|_| ArcSwap::from_pointee(Vec::new()))
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.shards
            .iter()
            .all(|snapshot| snapshot.load().is_empty())
    }

    pub fn select_healthy(&self, preferred_shard: usize) -> Option<Arc<SelectedConnection>> {
        pick_from_preferred_shards(self.shards.as_slice(), preferred_shard, |shard| {
            let snapshot = shard.load();
            pick_p2c_inflight_owned(
                snapshot.as_slice(),
                |c| c.handle.close_reason().is_none(),
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
}

enum RegistryMsg {
    Register {
        client_id: ClientId,
        group_id: GroupId,
        conn: Connection,
        shard_id: usize,
        negotiated: NegotiatedProtocol,
        reply: oneshot::Sender<Result<(), &'static str>>,
    },
    Unregister {
        client_id: ClientId,
    },
    PurgeDead {
        reply: oneshot::Sender<usize>,
    },
}

pub struct ClientRegistry {
    groups: Arc<DashMap<GroupId, Arc<ClientGroup>>>,
    shard_count: usize,
    next_register_shard: AtomicUsize,
    tx: mpsc::Sender<RegistryMsg>,
}

impl ClientRegistry {
    pub fn new(shard_count: usize, max_concurrent_streams: u32, max_pending_streams: usize) -> Self {
        let shard_count = shard_count.max(1);
        let groups = Arc::new(DashMap::new());
        let inflight_table = new_inflight_table(4096);
        let (tx, mut rx) = mpsc::channel(1024);
        let groups_clone = groups.clone();

        tokio::spawn(async move {
            let mut clients: HashMap<ClientId, ClientInfo> = HashMap::new();
            let mut group_conns: HashMap<GroupId, Vec<HashMap<ClientId, RegisteredConn>>> =
                HashMap::new();

            let build_snapshot = |idx: &HashMap<ClientId, RegisteredConn>| {
                idx.iter()
                    .map(|(client_id, registered)| {
                        Arc::new(SelectedConnection {
                            conn_id: client_id.clone(),
                            handle: registered.handle.clone(),
                            negotiated: registered.negotiated,
                        })
                    })
                    .collect::<Vec<_>>()
            };

            while let Some(msg) = rx.recv().await {
                match msg {
                    RegistryMsg::Register {
                        client_id,
                        group_id,
                        conn,
                        shard_id,
                        negotiated,
                        reply,
                    } => {
                        info!(
                            client_id = %client_id,
                            group_id = %group_id,
                            shard_id,
                            negotiated_version = negotiated.version,
                            capabilities = negotiated.capabilities,
                            "registering client"
                        );
                        let group = {
                            let entry = groups_clone
                                .entry(group_id.clone())
                                .or_insert_with(|| Arc::new(ClientGroup::new(shard_count)));
                            entry.value().clone()
                        };

                        let shards = group_conns
                            .entry(group_id.clone())
                            .or_insert_with(|| (0..shard_count).map(|_| HashMap::new()).collect());
                        let idx = &mut shards[shard_id % shard_count];
                        let slot_id = if let Some(existing) = idx.get(&client_id) {
                            existing.handle.slot_id()
                        } else if let Some(slot) = inflight_table.alloc_slot() {
                            slot
                        } else {
                            let _ = reply.send(Err("inflight slot table exhausted"));
                            continue;
                        };

                        let handle = ConnectionHandle::spawn(
                            conn,
                            inflight_table.clone(),
                            slot_id,
                            shard_id % shard_count,
                            max_concurrent_streams,
                            max_pending_streams,
                        );
                        idx.insert(client_id.clone(), RegisteredConn { handle, negotiated });
                        group.shards[shard_id % shard_count].store(Arc::new(build_snapshot(idx)));

                        if let Some(old_info) = clients.insert(
                            client_id.clone(),
                            ClientInfo {
                                group_id: group_id.clone(),
                                shard_id: shard_id % shard_count,
                            },
                        ) {
                            if old_info.group_id != group_id
                                || old_info.shard_id != shard_id % shard_count
                            {
                                if let Some(old_shards) = group_conns.get_mut(&old_info.group_id) {
                                    let old_idx = &mut old_shards[old_info.shard_id];
                                    if let Some(existing) = old_idx.remove(&client_id) {
                                        inflight_table.free_slot(existing.handle.slot_id());
                                    }
                                    if let Some(old_group) = groups_clone.get(&old_info.group_id) {
                                        old_group.shards[old_info.shard_id]
                                            .store(Arc::new(build_snapshot(old_idx)));
                                        if old_group.is_empty() {
                                            drop(old_group);
                                            groups_clone
                                                .remove_if(&old_info.group_id, |_, g| g.is_empty());
                                        }
                                    }
                                }
                            }
                        }
                        let _ = reply.send(Ok(()));
                    }
                    RegistryMsg::Unregister { client_id } => {
                        if let Some(info) = clients.remove(&client_id) {
                            info!(
                                client_id = %client_id,
                                group_id = %info.group_id,
                                shard_id = info.shard_id,
                                "unregistering client"
                            );
                            if let Some(shards) = group_conns.get_mut(&info.group_id) {
                                let idx = &mut shards[info.shard_id];
                                if let Some(existing) = idx.remove(&client_id) {
                                    inflight_table.free_slot(existing.handle.slot_id());
                                }
                                let should_remove =
                                    if let Some(group) = groups_clone.get(&info.group_id) {
                                        group.shards[info.shard_id]
                                            .store(Arc::new(build_snapshot(idx)));
                                        group.is_empty()
                                    } else {
                                        false
                                    };
                                if should_remove {
                                    groups_clone.remove_if(&info.group_id, |_, g| g.is_empty());
                                }
                            }
                        }
                    }
                    RegistryMsg::PurgeDead { reply } => {
                        let mut dead_count = 0;
                        let mut dead_clients = Vec::new();

                        for (gid, shards) in group_conns.iter_mut() {
                            for (shard_id, idx) in shards.iter_mut().enumerate() {
                                let mut dead_in_group = Vec::new();
                                for (cid, registered) in idx.iter() {
                                    if registered.handle.close_reason().is_some() {
                                        dead_in_group.push(cid.clone());
                                    }
                                }
                                if !dead_in_group.is_empty() {
                                    for cid in &dead_in_group {
                                        if let Some(existing) = idx.remove(cid) {
                                            inflight_table.free_slot(existing.handle.slot_id());
                                        }
                                        dead_clients.push(cid.clone());
                                        dead_count += 1;
                                    }
                                    if let Some(group) = groups_clone.get(gid) {
                                        group.shards[shard_id].store(Arc::new(build_snapshot(idx)));
                                    }
                                }
                            }
                        }

                        for cid in dead_clients {
                            if let Some(info) = clients.remove(&cid) {
                                info!(
                                    client_id = %cid,
                                    group_id = %info.group_id,
                                    shard_id = info.shard_id,
                                    "unregistering client"
                                );
                            }
                        }

                        let empty_gids: Vec<GroupId> = groups_clone
                            .iter()
                            .filter(|entry| entry.value().is_empty())
                            .map(|entry| entry.key().clone())
                            .collect();
                        for gid in empty_gids {
                            groups_clone.remove_if(&gid, |_, group| group.is_empty());
                        }

                        let _ = reply.send(dead_count);
                    }
                }
            }
        });

        Self {
            groups,
            shard_count,
            next_register_shard: AtomicUsize::new(0),
            tx,
        }
    }

    pub fn preferred_shard_for_group(&self, group_id: &str) -> usize {
        stable_shard_index(group_id, self.shard_count)
    }

    pub async fn register(
        &self,
        client_id: ClientId,
        group_id: GroupId,
        conn: Connection,
        negotiated: NegotiatedProtocol,
    ) -> Result<(), &'static str> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let shard_id = self.next_register_shard.fetch_add(1, Ordering::Relaxed) % self.shard_count;
        if self
            .tx
            .send(RegistryMsg::Register {
                client_id,
                group_id,
                conn,
                shard_id,
                negotiated,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
            return Err("registry actor channel closed");
        }
        reply_rx
            .await
            .unwrap_or(Err("registry actor response dropped"))
    }

    pub fn unregister(&self, client_id: &ClientId) {
        let _ = self.tx.try_send(RegistryMsg::Unregister {
            client_id: client_id.clone(),
        });
    }

    pub fn select_client_for_group(&self, group_id: &str) -> Option<Arc<SelectedConnection>> {
        let group = self.groups.get(group_id)?;
        group.select_healthy(self.preferred_shard_for_group(group_id))
    }

    pub async fn purge_dead(&self) -> usize {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .tx
            .send(RegistryMsg::PurgeDead { reply: reply_tx })
            .await
            .is_err()
        {
            return 0;
        }
        reply_rx.await.unwrap_or(0)
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new(1, 1000, 250)
    }
}

pub type SharedRegistry = Arc<ClientRegistry>;

pub fn new_shared_registry(
    shard_count: usize,
    max_concurrent_streams: u32,
    max_pending_streams: usize,
) -> SharedRegistry {
    Arc::new(ClientRegistry::new(
        shard_count,
        max_concurrent_streams,
        max_pending_streams,
    ))
}

pub fn unregister_if_connection_lost(
    registry: &SharedRegistry,
    selected: &SelectedConnection,
    err: &anyhow::Error,
) {
    let fatal_proxy_error = err.downcast_ref::<ProxyError>().is_some_and(|proxy_err| {
        matches!(
            proxy_err.kind,
            ErrorKind::QuicConnectionLost | ErrorKind::QuicConnectionFatal
        )
    });
    if selected.handle.close_reason().is_some() || fatal_proxy_error {
        registry.unregister(&selected.conn_id);
    }
}
