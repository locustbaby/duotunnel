use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data types (mirror server::config structures, kept here to avoid circular deps)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressVhostRule {
    pub match_host: String,
    pub action_client_group: String,
    pub action_proxy_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressTcpRule {
    pub match_port: u16,
    pub action_client_group: String,
    pub action_proxy_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamServer {
    pub address: String,
    pub resolve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamDef {
    pub servers: Vec<UpstreamServer>,
    pub lb_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressVhostRule {
    pub match_host: String,
    pub action_upstream: String,
}

/// One client group config (equivalent to GroupConfig in server/config.rs)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupConfig {
    pub config_version: String,
    pub upstreams: HashMap<String, UpstreamDef>,
    /// vhost routing rules for this group
    pub vhost_rules: Vec<EgressVhostRule>,
}

/// Full routing data that can be loaded from any ConfigSource
#[derive(Debug, Clone, Default)]
pub struct RoutingData {
    pub ingress_vhost: Vec<IngressVhostRule>,
    pub ingress_tcp: Vec<IngressTcpRule>,
    pub server_egress_upstreams: HashMap<String, UpstreamDef>,
    pub server_egress_vhost_rules: Vec<EgressVhostRule>,
    pub client_groups: HashMap<String, GroupConfig>,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

#[async_trait]
pub trait RuleStore: Send + Sync {
    /// Load all routing data from the backing store.
    async fn load_routing(&self) -> Result<RoutingData>;

    /// Persist a complete routing snapshot (upsert semantics).
    async fn save_routing(&self, data: &RoutingData) -> Result<()>;

    /// Returns true if the store has no routing rules (i.e. has never been seeded).
    /// Default impl loads all data and checks; backends may override with a cheaper query.
    async fn is_routing_empty(&self) -> Result<bool> {
        let data = self.load_routing().await?;
        Ok(data.ingress_vhost.is_empty()
            && data.ingress_tcp.is_empty()
            && data.client_groups.is_empty())
    }
}
