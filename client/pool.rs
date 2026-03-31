use anyhow::Result;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::config::ClientConfigFile;

pub async fn run_pool(
    config: ClientConfigFile,
    endpoint: quinn::Endpoint,
    cancel: CancellationToken,
) -> Result<()> {
    let n = config.quic.connections.max(1) as usize;
    info!(connections = n, "starting QUIC connection pool");

    let mut slots = JoinSet::new();

    for i in 0..n {
        let slot_id = if n == 1 {
            config.client_id.clone()
        } else {
            format!("{}-{}", config.client_id, i)
        };

        let mut slot_config = config.clone();
        slot_config.client_id = slot_id.clone();

        let endpoint = endpoint.clone();
        let slot_cancel = cancel.clone();

        slots.spawn(async move {
            crate::connect::run_supervisor(slot_config, endpoint, slot_cancel).await
        });
    }

    while let Some(join_result) = slots.join_next().await {
        match join_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                error!(error = %e, "pool slot failed; stopping all slots");
                cancel.cancel();
                while slots.join_next().await.is_some() {}
                return Err(e);
            }
            Err(e) => {
                error!(error = %e, "pool slot task panicked; stopping all slots");
                cancel.cancel();
                while slots.join_next().await.is_some() {}
                return Err(anyhow::anyhow!("pool slot task join error: {}", e));
            }
        }
    }

    Ok(())
}
