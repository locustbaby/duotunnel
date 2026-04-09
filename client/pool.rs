use crate::config::ClientConfigFile;
use crate::conn_pool::EntryConnPool;
use anyhow::Result;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
pub async fn run_pool(
    config: ClientConfigFile,
    endpoint: quinn::Endpoint,
    cancel: CancellationToken,
    ready: Arc<AtomicBool>,
    entry_pool: Arc<EntryConnPool>,
) -> Result<()> {
    let n = config.quic.connections.max(1) as usize;
    info!(connections = n, "starting QUIC connection pool");
    let mut slots = JoinSet::new();
    for _i in 0..n {
        let slot_config = config.clone();
        let endpoint = endpoint.clone();
        let slot_cancel = cancel.clone();
        let slot_ready = ready.clone();
        let slot_pool = entry_pool.clone();
        slots.spawn(async move {
            crate::connect::run_supervisor(slot_config, endpoint, slot_cancel, slot_ready, slot_pool).await
        });
    }
    while let Some(join_result) = slots.join_next().await {
        match join_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                error!(error = % e, "pool slot failed; stopping all slots");
                cancel.cancel();
                while slots.join_next().await.is_some() {}
                return Err(e);
            }
            Err(e) => {
                error!(error = % e, "pool slot task panicked; stopping all slots");
                cancel.cancel();
                while slots.join_next().await.is_some() {}
                return Err(anyhow::anyhow!("pool slot task join error: {}", e));
            }
        }
    }
    Ok(())
}
