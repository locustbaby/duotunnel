use crossbeam_utils::CachePadded;
use dashmap::DashMap;
use quinn::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};

struct ClientInfo {
    group_id: String,
    conn: Connection,
}

/// Per-group connection pool.
/// Uses DashMap for O(1) insert/remove and sharded locking (no global write lock).
/// select_healthy() collects a snapshot of connections then scans — avoids holding
/// any lock during the health check loop.
pub struct ClientGroup {
    entries: DashMap<String, Connection>,
    counter: CachePadded<AtomicUsize>,
}

impl ClientGroup {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            counter: CachePadded::new(AtomicUsize::new(0)),
        }
    }

    pub fn set(&self, client_id: String, conn: Connection) {
        self.entries.insert(client_id, conn);
    }

    pub fn remove(&self, client_id: &str) -> bool {
        self.entries.remove(client_id).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn select_healthy(&self) -> Option<Connection> {
        // Fast path: single connection (common case) — skip Vec allocation.
        if self.entries.len() == 1 {
            let conn = self.entries.iter().next()?.value().clone();
            self.counter.fetch_add(1, Ordering::Relaxed);
            return if conn.close_reason().is_none() { Some(conn) } else { None };
        }
        // General case: snapshot then round-robin scan.
        // Snapshot avoids holding DashMap shard locks during the health check loop.
        let conns: Vec<Connection> = self.entries.iter().map(|r| r.value().clone()).collect();
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
