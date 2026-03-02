use crossbeam_utils::CachePadded;
use dashmap::DashMap;
use parking_lot::RwLock;
use quinn::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};

struct ClientInfo {
    group_id: String,
    conn: Connection,
}

/// A group of QUIC connections that share a `group_id`.
///
/// Layout notes:
/// - `conns` and `ids` are stored as separate `Vec`s (SoA) so that the hot
///   path (`select_healthy`) only touches connection handles, leaving the
///   cold `String` id data in a different cacheline.
/// - `counter` is wrapped in `CachePadded` to prevent false-sharing with
///   `conns`/`ids` when multiple cores increment it concurrently.
pub struct ClientGroup {
    /// Active QUIC connections — touched on every routing decision.
    conns: RwLock<Vec<Connection>>,
    /// Corresponding client IDs — touched only on register/unregister.
    ids: RwLock<Vec<String>>,
    /// Round-robin cursor, padded to its own cacheline.
    counter: CachePadded<AtomicUsize>,
}

impl ClientGroup {
    pub fn new() -> Self {
        Self {
            conns: RwLock::new(Vec::new()),
            ids: RwLock::new(Vec::new()),
            counter: CachePadded::new(AtomicUsize::new(0)),
        }
    }

    pub fn add(&self, client_id: String, conn: Connection) {
        // Check if we're replacing an existing client_id
        {
            let mut ids = self.ids.write();
            if let Some(pos) = ids.iter().position(|id| id == &client_id) {
                ids[pos] = client_id.clone();
                drop(ids);
                self.conns.write()[pos] = conn;
                return;
            }
            ids.push(client_id);
        }
        self.conns.write().push(conn);
    }

    pub fn remove(&self, client_id: &str) -> bool {
        let mut ids = self.ids.write();
        if let Some(pos) = ids.iter().position(|id| id == client_id) {
            ids.swap_remove(pos);
            drop(ids);
            self.conns.write().swap_remove(pos);
            return true;
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.conns.read().is_empty()
    }

    /// Round-robin selection, skipping closed connections.
    /// Only acquires the read lock on `conns` — never touches `ids`.
    pub fn select_healthy(&self) -> Option<Connection> {
        let conns = self.conns.read();
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
