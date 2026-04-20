use std::sync::atomic::{AtomicUsize, Ordering};

use tunnel_lib::plugin::{LoadBalancer, PickCtx, Target};

/// Round-robin load balancer.
///
/// Stateful — holds a per-instance counter. Power-of-two list lengths use a
/// bitmask; other sizes fall back to modulo. Matches the behaviour of the
/// now-removed `UpstreamGroup::next`.
pub struct RoundRobinLb {
    counter: AtomicUsize,
}

impl RoundRobinLb {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinLb {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for RoundRobinLb {
    fn pick<'a>(&self, targets: &'a [Target], _ctx: &PickCtx) -> Option<&'a Target> {
        if targets.is_empty() {
            return None;
        }
        let raw = self.counter.fetch_add(1, Ordering::Relaxed);
        let len = targets.len();
        let idx = if len.is_power_of_two() {
            raw & (len - 1)
        } else {
            raw % len
        };
        targets.get(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tunnel_lib::proxy::tcp::UpstreamScheme;

    fn target(host: &str) -> Target {
        Target {
            host: host.to_string(),
            port: 80,
            scheme: UpstreamScheme::Http,
        }
    }

    #[test]
    fn round_robin_rotates_through_targets() {
        let lb = RoundRobinLb::new();
        let targets = vec![target("a"), target("b"), target("c")];
        let ctx = PickCtx {
            client_addr: "127.0.0.1:1".parse().unwrap(),
        };
        let picks: Vec<&str> = (0..6)
            .map(|_| lb.pick(&targets, &ctx).unwrap().host.as_str())
            .collect();
        assert_eq!(picks, vec!["a", "b", "c", "a", "b", "c"]);
    }

    #[test]
    fn round_robin_empty_returns_none() {
        let lb = RoundRobinLb::new();
        let ctx = PickCtx {
            client_addr: "127.0.0.1:1".parse().unwrap(),
        };
        assert!(lb.pick(&[], &ctx).is_none());
    }
}
