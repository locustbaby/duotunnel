use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverloadMode {
    #[default]
    InflightSlowpath,
    Burst,
}

/// Strategy for how to wait when inflight ≥ sleep threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackoffStrategy {
    /// Don't wait — skip straight to `open_bi` and rely on QUIC backpressure.
    None,
    /// Single fixed-duration sleep equal to the full budget (legacy behaviour).
    Fixed,
    /// Exponential backoff within the budget.  Rechecks inflight between
    /// steps so we stop as soon as a slot frees.  Derived shape:
    /// `min = budget/16`, `max = budget/4`, `multiplier = 2.0`.
    #[default]
    Exponential,
}

#[derive(Debug, Clone)]
pub struct OverloadLimits {
    pub mode: OverloadMode,
    pub inflight_yield_threshold: usize,
    pub inflight_sleep_threshold: usize,
    pub backoff: BackoffStrategy,
    /// Total time budget for the slow-path wait.
    pub inflight_sleep_budget: Duration,
}

impl OverloadLimits {
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        mode: OverloadMode,
        max_concurrent_streams: u32,
        yield_abs: usize,
        sleep_abs: usize,
        yield_pct: Option<f32>,
        sleep_pct: Option<f32>,
        sleep_budget_ms: u64,
        backoff: BackoffStrategy,
    ) -> Self {
        let max = max_concurrent_streams as usize;
        let yield_t = yield_pct
            .map(|p| ((p.clamp(0.0, 1.0) as f64) * max as f64).round() as usize)
            .unwrap_or(yield_abs);
        let sleep_t = sleep_pct
            .map(|p| ((p.clamp(0.0, 1.0) as f64) * max as f64).round() as usize)
            .unwrap_or(sleep_abs);
        Self {
            mode,
            inflight_yield_threshold: yield_t,
            inflight_sleep_threshold: sleep_t,
            backoff,
            inflight_sleep_budget: Duration::from_millis(sleep_budget_ms),
        }
    }
}

/// Slow-path invoked before `open_bi()` when a QUIC connection is near its
/// `max_concurrent_streams` cap.  `inflight` is a closure so the live count
/// is re-read between backoff steps.
pub async fn maybe_slow_path<F>(inflight: F, limits: &OverloadLimits)
where
    F: Fn() -> usize,
{
    if limits.mode == OverloadMode::Burst {
        return;
    }
    let cur = inflight();
    if cur < limits.inflight_yield_threshold {
        return;
    }
    if cur < limits.inflight_sleep_threshold {
        tokio::task::yield_now().await;
        return;
    }
    match limits.backoff {
        BackoffStrategy::None => {}
        BackoffStrategy::Fixed => {
            tokio::time::sleep(limits.inflight_sleep_budget).await;
        }
        BackoffStrategy::Exponential => {
            exponential_backoff(inflight, limits).await;
        }
    }
}

async fn exponential_backoff<F>(inflight: F, limits: &OverloadLimits)
where
    F: Fn() -> usize,
{
    let budget = limits.inflight_sleep_budget;
    if budget.is_zero() {
        return;
    }
    let min = (budget / 16).max(Duration::from_micros(50));
    let max = budget / 4;
    let deadline = tokio::time::Instant::now() + budget;
    let mut delay = min;
    loop {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            return;
        }
        let step = delay.min(deadline - now);
        tokio::time::sleep(step).await;
        if inflight() < limits.inflight_sleep_threshold {
            return;
        }
        delay = (delay * 2).min(max);
    }
}
