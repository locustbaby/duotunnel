use crate::lb::inflight::{inflight_load, inflight_notify, ConnectionState};
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
