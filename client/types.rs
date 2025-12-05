use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tunnel_lib::proto::tunnel::{Rule, Upstream};
use tunnel_lib::frame::ProtocolType;

#[derive(Debug, Clone)]
pub struct ClientIdentity {
    pub client_id: String,
    pub group_id: String,
    pub instance_id: String,
}

pub struct SessionState {

    pub protocol_type: ProtocolType,

    pub buffer: Vec<u8>,

    pub is_complete: bool,

    pub created_at: Instant,

    pub routing_info: Option<(String, String)>,
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


    pub fn add_frame(&mut self, payload: Vec<u8>, end_of_stream: bool) {
        self.buffer.extend_from_slice(&payload);
        self.is_complete = end_of_stream;
    }


    pub fn is_timed_out(&self, timeout: std::time::Duration) -> bool {
        self.created_at.elapsed() > timeout
    }
}

pub struct ClientState {

    pub rules: Arc<DashMap<String, Rule>>,

    pub upstreams: Arc<DashMap<String, Upstream>>,

    pub config_version: Arc<tokio::sync::RwLock<String>>,

    pub config_hash: Arc<tokio::sync::RwLock<String>>,

    pub quic_connection: Arc<tokio::sync::RwLock<Option<Arc<quinn::Connection>>>>,

    pub sessions: Arc<DashMap<u64, Arc<tokio::sync::Mutex<SessionState>>>>,

    pub egress_pool: Arc<tunnel_lib::egress_pool::EgressPool>,
}
