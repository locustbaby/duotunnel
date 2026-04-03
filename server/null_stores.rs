/// Stub implementations of RuleStore and ConfigSource used in ctld-managed mode.
/// In this mode the server receives all config via the ctld watch stream and never
/// queries SQLite directly; these satisfy the ServerState trait bounds without
/// opening a DB connection.
use anyhow::Result;
use crate::config::{ConfigSource, ServerEgressUpstream, TunnelManagement};
use tunnel_store::{RuleStore, rules::RoutingData};

pub struct NullRuleStore;

#[async_trait::async_trait]
impl RuleStore for NullRuleStore {
    async fn load_routing(&self) -> Result<RoutingData> {
        Ok(RoutingData {
            ingress_listeners: vec![],
            client_groups: vec![],
            egress_upstreams: vec![],
            egress_vhost_rules: vec![],
        })
    }
    async fn save_routing(&self, _: &RoutingData) -> Result<()> {
        anyhow::bail!("NullRuleStore: server is in ctld-managed mode")
    }
    async fn is_routing_empty(&self) -> Result<bool> {
        Ok(true)
    }
}

pub struct NullConfigSource;

#[async_trait::async_trait]
impl ConfigSource for NullConfigSource {
    async fn load(&self) -> Result<(TunnelManagement, ServerEgressUpstream)> {
        Ok((TunnelManagement::default(), ServerEgressUpstream::default()))
    }
}
