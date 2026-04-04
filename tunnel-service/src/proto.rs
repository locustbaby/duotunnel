/// Re-export the list-watch protocol types from tunnel-lib.
/// Types live in tunnel-lib so that server can depend on tunnel-lib only,
/// without depending on tunnel-service.
pub use tunnel_lib::ctld_proto::{
    ConfigSnapshot, ProtoClientGroup, ProtoClientUpstream, ProtoEgressUpstreamDef,
    ProtoEgressVhostRule, ProtoIngressListener, ProtoIngressListenerMode, ProtoIngressVhostRule,
    ProtoUpstreamServer, TokenCacheEntry, WatchEvent, WatchRequest,
};

use tunnel_store::rules::{
    ClientGroup, EgressUpstreamDef, EgressVhostRule, IngressListener, IngressListenerMode,
    RoutingData,
};

/// Convert tunnel_store RoutingData into the proto types used in ConfigSnapshot.
pub fn routing_data_to_proto(
    data: &RoutingData,
) -> (
    Vec<ProtoIngressListener>,
    Vec<ProtoClientGroup>,
    Vec<ProtoEgressUpstreamDef>,
    Vec<ProtoEgressVhostRule>,
) {
    let ingress = data.ingress_listeners.iter().map(proto_listener).collect();
    let groups = data.client_groups.iter().map(proto_group).collect();
    let egress_upstreams = data
        .egress_upstreams
        .iter()
        .map(proto_egress_upstream)
        .collect();
    let egress_vhost = data
        .egress_vhost_rules
        .iter()
        .map(proto_egress_vhost)
        .collect();
    (ingress, groups, egress_upstreams, egress_vhost)
}

fn proto_listener(l: &IngressListener) -> ProtoIngressListener {
    ProtoIngressListener {
        port: l.port,
        mode: match &l.mode {
            IngressListenerMode::Http { vhost } => ProtoIngressListenerMode::Http {
                vhost: vhost
                    .iter()
                    .map(|r| ProtoIngressVhostRule {
                        match_host: r.match_host.clone(),
                        group_id: r.group_id.clone(),
                        proxy_name: r.proxy_name.clone(),
                    })
                    .collect(),
            },
            IngressListenerMode::Tcp {
                group_id,
                proxy_name,
            } => ProtoIngressListenerMode::Tcp {
                group_id: group_id.clone(),
                proxy_name: proxy_name.clone(),
            },
        },
    }
}

fn proto_group(g: &ClientGroup) -> ProtoClientGroup {
    ProtoClientGroup {
        group_id: g.group_id.clone(),
        config_version: g.config_version.clone(),
        upstreams: g
            .upstreams
            .iter()
            .map(|u| ProtoClientUpstream {
                name: u.name.clone(),
                lb_policy: u.lb_policy.clone(),
                servers: u
                    .servers
                    .iter()
                    .map(|s| ProtoUpstreamServer {
                        address: s.address.clone(),
                        resolve: s.resolve,
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn proto_egress_upstream(u: &EgressUpstreamDef) -> ProtoEgressUpstreamDef {
    ProtoEgressUpstreamDef {
        name: u.name.clone(),
        lb_policy: u.lb_policy.clone(),
        servers: u
            .servers
            .iter()
            .map(|s| ProtoUpstreamServer {
                address: s.address.clone(),
                resolve: s.resolve,
            })
            .collect(),
    }
}

fn proto_egress_vhost(r: &EgressVhostRule) -> ProtoEgressVhostRule {
    ProtoEgressVhostRule {
        match_host: r.match_host.clone(),
        action_upstream: r.action_upstream.clone(),
    }
}
