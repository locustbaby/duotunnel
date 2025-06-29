use std::collections::HashMap;
use crate::config::Upstream;
use tunnel_lib::tunnel::Rule;
use tracing::{info, debug};

#[derive(Clone)]
pub struct ClientRulesEngine {
    http_rules: Vec<Rule>,
    grpc_rules: Vec<Rule>,
    upstreams: HashMap<String, Upstream>,
}

impl ClientRulesEngine {
    pub fn new() -> Self {
        Self {
            http_rules: Vec::new(),
            grpc_rules: Vec::new(),
            upstreams: HashMap::new(),
        }
    }

    pub async fn update_rules(&mut self, rules: Vec<Rule>, upstreams: Vec<tunnel_lib::tunnel::Upstream>) {
        let old_http = self.http_rules.clone();
        let old_grpc = self.grpc_rules.clone();
        let old_upstreams = self.upstreams.clone();
        // 构造新规则和 upstreams
        let mut new_http = Vec::new();
        let mut new_grpc = Vec::new();
        let mut new_upstreams = HashMap::new();
        for rule in rules {
            if !rule.match_service.is_empty() {
                new_grpc.push(rule);
            } else {
                new_http.push(rule);
            }
        }
        for upstream in upstreams {
            let up = Upstream {
                servers: upstream.servers.into_iter().map(|s| crate::config::ServerAddr {
                    address: s.address,
                    resolve: s.resolve,
                }).collect(),
                lb_policy: if upstream.lb_policy.is_empty() { None } else { Some(upstream.lb_policy) },
            };
            new_upstreams.insert(upstream.name, up);
        }
        debug!(
            "rules_update: old_http_rules={:?}, new_http_rules={:?}, old_grpc_rules={:?}, new_grpc_rules={:?}, old_upstreams={:?}, new_upstreams={:?}",
            old_http, new_http, old_grpc, new_grpc, old_upstreams, new_upstreams
        );
        if old_http != new_http || old_grpc != new_grpc || old_upstreams != new_upstreams {
            self.http_rules = new_http;
            self.grpc_rules = new_grpc;
            self.upstreams = new_upstreams;
            info!(
                event = "client_config_changed",
                old_rules_count = old_http.len() + old_grpc.len(),
                new_rules_count = self.http_rules.len() + self.grpc_rules.len(),
                old_upstreams_count = old_upstreams.len(),
                new_upstreams_count = self.upstreams.len(),
                message = "client config changed"
            );
        }
    }

    pub fn match_http_rule(&self, host: &str, path: &str) -> Option<&Rule> {
        self.http_rules.iter().find(|rule| {
            (rule.match_host.is_empty() || rule.match_host == host) &&
            (rule.match_path_prefix.is_empty() || path.starts_with(&rule.match_path_prefix))
        })
    }

    pub fn pick_backend(&self, upstream_name: &str) -> Option<String> {
        self.upstreams.get(upstream_name).and_then(|up| up.servers.get(0)).map(|s| s.address.clone())
    }

    pub fn debug_print_upstreams(&self) {
        println!("{:#?}", self.upstreams);
    }

    pub fn http_rules(&self) -> &Vec<Rule> {
        &self.http_rules
    }

    pub fn upstreams(&self) -> &HashMap<String, Upstream> {
        &self.upstreams
    }
} 