use anyhow::Result;
use async_trait::async_trait;
pub use duotunnel_lib::config::file::RoutingData;
pub use duotunnel_lib::{
    ClientGroupDef as ClientGroup, ClientUpstreamDef as ClientUpstream, EgressUpstreamDef,
    EgressVhostRuleDef as EgressVhostRule, GroupId, IngressListenerDef as IngressListener,
    IngressListenerModeDef as IngressListenerMode, IngressVhostRuleDef as IngressVhostRule,
    UpstreamServerDef as UpstreamServer,
};

#[async_trait]
pub trait RuleStore: Send + Sync {
    async fn load_routing(&self) -> Result<RoutingData>;
    #[cfg(test)]
    async fn save_routing(&self, data: &RoutingData) -> Result<()>;
}
