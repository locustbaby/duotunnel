/// Abstraction over metrics backends.
///
/// Injected into `ServerCtx` and `EgressCtx`; handlers call `ctx.metrics.incr()`
/// instead of importing concrete Prometheus symbols.
///
/// Label keys are `&'static str` so backends can use a zero-allocation static
/// registration path (e.g. `metrics::Label::from_static_parts`). Values stay
/// borrowed `&str` because they're often runtime-computed.
pub trait MetricsSink: Send + Sync + 'static {
    fn incr(&self, name: &'static str, labels: &[(&'static str, &str)]);
    fn observe(&self, name: &'static str, value: f64, labels: &[(&'static str, &str)]);
}

/// No-op implementation used when metrics are disabled or in tests.
pub struct NoopSink;

impl MetricsSink for NoopSink {
    #[inline]
    fn incr(&self, _name: &'static str, _labels: &[(&'static str, &str)]) {}
    #[inline]
    fn observe(&self, _name: &'static str, _value: f64, _labels: &[(&'static str, &str)]) {}
}
