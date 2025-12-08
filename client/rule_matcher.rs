use anyhow::Result;
use std::collections::HashMap;
use tunnel_lib::proto::tunnel::Rule;

pub struct RuleMatcher {
    by_protocol_and_host: HashMap<String, Rule>,
    by_protocol: HashMap<String, Vec<Rule>>,
}

impl RuleMatcher {
    pub fn new() -> Self {
        Self {
            by_protocol_and_host: HashMap::new(),
            by_protocol: HashMap::new(),
        }
    }

    pub fn update_rules(&mut self, rules: Vec<Rule>) {
        self.by_protocol_and_host.clear();
        self.by_protocol.clear();

        for rule in rules {
            let protocol = rule.r#type.clone();
            
            if !rule.match_host.is_empty() {
                let host_without_port = rule.match_host
                    .split(':')
                    .next()
                    .unwrap_or(&rule.match_host)
                    .trim()
                    .to_lowercase();
                
                let key = format!("{}:{}", protocol, host_without_port);
                self.by_protocol_and_host.insert(key, rule.clone());
            }
            
            self.by_protocol
                .entry(protocol)
                .or_insert_with(Vec::new)
                .push(rule);
        }
    }

    pub fn match_rule(&self, protocol: &str, host: &str) -> Option<Rule> {
        let host_without_port = host
            .split(':')
            .next()
            .unwrap_or(host)
            .trim()
            .to_lowercase();
        
        let key = format!("{}:{}", protocol, host_without_port);
        
        if let Some(rule) = self.by_protocol_and_host.get(&key) {
            return Some(rule.clone());
        }
        
        if let Some(rules) = self.by_protocol.get(protocol) {
            for rule in rules {
                if rule.match_host.is_empty() {
                    return Some(rule.clone());
                }
            }
        }
        
        None
    }

    pub fn len(&self) -> usize {
        self.by_protocol_and_host.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_protocol_and_host.is_empty()
    }
}

impl Default for RuleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let mut matcher = RuleMatcher::new();
        
        let rules = vec![
            Rule {
                rule_id: "1".to_string(),
                r#type: "http".to_string(),
                match_host: "example.com".to_string(),
                match_path_prefix: String::new(),
                match_header: HashMap::new(),
                action_proxy_pass: "upstream1".to_string(),
                action_set_host: String::new(),
            },
        ];
        
        matcher.update_rules(rules);
        
        let result = matcher.match_rule("http", "example.com");
        assert!(result.is_some());
        assert_eq!(result.unwrap().rule_id, "1");
        
        let result = matcher.match_rule("http", "example.com:8080");
        assert!(result.is_some());
        
        let result = matcher.match_rule("http", "other.com");
        assert!(result.is_none());
    }
}
