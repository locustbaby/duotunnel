use super::core::Protocol;
use crate::transport::listener::VhostRouter;
use std::collections::HashMap;

pub struct RuleMatchContext<'a> {
    pub port: Option<u16>,
    pub host: Option<&'a str>,
    pub protocol: Option<Protocol>,
}

impl<'a> RuleMatchContext<'a> {
    pub fn new(port: Option<u16>, host: Option<&'a str>, protocol: Option<Protocol>) -> Self {
        Self {
            port,
            host,
            protocol,
        }
    }
}

pub struct RuleSet<T: Clone + Send + Sync> {
    host_by_port: HashMap<u16, VhostRouter<T>>,
    host_only: VhostRouter<T>,
    port_only: HashMap<u16, T>,
    protocol_only: HashMap<Protocol, T>,
}

impl<T: Clone + Send + Sync> RuleSet<T> {
    pub fn new() -> Self {
        Self {
            host_by_port: HashMap::new(),
            host_only: VhostRouter::new(),
            port_only: HashMap::new(),
            protocol_only: HashMap::new(),
        }
    }

    pub fn add_port_host_rule(&mut self, port: u16, host: &str, value: T) {
        self.host_by_port
            .entry(port)
            .or_default()
            .add_route(host, value);
    }

    pub fn add_host_rule(&self, host: &str, value: T) {
        self.host_only.add_route(host, value);
    }

    pub fn add_port_rule(&mut self, port: u16, value: T) {
        self.port_only.insert(port, value);
    }

    pub fn add_protocol_rule(&mut self, protocol: Protocol, value: T) {
        self.protocol_only.insert(protocol, value);
    }

    pub fn resolve(&self, ctx: &RuleMatchContext<'_>) -> Option<T> {
        if let (Some(port), Some(host)) = (ctx.port, ctx.host) {
            if let Some(router) = self.host_by_port.get(&port) {
                if let Some(value) = router.get(host) {
                    return Some(value);
                }
            }
        }
        if let Some(host) = ctx.host {
            if let Some(value) = self.host_only.get(host) {
                return Some(value);
            }
        }
        if let Some(port) = ctx.port {
            if let Some(value) = self.port_only.get(&port) {
                return Some(value.clone());
            }
        }
        if let Some(protocol) = ctx.protocol {
            if let Some(value) = self.protocol_only.get(&protocol) {
                return Some(value.clone());
            }
        }
        None
    }
}

impl<T: Clone + Send + Sync> Default for RuleSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_port_host_before_port_only() {
        let mut rules = RuleSet::new();
        rules.add_port_rule(8080, "tcp".to_string());
        rules.add_port_host_rule(8080, "api.example.com", "http".to_string());
        let matched = rules.resolve(&RuleMatchContext::new(
            Some(8080),
            Some("api.example.com"),
            Some(Protocol::H1),
        ));
        assert_eq!(matched.as_deref(), Some("http"));
    }

    #[test]
    fn resolves_host_only_rules() {
        let rules = RuleSet::new();
        rules.add_host_rule("*.example.com", "egress".to_string());
        let matched = rules.resolve(&RuleMatchContext::new(
            None,
            Some("www.example.com"),
            Some(Protocol::H2),
        ));
        assert_eq!(matched.as_deref(), Some("egress"));
    }

    #[test]
    fn resolves_port_only_rules() {
        let mut rules = RuleSet::new();
        rules.add_port_rule(9000, "tcp".to_string());
        let matched = rules.resolve(&RuleMatchContext::new(
            Some(9000),
            None,
            Some(Protocol::Tcp),
        ));
        assert_eq!(matched.as_deref(), Some("tcp"));
    }
}
