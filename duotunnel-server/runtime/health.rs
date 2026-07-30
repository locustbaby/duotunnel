use parking_lot::RwLock;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Notify;

pub(crate) const CONTROL_DEGRADED_AFTER: Duration = Duration::from_secs(60);
pub(crate) const CONTROL_SECURITY_STALE_AFTER: Duration = Duration::from_secs(120);
pub(crate) const CONTROL_STALE_AFTER: Duration = Duration::from_secs(300);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlFreshness {
    Fresh,
    Degraded,
    Stale,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ListenerHealth {
    pub(crate) desired_generation: u64,
    pub(crate) active_generation: Option<u64>,
    pub(crate) error: Option<String>,
}

#[derive(Default)]
struct MutableFacts {
    listeners: BTreeMap<u16, ListenerHealth>,
    last_successfully_applied: Option<Instant>,
    last_control_confirmed: Option<Instant>,
    failed_component: Option<String>,
    required_components: BTreeSet<String>,
    running_components: BTreeSet<String>,
}

pub(crate) struct ServerHealthFacts {
    control_plane_required: bool,
    quic_bound: AtomicBool,
    config_valid: AtomicBool,
    config_applying: AtomicBool,
    security_fence_held: AtomicBool,
    mutable: RwLock<MutableFacts>,
    failed: Notify,
    degraded_after: Duration,
    security_stale_after: Duration,
    stale_after: Duration,
}

impl ServerHealthFacts {
    pub(crate) fn new(control_plane_required: bool) -> Self {
        Self::with_thresholds(
            control_plane_required,
            CONTROL_DEGRADED_AFTER,
            CONTROL_SECURITY_STALE_AFTER,
            CONTROL_STALE_AFTER,
        )
    }

    fn with_thresholds(
        control_plane_required: bool,
        degraded_after: Duration,
        security_stale_after: Duration,
        stale_after: Duration,
    ) -> Self {
        Self {
            control_plane_required,
            quic_bound: AtomicBool::new(false),
            config_valid: AtomicBool::new(!control_plane_required),
            config_applying: AtomicBool::new(false),
            security_fence_held: AtomicBool::new(false),
            mutable: RwLock::new(MutableFacts::default()),
            failed: Notify::new(),
            degraded_after,
            security_stale_after,
            stale_after,
        }
    }

    pub(crate) fn mark_quic_bound(&self, bound: bool) {
        self.quic_bound.store(bound, Ordering::Release);
    }

    pub(crate) fn begin_config_apply(&self) {
        self.config_applying.store(true, Ordering::Release);
    }

    pub(crate) fn finish_config_apply(&self) {
        let mut facts = self.mutable.write();
        let now = Instant::now();
        facts.last_successfully_applied = Some(now);
        facts.last_control_confirmed = Some(now);
        self.config_valid.store(true, Ordering::Release);
        self.security_fence_held.store(false, Ordering::Release);
        self.config_applying.store(false, Ordering::Release);
    }

    pub(crate) fn restore_config_applied(&self, age: Duration) {
        let mut facts = self.mutable.write();
        let restored = Instant::now().checked_sub(age);
        facts.last_successfully_applied = restored;
        facts.last_control_confirmed = restored;
        self.config_valid.store(true, Ordering::Release);
        self.security_fence_held.store(false, Ordering::Release);
        self.config_applying.store(false, Ordering::Release);
    }

    pub(crate) fn hold_config_apply_fence(&self) {
        self.security_fence_held.store(true, Ordering::Release);
        self.config_applying.store(true, Ordering::Release);
    }

    pub(crate) fn fail_config_apply(&self) {
        if !self.security_fence_held.load(Ordering::Acquire) {
            self.config_applying.store(false, Ordering::Release);
        }
    }

    pub(crate) fn confirm_control_freshness(&self) {
        self.mutable.write().last_control_confirmed = Some(Instant::now());
    }

    pub(crate) fn replace_listener_facts(
        &self,
        facts: BTreeMap<u16, ListenerHealth>,
        affected_ports: Option<&std::collections::HashSet<u16>>,
    ) {
        let mut mutable = self.mutable.write();
        if let Some(affected) = affected_ports {
            for port in affected {
                mutable.listeners.remove(port);
            }
            mutable.listeners.extend(facts);
        } else {
            mutable.listeners = facts;
        }
    }

    pub(crate) fn listener_worker_exited(&self, port: u16, generation: u64) {
        if let Some(listener) = self.mutable.write().listeners.get_mut(&port) {
            if listener.active_generation == Some(generation) {
                listener.active_generation = None;
                listener.error = Some("listener worker set exited unexpectedly".to_string());
            }
        }
    }

    pub(crate) fn set_required_components<'a>(
        &self,
        components: impl IntoIterator<Item = &'a str>,
    ) {
        self.mutable.write().required_components =
            components.into_iter().map(str::to_string).collect();
    }

    pub(crate) fn component_running(&self, component: &str) {
        self.mutable
            .write()
            .running_components
            .insert(component.to_string());
    }

    pub(crate) fn component_stopped(&self, component: &str) {
        self.mutable.write().running_components.remove(component);
    }

    pub(crate) fn component_failed(&self, component: &str, reason: &str) {
        let mut mutable = self.mutable.write();
        mutable.running_components.remove(component);
        mutable.failed_component = Some(format!("{component}: {reason}"));
        self.failed.notify_waiters();
    }

    pub(crate) async fn wait_for_failure(&self) -> String {
        loop {
            if let Some(failure) = self.mutable.read().failed_component.clone() {
                return failure;
            }
            self.failed.notified().await;
        }
    }

    pub(crate) fn control_freshness(&self) -> ControlFreshness {
        if !self.control_plane_required {
            return ControlFreshness::Fresh;
        }
        let Some(last_confirmed) = self.mutable.read().last_control_confirmed else {
            return ControlFreshness::Stale;
        };
        let age = last_confirmed.elapsed();
        if age >= self.stale_after {
            ControlFreshness::Stale
        } else if age >= self.degraded_after {
            ControlFreshness::Degraded
        } else {
            ControlFreshness::Fresh
        }
    }

    pub(crate) fn is_ready(&self) -> bool {
        let freshness = self.control_freshness();
        let mutable = self.mutable.read();
        self.quic_bound.load(Ordering::Acquire)
            && self.config_valid.load(Ordering::Acquire)
            && !self.config_applying.load(Ordering::Acquire)
            && freshness != ControlFreshness::Stale
            && mutable.failed_component.is_none()
            && mutable
                .required_components
                .is_subset(&mutable.running_components)
            && mutable.listeners.values().all(|listener| {
                listener.error.is_none()
                    && listener.active_generation == Some(listener.desired_generation)
            })
    }

    pub(crate) fn admits_new_work(&self) -> bool {
        self.is_ready()
    }

    pub(crate) fn admits_security_sensitive_work(&self) -> bool {
        if self.config_applying.load(Ordering::Acquire) {
            return false;
        }
        if !self.control_plane_required {
            return self.config_valid.load(Ordering::Acquire);
        }
        self.config_valid.load(Ordering::Acquire)
            && self
                .mutable
                .read()
                .last_control_confirmed
                .is_some_and(|last_confirmed| last_confirmed.elapsed() < self.security_stale_after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_stale_control_rejects_new_work() {
        let health = ServerHealthFacts::with_thresholds(
            true,
            Duration::from_millis(5),
            Duration::from_millis(8),
            Duration::from_millis(10),
        );
        health.mark_quic_bound(true);
        health.finish_config_apply();
        assert!(health.admits_new_work());

        std::thread::sleep(Duration::from_millis(12));

        assert_eq!(health.control_freshness(), ControlFreshness::Stale);
        assert!(!health.is_ready());
        assert!(!health.admits_new_work());
    }

    #[test]
    fn security_staleness_fences_auth_before_general_admission() {
        let health = ServerHealthFacts::with_thresholds(
            true,
            Duration::from_millis(2),
            Duration::from_millis(5),
            Duration::from_millis(20),
        );
        health.finish_config_apply();
        assert!(health.admits_security_sensitive_work());

        std::thread::sleep(Duration::from_millis(8));

        assert!(!health.admits_security_sensitive_work());
        assert_ne!(health.control_freshness(), ControlFreshness::Stale);
    }

    #[test]
    fn config_apply_fences_security_sensitive_work() {
        let health = ServerHealthFacts::new(false);
        assert!(health.admits_security_sensitive_work());

        health.begin_config_apply();

        assert!(!health.admits_security_sensitive_work());
        health.fail_config_apply();
        assert!(health.admits_security_sensitive_work());
    }

    #[test]
    fn held_security_fence_survives_failed_apply_until_success() {
        let health = ServerHealthFacts::new(false);
        health.begin_config_apply();
        health.hold_config_apply_fence();
        health.fail_config_apply();

        assert!(!health.admits_security_sensitive_work());
        assert!(!health.admits_new_work());

        health.finish_config_apply();
        assert!(health.admits_security_sensitive_work());
    }

    #[test]
    fn failed_required_listener_blocks_readiness() {
        let health = ServerHealthFacts::new(false);
        health.mark_quic_bound(true);
        health.replace_listener_facts(
            BTreeMap::from([(
                8080,
                ListenerHealth {
                    desired_generation: 2,
                    active_generation: None,
                    error: Some("bind failed".to_string()),
                },
            )]),
            None,
        );

        assert!(!health.is_ready());
    }
}
