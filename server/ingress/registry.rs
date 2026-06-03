use arc_swap::ArcSwap;
use dashmap::DashMap;
use quinn::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tunnel_lib::{
    inflight_load, new_inflight_table, pick_p2c_inflight, InflightSlotId, InflightTable,
};

struct ClientInfo {
    group_id: String,
}

#[derive(Clone)]
pub struct SelectedConnection {
    pub conn_id: Arc<str>,
    pub conn: Connection,
    pub inflight_table: Arc<InflightTable>,
    pub slot_id: InflightSlotId,
}

pub struct ClientGroup {
    snapshot: ArcSwap<Vec<Arc<SelectedConnection>>>,
}

impl ClientGroup {
    pub fn new() -> Self {
        Self {
            snapshot: ArcSwap::from_pointee(Vec::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.snapshot.load().is_empty()
    }

    pub fn select_healthy(&self) -> Option<Arc<SelectedConnection>> {
        let conns = self.snapshot.load();
        pick_p2c_inflight(
            conns.as_slice(),
            32,
            3,
            |c| c.conn.close_reason().is_none(),
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
}

enum RegistryMsg {
    Register {
        client_id: String,
        group_id: String,
        conn: Connection,
        reply: oneshot::Sender<Result<(), &'static str>>,
    },
    Unregister {
        client_id: String,
    },
    PurgeDead {
        reply: oneshot::Sender<usize>,
    },
}

pub struct ClientRegistry {
    groups: DashMap<String, Arc<ClientGroup>>,
    tx: mpsc::Sender<RegistryMsg>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        let groups = DashMap::new();
        let inflight_table = new_inflight_table(4096);
        let (tx, mut rx) = mpsc::channel(1024);
        let groups_clone = groups.clone();

        tokio::spawn(async move {
            let mut clients: HashMap<String, ClientInfo> = HashMap::new();
            let mut group_conns: HashMap<String, HashMap<String, (Connection, InflightSlotId)>> = HashMap::new();

            let build_snapshot = |table: &Arc<InflightTable>, idx: &HashMap<String, (Connection, InflightSlotId)>| {
                idx.iter()
                    .map(|(client_id, (conn, slot_id))| {
                        Arc::new(SelectedConnection {
                            conn_id: Arc::<str>::from(client_id.as_str()),
                            conn: conn.clone(),
                            inflight_table: table.clone(),
                            slot_id: *slot_id,
                        })
                    })
                    .collect::<Vec<_>>()
            };

            while let Some(msg) = rx.recv().await {
                match msg {
                    RegistryMsg::Register { client_id, group_id, conn, reply } => {
                        let group = groups_clone
                            .entry(group_id.clone())
                            .or_insert_with(|| Arc::new(ClientGroup::new()));

                        let idx = group_conns.entry(group_id.clone()).or_default();
                        let slot_id = if let Some((_, existing_slot)) = idx.get(&client_id) {
                            *existing_slot
                        } else {
                            if let Some(slot) = inflight_table.alloc_slot() {
                                slot
                            } else {
                                let _ = reply.send(Err("inflight slot table exhausted"));
                                continue;
                            }
                        };

                        idx.insert(client_id.clone(), (conn, slot_id));
                        group.snapshot.store(Arc::new(build_snapshot(&inflight_table, idx)));

                        if let Some(old_info) = clients.insert(client_id.clone(), ClientInfo { group_id: group_id.clone() }) {
                            if old_info.group_id != group_id {
                                if let Some(old_idx) = group_conns.get_mut(&old_info.group_id) {
                                    if let Some((_, slot)) = old_idx.remove(&client_id) {
                                        inflight_table.free_slot(slot);
                                    }
                                    if let Some(old_grp) = groups_clone.get(&old_info.group_id) {
                                        old_grp.snapshot.store(Arc::new(build_snapshot(&inflight_table, old_idx)));
                                        if old_grp.is_empty() {
                                            drop(old_grp);
                                            groups_clone.remove_if(&old_info.group_id, |_, g| g.is_empty());
                                        }
                                    }
                                }
                            }
                        }
                        let _ = reply.send(Ok(()));
                    }
                    RegistryMsg::Unregister { client_id } => {
                        if let Some(info) = clients.remove(&client_id) {
                            if let Some(idx) = group_conns.get_mut(&info.group_id) {
                                if let Some((_, slot_id)) = idx.remove(&client_id) {
                                    inflight_table.free_slot(slot_id);
                                }
                                if let Some(group) = groups_clone.get(&info.group_id) {
                                    group.snapshot.store(Arc::new(build_snapshot(&inflight_table, idx)));
                                    if group.is_empty() {
                                        drop(group);
                                        groups_clone.remove_if(&info.group_id, |_, g| g.is_empty());
                                    }
                                }
                            }
                        }
                    }
                    RegistryMsg::PurgeDead { reply } => {
                        let mut dead_count = 0;
                        let mut dead_clients = Vec::new();

                        for (gid, idx) in group_conns.iter_mut() {
                            let mut dead_in_group = Vec::new();
                            for (cid, (conn, _)) in idx.iter() {
                                if conn.close_reason().is_some() {
                                    dead_in_group.push(cid.clone());
                                }
                            }
                            if !dead_in_group.is_empty() {
                                for cid in &dead_in_group {
                                    if let Some((_, slot_id)) = idx.remove(cid) {
                                        inflight_table.free_slot(slot_id);
                                    }
                                    dead_clients.push(cid.clone());
                                    dead_count += 1;
                                }
                                if let Some(group) = groups_clone.get(gid) {
                                    group.snapshot.store(Arc::new(build_snapshot(&inflight_table, idx)));
                                }
                            }
                        }

                        for cid in dead_clients {
                            clients.remove(&cid);
                        }

                        let empty_gids: Vec<String> = groups_clone
                            .iter()
                            .filter(|r| r.value().is_empty())
                            .map(|r| r.key().clone())
                            .collect();
                        for gid in empty_gids {
                            groups_clone.remove_if(&gid, |_, g| g.is_empty());
                        }

                        let _ = reply.send(dead_count);
                    }
                }
            }
        });

        Self { groups, tx }
    }

    pub async fn register(
        &self,
        client_id: String,
        group_id: String,
        conn: Connection,
    ) -> Result<(), &'static str> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self.tx.send(RegistryMsg::Register { client_id, group_id, conn, reply: reply_tx }).await.is_err() {
            return Err("registry actor channel closed");
        }
        reply_rx.await.unwrap_or(Err("registry actor response dropped"))
    }

    pub fn unregister(&self, client_id: &str) {
        let _ = self.tx.try_send(RegistryMsg::Unregister {
            client_id: client_id.to_string(),
        });
    }

    pub fn select_client_for_group(&self, group_id: &str) -> Option<Arc<SelectedConnection>> {
        let group = self.groups.get(group_id)?;
        group.select_healthy()
    }

    pub async fn purge_dead(&self) -> usize {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self.tx.send(RegistryMsg::PurgeDead { reply: reply_tx }).await.is_err() {
            return 0;
        }
        reply_rx.await.unwrap_or(0)
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedRegistry = Arc<ClientRegistry>;
pub fn new_shared_registry() -> SharedRegistry {
    Arc::new(ClientRegistry::new())
}
