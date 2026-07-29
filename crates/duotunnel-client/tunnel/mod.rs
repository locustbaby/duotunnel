use crate::bootstrap::config::ClientConfigFile;
use crate::egress::udp_listener::UdpListenerRegistry;
use crate::runtime::engine::ClientService;
use crate::tunnel::conn_pool::EntryConnPool;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub mod client;
pub mod conn_pool;
pub mod endpoint;
pub mod pool;
pub mod supervisor;

pub(crate) struct TunnelPoolService {
    pub(crate) config: ClientConfigFile,
    pub(crate) endpoint: quinn::Endpoint,
    pub(crate) entry_pool: Arc<EntryConnPool>,
    pub(crate) udp_registry: Arc<UdpListenerRegistry>,
    pub(crate) resolved_connections: u32,
}

#[async_trait::async_trait]
impl ClientService for TunnelPoolService {
    fn name(&self) -> &'static str {
        "wan-tunnel-pool"
    }
    async fn start(&self, shutdown: CancellationToken) -> anyhow::Result<()> {
        if self.resolved_connections > 1 {
            info!(
                connections = self.resolved_connections,
                "using multi-QUIC connection pool"
            );
            pool::run_pool(
                self.config.clone(),
                self.endpoint.clone(),
                shutdown,
                self.entry_pool.clone(),
                self.udp_registry.clone(),
                self.resolved_connections,
            )
            .await
            .map_err(|e| anyhow::anyhow!("run_pool failed: {}", e))
        } else {
            supervisor::run_supervisor(
                self.config.clone(),
                self.endpoint.clone(),
                shutdown,
                self.entry_pool.clone(),
                self.udp_registry.clone(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("run_supervisor failed: {}", e))
        }
    }
}
