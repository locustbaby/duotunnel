use std::time::Duration;
use tokio::sync::mpsc;
use tracing::warn;

use crate::service::ControlService;

const PUBLISH_DEBOUNCE_MS: u64 = 50;

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
    let mut last_fingerprint: Option<String> = None;
    loop {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let Some(svc) = svc.upgrade() else { break };
        match svc.load_token_cache().await {
            Err(e) => {
                warn!(error = %e, "db_poll: failed to load token cache");
            }
            Ok(entries) => {
                let mut parts: Vec<String> = entries
                    .iter()
                    .map(|e| format!("{}:{}:{}", e.hash_hex, e.token_status, e.client_status))
                    .collect();
                parts.sort_unstable();
                let fingerprint = parts.join("|");
                let changed = last_fingerprint.as_deref() != Some(&fingerprint);
                last_fingerprint = Some(fingerprint);
                if changed {
                    tracing::debug!("db_poll: token change detected, triggering publish");
                    svc.publish();
                }
            }
        }
    }
}
