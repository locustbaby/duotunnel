use crate::bootstrap::config::ClientConfigFile;
use crate::egress::udp_listener::UdpListenerRegistry;
use crate::tunnel::client::{run_client, RunClientOutcome};
use crate::tunnel::conn_pool::EntryConnPool;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

const STABLE_SESSION_WINDOW: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FailureClass {
    Fatal,
    Transient,
}
#[derive(Debug)]
pub struct ConnectError {
    class: FailureClass,
    inner: anyhow::Error,
}
impl ConnectError {
    pub fn transient(err: impl Into<anyhow::Error>) -> Self {
        Self {
            class: FailureClass::Transient,
            inner: err.into(),
        }
    }
    pub fn fatal(err: impl Into<anyhow::Error>) -> Self {
        Self {
            class: FailureClass::Fatal,
            inner: err.into(),
        }
    }
    pub fn class(&self) -> FailureClass {
        self.class
    }
    pub fn into_anyhow(self) -> anyhow::Error {
        self.inner
    }
}
impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl std::error::Error for ConnectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}
pub async fn run_supervisor(
    config: ClientConfigFile,
    endpoint: quinn::Endpoint,
    cancel: CancellationToken,
    entry_pool: Arc<EntryConnPool>,
    udp_registry: Arc<UdpListenerRegistry>,
) -> Result<()> {
    let initial_delay = Duration::from_millis(config.reconnect.initial_delay_ms);
    let max_delay = Duration::from_millis(config.reconnect.max_delay_ms);
    let mut backoff = JitterBackoff::new(initial_delay, max_delay);
    let startup_jitter = Duration::from_millis(config.reconnect.startup_jitter_ms);
    if !startup_jitter.is_zero() {
        let wait = random_delay_up_to(startup_jitter);
        if !wait.is_zero() {
            tokio::select! {
                _ = cancel.cancelled() => return Ok(()), _ = tokio::time::sleep(wait) =>
                { info!(server = % config.server_address(), startup_jitter_ms = wait
                .as_millis(), "startup jitter elapsed"); }
            }
        }
    }
    loop {
        if cancel.is_cancelled() {
            info!(server = % config.server_address(), "shutdown signal received");
            return Ok(());
        }
        // run_client observes `cancel` itself so it can drain in-flight
        // streams and close the connection gracefully; racing it against
        // cancel.cancelled() here would drop that future mid-drain. The
        // connect/login phase is bounded by its own timeouts.
        let result = run_client(
            &config,
            &endpoint,
            cancel.clone(),
            entry_pool.clone(),
            udp_registry.clone(),
        )
        .await;
        if cancel.is_cancelled() {
            info!(server = % config.server_address(), "shutdown signal received");
            return Ok(());
        }
        let error = match result {
            Ok(RunClientOutcome::Shutdown) => return Ok(()),
            Ok(RunClientOutcome::SessionEnded(report)) => {
                let reset = backoff.observe_session(
                    report.lifetime,
                    report.completed_business,
                    STABLE_SESSION_WINDOW,
                );
                info!(
                    server = %config.server_address(),
                    session_lifetime_ms = report.lifetime.as_millis(),
                    completed_business = report.completed_business,
                    backoff_reset = reset,
                    "tunnel session ended"
                );
                report.error
            }
            Err(error) => error,
        };
        if error.class() == FailureClass::Fatal {
            error!(server = % config.server_address(), error = % error, "fatal connect error");
            return Err(error.into_anyhow());
        }
        let retry_delay = backoff.next_delay();
        warn!(
            server = % config.server_address(),
            error = % error,
            retry_in_ms = retry_delay.as_millis(),
            "transient connect error, scheduling retry"
        );
        tokio::select! {
            _ = cancel.cancelled() => return Ok(()),
            _ = tokio::time::sleep(retry_delay) => {}
        }
    }
}
struct JitterBackoff {
    initial: Duration,
    max: Duration,
    current: Duration,
}
impl JitterBackoff {
    fn new(initial: Duration, max: Duration) -> Self {
        let initial = if initial.is_zero() {
            Duration::from_millis(1)
        } else {
            initial
        };
        let max = if max < initial { initial } else { max };
        Self {
            initial,
            max,
            current: initial,
        }
    }
    fn reset(&mut self) {
        self.current = self.initial;
    }
    fn observe_session(
        &mut self,
        lifetime: Duration,
        completed_business: bool,
        stable_window: Duration,
    ) -> bool {
        if completed_business || lifetime >= stable_window {
            self.reset();
            true
        } else {
            false
        }
    }
    fn next_delay(&mut self) -> Duration {
        let cap = self.current;
        self.current = std::cmp::min(self.current.saturating_mul(2), self.max);
        let min_delay = cap / 2;
        random_delay_range(min_delay, cap).max(Duration::from_millis(1))
    }
}
fn random_delay_range(min: Duration, max: Duration) -> Duration {
    use rand::Rng;
    let min_ms = min.as_millis() as u64;
    let max_ms = max.as_millis() as u64;
    if min_ms >= max_ms {
        return min;
    }
    let jitter_ms = rand::rng().random_range(min_ms..=max_ms);
    Duration::from_millis(jitter_ms)
}
fn random_delay_up_to(cap: Duration) -> Duration {
    random_delay_range(Duration::ZERO, cap)
}
/// The server sets `retryable` explicitly; error text is deliberately generic
/// for unauthenticated peers and must not be pattern-matched to decide whether
/// to reconnect — a transient backing-store fault would otherwise be read as a
/// rejected token and stop the client for good.
pub fn classify_login_failure(retryable: bool, resp_error: Option<&str>) -> ConnectError {
    let msg = resp_error.unwrap_or("unknown login error");
    if retryable {
        ConnectError::transient(anyhow!("login rejected by server: {}", msg))
    } else {
        ConnectError::fatal(anyhow!("login rejected by server: {}", msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_session_preserves_exponential_backoff() {
        let mut backoff = JitterBackoff::new(Duration::from_millis(10), Duration::from_millis(80));
        let _ = backoff.next_delay();
        assert_eq!(backoff.current, Duration::from_millis(20));

        assert!(!backoff.observe_session(Duration::from_secs(5), false, Duration::from_secs(30)));
        assert_eq!(backoff.current, Duration::from_millis(20));
    }

    #[test]
    fn stable_session_resets_backoff() {
        let mut backoff = JitterBackoff::new(Duration::from_millis(10), Duration::from_millis(80));
        let _ = backoff.next_delay();

        assert!(backoff.observe_session(Duration::from_secs(30), false, Duration::from_secs(30)));
        assert_eq!(backoff.current, Duration::from_millis(10));
    }

    #[test]
    fn completed_business_resets_backoff_even_for_short_session() {
        let mut backoff = JitterBackoff::new(Duration::from_millis(10), Duration::from_millis(80));
        let _ = backoff.next_delay();

        assert!(backoff.observe_session(Duration::from_secs(1), true, Duration::from_secs(30)));
        assert_eq!(backoff.current, Duration::from_millis(10));
    }
}
