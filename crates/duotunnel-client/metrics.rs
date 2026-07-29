use crate::health::ClientHealth;
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::OnceLock;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub fn set_handle(handle: PrometheusHandle) {
    HANDLE.set(handle).ok();
}

pub fn encode(health: &ClientHealth) -> String {
    let mut body = HANDLE.get().map(|h| h.render()).unwrap_or_default();
    append_resource_metrics(&mut body, health);
    body
}

fn append_resource_metrics(body: &mut String, health: &ClientHealth) {
    if !body.is_empty() && !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str("# HELP duotunnel_slowpath_waiting_tasks Active tasks waiting in slowpath overload backoff.\n");
    body.push_str("# TYPE duotunnel_slowpath_waiting_tasks gauge\n");
    body.push_str(&format!(
        "duotunnel_slowpath_waiting_tasks {}\n",
        duotunnel_core::METRICS.waiting_tasks()
    ));
    let snapshot = health.snapshot();
    body.push_str("# HELP duotunnel_client_active_tunnels Active committed QUIC tunnels.\n");
    body.push_str("# TYPE duotunnel_client_active_tunnels gauge\n");
    body.push_str(&format!(
        "duotunnel_client_active_tunnels {}\n",
        snapshot.active_tunnels
    ));
    body.push_str("# HELP duotunnel_client_desired_tunnels Desired QUIC tunnel count.\n");
    body.push_str("# TYPE duotunnel_client_desired_tunnels gauge\n");
    body.push_str(&format!(
        "duotunnel_client_desired_tunnels {}\n",
        snapshot.desired_tunnels
    ));
    body.push_str(
        "# HELP duotunnel_client_tunnel_degraded Whether active tunnels are below desired.\n",
    );
    body.push_str("# TYPE duotunnel_client_tunnel_degraded gauge\n");
    body.push_str(&format!(
        "duotunnel_client_tunnel_degraded {}\n",
        usize::from(snapshot.degraded)
    ));
    body.push_str(
        "# HELP duotunnel_client_ready Whether aggregate client readiness is satisfied.\n",
    );
    body.push_str("# TYPE duotunnel_client_ready gauge\n");
    body.push_str(&format!(
        "duotunnel_client_ready {}\n",
        usize::from(snapshot.ready)
    ));
    body.push_str(
        "# HELP duotunnel_client_pool_actor_alive Whether the connection-pool actor is alive.\n",
    );
    body.push_str("# TYPE duotunnel_client_pool_actor_alive gauge\n");
    body.push_str(&format!(
        "duotunnel_client_pool_actor_alive {}\n",
        usize::from(snapshot.pool_actor_alive)
    ));
}

pub fn egress_rejection(reason: &'static str) {
    metrics::counter!("egress_rejections_total", "reason" => reason).increment(1);
}

pub fn udp_datagram_dropped(reason: &'static str) {
    metrics::counter!("duotunnel_udp_datagram_dropped_total", "reason" => reason).increment(1);
}
