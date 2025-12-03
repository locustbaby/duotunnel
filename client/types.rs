use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tunnel_lib::proto::tunnel::{Rule, Upstream};
use tunnel_lib::frame::ProtocolType;

/// Client Identity information
#[derive(Debug, Clone)]
pub struct ClientIdentity {
    pub client_id: String,
    pub group_id: String,
    pub instance_id: String,
}

/// Session state for frame reassembly
pub struct SessionState {
    /// Protocol type
    pub protocol_type: ProtocolType,
    /// Accumulated payload data
    pub buffer: Vec<u8>,
    /// Whether the session is complete (received END_OF_STREAM)
    pub is_complete: bool,
    /// Timestamp when session was created
    pub created_at: Instant,
    /// Routing information (from first frame)
    pub routing_info: Option<(String, String)>, // (type, host)
}

impl SessionState {
    pub fn new(protocol_type: ProtocolType) -> Self {
        Self {
            protocol_type,
            buffer: Vec::new(),
            is_complete: false,
            created_at: Instant::now(),
            routing_info: None,
        }
    }

    /// Add frame payload to buffer
    pub fn add_frame(&mut self, payload: Vec<u8>, end_of_stream: bool) {
        self.buffer.extend_from_slice(&payload);
        self.is_complete = end_of_stream;
    }

    /// Check if session has timed out
    pub fn is_timed_out(&self, timeout: std::time::Duration) -> bool {
        self.created_at.elapsed() > timeout
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
    /// Active QUIC connection to server (wrapped in RwLock for dynamic updates)
    pub quic_connection: Arc<tokio::sync::RwLock<Option<Arc<quinn::Connection>>>>,
    /// Session state map: session_id -> SessionState
    pub sessions: Arc<DashMap<u64, Arc<tokio::sync::Mutex<SessionState>>>>,
    /// Egress connection pool (manages HTTP/HTTPS clients for upstreams)
    pub egress_pool: Arc<crate::egress_pool::EgressPool>,
}
