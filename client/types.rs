use dashmap::DashMap;
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::Mutex;
use tokio_rustls;
use tunnel_lib::proto::tunnel::{Rule, Upstream};

/// Pooled connection entry
pub enum PooledConnection {
    Tcp(TokioTcpStream),
    Tls(tokio_rustls::client::TlsStream<TokioTcpStream>),
}

/// Connection pool entry
pub struct ConnectionPoolEntry {
    pub connections: Vec<(PooledConnection, Instant)>, // (connection, created_at)
    pub max_size: usize,
    pub idle_timeout: std::time::Duration,
}

impl ConnectionPoolEntry {
    pub fn new(max_size: usize, idle_timeout: std::time::Duration) -> Self {
        Self {
            connections: Vec::new(),
            max_size,
            idle_timeout,
        }
    }
    
    /// Get a connection from pool, or return None if pool is empty
    pub fn get(&mut self) -> Option<PooledConnection> {
        // Clean up idle connections first
        let now = Instant::now();
        self.connections.retain(|(_, created_at)| {
            now.duration_since(*created_at) < self.idle_timeout
        });
        
        // Return the first available connection
        if let Some((conn, _)) = self.connections.pop() {
            Some(conn)
        } else {
            None
        }
    }
    
    /// Return a connection to the pool
    pub fn put(&mut self, conn: PooledConnection) {
        // Only keep connections if pool is not full
        if self.connections.len() < self.max_size {
            self.connections.push((conn, Instant::now()));
        }
        // Otherwise, drop the connection (pool is full)
    }
}

pub struct ClientState {
    /// Current rules from server
    pub rules: Arc<DashMap<String, Rule>>,
    /// Current upstreams from server
    pub upstreams: Arc<DashMap<String, Upstream>>,
    /// Current config version
    pub config_version: Arc<tokio::sync::RwLock<String>>,
    /// Current config hash
    pub config_hash: Arc<tokio::sync::RwLock<String>>,
    /// TLS connector (reused for all HTTPS connections)
    pub tls_connector: Arc<tokio_rustls::TlsConnector>,
    /// Connection pool: key is "hostname:port:is_ssl", value is connection pool
    pub connection_pool: Arc<DashMap<String, Arc<Mutex<ConnectionPoolEntry>>>>,
    /// Active QUIC connection to server (wrapped in RwLock for dynamic updates)
    pub quic_connection: Arc<tokio::sync::RwLock<Option<Arc<quinn::Connection>>>>,
}

