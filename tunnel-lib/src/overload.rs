use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverloadMode {
    #[default]
    InflightSlowpath,
    Burst,
}

#[derive(Debug, Clone)]
pub struct OverloadLimits {
    pub mode: OverloadMode,
    pub inflight_yield_threshold: usize,
    pub inflight_sleep_threshold: usize,
    pub inflight_sleep: Duration,
}

impl OverloadLimits {
    pub fn resolve(
        mode: OverloadMode,
        max_concurrent_streams: u32,
        yield_abs: usize,
        sleep_abs: usize,
        yield_pct: Option<f32>,
        sleep_pct: Option<f32>,
        sleep_ms: u64,
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
            inflight_sleep: Duration::from_millis(sleep_ms),
        }
    }
}

pub async fn maybe_slow_path(inflight: usize, limits: &OverloadLimits) {
    if limits.mode == OverloadMode::Burst {
        return;
    }
    if inflight >= limits.inflight_sleep_threshold {
        tokio::time::sleep(limits.inflight_sleep).await;
    } else if inflight >= limits.inflight_yield_threshold {
        tokio::task::yield_now().await;
    }
}
