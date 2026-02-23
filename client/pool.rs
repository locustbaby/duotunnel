use anyhow::Result;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::config::ClientConfigFile;

pub async fn run_pool(
    config: &ClientConfigFile,
    endpoint: &quinn::Endpoint,
    cancel: CancellationToken,
) -> Result<()> {
    let n = config.quic.connections.max(1) as usize;
    info!(connections = n, "starting QUIC connection pool");

    let mut handles = Vec::with_capacity(n);

    for i in 0..n {
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

    for h in handles {
        let _ = h.await;
    }

    Ok(())
}

async fn run_slot(config: ClientConfigFile, endpoint: quinn::Endpoint, cancel: CancellationToken) {
    let initial_delay = std::time::Duration::from_millis(config.reconnect.initial_delay_ms);
    let max_delay = std::time::Duration::from_millis(config.reconnect.max_delay_ms);
    let mut retry_delay = initial_delay;

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
                        retry_delay = initial_delay;
                    }
                    Err(e) => {
                        error!(
                            client_id = %config.client_id,
                            error = %e,
                            retry_in_ms = %retry_delay.as_millis(),
                            "connection error, reconnecting..."
                        );
                        tokio::select! {
                            _ = cancel.cancelled() => return,
                            _ = tokio::time::sleep(retry_delay) => {}
                        }
                        retry_delay = std::cmp::min(retry_delay * 2, max_delay);
                    }
                }
            }
        }
    }
}
