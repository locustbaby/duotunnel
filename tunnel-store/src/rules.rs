use anyhow::Result;
use async_trait::async_trait;
pub use tunnel_lib::{
    ClientGroupDef as ClientGroup, ClientUpstreamDef as ClientUpstream,
    EgressUpstreamDef, EgressVhostRuleDef as EgressVhostRule,
    IngressListenerDef as IngressListener, IngressListenerModeDef as IngressListenerMode,
    IngressVhostRuleDef as IngressVhostRule, UpstreamServerDef as UpstreamServer,
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
