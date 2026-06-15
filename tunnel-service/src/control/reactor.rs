use std::time::Duration;
use tokio::sync::mpsc;
use tracing::warn;

use crate::control::service::ControlService;

const PUBLISH_DEBOUNCE_MS: u64 = 50;
const POLL_INTERVAL_MS: u64 = 1500;

pub(crate) async fn debounce_publish_task(
    svc: std::sync::Weak<ControlService>,
    mut rx: mpsc::Receiver<()>,
) {
    let mut consecutive_failures: u32 = 0;
    loop {
        if rx.recv().await.is_none() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(PUBLISH_DEBOUNCE_MS)).await;
        while rx.try_recv().is_ok() {}

        match svc.upgrade() {
            None => break,
            Some(svc) => {
                if let Err(e) = svc.do_publish().await {
                    consecutive_failures += 1;
                    let backoff = Duration::from_secs(1 << consecutive_failures.min(5));
                    warn!(
                        error = %e,
                        consecutive_failures,
                        backoff_secs = backoff.as_secs(),
                        "debounced publish failed, backing off"
                    );
                    tokio::time::sleep(backoff).await;
                } else {
                    consecutive_failures = 0;
                }
            }
        }
    }
}

pub(crate) async fn db_poll_task(svc: std::sync::Weak<ControlService>) {
    loop {
        tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
        let Some(svc) = svc.upgrade() else { break };
        match svc.load_token_cache().await {
            Err(e) => {
                warn!(error = %e, "db_poll: failed to load token cache");
            }
            Ok(entries) => match svc.load_routing().await {
                Err(e) => {
                    warn!(error = %e, "db_poll: failed to load routing rules");
                }
                Ok(routing) => {
                    let (ingress_listeners, client_groups, egress_upstreams, egress_vhost_rules) =
                        crate::control::proto::routing_data_to_proto(&routing);
                    let current = svc.snapshot();
                    let changed = entries != current.token_cache
                        || ingress_listeners != current.ingress_listeners
                        || client_groups != current.client_groups
                        || egress_upstreams != current.egress_upstreams
                        || egress_vhost_rules != current.egress_vhost_rules;
                    if changed {
                        tracing::info!("db_poll: database change detected, triggering publish");
                        svc.publish();
                    }
                }
            },
        }
    }
}
