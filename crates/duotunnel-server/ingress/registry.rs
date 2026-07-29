use arc_swap::ArcSwap;
use dashmap::DashMap;
use duotunnel_core::{
    inflight_load, new_inflight_table, pick_from_preferred_shards, pick_p2c_inflight_owned,
    stable_shard_index, ClientId, ConnectionHandle, ErrorKind, GroupId, NegotiatedProtocol,
    ProxyError,
};
use quinn::Connection;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

const MAX_PENDING_UNREGISTERS: usize = 65_536;

struct ClientInfo {
    group_id: GroupId,
    shard_id: usize,
}

struct RegisteredConn {
    handle: Arc<ConnectionHandle>,
    negotiated: NegotiatedProtocol,
    token_hash: [u8; 32],
}

fn fail_closed_registry(groups: &DashMap<GroupId, Arc<ClientGroup>>) {
    for group in groups.iter() {
        for shard in &group.shards {
            let snapshot = shard.load_full();
            for selected in snapshot.iter() {
                selected.handle.retire();
                selected.handle.close(0, b"registry unavailable");
            }
            shard.store(Arc::new(Vec::new()));
        }
    }
    groups.clear();
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
                |c| {
                    c.handle.connection_state().is_selectable() && c.handle.close_reason().is_none()
                },
                |c| inflight_load(c.handle.connection_state(), Ordering::Relaxed),
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
        token_hash: [u8; 32],
        reply: oneshot::Sender<Result<(), &'static str>>,
    },
    Unregister {
        client_id: ClientId,
    },
    PurgeDead {
        reply: oneshot::Sender<usize>,
    },
    RevokeTokens {
        token_hashes: Arc<std::collections::HashSet<[u8; 32]>>,
        reply: oneshot::Sender<usize>,
    },
}

pub struct ClientRegistry {
    groups: Arc<DashMap<GroupId, Arc<ClientGroup>>>,
    client_to_group: Arc<DashMap<ClientId, (GroupId, usize)>>,
    actor_alive: Arc<AtomicBool>,
    shard_count: usize,
    next_register_shard: AtomicUsize,
    tx: mpsc::Sender<RegistryMsg>,
    pending_unregisters: Arc<parking_lot::Mutex<HashSet<ClientId>>>,
    pending_notify: Arc<tokio::sync::Notify>,
}

impl ClientRegistry {
    pub fn new(
        shard_count: usize,
        max_concurrent_streams: u32,
        max_pending_streams: usize,
    ) -> Self {
        let shard_count = shard_count.max(1);
        let groups = Arc::new(DashMap::<GroupId, Arc<ClientGroup>>::new());
        let client_to_group = Arc::new(DashMap::<ClientId, (GroupId, usize)>::new());
        let inflight_table = new_inflight_table(4096);
        let (tx, mut rx) = mpsc::channel(1024);
        let groups_clone = groups.clone();
        let client_to_group_clone = client_to_group.clone();
        let actor_alive = Arc::new(AtomicBool::new(true));
        let pending_unregisters = Arc::new(parking_lot::Mutex::new(HashSet::new()));
        let pending_unregisters_clone = pending_unregisters.clone();
        let pending_notify = Arc::new(tokio::sync::Notify::new());
        let pending_notify_clone = pending_notify.clone();
        let actor_alive_for_actor = actor_alive.clone();

        let actor = tokio::spawn(async move {
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

            let pending_notify_clone2 = pending_notify_clone.clone();
            loop {
                if !actor_alive_for_actor.load(Ordering::Acquire) {
                    break;
                }
                // Drain pending unregisters
                let pending = {
                    let mut guard = pending_unregisters_clone.lock();
                    std::mem::take(&mut *guard)
                };
                for cid in pending {
                    actor_unregister(
                        &cid,
                        &mut clients,
                        &mut group_conns,
                        &client_to_group_clone,
                        &groups_clone,
                        shard_count,
                    );
                }

                tokio::select! {
                    _ = pending_notify_clone2.notified() => {}
                    msg = rx.recv() => {
                        let Some(msg) = msg else { break; };
                        match msg {
                    RegistryMsg::Register {
                        client_id,
                        group_id,
                        conn,
                        shard_id,
                        negotiated,
                        token_hash,
                        reply,
                    } => {
                        if clients.contains_key(&client_id) {
                            let _ = reply.send(Err("client id already registered"));
                            continue;
                        }
                        info!(
                            client_id = %client_id,
                            group_id = %group_id,
                            shard_id,
                            negotiated_version = negotiated.version,
                            capabilities = negotiated.capabilities,
                            "registering client"
                        );
                        let predecessor = clients.get(&client_id).and_then(|info| {
                            group_conns.get(&info.group_id)
                                .and_then(|shards| shards[info.shard_id].get(&client_id))
                                .map(|rc| rc.handle.clone())
                        });
                        let Some(connection_state) = inflight_table.allocate() else {
                            let _ = reply.send(Err("connection state capacity exhausted"));
                            continue;
                        };
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

                        let handle = ConnectionHandle::spawn(
                            conn,
                            connection_state,
                            shard_id % shard_count,
                            max_concurrent_streams,
                            max_pending_streams,
                        );

                        let mut replaced = idx.insert(
                            client_id.clone(),
                            RegisteredConn {
                                handle,
                                negotiated,
                                token_hash,
                            },
                        );
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
                                        replaced = Some(existing);
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

                        // Publish to public index during Actor commit phase
                        client_to_group_clone.insert(
                            client_id.clone(),
                            (group_id.clone(), shard_id % shard_count),
                        );

                        // Retire the predecessor after successful commit
                        if let Some(existing) = replaced {
                            existing.handle.retire();
                        } else if let Some(existing_handle) = predecessor {
                            existing_handle.retire();
                        }

                        let _ = reply.send(Ok(()));
                    }
                    RegistryMsg::Unregister { client_id } => {
                        actor_unregister(
                            &client_id,
                            &mut clients,
                            &mut group_conns,
                            &client_to_group_clone,
                            &groups_clone,
                            shard_count,
                        );
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
                                            existing.handle.retire();
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
                            actor_unregister(
                                &cid,
                                &mut clients,
                                &mut group_conns,
                                &client_to_group_clone,
                                &groups_clone,
                                shard_count,
                            );
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
                    RegistryMsg::RevokeTokens {
                        token_hashes,
                        reply,
                    } => {
                        let mut revoked = 0;
                        for shards in group_conns.values_mut() {
                            for idx in shards {
                                for registered in idx.values() {
                                    if token_hashes.contains(&registered.token_hash)
                                        && registered.handle.retire()
                                    {
                                        registered.handle.close(0, b"token revoked");
                                        revoked += 1;
                                    }
                                }
                            }
                        }
                        let _ = reply.send(revoked);
                    }
                }
                    }
                }
            }
        });
        let monitor_groups = groups.clone();
        let monitor_alive = actor_alive.clone();
        tokio::spawn(async move {
            if let Err(join_error) = actor.await {
                error!(error = %join_error, "registry actor failed");
            } else {
                error!("registry actor exited");
            }
            monitor_alive.store(false, Ordering::Release);
            fail_closed_registry(&monitor_groups);
        });

        Self {
            groups,
            client_to_group,
            actor_alive,
            shard_count,
            next_register_shard: AtomicUsize::new(0),
            tx,
            pending_unregisters,
            pending_notify,
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
        token_hash: [u8; 32],
    ) -> Result<(), &'static str> {
        if !self.actor_alive.load(Ordering::Acquire) {
            return Err("registry unavailable");
        }
        let (reply_tx, reply_rx) = oneshot::channel();
        let shard_id = self.next_register_shard.fetch_add(1, Ordering::Relaxed) % self.shard_count;
        if self
            .tx
            .send(RegistryMsg::Register {
                client_id: client_id.clone(),
                group_id: group_id.clone(),
                conn,
                shard_id,
                negotiated,
                token_hash,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
            self.fail_closed();
            return Err("registry actor channel closed");
        }
        match reply_rx.await {
            Ok(result) => result,
            Err(_) => {
                self.fail_closed();
                Err("registry actor response dropped")
            }
        }
    }

    pub fn unregister(&self, client_id: &ClientId) {
        if !self.actor_alive.load(Ordering::Acquire) {
            return;
        }
        self.retire_visible_client(client_id);
        let message = RegistryMsg::Unregister {
            client_id: client_id.clone(),
        };
        match self.tx.try_send(message) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                let (inserted, overflowed) = {
                    let mut guard = self.pending_unregisters.lock();
                    if guard.contains(client_id) {
                        (false, false)
                    } else if guard.len() >= MAX_PENDING_UNREGISTERS {
                        (false, true)
                    } else {
                        guard.insert(client_id.clone());
                        (true, false)
                    }
                };
                if overflowed {
                    error!(
                        client_id = %client_id,
                        limit = MAX_PENDING_UNREGISTERS,
                        "registry unregister reconcile overflowed; failing closed"
                    );
                    self.fail_closed();
                } else if inserted {
                    self.pending_notify.notify_one();
                }
            }
            Err(mpsc::error::TrySendError::Closed(_)) => self.fail_closed(),
        }
    }

    pub fn select_client_for_group(&self, group_id: &str) -> Option<Arc<SelectedConnection>> {
        if !self.actor_alive.load(Ordering::Acquire) {
            return None;
        }
        let group = self.groups.get(group_id).map(|r| r.value().clone())?;
        group.select_healthy(self.preferred_shard_for_group(group_id))
    }

    pub async fn purge_dead(&self) -> Result<usize, &'static str> {
        if !self.actor_alive.load(Ordering::Acquire) {
            return Err("registry unavailable");
        }
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .tx
            .send(RegistryMsg::PurgeDead { reply: reply_tx })
            .await
            .is_err()
        {
            self.fail_closed();
            return Err("registry actor channel closed");
        }
        reply_rx.await.map_err(|_| {
            self.fail_closed();
            "registry actor response dropped"
        })
    }

    pub async fn revoke_tokens(
        &self,
        token_hashes: std::collections::HashSet<[u8; 32]>,
    ) -> Result<usize, &'static str> {
        if token_hashes.is_empty() {
            return Ok(0);
        }
        if !self.actor_alive.load(Ordering::Acquire) {
            return Err("registry unavailable");
        }
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(RegistryMsg::RevokeTokens {
                token_hashes: Arc::new(token_hashes),
                reply: reply_tx,
            })
            .await
            .map_err(|_| {
                self.fail_closed();
                "registry actor channel closed"
            })?;
        reply_rx.await.map_err(|_| {
            self.fail_closed();
            "registry actor response dropped"
        })
    }

    fn fail_closed(&self) {
        self.actor_alive.store(false, Ordering::Release);
        fail_closed_registry(&self.groups);
        self.client_to_group.clear();
        self.pending_unregisters.lock().clear();
    }

    fn retire_visible_client(&self, client_id: &ClientId) {
        if let Some((_, (group_id, shard_id))) = self.client_to_group.remove(client_id) {
            if let Some(group) = self.groups.get(&group_id) {
                if shard_id < group.shards.len() {
                    let snapshot = group.shards[shard_id].load();
                    if let Some(selected) =
                        snapshot.iter().find(|entry| &entry.conn_id == client_id)
                    {
                        selected.handle.retire();
                    }
                }
            }
        }
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

fn actor_unregister(
    client_id: &ClientId,
    clients: &mut HashMap<ClientId, ClientInfo>,
    group_conns: &mut HashMap<GroupId, Vec<HashMap<ClientId, RegisteredConn>>>,
    client_to_group: &DashMap<ClientId, (GroupId, usize)>,
    groups: &DashMap<GroupId, Arc<ClientGroup>>,
    _shard_count: usize,
) {
    client_to_group.remove_if(client_id, |_, _| true);
    if let Some(info) = clients.remove(client_id) {
        tracing::info!(
            client_id = %client_id,
            group_id = %info.group_id,
            shard_id = info.shard_id,
            "unregistering client"
        );
        if let Some(shards) = group_conns.get_mut(&info.group_id) {
            let idx = &mut shards[info.shard_id];
            if let Some(existing) = idx.remove(client_id) {
                existing.handle.retire();
            }

            let snapshot = idx
                .iter()
                .map(|(cid, registered)| {
                    Arc::new(SelectedConnection {
                        conn_id: cid.clone(),
                        handle: registered.handle.clone(),
                        negotiated: registered.negotiated,
                    })
                })
                .collect::<Vec<_>>();

            let should_remove = if let Some(group) = groups.get(&info.group_id) {
                group.shards[info.shard_id].store(Arc::new(snapshot));
                group.is_empty()
            } else {
                false
            };
            if should_remove {
                groups.remove_if(&info.group_id, |_, g| g.is_empty());
            }
        }
    }
}
