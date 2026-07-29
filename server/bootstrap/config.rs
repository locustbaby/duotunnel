// Re-export the canonical config types from tunnel-store so the rest of the
// server crate can keep importing from `crate::config`.
pub use tunnel_store::server_config::{
    ClientConfigs, EgressHttpRule, EgressRules, GroupConfig, HttpListenerDef as HttpListenerConfig,
    IngressListenerDef as IngressListener, IngressModeDef as IngressMode, IngressRouting,
    ServerConfigFile, ServerDef, ServerEgressUpstream, TcpListenerDef as TcpListenerConfig,
    TunnelManagement, UpstreamDef, VhostRuleDef as VhostRule,
};

// ── Client config helper ──────────────────────────────────────────────────────

pub fn build_client_config_for_group(
    tm: &TunnelManagement,
    egress_rules: &[tunnel_lib::EgressVhostRuleDef],
    group_id: &str,
) -> Option<tunnel_lib::ClientConfig> {
    let group = tm.client_configs.groups.get(group_id)?;
    let upstreams = group
        .upstreams
        .iter()
        .map(|(name, def)| tunnel_lib::UpstreamConfig {
            name: name.clone(),
            servers: def
                .servers
                .iter()
                .map(|s| tunnel_lib::UpstreamServer {
                    address: s.address.clone(),
                    resolve: s.resolve,
                })
                .collect(),
            lb_policy: def.lb_policy.clone(),
        })
        .collect();
    Some(tunnel_lib::ClientConfig {
        config_version: group.config_version.clone(),
        upstreams,
        egress_rules: egress_rules.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_build_client_config_for_group_not_found() {
        let tm = TunnelManagement {
            server_ingress_routing: Default::default(),
            client_configs: ClientConfigs {
                groups: HashMap::new(),
            },
        };
        assert!(build_client_config_for_group(&tm, &[], "nonexistent").is_none());
    }

    #[test]
    fn test_build_client_config_for_group_empty() {
        let mut groups = HashMap::new();
        groups.insert(
            "group1".to_string().into(),
            GroupConfig {
                config_version: "v1".to_string(),
                upstreams: HashMap::new(),
            },
        );
        let tm = TunnelManagement {
            server_ingress_routing: Default::default(),
            client_configs: ClientConfigs { groups },
        };

        let config =
            build_client_config_for_group(&tm, &[], "group1").expect("group1 should exist");
        assert_eq!(config.config_version, "v1");
        assert!(config.upstreams.is_empty());
    }

    #[test]
    fn test_build_client_config_for_group_success() {
        let mut upstreams = HashMap::new();
        upstreams.insert(
            "upstream1".to_string(),
            UpstreamDef {
                servers: vec![
                    ServerDef {
                        address: "127.0.0.1:8080".to_string(),
                        resolve: false,
                    },
                    ServerDef {
                        address: "example.com:80".to_string(),
                        resolve: true,
                    },
                ],
                lb_policy: "round_robin".to_string(),
            },
        );

        let mut groups = HashMap::new();
        groups.insert(
            "group_prod".to_string().into(),
            GroupConfig {
                config_version: "v2".to_string(),
                upstreams,
            },
        );

        let tm = TunnelManagement {
            server_ingress_routing: Default::default(),
            client_configs: ClientConfigs { groups },
        };

        let config =
            build_client_config_for_group(&tm, &[], "group_prod").expect("group_prod should exist");
        assert_eq!(config.config_version, "v2");
        assert_eq!(config.upstreams.len(), 1);

        let upstream = config
            .upstreams
            .iter()
            .find(|u| u.name == "upstream1")
            .expect("upstream1 missing");

        assert_eq!(upstream.lb_policy, "round_robin");
        assert_eq!(upstream.servers.len(), 2);

        assert_eq!(upstream.servers[0].address, "127.0.0.1:8080");
        assert!(!upstream.servers[0].resolve);

        assert_eq!(upstream.servers[1].address, "example.com:80");
        assert!(upstream.servers[1].resolve);
    }
}
