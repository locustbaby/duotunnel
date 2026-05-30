pub use tunnel_lib::ctld_proto::{
    ConfigPatch, ConfigSnapshot, ProtoClientGroup, ProtoEgressUpstreamDef, ProtoEgressVhostRule,
    ProtoIngressListener, ResourceOp, TokenCacheEntry, WatchEvent,
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

pub fn build_patch(previous: &ConfigSnapshot, next: &ConfigSnapshot) -> ConfigPatch {
    ConfigPatch {
        resource_version: next.resource_version,
        ingress_listeners: diff_by_key(
            &previous.ingress_listeners,
            &next.ingress_listeners,
            |item| item.port.to_string(),
        ),
        client_groups: diff_by_key(&previous.client_groups, &next.client_groups, |item| {
            item.group_id.clone()
        }),
        egress_upstreams: diff_by_key(
            &previous.egress_upstreams,
            &next.egress_upstreams,
            |item| item.name.clone(),
        ),
        egress_vhost_rules: diff_by_key(
            &previous.egress_vhost_rules,
            &next.egress_vhost_rules,
            |item| item.match_host.clone(),
        ),
        token_cache: diff_by_key(&previous.token_cache, &next.token_cache, |item| {
            item.hash_hex.clone()
        }),
    }
}

fn diff_by_key<T, F>(previous: &[T], next: &[T], key_of: F) -> Vec<ResourceOp<T>>
where
    T: Clone + PartialEq,
    F: Fn(&T) -> String,
{
    let prev_map: BTreeMap<String, &T> = previous.iter().map(|item| (key_of(item), item)).collect();
    let next_map: BTreeMap<String, &T> = next.iter().map(|item| (key_of(item), item)).collect();
    let mut keys = BTreeSet::new();
    keys.extend(prev_map.keys().cloned());
    keys.extend(next_map.keys().cloned());
    let mut ops = Vec::new();
    for key in keys {
        match (prev_map.get(&key), next_map.get(&key)) {
            (Some(prev), Some(next)) if *prev != *next => ops.push(ResourceOp::Upsert((*next).clone())),
            (None, Some(next)) => ops.push(ResourceOp::Upsert((*next).clone())),
            (Some(_), None) => ops.push(ResourceOp::Delete { key }),
            _ => {}
        }
    }
    ops
}
