use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::config::{ServerConfig, Upstream, Rule, ClientGroupConfig};

pub struct RulesEngine {
    pub config: ServerConfig,
}

impl RulesEngine {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// 匹配 reverse_proxy 规则（入口请求 -> client group）
    pub fn match_reverse_proxy_rule(&self, host: &str, path: &str, service: Option<&str>) -> Option<&Rule> {
        let rules = &self.config.reverse_proxy.rules;
        Self::find_matching_rule(&rules.http, &rules.grpc, host, path, service)
    }

    /// 匹配 forward 规则（client tunnel 请求 -> 本地 upstream）
    pub fn match_forward_rule(&self, host: &str, path: &str, service: Option<&str>) -> Option<&Rule> {
        let rules = &self.config.forward.rules;
        Self::find_matching_rule(&rules.http, &rules.grpc, host, path, service)
    }

    /// 匹配 client group 下发规则（client 本地分发）
    pub fn match_client_group_rule<'a>(group: &'a ClientGroupConfig, host: &'a str, path: &'a str, service: Option<&'a str>) -> Option<&'a Rule> {
        let rules = &group.rules;
        Self::find_matching_rule(&rules.http, &rules.grpc, host, path, service)
    }

    fn find_matching_rule<'a>(http_rules: &'a [Rule], grpc_rules: &'a [Rule], host: &str, path: &str, service: Option<&str>) -> Option<&'a Rule> {
        // HTTP
        if service.is_none() {
            http_rules.iter().find(|rule| {
                rule.match_host.as_deref().map_or(true, |h| h == host) &&
                rule.match_path_prefix.as_deref().map_or(true, |p| path.starts_with(p))
            })
        } else {
            // gRPC
            grpc_rules.iter().find(|rule| {
                rule.match_host.as_deref().map_or(true, |h| h == host) &&
                rule.match_service.as_deref().map_or(true, |s| Some(s) == service)
            })
        }
    }

    pub fn get_upstream(&self, name: &str) -> Option<&Upstream> {
        self.config.forward.upstreams.get(name)
    }
    pub fn get_group(&self, group: &str) -> Option<&ClientGroupConfig> {
        self.config.client_groups.get(group)
    }
}

// Rule helper methods for proxy logic
impl Rule {
    /// Returns true if this rule is a reverse proxy rule (matches client group)
    pub fn is_reverse_proxy_rule(&self) -> bool {
        self.action_client_group.is_some()
    }
    /// Returns true if this rule is a client group rule (reverse proxy to group)
    pub fn is_client_group_rule(&self) -> bool {
        self.action_client_group.is_some()
    }
    /// Returns the client group name if this is a client group rule
    pub fn extract_client_group(&self) -> Option<&str> {
        self.action_client_group.as_deref()
    }
    /// Returns true if this rule is a forward proxy rule (matches upstream)
    pub fn is_forward_proxy_rule(&self) -> bool {
        self.action_upstream.is_some()
    }
} 