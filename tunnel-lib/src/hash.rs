use sha2::{Sha256, Digest};
use crate::proto::tunnel::{Rule, Upstream};

pub fn calculate_config_hash(rules: &[Rule], upstreams: &[Upstream]) -> String {
    let mut hasher = Sha256::new();
    

    let mut sorted_rules = rules.to_vec();
    sorted_rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
    
    for rule in sorted_rules {
        hasher.update(rule.rule_id.as_bytes());
        hasher.update(rule.r#type.as_bytes());
        hasher.update(rule.match_host.as_bytes());
        hasher.update(rule.match_path_prefix.as_bytes());
        hasher.update(rule.action_proxy_pass.as_bytes());
        hasher.update(rule.action_set_host.as_bytes());
        

        let mut headers: Vec<_> = rule.match_header.iter().collect();
        headers.sort_by_key(|(k, _)| *k);
        for (key, value) in headers {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }
    }
    

    let mut sorted_upstreams = upstreams.to_vec();
    sorted_upstreams.sort_by(|a, b| a.name.cmp(&b.name));
    
    for upstream in sorted_upstreams {
        hasher.update(upstream.name.as_bytes());
        hasher.update(upstream.lb_policy.as_bytes());
        

        let mut sorted_servers = upstream.servers.clone();
        sorted_servers.sort_by(|a, b| a.address.cmp(&b.address));
        for server in sorted_servers {
            hasher.update(server.address.as_bytes());
            hasher.update(&[if server.resolve { 1 } else { 0 }]);
        }
    }
    
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_hash_consistency() {
        let rules = vec![
            Rule {
                rule_id: "rule1".to_string(),
                r#type: "http".to_string(),
                match_host: "example.com".to_string(),
                match_path_prefix: "/api".to_string(),
                match_header: HashMap::new(),
                action_proxy_pass: "backend".to_string(),
                action_set_host: "".to_string(),
            },
        ];
        
        let upstreams = vec![
            Upstream {
                name: "backend".to_string(),
                servers: vec![],
                lb_policy: "round_robin".to_string(),
            },
        ];
        
        let hash1 = calculate_config_hash(&rules, &upstreams);
        let hash2 = calculate_config_hash(&rules, &upstreams);
        
        assert_eq!(hash1, hash2, "Hash should be consistent");
    }
    
    #[test]
    fn test_hash_changes_on_modification() {
        let rules1 = vec![
            Rule {
                rule_id: "rule1".to_string(),
                r#type: "http".to_string(),
                match_host: "example.com".to_string(),
                match_path_prefix: "/api".to_string(),
                match_header: HashMap::new(),
                action_proxy_pass: "backend".to_string(),
                action_set_host: "".to_string(),
            },
        ];
        
        let rules2 = vec![
            Rule {
                rule_id: "rule1".to_string(),
                r#type: "http".to_string(),
                match_host: "changed.com".to_string(),
                match_path_prefix: "/api".to_string(),
                match_header: HashMap::new(),
                action_proxy_pass: "backend".to_string(),
                action_set_host: "".to_string(),
            },
        ];
        
        let upstreams = vec![];
        
        let hash1 = calculate_config_hash(&rules1, &upstreams);
        let hash2 = calculate_config_hash(&rules2, &upstreams);
        
        assert_ne!(hash1, hash2, "Hash should change when config changes");
    }
}
