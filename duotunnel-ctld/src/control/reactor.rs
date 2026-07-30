use std::time::Duration;
use tokio::sync::mpsc;
use tracing::warn;

use crate::control::service::{ControlService, DegradedSource};

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
                    svc.publish();
                    if let Err(mark_error) = svc
                        .set_source_degraded(DegradedSource::Coordinator, true)
                        .await
                    {
                        warn!(error = %mark_error, "failed to persist degraded config state");
                    }
                    warn!(
                        error = %e,
                        consecutive_failures,
                        backoff_secs = backoff.as_secs(),
                        "debounced publish failed, backing off"
                    );
                    tokio::time::sleep(backoff).await;
                } else {
                    consecutive_failures = 0;
                    if let Err(clear_error) = svc
                        .set_source_degraded(DegradedSource::Coordinator, false)
                        .await
                    {
                        warn!(error = %clear_error, "failed to clear degraded config state");
                    }
                }
            }
        }
    }
}
