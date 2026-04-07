use parking_lot::Mutex;
use quinn::Connection;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::warn;

/// Per-thread client registry for connection-affine ingress dispatch.
///
/// Each QUIC endpoint thread maintains its own `LocalRegistry` containing only
/// the connections that arrived on *that* endpoint.  Ingress listeners running
/// on the same thread select from this registry, so every `open_bi()` call is
/// made within the same `current_thread` runtime — eliminating cross-runtime
/// park/unpark overhead.
pub struct LocalRegistry {
    /// group_id → list of (conn_id, Connection) pairs registered on this thread.
    groups: Mutex<HashMap<String, Vec<(String, Connection)>>>,
    /// Round-robin cursor per group (indexed by a stable position in the Vec).
    counters: Mutex<HashMap<String, AtomicUsize>>,
}

impl LocalRegistry {
    pub fn new() -> Self {
        Self {
            groups: Mutex::new(HashMap::new()),
            counters: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, conn_id: String, group_id: String, conn: Connection) {
        let mut groups = self.groups.lock();
        groups
            .entry(group_id)
            .or_default()
            .push((conn_id, conn));
    }

    pub fn unregister(&self, conn_id: &str) {
        let mut groups = self.groups.lock();
        for conns in groups.values_mut() {
            conns.retain(|(id, _)| id != conn_id);
        }
        // Remove empty groups.
        groups.retain(|_, v| !v.is_empty());
    }

    /// Select a healthy connection for `group_id` using round-robin.
    /// Returns `None` if no healthy connection exists on this thread.
    pub fn select_for_group(&self, group_id: &str) -> Option<Connection> {
        let groups = self.groups.lock();
        let conns = groups.get(group_id)?;
        let len = conns.len();
        if len == 0 {
            return None;
        }
        // Grab or create the counter for this group.
        // We hold groups lock here — use a simple index scan rather than a
        // separate atomic so we don't need two locks at once.
        let start = {
            drop(groups); // release groups lock before taking counters
            let mut counters = self.counters.lock();
            let ctr = counters
                .entry(group_id.to_string())
                .or_insert_with(|| AtomicUsize::new(0));
            ctr.fetch_add(1, Ordering::Relaxed)
        };
        // Re-acquire groups to do the selection.
        let groups = self.groups.lock();
        let conns = groups.get(group_id)?;
        let len = conns.len();
        if len == 0 {
            return None;
        }
        for i in 0..len {
            let (_, conn) = &conns[(start + i) % len];
            if conn.close_reason().is_none() {
                return Some(conn.clone());
            }
        }
        warn!(group_id = %group_id, "local_registry: no healthy connection on this thread");
        None
    }

}

pub type SharedLocalRegistry = Arc<LocalRegistry>;

pub fn new_local_registry() -> SharedLocalRegistry {
    Arc::new(LocalRegistry::new())
}
