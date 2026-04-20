use metrics::{counter, histogram, Label};

use tunnel_lib::plugin::MetricsSink;

/// `MetricsSink` implementation backed by the `metrics` crate.
///
/// Label keys arrive as `&'static str` and pass through `Label::new` without
/// allocation (the crate's `SharedString` can hold a static borrow via
/// `const_str`). Values are borrowed `&str` so each runtime-computed value
/// costs one `String` allocation. The `metrics` crate internally materialises
/// labels into a `Vec<Label>` no matter what we pass in, so no upstream
/// zero-alloc path exists for dynamic labels.
pub struct PrometheusSink;

fn build_labels(labels: &[(&'static str, &str)]) -> Vec<Label> {
    labels
        .iter()
        .map(|(k, v)| Label::new(*k, v.to_string()))
        .collect()
}

impl MetricsSink for PrometheusSink {
    fn incr(&self, name: &'static str, labels: &[(&'static str, &str)]) {
        counter!(name, build_labels(labels)).increment(1);
    }

    fn observe(&self, name: &'static str, value: f64, labels: &[(&'static str, &str)]) {
        histogram!(name, build_labels(labels)).record(value);
    }
}
