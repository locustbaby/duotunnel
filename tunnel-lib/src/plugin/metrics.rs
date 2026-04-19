/// Abstraction over metrics backends.
///
/// Injected into `ServerCtx` and `EgressCtx`; handlers call `ctx.metrics.incr()`
/// instead of importing concrete Prometheus symbols.
pub trait MetricsSink: Send + Sync + 'static {
    fn incr(&self, name: &'static str, labels: &[(&str, &str)]);
    fn observe(&self, name: &'static str, value: f64, labels: &[(&str, &str)]);
}

/// No-op implementation used when metrics are disabled or in tests.
pub struct NoopSink;

impl MetricsSink for NoopSink {
    #[inline]
    fn incr(&self, _name: &'static str, _labels: &[(&str, &str)]) {}
    #[inline]
    fn observe(&self, _name: &'static str, _value: f64, _labels: &[(&str, &str)]) {}
}
