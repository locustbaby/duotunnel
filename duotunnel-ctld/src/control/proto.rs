pub use duotunnel_lib::ctld_proto::{
    ConfigDelta, ConfigEvent, ConfigOperation, ConfigSnapshot, ProtoClientGroup,
    ProtoEgressUpstreamDef, ProtoEgressVhostRule, ProtoIngressListener, TokenCacheEntry,
};

use crate::storage::rules::RoutingData;
use std::collections::{BTreeMap, BTreeSet};

pub fn routing_data_to_proto(
    data: &RoutingData,
) -> (
    Vec<ProtoIngressListener>,
    Vec<ProtoClientGroup>,
    Vec<ProtoEgressUpstreamDef>,
    Vec<ProtoEgressVhostRule>,
) {
    (
        data.ingress_listeners.clone(),
        data.client_groups.clone(),
        data.egress_upstreams.clone(),
        data.egress_vhost_rules.clone(),
    )
}

pub fn diff_snapshots(previous: &ConfigSnapshot, next: &ConfigSnapshot) -> Vec<ConfigOperation> {
    let mut operations = Vec::new();
    let previous_ingress: BTreeMap<_, _> = previous
        .ingress_listeners
        .iter()
        .map(|item| (item.port, item))
        .collect();
    let next_ingress: BTreeMap<_, _> = next
        .ingress_listeners
        .iter()
        .map(|item| (item.port, item))
        .collect();
    let mut ingress_keys = BTreeSet::new();
    ingress_keys.extend(previous_ingress.keys().copied());
    ingress_keys.extend(next_ingress.keys().copied());
    for key in ingress_keys {
        match (previous_ingress.get(&key), next_ingress.get(&key)) {
            (Some(previous), Some(next)) if *previous != *next => {
                operations.push(ConfigOperation::UpsertIngressListener((*next).clone()))
            }
            (None, Some(next)) => {
                operations.push(ConfigOperation::UpsertIngressListener((*next).clone()))
            }
            (Some(_), None) => {
                operations.push(ConfigOperation::DeleteIngressListener { port: key })
            }
            _ => {}
        }
    }

    let previous_groups: BTreeMap<_, _> = previous
        .client_groups
        .iter()
        .map(|item| (item.group_id.as_str().to_owned(), item))
        .collect();
    let next_groups: BTreeMap<_, _> = next
        .client_groups
        .iter()
        .map(|item| (item.group_id.as_str().to_owned(), item))
        .collect();
    let mut group_keys = BTreeSet::new();
    group_keys.extend(previous_groups.keys().cloned());
    group_keys.extend(next_groups.keys().cloned());
    for key in group_keys {
        match (previous_groups.get(&key), next_groups.get(&key)) {
            (Some(previous), Some(next)) if *previous != *next => {
                operations.push(ConfigOperation::UpsertClientGroup((*next).clone()))
            }
            (None, Some(next)) => {
                operations.push(ConfigOperation::UpsertClientGroup((*next).clone()))
            }
            (Some(_), None) => {
                operations.push(ConfigOperation::DeleteClientGroup { group_id: key })
            }
            _ => {}
        }
    }

    let previous_upstreams: BTreeMap<_, _> = previous
        .egress_upstreams
        .iter()
        .map(|item| (item.name.clone(), item))
        .collect();
    let next_upstreams: BTreeMap<_, _> = next
        .egress_upstreams
        .iter()
        .map(|item| (item.name.clone(), item))
        .collect();
    let mut upstream_keys = BTreeSet::new();
    upstream_keys.extend(previous_upstreams.keys().cloned());
    upstream_keys.extend(next_upstreams.keys().cloned());
    for key in upstream_keys {
        match (previous_upstreams.get(&key), next_upstreams.get(&key)) {
            (Some(previous), Some(next)) if *previous != *next => {
                operations.push(ConfigOperation::UpsertEgressUpstream((*next).clone()))
            }
            (None, Some(next)) => {
                operations.push(ConfigOperation::UpsertEgressUpstream((*next).clone()))
            }
            (Some(_), None) => operations.push(ConfigOperation::DeleteEgressUpstream { name: key }),
            _ => {}
        }
    }

    let previous_vhosts: BTreeMap<_, _> = previous
        .egress_vhost_rules
        .iter()
        .map(|item| (item.match_host.clone(), item))
        .collect();
    let next_vhosts: BTreeMap<_, _> = next
        .egress_vhost_rules
        .iter()
        .map(|item| (item.match_host.clone(), item))
        .collect();
    let mut vhost_keys = BTreeSet::new();
    vhost_keys.extend(previous_vhosts.keys().cloned());
    vhost_keys.extend(next_vhosts.keys().cloned());
    for key in vhost_keys {
        match (previous_vhosts.get(&key), next_vhosts.get(&key)) {
            (Some(previous), Some(next)) if *previous != *next => {
                operations.push(ConfigOperation::UpsertEgressVhostRule((*next).clone()))
            }
            (None, Some(next)) => {
                operations.push(ConfigOperation::UpsertEgressVhostRule((*next).clone()))
            }
            (Some(_), None) => {
                operations.push(ConfigOperation::DeleteEgressVhostRule { match_host: key })
            }
            _ => {}
        }
    }

    let previous_tokens: BTreeMap<_, _> = previous
        .token_cache
        .iter()
        .map(|item| (item.hash_hex.clone(), item))
        .collect();
    let next_tokens: BTreeMap<_, _> = next
        .token_cache
        .iter()
        .map(|item| (item.hash_hex.clone(), item))
        .collect();
    let mut token_keys = BTreeSet::new();
    token_keys.extend(previous_tokens.keys().cloned());
    token_keys.extend(next_tokens.keys().cloned());
    for key in token_keys {
        match (previous_tokens.get(&key), next_tokens.get(&key)) {
            (Some(previous), Some(next)) if *previous != *next => {
                operations.push(ConfigOperation::UpsertToken((*next).clone()))
            }
            (None, Some(next)) => operations.push(ConfigOperation::UpsertToken((*next).clone())),
            (Some(_), None) => operations.push(ConfigOperation::DeleteToken { hash_hex: key }),
            _ => {}
        }
    }

    operations
}

#[cfg(test)]
mod tests {
    use super::*;
    use duotunnel_lib::ctld_proto::apply_config_operations;

    #[test]
    fn diff_and_apply_reconstruct_target() {
        let base = ConfigSnapshot {
            resource_version: 1,
            ingress_listeners: vec![],
            client_groups: vec![],
            egress_upstreams: vec![],
            egress_vhost_rules: vec![],
            token_cache: vec![],
        };
        let target = ConfigSnapshot {
            resource_version: 2,
            egress_upstreams: vec![duotunnel_lib::EgressUpstreamDef {
                name: "api".into(),
                lb_policy: "round_robin".into(),
                servers: vec![],
            }],
            ..base.clone()
        };
        let operations = diff_snapshots(&base, &target);
        let mut applied = base;
        apply_config_operations(&mut applied, &operations).unwrap();
        applied.resource_version = target.resource_version;
        assert_eq!(applied, target);
    }
}
