use arc_swap::ArcSwap;
use crossbeam_utils::CachePadded;
use dashmap::DashMap;
use parking_lot::Mutex;
use quinn::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};

struct ClientInfo {
    group_id: String,
    conn: Connection,
}

/// Per-group connection pool using RCU (Read-Copy-Update) for routing reads.
///
/// `snapshot` is an `ArcSwap<Vec<Connection>>` — readers call `.load()` which is
/// a single atomic pointer load with no allocation.  Writers hold a `Mutex` to
/// serialize mutations, rebuild the Vec, and swap atomically.
///
/// This replaces the previous `DashMap<String, Connection>` pattern that caused
/// a heap allocation + many Arc reference-count bumps on *every* routing lookup.
pub struct ClientGroup {
    /// Mutable index: client_id → Connection.  Only touched by writers.
    index: Mutex<std::collections::HashMap<String, Connection>>,
    /// Read-side snapshot.  Rebuilt on every write; readers pay only one atomic load.
    snapshot: ArcSwap<Vec<Connection>>,
    counter: CachePadded<AtomicUsize>,
}

impl ClientGroup {
    pub fn new() -> Self {
        Self {
            index: Mutex::new(std::collections::HashMap::new()),
            snapshot: ArcSwap::from_pointee(Vec::new()),
            counter: CachePadded::new(AtomicUsize::new(0)),
        }
    }

    pub fn set(&self, client_id: String, conn: Connection) {
        let mut idx = self.index.lock();
        idx.insert(client_id, conn);
        let snap: Vec<Connection> = idx.values().cloned().collect();
        self.snapshot.store(Arc::new(snap));
    }

    pub fn remove(&self, client_id: &str) -> bool {
        let mut idx = self.index.lock();
        let removed = idx.remove(client_id).is_some();
        if removed {
            let snap: Vec<Connection> = idx.values().cloned().collect();
            self.snapshot.store(Arc::new(snap));
        }
        removed
    }

    pub fn is_empty(&self) -> bool {
        self.snapshot.load().is_empty()
    }

    /// O(1) allocation-free routing read via RCU snapshot.
    pub fn select_healthy(&self) -> Option<Connection> {
        let conns = self.snapshot.load();
        let len = conns.len();
        if len == 0 {
            return None;
        }
        let start = self.counter.fetch_add(1, Ordering::Relaxed);
        for i in 0..len {
            let conn = &conns[(start + i) % len];
            if conn.close_reason().is_none() {
                return Some(conn.clone());
            }
        }
        None
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

    fn replace_or_register(
        &self,
        client_id: String,
        group_id: String,
        conn: Connection,
    ) {
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

    pub fn select_client_for_group(&self, group_id: &str) -> Option<Connection> {
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
