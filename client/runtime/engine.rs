use async_trait::async_trait;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait ClientService: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self, shutdown: CancellationToken) -> anyhow::Result<()>;
}

pub struct RuntimeEngine {
    services: Vec<Arc<dyn ClientService>>,
    shutdown: CancellationToken,
}

impl RuntimeEngine {
    pub fn new(shutdown: CancellationToken) -> Self {
        Self {
            services: Vec::new(),
            shutdown,
        }
    }

    pub fn add_service(&mut self, service: Arc<dyn ClientService>) {
        self.services.push(service);
    }

    pub async fn run_until_shutdown(self) -> anyhow::Result<()> {
        let RuntimeEngine { services, shutdown } = self;
        use futures_util::StreamExt;
        let mut handles = futures_util::stream::FuturesUnordered::new();
        for service in services {
            let service_shutdown = shutdown.clone();
            let name = service.name();
            handles.push(tokio::spawn(async move {
                if let Err(e) = service.start(service_shutdown).await {
                    tracing::error!(service = name, error = %e, "service crashed");
                    return Err(e);
                }
                Ok(())
            }));
        }
        let mut first_err = None;
        while let Some(res) = handles.next().await {
            match res {
                Ok(Ok(())) => continue,
                Ok(Err(e)) => {
                    shutdown.cancel();
                    first_err = Some(e);
                    break;
                }
                Err(e) => {
                    shutdown.cancel();
                    first_err = Some(anyhow::anyhow!("Service task panicked: {}", e));
                    break;
                }
            }
        }
        shutdown.cancel();
        while let Some(res) = handles.next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::error!(error = %e, "secondary service error during shutdown drain");
                }
                Err(e) => {
                    tracing::error!(error = %e, "secondary service panicked during shutdown drain");
                }
            }
        }
        match first_err {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
