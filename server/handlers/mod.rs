pub mod http;
pub mod metrics;
pub mod quic;
pub mod tcp;

use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use std::sync::Arc;

/// How long to queue waiting for a connection slot before rejecting.
/// Tolerates short bursts at the connection limit without immediately dropping.
pub const SEMAPHORE_WAIT_MS: u64 = 500;

/// Acquire a semaphore permit, waiting up to SEMAPHORE_WAIT_MS.
/// Returns None if the semaphore is closed (shutdown) or the timeout elapsed.
/// Callers should reject the connection and `continue` on None.
pub async fn try_acquire_permit(sem: Arc<Semaphore>) -> Option<OwnedSemaphorePermit> {
    match tokio::time::timeout(
        Duration::from_millis(SEMAPHORE_WAIT_MS),
        sem.acquire_owned(),
    )
    .await
    {
        Ok(Ok(permit)) => Some(permit),
        _ => None,
    }
}
