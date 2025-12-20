use std::collections::HashMap;
use crate::types::{IngressRule, EgressRule, GrpcEgressRule};

/// RuleMatcher for server-side routing with O(1) lookup performance
/// Supports ingress rules (HTTP, WSS) and egress rules (HTTP, gRPC)
pub struct ServerRuleMatcher {
    // Ingress rules: protocol:host -> IngressRule
    ingress_by_protocol_and_host: HashMap<String, IngressRule>,
    // Egress HTTP rules: host -> EgressRule
    egress_http_by_host: HashMap<String, EgressRule>,
    // Egress gRPC rules: host:service -> GrpcEgressRule
    egress_grpc_by_host_and_service: HashMap<String, GrpcEgressRule>,
}

impl ServerRuleMatcher {
    pub fn new() -> Self {
        Self {
            ingress_by_protocol_and_host: HashMap::new(),
            egress_http_by_host: HashMap::new(),
            egress_grpc_by_host_and_service: HashMap::new(),
        }
    }

    /// Update ingress rules (HTTP, WSS)
    pub fn update_ingress_rules(&mut self, rules: Vec<IngressRule>, protocol: &str) {
        for rule in rules {
            let host_without_port = rule.match_host
                .split(':')
                .next()
                .unwrap_or(&rule.match_host)
                .trim()
                .to_lowercase();
            
            let key = format!("{}:{}", protocol, host_without_port);
            self.ingress_by_protocol_and_host.insert(key, rule);
        }
    }

    /// Update egress HTTP rules
    pub fn update_egress_http_rules(&mut self, rules: Vec<EgressRule>) {
        self.egress_http_by_host.clear();
        for rule in rules {
            let host_without_port = rule.match_host
                .split(':')
                .next()
                .unwrap_or(&rule.match_host)
                .trim()
                .to_lowercase();
            
            self.egress_http_by_host.insert(host_without_port, rule);
        }
    }

    /// Update egress gRPC rules
    pub fn update_egress_grpc_rules(&mut self, rules: Vec<GrpcEgressRule>) {
        self.egress_grpc_by_host_and_service.clear();
        for rule in rules {
            let host_without_port = rule.match_host
                .split(':')
                .next()
                .unwrap_or(&rule.match_host)
                .trim()
                .to_lowercase();
            
            let key = format!("{}:{}", host_without_port, rule.match_service);
            self.egress_grpc_by_host_and_service.insert(key, rule);
        }
    }

    /// Match ingress rule (HTTP, WSS)
    pub fn match_ingress_rule(&self, protocol: &str, host: &str) -> Option<IngressRule> {
        let host_without_port = host
            .split(':')
            .next()
            .unwrap_or(host)
            .trim()
            .to_lowercase();
        
        let key = format!("{}:{}", protocol, host_without_port);
        self.ingress_by_protocol_and_host.get(&key).cloned()
    }

    /// Match egress HTTP rule
    pub fn match_egress_http_rule(&self, host: &str) -> Option<EgressRule> {
        let host_without_port = host
            .split(':')
            .next()
            .unwrap_or(host)
            .trim()
            .to_lowercase();
        
        self.egress_http_by_host.get(&host_without_port).cloned()
    }

    /// Match egress gRPC rule
    pub fn match_egress_grpc_rule(&self, host: &str, service: &str) -> Option<GrpcEgressRule> {
        let host_without_port = host
            .split(':')
            .next()
            .unwrap_or(host)
            .trim()
            .to_lowercase();
        
        let key = format!("{}:{}", host_without_port, service);
        self.egress_grpc_by_host_and_service.get(&key).cloned()
    }

    pub fn ingress_rule_count(&self) -> usize {
        self.ingress_by_protocol_and_host.len()
    }

    pub fn egress_http_rule_count(&self) -> usize {
        self.egress_http_by_host.len()
    }

    pub fn egress_grpc_rule_count(&self) -> usize {
        self.egress_grpc_by_host_and_service.len()
    }
}

impl Default for ServerRuleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingress_match() {
        let mut matcher = ServerRuleMatcher::new();
        
        let rules = vec![
            IngressRule {
                match_host: "example.com".to_string(),
                action_client_group: "group-a".to_string(),
            },
        ];
        
        matcher.update_ingress_rules(rules, "http");
        
        let result = matcher.match_ingress_rule("http", "example.com");
        assert!(result.is_some());
        assert_eq!(result.unwrap().action_client_group, "group-a");
        
        let result = matcher.match_ingress_rule("http", "example.com:8080");
        assert!(result.is_some());
        
        let result = matcher.match_ingress_rule("http", "other.com");
        assert!(result.is_none());
    }

    #[test]
    fn test_egress_http_match() {
        let mut matcher = ServerRuleMatcher::new();
        
        let rules = vec![
            EgressRule {
                match_host: "backend.com".to_string(),
                action_upstream: "backend".to_string(),
            },
        ];
        
        matcher.update_egress_http_rules(rules);
        
        let result = matcher.match_egress_http_rule("backend.com");
        assert!(result.is_some());
        assert_eq!(result.unwrap().action_upstream, "backend");
        
        let result = matcher.match_egress_http_rule("backend.com:443");
        assert!(result.is_some());
        
        let result = matcher.match_egress_http_rule("other.com");
        assert!(result.is_none());
    }

    #[test]
    fn test_egress_grpc_match() {
        let mut matcher = ServerRuleMatcher::new();
        
        let rules = vec![
            GrpcEgressRule {
                match_host: "grpc.backend.com".to_string(),
                match_service: "com.example.Service".to_string(),
                action_upstream: "grpc_backend".to_string(),
            },
        ];
        
        matcher.update_egress_grpc_rules(rules);
        
        let result = matcher.match_egress_grpc_rule("grpc.backend.com", "com.example.Service");
        assert!(result.is_some());
        assert_eq!(result.unwrap().action_upstream, "grpc_backend");
        
        let result = matcher.match_egress_grpc_rule("grpc.backend.com:9000", "com.example.Service");
        assert!(result.is_some());
        
        let result = matcher.match_egress_grpc_rule("grpc.backend.com", "other.Service");
        assert!(result.is_none());
    }
}

