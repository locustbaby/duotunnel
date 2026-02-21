/// Multi-QUIC connection pool.
///
/// Maintains `target_size` parallel QUIC connections to the server.
/// Each connection gets a suffixed client_id (e.g. `my-client-0`, `my-client-1`)
/// so the server registry treats them as independent clients within the same group.
///
/// Each slot runs its own exponential-backoff reconnect loop independently,
/// so a single dropped connection does not stall the others.
use anyhow::Result;
use tokio_util::sync::CancellationToken;
use tracing::{info, error};

use crate::config::ClientConfigFile;

/// Spawn `config.quic_connections` independent reconnect tasks and wait for
/// all of them (or the cancel token to fire).
pub async fn run_pool(
    config: &ClientConfigFile,
    endpoint: &quinn::Endpoint,
    cancel: CancellationToken,
) -> Result<()> {
    let n = config.quic_connections.max(1) as usize;
    info!(connections = n, "starting QUIC connection pool");

    let mut handles = Vec::with_capacity(n);

    for i in 0..n {
        // Each slot uses a unique client_id so the server registry sees N distinct clients.
        let slot_id = if n == 1 {
            config.client_id.clone()
        } else {
            format!("{}-{}", config.client_id, i)
        };

        let mut slot_config = config.clone();
        slot_config.client_id = slot_id.clone();

        let endpoint = endpoint.clone();
        let cancel = cancel.clone();

        let handle = tokio::spawn(async move {
            run_slot(slot_config, endpoint, cancel).await;
        });
        handles.push(handle);
    }

    // Wait for all slots (they only exit when cancel fires or all reconnects fail
    // catastrophically — in practice they loop forever until cancel).
    for h in handles {
        let _ = h.await;
    }

    Ok(())
}

/// Single connection slot: connects, runs, reconnects with exponential backoff.
/// Exits when the cancel token is cancelled.
async fn run_slot(
    config: ClientConfigFile,
    endpoint: quinn::Endpoint,
    cancel: CancellationToken,
) {
    let mut retry_delay = std::time::Duration::from_secs(1);
    const MAX_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(60);
    const INITIAL_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(1);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                info!(client_id = %config.client_id, "pool slot cancelled");
                return;
            }
            result = crate::run_client(&config, &endpoint) => {
                match result {
                    Ok(_) => {
                        info!(client_id = %config.client_id, "connection closed gracefully, reconnecting...");
                        retry_delay = INITIAL_RETRY_DELAY;
                    }
                    Err(e) => {
                        error!(
                            client_id = %config.client_id,
                            error = %e,
                            retry_in_secs = %retry_delay.as_secs(),
                            "connection error, reconnecting..."
                        );
                        tokio::select! {
                            _ = cancel.cancelled() => return,
                            _ = tokio::time::sleep(retry_delay) => {}
                        }
                        retry_delay = std::cmp::min(retry_delay * 2, MAX_RETRY_DELAY);
                    }
                }
            }
        }
    }
}
