pub use tunnel_lib::ctld_proto::{
    ConfigDelta, ConfigEvent, ConfigOperation, ConfigSnapshot, ProtoClientGroup,
    ProtoEgressUpstreamDef, ProtoEgressVhostRule, ProtoIngressListener, TokenCacheEntry,
};

use std::collections::{BTreeMap, BTreeSet};
use tunnel_store::rules::RoutingData;

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
    diff_by_key(
        &previous.ingress_listeners,
        &next.ingress_listeners,
        |item| item.port,
        |item| ConfigOperation::UpsertIngressListener(item.clone()),
        |key| ConfigOperation::DeleteIngressListener { port: *key },
        &mut operations,
    );
    diff_by_key(
        &previous.client_groups,
        &next.client_groups,
        |item| item.group_id.as_str().to_owned(),
        |item| ConfigOperation::UpsertClientGroup(item.clone()),
        |key| ConfigOperation::DeleteClientGroup {
            group_id: key.clone(),
        },
        &mut operations,
    );
    diff_by_key(
        &previous.egress_upstreams,
        &next.egress_upstreams,
        |item| item.name.clone(),
        |item| ConfigOperation::UpsertEgressUpstream(item.clone()),
        |key| ConfigOperation::DeleteEgressUpstream { name: key.clone() },
        &mut operations,
    );
    diff_by_key(
        &previous.egress_vhost_rules,
        &next.egress_vhost_rules,
        |item| item.match_host.clone(),
        |item| ConfigOperation::UpsertEgressVhostRule(item.clone()),
        |key| ConfigOperation::DeleteEgressVhostRule {
            match_host: key.clone(),
        },
        &mut operations,
    );
    diff_by_key(
        &previous.token_cache,
        &next.token_cache,
        |item| item.hash_hex.clone(),
        |item| ConfigOperation::UpsertToken(item.clone()),
        |key| ConfigOperation::DeleteToken {
            hash_hex: key.clone(),
        },
        &mut operations,
    );
    operations
}

fn diff_by_key<T, K, FU, FD>(
    previous: &[T],
    next: &[T],
    key_of: impl Fn(&T) -> K,
    upsert: FU,
    delete: FD,
    operations: &mut Vec<ConfigOperation>,
) where
    T: PartialEq,
    K: Ord + Clone,
    FU: Fn(&T) -> ConfigOperation,
    FD: Fn(&K) -> ConfigOperation,
{
    let prev_map: BTreeMap<K, &T> = previous.iter().map(|item| (key_of(item), item)).collect();
    let next_map: BTreeMap<K, &T> = next.iter().map(|item| (key_of(item), item)).collect();
    let mut keys = BTreeSet::new();
    keys.extend(prev_map.keys().cloned());
    keys.extend(next_map.keys().cloned());
    for key in keys {
        match (prev_map.get(&key), next_map.get(&key)) {
            (Some(previous), Some(next)) if *previous != *next => operations.push(upsert(next)),
            (None, Some(next)) => operations.push(upsert(next)),
            (Some(_), None) => operations.push(delete(&key)),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tunnel_lib::ctld_proto::apply_config_operations;

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
            egress_upstreams: vec![tunnel_lib::EgressUpstreamDef {
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
