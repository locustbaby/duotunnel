use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use dashmap::DashMap;
use quinn::Connection;
use tracing::{info, warn, debug};

/// Stores metadata for client unregistration lookup
struct ClientInfo {
    group_id: String,
    conn: Connection,
}

/// Thread-safe client group with O(1) stable round-robin selection.
///
/// Uses `RwLock<Vec<(String, Connection)>>` instead of `DashMap` so that
/// `select_healthy` can index by position in O(1) with a deterministic order.
/// Write operations (add/remove) are rare — only on connect/disconnect — so
/// write-lock contention is negligible.
pub struct ClientGroup {
    clients: RwLock<Vec<(String, Connection)>>,
    counter: AtomicUsize,
}

impl ClientGroup {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(Vec::new()),
            counter: AtomicUsize::new(0),
        }
    }

    pub fn add(&self, client_id: String, conn: Connection) {
        let mut c = self.clients.write().unwrap();
        // Replace if the same client_id already exists (idempotent upsert)
        c.retain(|(id, _)| id != &client_id);
        c.push((client_id, conn));
    }

    pub fn remove(&self, client_id: &str) -> bool {
        let mut c = self.clients.write().unwrap();
        let before = c.len();
        c.retain(|(id, _)| id != client_id);
        c.len() < before
    }

    pub fn is_empty(&self) -> bool {
        self.clients.read().unwrap().is_empty()
    }

    /// Select a healthy client using O(1) round-robin.
    ///
    /// Iterates at most `len` slots starting from the atomic counter position,
    /// skipping any connection that has already been closed.
    pub fn select_healthy(&self) -> Option<Connection> {
        let c = self.clients.read().unwrap();
        let len = c.len();
        if len == 0 {
            return None;
        }

        let start = self.counter.fetch_add(1, Ordering::Relaxed);
        for i in 0..len {
            let (_, conn) = &c[(start + i) % len];
            if conn.close_reason().is_none() {
                return Some(conn.clone());
            }
        }

        None
    }
}

/// Lock-free client registry using DashMap
pub struct ClientRegistry {
    /// Group ID -> ClientGroup
    groups: DashMap<String, Arc<ClientGroup>>,
    /// Client ID -> ClientInfo (for unregistration lookup)
    clients: DashMap<String, ClientInfo>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            groups: DashMap::new(),
            clients: DashMap::new(),
        }
    }

    /// Atomically replace an existing registration or create a new one.
    ///
    /// Uses `DashMap::entry()` which holds a shard-level lock across the
    /// check-and-swap, eliminating the race window that existed with the
    /// previous get → unregister → register three-step sequence.
    ///
    /// Returns the old `Connection` if one was displaced (caller should close it).
    pub fn replace_or_register(
        &self,
        client_id: String,
        group_id: String,
        conn: Connection,
    ) -> Option<Connection> {
        use dashmap::mapref::entry::Entry;

        let old_conn = match self.clients.entry(client_id.clone()) {
            Entry::Occupied(mut occ) => {
                let old_info = occ.get();
                let old_group_id = old_info.group_id.clone();
                let old_conn = old_info.conn.clone();

                // Remove from the old group before overwriting
                if let Some(grp) = self.groups.get(&old_group_id) {
                    grp.remove(&client_id);
                    if grp.is_empty() {
                        drop(grp);
                        debug!(group_id = %old_group_id, "removing empty group");
                        self.groups.remove(&old_group_id);
                    }
                }

                occ.insert(ClientInfo {
                    group_id: group_id.clone(),
                    conn: conn.clone(),
                });

                Some(old_conn)
            }
            Entry::Vacant(vac) => {
                vac.insert(ClientInfo {
                    group_id: group_id.clone(),
                    conn: conn.clone(),
                });
                None
            }
        };

        // Add to the (possibly new) group
        self.groups
            .entry(group_id)
            .or_insert_with(|| Arc::new(ClientGroup::new()))
            .add(client_id, conn);

        old_conn
    }

    pub fn register(&self, client_id: String, group_id: String, conn: Connection) {
        info!(client_id = %client_id, group_id = %group_id, "registering client");
        self.replace_or_register(client_id, group_id, conn);
    }

    /// Get the connection for a specific client (for duplicate detection)
    pub fn get_client_connection(&self, client_id: &str) -> Option<Connection> {
        self.clients.get(client_id).map(|info| info.conn.clone())
    }

    pub fn unregister(&self, client_id: &str) {
        if let Some((_, info)) = self.clients.remove(client_id) {
            info!(client_id = %client_id, group_id = %info.group_id, "unregistering client");

            if let Some(group) = self.groups.get(&info.group_id) {
                group.remove(client_id);

                if group.is_empty() {
                    drop(group);
                    debug!(group_id = %info.group_id, "removing empty group");
                    self.groups.remove(&info.group_id);
                }
            }
        }
    }

    /// Select a healthy client from the specified group using round-robin
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
