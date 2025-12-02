use dashmap::DashMap;
use std::collections::HashMap;
use tunnel_lib::proto::tunnel::{Rule, Upstream};

/// Server state
pub struct ServerState {
    /// Map client_id -> Connection
    pub clients: DashMap<String, quinn::Connection>,
    /// Map client_id -> client_group
    pub client_groups: DashMap<String, String>,
    /// Map client_group -> Vec<client_id>
    pub group_clients: DashMap<String, Vec<String>>,
    /// Server ingress routing rules (for routing external requests to client groups)
    pub ingress_rules: Vec<IngressRule>,
    /// Client-specific configurations: Map client_group -> (rules, upstreams, config_version)
    pub client_configs: HashMap<String, (Vec<Rule>, Vec<Upstream>, String)>,
    /// Config version
    pub config_version: String,
}

#[derive(Debug, Clone)]
pub struct IngressRule {
    pub match_host: String,
    pub action_client_group: String,
}

