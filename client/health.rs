use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HealthSnapshot {
    pub(crate) ready: bool,
    pub(crate) degraded: bool,
    pub(crate) entry_listener_ready: bool,
    pub(crate) pool_actor_alive: bool,
    pub(crate) active_tunnels: usize,
    pub(crate) desired_tunnels: usize,
    pub(crate) min_ready_tunnels: usize,
}

pub(crate) struct ClientHealth {
    entry_listener_required: bool,
    entry_listener_up: AtomicBool,
    pool_actor_alive: AtomicBool,
    active_tunnels: AtomicUsize,
    desired_tunnels: usize,
    min_ready_tunnels: usize,
}

impl ClientHealth {
    pub(crate) fn new(
        entry_listener_required: bool,
        desired_tunnels: usize,
        min_ready_tunnels: usize,
    ) -> Self {
        let desired_tunnels = desired_tunnels.max(1);
        Self {
            entry_listener_required,
            entry_listener_up: AtomicBool::new(false),
            pool_actor_alive: AtomicBool::new(false),
            active_tunnels: AtomicUsize::new(0),
            desired_tunnels,
            min_ready_tunnels: min_ready_tunnels.max(1).min(desired_tunnels),
        }
    }

    pub(crate) fn set_entry_listener_up(&self, up: bool) {
        self.entry_listener_up.store(up, Ordering::Release);
    }

    pub(crate) fn set_pool_actor_alive(&self, alive: bool) {
        self.pool_actor_alive.store(alive, Ordering::Release);
    }

    pub(crate) fn set_active_tunnels(&self, active: usize) {
        self.active_tunnels.store(active, Ordering::Release);
    }

    pub(crate) fn snapshot(&self) -> HealthSnapshot {
        let active_tunnels = self.active_tunnels.load(Ordering::Acquire);
        let pool_actor_alive = self.pool_actor_alive.load(Ordering::Acquire);
        let entry_listener_ready =
            !self.entry_listener_required || self.entry_listener_up.load(Ordering::Acquire);
        HealthSnapshot {
            ready: entry_listener_ready
                && pool_actor_alive
                && active_tunnels >= self.min_ready_tunnels,
            degraded: active_tunnels < self.desired_tunnels,
            entry_listener_ready,
            pool_actor_alive,
            active_tunnels,
            desired_tunnels: self.desired_tunnels,
            min_ready_tunnels: self.min_ready_tunnels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_tunnel_removal_does_not_clear_aggregate_readiness() {
        let health = ClientHealth::new(false, 2, 1);
        health.set_pool_actor_alive(true);
        health.set_active_tunnels(2);
        assert!(health.snapshot().ready);
        assert!(!health.snapshot().degraded);

        health.set_active_tunnels(1);
        let snapshot = health.snapshot();
        assert!(snapshot.ready);
        assert!(snapshot.degraded);
        assert_eq!(snapshot.active_tunnels, 1);
    }

    #[test]
    fn required_entry_listener_participates_in_readiness() {
        let health = ClientHealth::new(true, 1, 1);
        health.set_pool_actor_alive(true);
        health.set_active_tunnels(1);
        assert!(!health.snapshot().ready);

        health.set_entry_listener_up(true);
        assert!(health.snapshot().ready);
    }
}
