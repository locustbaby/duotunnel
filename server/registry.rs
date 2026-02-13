use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use dashmap::DashMap;
use quinn::Connection;
use tracing::{info, warn, debug};

/// Stores metadata for client unregistration lookup
struct ClientInfo {
    group_id: String,
    conn: Connection,
}

/// Thread-safe client group with O(1) round-robin selection
pub struct ClientGroup {
    /// Direct references to connections for O(1) access
    clients: DashMap<String, Connection>,
    /// Round-robin counter
    counter: AtomicUsize,
}

impl ClientGroup {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
            counter: AtomicUsize::new(0),
        }
    }

    pub fn add(&self, client_id: String, conn: Connection) {
        self.clients.insert(client_id, conn);
    }

    pub fn remove(&self, client_id: &str) -> bool {
        self.clients.remove(client_id).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Select a healthy client using round-robin. O(n) worst case when all clients
    /// are unhealthy, but O(1) amortized when most clients are healthy.
    pub fn select_healthy(&self) -> Option<Connection> {
        let len = self.clients.len();
        if len == 0 {
            return None;
        }

        let start_idx = self.counter.fetch_add(1, Ordering::Relaxed);

        // Try up to `len` clients starting from counter position
        for i in 0..len {
            let idx = (start_idx + i) % len;

            // Get the client at this index position
            if let Some(entry) = self.clients.iter().nth(idx) {
                let conn = entry.value();
                if conn.close_reason().is_none() {
                    return Some(conn.clone());
                }
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

    pub fn register(&self, client_id: String, group_id: String, conn: Connection) {
        info!(client_id = %client_id, group_id = %group_id, "registering client");

        // Store client info for unregistration lookup
        self.clients.insert(client_id.clone(), ClientInfo {
            group_id: group_id.clone(),
            conn: conn.clone(),
        });

        // Add to group (create if needed)
        self.groups
            .entry(group_id)
            .or_insert_with(|| Arc::new(ClientGroup::new()))
            .add(client_id, conn);
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

                // Optionally clean up empty groups
                if group.is_empty() {
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
