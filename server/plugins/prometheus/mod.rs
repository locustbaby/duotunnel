use tunnel_lib::plugin::MetricsSink;

/// `MetricsSink` implementation backed by the `metrics` crate, which the
/// server already uses via `metrics_exporter_prometheus` (see
/// `server/metrics.rs`).
///
/// Runtime-computed label values are forwarded by cloning the string — this
/// matches how `server/metrics.rs` already does it for `group_id` / `status`
/// labels. The metric name is `&'static str`, so no allocation on the name.
pub struct PrometheusSink;

impl MetricsSink for PrometheusSink {
    fn incr(&self, name: &'static str, labels: &[(&str, &str)]) {
        let owned: Vec<(String, String)> = labels
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        metrics::counter!(name, &owned).increment(1);
    }

    fn observe(&self, name: &'static str, value: f64, labels: &[(&str, &str)]) {
        let owned: Vec<(String, String)> = labels
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        metrics::histogram!(name, &owned).record(value);
    }
}
