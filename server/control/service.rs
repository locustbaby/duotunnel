use crate::ServerState;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub trait BackgroundService: Send + 'static {
    fn name(&self) -> &'static str;
    fn run(
        self: Box<Self>,
        state: Arc<ServerState>,
        shutdown: CancellationToken,
        proxy_handle: tokio::runtime::Handle,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;
}
