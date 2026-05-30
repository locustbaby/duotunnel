use arc_swap::ArcSwap;
use dashmap::DashMap;
use parking_lot::Mutex;
use quinn::Connection;
use std::sync::Arc;
use tracing::{debug, info, warn};
use tunnel_lib::{inflight_load, new_inflight_table, pick_least_inflight, InflightSlotId, InflightTable};

struct ClientInfo {
    group_id: String,
    conn: Connection,
}

#[derive(Clone)]
pub struct SelectedConnection {
    pub conn_id: Arc<str>,
    pub conn: Connection,
    pub inflight_table: Arc<InflightTable>,
    pub slot_id: InflightSlotId,
}

/// Per-group connection pool using RCU (Read-Copy-Update) for routing reads.
///
/// `snapshot` is an `ArcSwap<Vec<Arc<SelectedConnection>>>` — readers call `.load()` which is
/// a single atomic pointer load with no allocation.  Writers hold a `Mutex` to
/// serialize mutations, rebuild the Vec, and swap atomically.
///
/// This replaces the previous `DashMap<String, Connection>` pattern that caused
/// a heap allocation + many Arc reference-count bumps on *every* routing lookup.
type ClientIndex = std::collections::HashMap<String, (Connection, InflightSlotId)>;

pub struct ClientGroup {
    index: Mutex<ClientIndex>,
    snapshot: ArcSwap<Vec<Arc<SelectedConnection>>>,
    inflight_table: Arc<InflightTable>,
}

impl ClientGroup {
    pub fn new() -> Self {
        Self {
            index: Mutex::new(std::collections::HashMap::new()),
            snapshot: ArcSwap::from_pointee(Vec::new()),
            inflight_table: new_inflight_table(4096),
        }
    }

    fn build_snapshot(
        table: &Arc<InflightTable>,
        idx: &ClientIndex,
    ) -> Vec<Arc<SelectedConnection>> {
        idx.iter()
            .map(|(client_id, (conn, slot_id))| {
                Arc::new(SelectedConnection {
                    conn_id: Arc::<str>::from(client_id.as_str()),
                    conn: conn.clone(),
                    inflight_table: table.clone(),
                    slot_id: *slot_id,
                })
            })
            .collect()
    }

    pub fn set(&self, client_id: String, conn: Connection) {
        let mut idx = self.index.lock();
        let slot_id = idx
            .get(&client_id)
            .map(|(_, slot_id)| *slot_id)
            .unwrap_or_else(|| {
                self.inflight_table
                    .alloc_slot()
                    .expect("inflight slot table exhausted")
            });
        idx.insert(client_id, (conn, slot_id));
        self.snapshot
            .store(Arc::new(Self::build_snapshot(&self.inflight_table, &idx)));
    }

    pub fn remove(&self, client_id: &str) -> bool {
        let mut idx = self.index.lock();
        let removed = idx.remove(client_id);
        if let Some((_, slot_id)) = removed {
            self.inflight_table.free_slot(slot_id);
        }
        let removed = removed.is_some();
        if removed {
            self.snapshot
                .store(Arc::new(Self::build_snapshot(&self.inflight_table, &idx)));
        }
        removed
    }

    pub fn is_empty(&self) -> bool {
        self.snapshot.load().is_empty()
    }

    /// Select the healthy connection with the fewest in-flight streams (least-inflight).
    ///
    /// Reads are allocation-free: one atomic pointer load from the RCU snapshot,
    /// then a linear scan over healthy connections comparing `AtomicUsize` inflight counts.
    pub fn select_healthy(&self) -> Option<Arc<SelectedConnection>> {
        let conns = self.snapshot.load();
        let len = conns.len();
        if len > 32 {
            let idx1 = fastrand::usize(..len);
            let idx2 = {
                let r = fastrand::usize(..len - 1);
                if r >= idx1 {
                    r + 1
                } else {
                    r
                }
            };
            let c1 = &conns[idx1];
            let c2 = &conns[idx2];
            let c1_healthy = c1.conn.close_reason().is_none();
            let c2_healthy = c2.conn.close_reason().is_none();
            match (c1_healthy, c2_healthy) {
                (true, true) => {
                    if inflight_load(
                        &c1.inflight_table,
                        c1.slot_id,
                        std::sync::atomic::Ordering::Relaxed,
                    ) <= inflight_load(
                        &c2.inflight_table,
                        c2.slot_id,
                        std::sync::atomic::Ordering::Relaxed,
                    )
                    {
                        Some(c1.clone())
                    } else {
                        Some(c2.clone())
                    }
                }
                (true, false) => Some(c1.clone()),
                (false, true) => Some(c2.clone()),
                (false, false) => pick_least_inflight(
                    conns.as_slice(),
                    |c| c.conn.close_reason().is_none(),
                    |c| {
                        inflight_load(
                            &c.inflight_table,
                            c.slot_id,
                            std::sync::atomic::Ordering::Relaxed,
                        )
                    },
                )
                .cloned(),
            }
        } else {
            pick_least_inflight(
                conns.as_slice(),
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
}

pub struct ClientRegistry {
    groups: DashMap<String, Arc<ClientGroup>>,
    clients: DashMap<String, ClientInfo>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            groups: DashMap::new(),
            clients: DashMap::new(),
        }
    }

    fn replace_or_register(&self, client_id: String, group_id: String, conn: Connection) {
        use dashmap::mapref::entry::Entry;
        match self.clients.entry(client_id.clone()) {
            Entry::Occupied(mut occ) => {
                let old_group_id = occ.get().group_id.clone();
                occ.insert(ClientInfo {
                    group_id: group_id.clone(),
                    conn: conn.clone(),
                });
                if let Some(grp) = self.groups.get(&old_group_id) {
                    grp.remove(&client_id);
                    if grp.is_empty() {
                        drop(grp);
                        self.groups.remove_if(&old_group_id, |_, g| g.is_empty());
                        debug!(group_id = %old_group_id, "removing empty group");
                    }
                }
            }
            Entry::Vacant(vac) => {
                vac.insert(ClientInfo {
                    group_id: group_id.clone(),
                    conn: conn.clone(),
                });
            }
        }
        self.groups
            .entry(group_id)
            .or_insert_with(|| Arc::new(ClientGroup::new()))
            .set(client_id, conn);
    }

    pub fn register(&self, client_id: String, group_id: String, conn: Connection) {
        info!(client_id = %client_id, group_id = %group_id, "registering client");
        self.replace_or_register(client_id, group_id, conn);
    }

    pub fn get_client_connection(&self, client_id: &str) -> Option<Connection> {
        self.clients.get(client_id).map(|info| info.conn.clone())
    }

    pub fn unregister(&self, client_id: &str) {
        if let Some((_, info)) = self.clients.remove(client_id) {
            info!(
                client_id = %client_id, group_id = %info.group_id,
                "unregistering client"
            );
            if let Some(group) = self.groups.get(&info.group_id) {
                group.remove(client_id);
                if group.is_empty() {
                    drop(group);
                    self.groups.remove_if(&info.group_id, |_, g| g.is_empty());
                    debug!(group_id = %info.group_id, "removing empty group");
                }
            }
        }
    }

    pub fn select_client_for_group(&self, group_id: &str) -> Option<Arc<SelectedConnection>> {
        let group = self.groups.get(group_id)?;
        let conn = group.select_healthy();
        if conn.is_none() {
            warn!(group_id = %group_id, "no healthy clients in group");
        }
        conn
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
