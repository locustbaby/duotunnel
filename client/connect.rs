use crate::config::ClientConfigFile;
use crate::conn_pool::EntryConnPool;
use anyhow::{anyhow, Result};
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
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
    ready: Arc<AtomicBool>,
    entry_pool: Arc<EntryConnPool>,
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
        tokio::select! {
            _ = cancel.cancelled() => { info!(server = % config.server_address(),
            "shutdown signal received"); return Ok(()); } result = crate::run_client(&
            config, &endpoint, cancel.clone(), ready.clone(), entry_pool.clone()) => { match result { Ok(_) => { backoff.reset();
            info!(server = % config.server_address(),
            "connection ended, restarting loop"); } Err(e) => { if e.class() ==
            FailureClass::Fatal { error!(server = % config.server_address(), error = % e,
            "fatal connect error"); return Err(e.into_anyhow()); } let retry_delay =
            backoff.next_delay(); warn!(server = % config.server_address(), error = % e,
            retry_in_ms = retry_delay.as_millis(),
            "transient connect error, scheduling retry"); tokio::select! { _ = cancel
            .cancelled() => return Ok(()), _ = tokio::time::sleep(retry_delay) => {} } }
            } }
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
    fn next_delay(&mut self) -> Duration {
        let cap = self.current;
        self.current = std::cmp::min(self.current.saturating_mul(2), self.max);
        random_delay_up_to(cap).max(Duration::from_millis(1))
    }
}
fn random_delay_up_to(cap: Duration) -> Duration {
    use rand::Rng;
    let max_ms_u128 = cap.as_millis();
    if max_ms_u128 == 0 {
        return Duration::ZERO;
    }
    let max_ms = std::cmp::min(max_ms_u128, u64::MAX as u128) as u64;
    if max_ms <= 1 {
        return Duration::from_millis(max_ms);
    }
    let jitter_ms = rand::rng().random_range(1..=max_ms);
    Duration::from_millis(jitter_ms)
}
pub fn classify_login_failure(resp_error: Option<&str>) -> ConnectError {
    let msg = resp_error.unwrap_or("unknown login error");
    if msg.contains("timeout") || msg.contains("unexpected message type") {
        ConnectError::transient(anyhow!("login rejected by server: {}", msg))
    } else {
        ConnectError::fatal(anyhow!("login rejected by server: {}", msg))
    }
}
