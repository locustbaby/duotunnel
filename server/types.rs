use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tunnel_lib::proto::tunnel::{Rule, Upstream};
use tunnel_lib::frame::ProtocolType;

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

pub struct ServerState {
    pub clients: DashMap<String, quinn::Connection>,
    pub client_groups: DashMap<String, String>,
    pub group_clients: DashMap<String, Vec<String>>,
    pub addr_to_client: DashMap<std::net::SocketAddr, String>,
    pub data_stream_semaphore: Arc<tokio::sync::Semaphore>,
    pub ingress_rules: Vec<IngressRule>,
    pub egress_rules_http: Vec<EgressRule>,
    pub egress_rules_grpc: Vec<GrpcEgressRule>,
    pub egress_upstreams: HashMap<String, EgressUpstream>,
    pub client_configs: HashMap<String, (Vec<Rule>, Vec<Upstream>, String)>,
    pub config_version: String,
    pub sessions: Arc<DashMap<u64, Arc<Mutex<SessionState>>>>,
    pub egress_pool: Arc<tunnel_lib::egress_pool::EgressPool>,
}

#[derive(Debug, Clone)]
pub struct IngressRule {
    pub match_host: String,
    pub action_client_group: String,
}

#[derive(Debug, Clone)]
pub struct EgressRule {
    pub match_host: String,
    pub action_upstream: String,
}

#[derive(Debug, Clone)]
pub struct GrpcEgressRule {
    pub match_host: String,
    pub match_service: String,
    pub action_upstream: String,
}

#[derive(Debug, Clone)]
pub struct EgressUpstream {
    pub name: String,
    pub address: String,
    pub is_ssl: bool,
}

