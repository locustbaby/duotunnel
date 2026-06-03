use crate::bootstrap::config::ClientConfigFile;
use crate::runtime::engine::ClientService;
use crate::tunnel::conn_pool::EntryConnPool;
use std::sync::{atomic::AtomicBool, Arc};
use tokio_util::sync::CancellationToken;
use tracing::info;

pub mod client;
pub mod conn_pool;
pub mod pool;
pub mod supervisor;

pub struct TunnelPoolService {
    pub config: ClientConfigFile,
    pub endpoint: quinn::Endpoint,
    pub entry_pool: Arc<EntryConnPool>,
    pub ready: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl ClientService for TunnelPoolService {
    fn name(&self) -> &'static str {
        "wan-tunnel-pool"
    }
    async fn start(&self, shutdown: CancellationToken) -> anyhow::Result<()> {
        if self.config.quic.connections > 1 {
            info!(connections = %self.config.quic.connections, "using multi-QUIC connection pool");
            pool::run_pool(
                self.config.clone(),
                self.endpoint.clone(),
                shutdown,
                self.ready.clone(),
                self.entry_pool.clone(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("run_pool failed: {}", e))
        } else {
            supervisor::run_supervisor(
                self.config.clone(),
                self.endpoint.clone(),
                shutdown,
                self.ready.clone(),
                self.entry_pool.clone(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("run_supervisor failed: {}", e))
        }
    }
}
