use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressVhostRule {
    pub match_host: String,
    pub group_id: String,
    pub proxy_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressListener {
    pub id: i64,
    pub port: u16,
    pub mode: IngressListenerMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngressListenerMode {
    Http { vhost: Vec<IngressVhostRule> },
    Tcp { group_id: String, proxy_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamServer {
    pub address: String,
    pub resolve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUpstream {
    pub name: String,
    pub lb_policy: String,
    pub servers: Vec<UpstreamServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientGroup {
    pub group_id: String,
    pub config_version: String,
    pub upstreams: Vec<ClientUpstream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressVhostRule {
    pub match_host: String,
    pub action_upstream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressUpstreamDef {
    pub name: String,
    pub lb_policy: String,
    pub servers: Vec<UpstreamServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingData {
    pub ingress_listeners: Vec<IngressListener>,
    pub client_groups: Vec<ClientGroup>,
    pub egress_upstreams: Vec<EgressUpstreamDef>,
    pub egress_vhost_rules: Vec<EgressVhostRule>,
}
#[async_trait]
pub trait RuleStore: Send + Sync {
    async fn load_routing(&self) -> Result<RoutingData>;
    async fn save_routing(&self, data: &RoutingData) -> Result<()>;
    async fn is_routing_empty(&self) -> Result<bool>;
}
