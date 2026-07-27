use crate::bootstrap::{ServerBootstrap, ServerState};
use crate::control::control_client;
use crate::control::hot_reload;
use crate::control::service::BackgroundService;
use crate::ingress::handlers;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

const COMPONENT_RESTART_BUDGET: usize = 2;
const COMPONENT_RESTART_BACKOFF: Duration = Duration::from_millis(250);

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
    pub(crate) proxy_handle: tokio::runtime::Handle,
}

impl Clone for ComponentContext {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            bootstrap: self.bootstrap.clone(),
            shutdown: self.shutdown.clone(),
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
        ctx.state
            .health()
            .set_required_components(components.iter().map(|component| component.name()));
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

fn supervise_component<F>(
    name: &'static str,
    status: ComponentStatus,
    health: Arc<crate::runtime::health::ServerHealthFacts>,
    shutdown: CancellationToken,
    mut run_once: F,
) where
    F: FnMut() -> anyhow::Result<()>,
{
    for attempt in 0..=COMPONENT_RESTART_BUDGET {
        status.set(ComponentState::Running);
        health.component_running(name);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(&mut run_once));
        if shutdown.is_cancelled() {
            health.component_stopped(name);
            status.set(ComponentState::Stopped);
            return;
        }
        let reason = match outcome {
            Ok(Ok(())) => "unexpected clean exit".to_string(),
            Ok(Err(error)) => error.to_string(),
            Err(_) => "component panicked".to_string(),
        };
        health.component_stopped(name);
        if attempt < COMPONENT_RESTART_BUDGET {
            error!(
                component = name,
                attempt = attempt + 1,
            error = %reason,
            "component exited unexpectedly; restarting"
            );
            if !wait_restart_backoff(&shutdown) {
                status.set(ComponentState::Stopped);
                return;
            }
            continue;
        }
        status.set(ComponentState::Failed);
        health.component_failed(name, &reason);
        error!(
            component = name,
            attempts = attempt + 1,
            error = %reason,
            "component restart budget exhausted"
        );
        return;
    }
}

fn wait_restart_backoff(shutdown: &CancellationToken) -> bool {
    let deadline = std::time::Instant::now() + COMPONENT_RESTART_BACKOFF;
    while !shutdown.is_cancelled() {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return true;
        }
        std::thread::sleep(remaining.min(Duration::from_millis(10)));
    }
    false
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
        let health = ctx.state.health().clone();
        let shutdown = ctx.shutdown.clone();
        let join = std::thread::spawn(move || {
            let runner_health = health.clone();
            let runner_shutdown = shutdown.clone();
            supervise_component("metrics", status_clone, health, shutdown, move || {
                let rt = tunnel_lib::build_single_thread_runtime("metrics-worker");
                rt.block_on(handlers::metrics::run_metrics_server(
                    metrics_port,
                    runner_health.clone(),
                    runner_shutdown.clone(),
                ))
            });
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
        let health = ctx.state.health().clone();
        let shutdown = ctx.shutdown.clone();
        let join = std::thread::spawn(move || {
            supervise_component("background", status_clone, health, shutdown, move || {
                let rt = tunnel_lib::build_single_thread_runtime("bg-worker");
                rt.block_on(background_main(ctx.clone()))
            });
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
                    config_path: ctx.bootstrap.config_path().to_string(),
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

    let registry = ctx.state.registry().clone();
    let purge_shutdown = ctx.shutdown.clone();
    let mut purge = tokio::spawn(purge_loop(registry, purge_shutdown));
    let mut service = svc.run(ctx.state, ctx.shutdown.clone(), ctx.proxy_handle.clone());

    tokio::select! {
        result = &mut service => {
            if ctx.shutdown.is_cancelled() {
                result
            } else {
                match result {
                    Ok(()) => anyhow::bail!("background service exited unexpectedly"),
                    Err(error) => Err(error),
                }
            }
        }
        result = &mut purge => {
            if ctx.shutdown.is_cancelled() {
                result??;
                Ok(())
            } else {
                match result {
                    Ok(Ok(())) => anyhow::bail!("purge task exited unexpectedly"),
                    Ok(Err(error)) => Err(error),
                    Err(error) => Err(error.into()),
                }
            }
        }
    }
}

async fn purge_loop(
    registry: crate::ingress::registry::SharedRegistry,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = interval.tick() => {
                let purged = registry
                    .purge_dead()
                    .await
                    .map_err(anyhow::Error::msg)?;
                if purged > 0 {
                    info!(purged, "registry: purged dead connections");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unexpected_exit_exhausts_restart_budget_and_fails_health() {
        let health = Arc::new(crate::runtime::health::ServerHealthFacts::new(false));
        let status = ComponentStatus::new();
        let shutdown = CancellationToken::new();
        let mut attempts = 0;

        supervise_component(
            "test-component",
            status.clone(),
            health.clone(),
            shutdown,
            || {
                attempts += 1;
                Ok(())
            },
        );

        assert_eq!(attempts, COMPONENT_RESTART_BUDGET + 1);
        assert_eq!(status.state(), ComponentState::Failed);
        assert!(!health.is_ready());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let failure = runtime.block_on(health.wait_for_failure());
        assert!(failure.contains("test-component"));
    }

    #[test]
    fn shutdown_interrupts_component_restart_backoff() {
        let health = Arc::new(crate::runtime::health::ServerHealthFacts::new(false));
        let status = ComponentStatus::new();
        let shutdown = CancellationToken::new();
        let cancel = shutdown.clone();
        let canceller = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            cancel.cancel();
        });
        let started = std::time::Instant::now();

        supervise_component("test-component", status.clone(), health, shutdown, || {
            anyhow::bail!("failed")
        });
        canceller.join().unwrap();

        assert!(started.elapsed() < COMPONENT_RESTART_BACKOFF);
        assert_eq!(status.state(), ComponentState::Stopped);
    }
}
