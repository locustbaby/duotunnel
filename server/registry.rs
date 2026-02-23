use dashmap::DashMap;
use quinn::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

struct ClientInfo {
    group_id: String,
    conn: Connection,
}

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
