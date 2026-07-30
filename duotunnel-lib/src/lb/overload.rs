use crate::lb::inflight::{inflight_load, inflight_notify, ConnectionState};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverloadMode {
    #[default]
    InflightSlowpath,
    Burst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackoffStrategy {
    None,
    Fixed,
    #[default]
    Exponential,
}

#[derive(Debug, Clone)]
pub struct OverloadLimits {
    pub mode: OverloadMode,
    pub inflight_yield_threshold: usize,
    pub inflight_sleep_threshold: usize,
    pub max_pending_streams: usize,
    pub backoff: BackoffStrategy,
    pub inflight_sleep_budget: Duration,
}

/// Admission limits for resources whose lifetime is owned by a caller.
///
/// `global` bounds all permits. `per_group` adds independent bounds for the
/// named groups; groups without an entry remain governed only by `global`.
/// A zero limit is valid and intentionally rejects every attempt in that
/// scope, which is useful while draining or disabling a group.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdmissionLimits {
    pub global: Option<usize>,
    pub per_group: BTreeMap<String, usize>,
}

impl AdmissionLimits {
    pub fn new(global: Option<usize>) -> Self {
        Self {
            global,
            per_group: BTreeMap::new(),
        }
    }

    pub fn with_group_limit(mut self, group: impl Into<String>, limit: usize) -> Self {
        self.per_group.insert(group.into(), limit);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionConfigError {
    EmptyGroupId,
}

impl fmt::Display for AdmissionConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGroupId => formatter.write_str("admission group id must not be empty"),
        }
    }
}

impl std::error::Error for AdmissionConfigError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionRejectScope {
    Global,
    Group,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionRejected {
    pub scope: AdmissionRejectScope,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdmissionStats {
    pub admitted: u64,
    pub rejected: u64,
    pub active: u64,
}

struct AdmissionCounters {
    limit: Option<usize>,
    active: AtomicUsize,
    admitted: AtomicU64,
    rejected: AtomicU64,
}

impl AdmissionCounters {
    fn new(limit: Option<usize>) -> Self {
        Self {
            limit,
            active: AtomicUsize::new(0),
            admitted: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
        }
    }

    fn stats(&self) -> AdmissionStats {
        AdmissionStats {
            admitted: self.admitted.load(Ordering::Relaxed),
            rejected: self.rejected.load(Ordering::Relaxed),
            active: self.active.load(Ordering::Relaxed) as u64,
        }
    }
}

struct AdmissionState {
    global: AdmissionCounters,
    groups: HashMap<String, Arc<AdmissionCounters>>,
}

/// A shared, non-blocking admission controller.
///
/// There is deliberately no waiter queue: `try_acquire` either returns a
/// permit immediately or rejects the attempt. This keeps overload decisions
/// off the latency-sensitive caller path and makes backpressure explicit to
/// the protocol handler. Per-group limits provide isolation, not scheduling
/// fairness; callers that need a fair queue must add it above this primitive.
#[derive(Clone)]
pub struct AdmissionController {
    state: Arc<AdmissionState>,
}

impl fmt::Debug for AdmissionController {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AdmissionController")
            .field("stats", &self.stats())
            .finish_non_exhaustive()
    }
}

impl AdmissionController {
    pub fn new(limits: AdmissionLimits) -> Result<Self, AdmissionConfigError> {
        if limits.per_group.keys().any(|group| group.is_empty()) {
            return Err(AdmissionConfigError::EmptyGroupId);
        }

        let groups = limits
            .per_group
            .into_iter()
            .map(|(group, limit)| (group, Arc::new(AdmissionCounters::new(Some(limit)))))
            .collect();

        Ok(Self {
            state: Arc::new(AdmissionState {
                global: AdmissionCounters::new(limits.global),
                groups,
            }),
        })
    }

    pub fn stats(&self) -> AdmissionStats {
        self.state.global.stats()
    }

    pub fn group_stats(&self, group: &str) -> Option<AdmissionStats> {
        self.state
            .groups
            .get(group)
            .map(|counters| counters.stats())
    }

    /// Reserves the global budget for a resource without applying a group
    /// budget. The permit must be held until the resource has fully stopped.
    #[must_use = "the admission permit must be held until the resource stops"]
    pub fn try_acquire_global(&self) -> Result<AdmissionPermit, AdmissionRejected> {
        self.try_acquire_inner(None)
    }

    /// Reserves the global budget and the configured budget for `group`. A
    /// group with no configured limit still participates in the global budget.
    #[must_use = "the admission permit must be held until the resource stops"]
    pub fn try_acquire_group(&self, group: &str) -> Result<AdmissionPermit, AdmissionRejected> {
        self.try_acquire_inner(Some(group))
    }

    fn try_acquire_inner(&self, group: Option<&str>) -> Result<AdmissionPermit, AdmissionRejected> {
        let group_counters = group.and_then(|id| self.state.groups.get(id).cloned());
        if !try_reserve(&self.state.global) {
            self.state.global.rejected.fetch_add(1, Ordering::Relaxed);
            return Err(AdmissionRejected {
                scope: AdmissionRejectScope::Global,
            });
        }

        if let Some(counters) = group_counters.as_deref() {
            if !try_reserve(counters) {
                release_reservation(&self.state.global);
                record_rejection(&self.state.global, Some(counters));
                return Err(AdmissionRejected {
                    scope: AdmissionRejectScope::Group,
                });
            }
        }

        self.state.global.admitted.fetch_add(1, Ordering::Relaxed);
        if let Some(counters) = group_counters.as_deref() {
            counters.admitted.fetch_add(1, Ordering::Relaxed);
        }

        Ok(AdmissionPermit {
            state: Some(self.state.clone()),
            group: group_counters,
        })
    }

    /// Compatibility wrapper for callers that do not need the rejection
    /// reason. New resource domains should use the explicit methods above.
    #[must_use]
    pub fn try_acquire(&self, group: Option<&str>) -> Option<AdmissionPermit> {
        match group {
            Some(group) => self.try_acquire_group(group).ok(),
            None => self.try_acquire_global().ok(),
        }
    }
}

#[must_use]
pub struct AdmissionPermit {
    state: Option<Arc<AdmissionState>>,
    group: Option<Arc<AdmissionCounters>>,
}

impl fmt::Debug for AdmissionPermit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AdmissionPermit")
            .field("active", &self.state.is_some())
            .finish()
    }
}

impl AdmissionPermit {
    pub fn release(mut self) {
        self.release_inner();
    }

    fn release_inner(&mut self) {
        let Some(state) = self.state.take() else {
            return;
        };
        release_reservation(&state.global);
        if let Some(group) = self.group.take() {
            release_reservation(&group);
        }
    }
}

impl Drop for AdmissionPermit {
    fn drop(&mut self) {
        self.release_inner();
    }
}

fn try_reserve(counters: &AdmissionCounters) -> bool {
    let mut active = counters.active.load(Ordering::Acquire);
    loop {
        if active == usize::MAX || counters.limit.is_some_and(|limit| active >= limit) {
            return false;
        }
        match counters.active.compare_exchange_weak(
            active,
            active + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return true,
            Err(actual) => active = actual,
        }
    }
}

fn release_reservation(counters: &AdmissionCounters) {
    let previous = counters.active.fetch_sub(1, Ordering::AcqRel);
    debug_assert!(previous > 0, "admission reservation underflow");
}

fn record_rejection(global: &AdmissionCounters, group: Option<&AdmissionCounters>) {
    global.rejected.fetch_add(1, Ordering::Relaxed);
    if let Some(group) = group {
        group.rejected.fetch_add(1, Ordering::Relaxed);
    }
}

impl OverloadLimits {
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        mode: OverloadMode,
        max_concurrent_streams: u32,
        yield_abs: usize,
        sleep_abs: usize,
        max_pending_streams: Option<usize>,
        yield_pct: Option<f32>,
        sleep_pct: Option<f32>,
        sleep_budget_ms: u64,
        backoff: BackoffStrategy,
    ) -> Self {
        let max = max_concurrent_streams as usize;
        let mut yield_t = yield_pct
            .map(|p| ((p.clamp(0.0, 1.0) as f64) * max as f64).round() as usize)
            .unwrap_or(yield_abs);
        let sleep_t = sleep_pct
            .map(|p| ((p.clamp(0.0, 1.0) as f64) * max as f64).round() as usize)
            .unwrap_or(sleep_abs);
        if yield_t > sleep_t {
            yield_t = sleep_t;
        }
        let pending_t = max_pending_streams.unwrap_or_else(|| (max / 4).max(1));
        Self {
            mode,
            inflight_yield_threshold: yield_t,
            inflight_sleep_threshold: sleep_t,
            max_pending_streams: pending_t,
            backoff,
            inflight_sleep_budget: Duration::from_millis(sleep_budget_ms),
        }
    }
}

pub async fn maybe_slow_path(state: &ConnectionState, limits: &OverloadLimits) {
    if limits.mode == OverloadMode::Burst {
        return;
    }
    let cur = inflight_load(state, std::sync::atomic::Ordering::Relaxed);
    if cur < limits.inflight_yield_threshold {
        return;
    }

    crate::infra::metrics::METRICS
        .slowpath_waiting_tasks
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    struct SlowpathGuard;
    impl Drop for SlowpathGuard {
        fn drop(&mut self) {
            crate::infra::metrics::METRICS
                .slowpath_waiting_tasks
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    let _guard = SlowpathGuard;

    let notify = inflight_notify(state);
    if cur < limits.inflight_sleep_threshold {
        let notified = notify.notified();
        tokio::select! {
            _ = notified => {}
            _ = tokio::time::sleep(Duration::from_millis(1)) => {}
        }
        return;
    }
    if matches!(limits.backoff, BackoffStrategy::None) || limits.inflight_sleep_budget.is_zero() {
        return;
    }
    let deadline = tokio::time::Instant::now() + limits.inflight_sleep_budget;
    loop {
        if inflight_load(state, std::sync::atomic::Ordering::Relaxed)
            < limits.inflight_sleep_threshold
        {
            return;
        }
        let now = tokio::time::Instant::now();
        if now >= deadline {
            return;
        }
        let wait = match limits.backoff {
            BackoffStrategy::Fixed => deadline - now,
            BackoffStrategy::Exponential => (deadline - now).min(Duration::from_millis(10)),
            BackoffStrategy::None => return,
        };
        let notified = notify.notified();
        tokio::select! {
            _ = notified => {}
            _ = tokio::time::sleep(wait) => {}
        }
    }
}

#[cfg(test)]
mod admission_tests {
    use super::AdmissionRejectScope;
    use super::{AdmissionConfigError, AdmissionController, AdmissionLimits};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;
    use tokio::sync::oneshot;

    #[test]
    fn rejects_invalid_group_configuration() {
        let limits = AdmissionLimits::new(Some(1)).with_group_limit("", 1);
        assert_eq!(
            AdmissionController::new(limits).unwrap_err().to_string(),
            AdmissionConfigError::EmptyGroupId.to_string()
        );
    }

    #[test]
    fn permit_release_updates_global_and_group_counters() {
        let limits = AdmissionLimits::new(Some(2)).with_group_limit("blue", 1);
        let controller = AdmissionController::new(limits).expect("valid limits");

        let blue = controller.try_acquire(Some("blue")).expect("blue slot");
        assert!(controller.try_acquire(Some("blue")).is_none());
        let other = controller.try_acquire(Some("other")).expect("global slot");

        assert_eq!(controller.stats().admitted, 2);
        assert_eq!(controller.stats().rejected, 1);
        assert_eq!(controller.stats().active, 2);
        assert_eq!(controller.group_stats("blue").unwrap().admitted, 1);
        assert_eq!(controller.group_stats("blue").unwrap().rejected, 1);
        assert_eq!(controller.group_stats("blue").unwrap().active, 1);

        drop(blue);
        drop(other);
        assert_eq!(controller.stats().active, 0);
        assert_eq!(controller.group_stats("blue").unwrap().active, 0);
    }

    #[test]
    fn detailed_rejection_identifies_global_or_group_scope() {
        let group_limited =
            AdmissionController::new(AdmissionLimits::new(Some(2)).with_group_limit("blue", 1))
                .expect("valid limits");
        let _blue = group_limited
            .try_acquire_group("blue")
            .expect("first blue slot");
        assert_eq!(
            group_limited.try_acquire_group("blue").unwrap_err().scope,
            AdmissionRejectScope::Group
        );

        let global_limited =
            AdmissionController::new(AdmissionLimits::new(Some(1))).expect("valid limits");
        let _first = global_limited
            .try_acquire_global()
            .expect("first global slot");
        assert_eq!(
            global_limited.try_acquire_global().unwrap_err().scope,
            AdmissionRejectScope::Global
        );

        let isolated_global =
            AdmissionController::new(AdmissionLimits::new(Some(1)).with_group_limit("blue", 1))
                .expect("valid limits");
        let _other = isolated_global.try_acquire_global().expect("global slot");
        assert_eq!(
            isolated_global.try_acquire_group("blue").unwrap_err().scope,
            AdmissionRejectScope::Global
        );
        assert_eq!(isolated_global.group_stats("blue").unwrap().rejected, 0);
    }

    #[test]
    fn explicit_permit_release_is_idempotent_and_reclaims_capacity() {
        let controller =
            AdmissionController::new(AdmissionLimits::new(Some(1))).expect("valid limits");
        let permit = controller.try_acquire_global().expect("first global slot");
        permit.release();

        assert_eq!(controller.stats().active, 0);
        let _second = controller
            .try_acquire_global()
            .expect("released slot should be reusable");
    }

    #[tokio::test]
    async fn cancelling_owner_task_releases_permit() {
        let controller =
            AdmissionController::new(AdmissionLimits::new(Some(1))).expect("valid limits");
        let task_controller = controller.clone();
        let (started_tx, started_rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            let _permit = task_controller
                .try_acquire_global()
                .expect("task should acquire the slot");
            started_tx
                .send(())
                .expect("test task should signal startup");
            std::future::pending::<()>().await;
        });

        tokio::time::timeout(Duration::from_secs(1), started_rx)
            .await
            .expect("task should acquire the permit in time")
            .expect("test task should signal startup");
        assert_eq!(controller.stats().active, 1);
        task.abort();
        task.await.expect_err("task should be cancelled");
        assert_eq!(controller.stats().active, 0);
    }

    #[test]
    fn configured_groups_are_isolated_under_global_capacity() {
        let limits = AdmissionLimits::new(Some(4))
            .with_group_limit("a", 2)
            .with_group_limit("b", 2);
        let controller = AdmissionController::new(limits).expect("valid limits");

        let a1 = controller.try_acquire(Some("a")).expect("a slot 1");
        let a2 = controller.try_acquire(Some("a")).expect("a slot 2");
        assert!(controller.try_acquire(Some("a")).is_none());

        let b1 = controller.try_acquire(Some("b")).expect("b slot 1");
        let b2 = controller.try_acquire(Some("b")).expect("b slot 2");
        assert!(controller.try_acquire(Some("b")).is_none());
        assert_eq!(controller.stats().active, 4);

        drop(a1);
        drop(a2);
        drop(b1);
        drop(b2);
        assert_eq!(controller.stats().active, 0);
    }

    #[test]
    fn concurrent_groups_cannot_monopolize_shared_capacity() {
        let workers_per_group = 8;
        let total_workers = workers_per_group * 2;
        let controller = Arc::new(
            AdmissionController::new(
                AdmissionLimits::new(Some(4))
                    .with_group_limit("a", 2)
                    .with_group_limit("b", 2),
            )
            .expect("valid limits"),
        );
        let start = Arc::new(Barrier::new(total_workers));
        let attempts_complete = Arc::new(Barrier::new(total_workers));
        let admitted_a = Arc::new(AtomicUsize::new(0));
        let admitted_b = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..total_workers)
            .map(|index| {
                let group = if index < workers_per_group { "a" } else { "b" };
                let controller = controller.clone();
                let start = start.clone();
                let attempts_complete = attempts_complete.clone();
                let admitted = if group == "a" {
                    admitted_a.clone()
                } else {
                    admitted_b.clone()
                };
                thread::spawn(move || {
                    start.wait();
                    let permit = controller.try_acquire(Some(group));
                    if permit.is_some() {
                        admitted.fetch_add(1, Ordering::Relaxed);
                    }
                    attempts_complete.wait();
                    drop(permit);
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("worker should not panic");
        }

        assert_eq!(admitted_a.load(Ordering::Relaxed), 2);
        assert_eq!(admitted_b.load(Ordering::Relaxed), 2);
        assert_eq!(controller.stats().active, 0);
    }

    #[test]
    fn concurrent_attempts_never_exceed_global_limit() {
        let limit = 4;
        let workers = 32;
        let controller = Arc::new(
            AdmissionController::new(AdmissionLimits::new(Some(limit))).expect("valid limits"),
        );
        let start = Arc::new(Barrier::new(workers));
        let attempts_complete = Arc::new(Barrier::new(workers));
        let admitted = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..workers)
            .map(|_| {
                let controller = controller.clone();
                let start = start.clone();
                let attempts_complete = attempts_complete.clone();
                let admitted = admitted.clone();
                thread::spawn(move || {
                    start.wait();
                    let permit = controller.try_acquire(None);
                    if permit.is_some() {
                        admitted.fetch_add(1, Ordering::Relaxed);
                    }
                    attempts_complete.wait();
                    drop(permit);
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("worker should not panic");
        }

        assert_eq!(admitted.load(Ordering::Relaxed), limit);
        assert_eq!(controller.stats().admitted, limit as u64);
        assert_eq!(controller.stats().rejected, (workers - limit) as u64);
        assert_eq!(controller.stats().active, 0);
    }
}
