use crate::bootstrap::{ServerBootstrap, ServerState};
use crate::control::control_client;
use crate::control::hot_reload;
use crate::control::service::BackgroundService;
use crate::ingress::handlers;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ComponentState {
    Starting = 0,
    Running = 1,
    Stopping = 2,
    Stopped = 3,
    Failed = 4,
}

impl ComponentState {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Starting,
            1 => Self::Running,
            2 => Self::Stopping,
            3 => Self::Stopped,
            _ => Self::Failed,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ComponentStatus {
    inner: Arc<AtomicU8>,
}

impl ComponentStatus {
    fn new() -> Self {
        Self {
            inner: Arc::new(AtomicU8::new(ComponentState::Starting as u8)),
        }
    }

    pub(crate) fn state(&self) -> ComponentState {
        ComponentState::from_u8(self.inner.load(Ordering::Acquire))
    }

    fn set(&self, state: ComponentState) {
        self.inner.store(state as u8, Ordering::Release);
    }
}

pub(crate) struct ComponentContext {
    pub(crate) state: Arc<ServerState>,
    pub(crate) bootstrap: Arc<ServerBootstrap>,
    pub(crate) shutdown: CancellationToken,
    pub(crate) ready: Arc<AtomicBool>,
    pub(crate) proxy_handle: tokio::runtime::Handle,
}

impl Clone for ComponentContext {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            bootstrap: self.bootstrap.clone(),
            shutdown: self.shutdown.clone(),
            ready: self.ready.clone(),
            proxy_handle: self.proxy_handle.clone(),
        }
    }
}

pub(crate) trait ServerComponent: Send + 'static {
    fn name(&self) -> &'static str;
    fn start(self: Box<Self>, ctx: ComponentContext) -> ComponentHandle;
}

pub(crate) struct ComponentHandle {
    _name: &'static str,
    status: ComponentStatus,
    join: Option<std::thread::JoinHandle<()>>,
}

impl ComponentHandle {
    fn new(name: &'static str, status: ComponentStatus, join: std::thread::JoinHandle<()>) -> Self {
        Self {
            _name: name,
            status,
            join: Some(join),
        }
    }

    pub(crate) fn state(&self) -> ComponentState {
        self.status.state()
    }

    fn begin_shutdown(&self) {
        self.status.set(ComponentState::Stopping);
    }

    fn join(mut self) {
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

pub(crate) struct ServerSupervisor {
    handles: Vec<ComponentHandle>,
}

impl ServerSupervisor {
    pub(crate) fn start(components: Vec<Box<dyn ServerComponent>>, ctx: ComponentContext) -> Self {
        let mut handles = Vec::with_capacity(components.len());
        for component in components {
            let name = component.name();
            let handle = component.start(ctx.clone());
            info!(component = name, state = ?handle.state(), "component started");
            handles.push(handle);
        }
        Self { handles }
    }

    pub(crate) fn shutdown(self) {
        for handle in &self.handles {
            handle.begin_shutdown();
        }
        for handle in self.handles {
            handle.join();
        }
    }
}

pub(crate) struct MetricsComponent;

impl ServerComponent for MetricsComponent {
    fn name(&self) -> &'static str {
        "metrics"
    }

    fn start(self: Box<Self>, ctx: ComponentContext) -> ComponentHandle {
        let status = ComponentStatus::new();
        let status_clone = status.clone();
        let metrics_port = ctx
            .bootstrap
            .metrics_port()
            .expect("metrics component requires metrics_port");
        let ready = ctx.ready.clone();
        let shutdown = ctx.shutdown.clone();
        let join = std::thread::spawn(move || {
            status_clone.set(ComponentState::Running);
            let rt = tunnel_lib::build_single_thread_runtime("metrics-worker");
            let result = rt.block_on(async move {
                handlers::metrics::run_metrics_server(metrics_port, ready, shutdown).await
            });
            if let Err(e) = result {
                status_clone.set(ComponentState::Failed);
                error!(port = %metrics_port, error = %e, "Metrics server failed");
            } else {
                status_clone.set(ComponentState::Stopped);
            }
        });
        ComponentHandle::new(self.name(), status, join)
    }
}

pub(crate) struct BackgroundComponent;

impl ServerComponent for BackgroundComponent {
    fn name(&self) -> &'static str {
        "background"
    }

    fn start(self: Box<Self>, ctx: ComponentContext) -> ComponentHandle {
        let status = ComponentStatus::new();
        let status_clone = status.clone();
        let join = std::thread::spawn(move || {
            status_clone.set(ComponentState::Running);
            let rt = tunnel_lib::build_single_thread_runtime("bg-worker");
            let result = rt.block_on(async move { background_main(ctx).await });
            if let Err(e) = result {
                status_clone.set(ComponentState::Failed);
                error!(error = %e, "background component failed");
            } else {
                status_clone.set(ComponentState::Stopped);
            }
        });
        ComponentHandle::new(self.name(), status, join)
    }
}

async fn background_main(ctx: ComponentContext) -> anyhow::Result<()> {
    let svc: Box<dyn BackgroundService> = match ctx.bootstrap.mode() {
        crate::bootstrap::ServerMode::Managed {
            ctld_addr,
            ctld_token,
        } => match ctld_addr.parse::<std::net::SocketAddr>() {
            Ok(addr) => {
                info!(addr = %addr, "starting ctld watch client");
                Box::new(control_client::ControlClientService {
                    ctld_addr: addr,
                    auth_token: ctld_token.clone(),
                })
            }
            Err(e) => {
                anyhow::bail!("invalid ctld_addr: {}", e);
            }
        },
        crate::bootstrap::ServerMode::Standalone => Box::new(hot_reload::HotReloadService {
            config_path: ctx.bootstrap.config_path().to_string(),
        }),
    };

    let name = svc.name();
    let registry = ctx.state.registry().clone();
    let purge_shutdown = ctx.shutdown.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = purge_shutdown.cancelled() => break,
                _ = interval.tick() => {
                    let purged = registry.purge_dead().await;
                    if purged > 0 {
                        info!(purged, "registry: purged dead connections");
                    }
                }
            }
        }
    });

    if let Err(e) = svc.run(ctx.state, ctx.shutdown, ctx.proxy_handle).await {
        error!(service = name, error = %e, "background service exited with error");
    }
    Ok(())
}
