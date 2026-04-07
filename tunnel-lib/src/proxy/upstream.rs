use super::balancer::{Balancer, BalancerExt, RoundRobin};
use std::sync::Arc;

/// A named group of upstream server addresses with a pluggable load-balancing
/// policy. Defaults to round-robin; swap the balancer to change the algorithm
/// without touching callers.
pub struct UpstreamGroup {
    pub servers: Vec<String>,
    balancer: Arc<dyn Balancer>,
}

impl UpstreamGroup {
    /// Create a group with the default round-robin balancer.
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            balancer: Arc::new(RoundRobin::new()),
        }
    }

    /// Create a group from a policy name string (e.g. from config YAML).
    ///
    /// Recognises `"round_robin"` (the default). Any unrecognised value falls
    /// back to round-robin and logs a warning.
    pub fn from_policy(servers: Vec<String>, policy: &str) -> Self {
        let balancer: Arc<dyn Balancer> = match policy {
            "round_robin" | "" => Arc::new(RoundRobin::new()),
            other => {
                tracing::warn!(
                    lb_policy = other,
                    "unknown lb_policy — falling back to round_robin"
                );
                Arc::new(RoundRobin::new())
            }
        };
        Self { servers, balancer }
    }

    /// Create a group with a custom balancer.
    pub fn with_balancer(servers: Vec<String>, balancer: Arc<dyn Balancer>) -> Self {
        Self { servers, balancer }
    }

    /// Pick the next server address according to the active balancing policy.
    pub fn next(&self) -> Option<&String> {
        self.balancer.pick(&self.servers)
    }

    pub fn first(&self) -> Option<&String> {
        self.servers.first()
    }
}
