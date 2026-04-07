use parking_lot::Mutex;
use std::sync::Arc;
use tracing::debug;
use tunnel_lib::{Balancer, BalancerExt, RoundRobin};

/// Process-level pool of active QUIC connections to the server.
///
/// Each endpoint thread registers its connection here on login and removes it
/// on disconnect. Callers use `next()` to pick a healthy connection according
/// to the active balancing policy (default: round-robin).
pub struct ConnectionPool {
    conns: Mutex<Vec<quinn::Connection>>,
    balancer: Arc<dyn Balancer>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self::with_balancer(Arc::new(RoundRobin::new()))
    }

    pub fn with_balancer(balancer: Arc<dyn Balancer>) -> Self {
        Self {
            conns: Mutex::new(Vec::new()),
            balancer,
        }
    }

    /// Register a connection. Called by each thread after successful login.
    pub fn add(&self, conn: quinn::Connection) {
        let mut guard = self.conns.lock();
        guard.push(conn);
        debug!(total = guard.len(), "connection pool: added connection");
    }

    /// Remove a connection by stable_id. Called when a connection closes.
    pub fn remove(&self, conn: &quinn::Connection) {
        let mut guard = self.conns.lock();
        let before = guard.len();
        guard.retain(|c: &quinn::Connection| c.stable_id() != conn.stable_id());
        debug!(
            removed = before - guard.len(),
            total = guard.len(),
            "connection pool: removed connection"
        );
    }

    /// Pick the next healthy connection via the active balancing policy.
    ///
    /// Closed connections are filtered out before the balancer sees them, so
    /// the policy operates only on live connections.
    pub fn next(&self) -> Option<quinn::Connection> {
        let guard = self.conns.lock();
        let healthy: Vec<&quinn::Connection> =
            guard.iter().filter(|c| c.close_reason().is_none()).collect();
        self.balancer.pick(&healthy).map(|c| (*c).clone())
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.conns.lock().len()
    }
}

pub type SharedConnectionPool = Arc<ConnectionPool>;

pub fn new_connection_pool() -> SharedConnectionPool {
    Arc::new(ConnectionPool::new())
}
