use crate::bootstrap::config::{IngressListener, IngressMode};
use crate::ServerState;
use parking_lot::Mutex as ParkingMutex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use tunnel_lib::{GroupId, ProxyName};

struct ListenerEntry {
    generation: u64,
    kind: ListenerKind,
    state: ListenerState,
    cancel: CancellationToken,
    drained: CancellationToken,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ListenerState {
    Starting,
    Active,
    Draining,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ListenerKind {
    Http,
    Tcp {
        group_id: GroupId,
        proxy_name: ProxyName,
    },
}

struct StartReservation {
    port: u16,
    generation: u64,
    cancel: CancellationToken,
    drained: CancellationToken,
    predecessor_drained: Option<CancellationToken>,
}

struct DrainReservation {
    port: u16,
    generation: u64,
    drained: CancellationToken,
}

enum ListenerOperation {
    Start {
        reservation: StartReservation,
        listener: IngressListener,
    },
    Drain(DrainReservation),
}

struct OperationFence {
    port: u16,
    generation: u64,
    state: ListenerState,
    cancel: CancellationToken,
    drained: CancellationToken,
}

impl ListenerOperation {
    fn fence(&self) -> OperationFence {
        match self {
            Self::Start { reservation, .. } => OperationFence {
                port: reservation.port,
                generation: reservation.generation,
                state: ListenerState::Starting,
                cancel: reservation.cancel.clone(),
                drained: reservation.drained.clone(),
            },
            Self::Drain(reservation) => OperationFence {
                port: reservation.port,
                generation: reservation.generation,
                state: ListenerState::Draining,
                cancel: CancellationToken::new(),
                drained: reservation.drained.clone(),
            },
        }
    }
}

struct ListenerTable {
    next_generation: u64,
    shutting_down: bool,
    entries: HashMap<u16, ListenerEntry>,
}

impl ListenerTable {
    fn new() -> Self {
        Self {
            next_generation: 1,
            shutting_down: false,
            entries: HashMap::new(),
        }
    }

    fn allocate_generation(&mut self) -> u64 {
        let generation = self.next_generation;
        self.next_generation = self.next_generation.wrapping_add(1).max(1);
        generation
    }

    fn begin_start(&mut self, port: u16, kind: ListenerKind) -> Option<StartReservation> {
        if self.shutting_down {
            return None;
        }
        if self
            .entries
            .get(&port)
            .is_some_and(|entry| entry.state != ListenerState::Draining && entry.kind == kind)
        {
            return None;
        }

        let predecessor = self.entries.remove(&port);
        if let Some(entry) = &predecessor {
            entry.cancel.cancel();
        }

        let generation = self.allocate_generation();
        let cancel = CancellationToken::new();
        let drained = CancellationToken::new();
        self.entries.insert(
            port,
            ListenerEntry {
                generation,
                kind,
                state: ListenerState::Starting,
                cancel: cancel.clone(),
                drained: drained.clone(),
            },
        );
        Some(StartReservation {
            port,
            generation,
            cancel,
            drained,
            predecessor_drained: predecessor.map(|entry| entry.drained),
        })
    }

    fn begin_drain(&mut self, port: u16) -> Option<DrainReservation> {
        let entry = self.entries.remove(&port)?;
        if entry.state == ListenerState::Draining {
            self.entries.insert(port, entry);
            return None;
        }

        entry.cancel.cancel();
        let generation = self.allocate_generation();
        let drained = entry.drained.clone();
        self.entries.insert(
            port,
            ListenerEntry {
                generation,
                kind: entry.kind,
                state: ListenerState::Draining,
                cancel: entry.cancel,
                drained: drained.clone(),
            },
        );
        Some(DrainReservation {
            port,
            generation,
            drained,
        })
    }

    fn is_current_start(&self, port: u16, generation: u64) -> bool {
        !self.shutting_down
            && self.entries.get(&port).is_some_and(|entry| {
                entry.generation == generation && entry.state == ListenerState::Starting
            })
    }

    fn complete_start(&mut self, port: u16, generation: u64) -> bool {
        if !self.is_current_start(port, generation) {
            return false;
        }
        self.entries
            .get_mut(&port)
            .expect("current listener reservation disappeared")
            .state = ListenerState::Active;
        true
    }

    fn remove_if_current(&mut self, port: u16, generation: u64, state: ListenerState) -> bool {
        let is_current = self
            .entries
            .get(&port)
            .is_some_and(|entry| entry.generation == generation && entry.state == state);
        if is_current {
            self.entries.remove(&port);
        }
        is_current
    }

    fn begin_shutdown(&mut self) -> Vec<(u16, CancellationToken)> {
        self.shutting_down = true;
        self.entries
            .drain()
            .map(|(port, entry)| {
                entry.cancel.cancel();
                (port, entry.drained)
            })
            .collect()
    }
}

pub(crate) struct ListenerManager {
    table: ParkingMutex<ListenerTable>,
}

impl ListenerManager {
    pub(crate) fn new() -> Self {
        Self {
            table: ParkingMutex::new(ListenerTable::new()),
        }
    }

    fn plan_sync(
        &self,
        desired: &[IngressListener],
        affected_ports: Option<&HashSet<u16>>,
    ) -> Vec<ListenerOperation> {
        let desired_by_port: HashMap<u16, &IngressListener> = desired
            .iter()
            .filter(|listener| affected_ports.is_none_or(|ports| ports.contains(&listener.port)))
            .map(|listener| (listener.port, listener))
            .collect();

        let mut table = self.table.lock();
        if table.shutting_down {
            return Vec::new();
        }

        let existing_ports: Vec<u16> = table
            .entries
            .keys()
            .copied()
            .filter(|port| affected_ports.is_none_or(|ports| ports.contains(port)))
            .collect();
        let mut operations = Vec::new();

        for port in existing_ports {
            if !desired_by_port.contains_key(&port) {
                if let Some(reservation) = table.begin_drain(port) {
                    operations.push(ListenerOperation::Drain(reservation));
                }
            }
        }

        for (port, listener) in desired_by_port {
            let kind = ListenerKind::from_listener(listener);
            if let Some(reservation) = table.begin_start(port, kind) {
                operations.push(ListenerOperation::Start {
                    reservation,
                    listener: listener.clone(),
                });
            }
        }
        operations
    }

    fn is_current_start(&self, port: u16, generation: u64) -> bool {
        self.table.lock().is_current_start(port, generation)
    }

    fn complete_start(&self, port: u16, generation: u64) -> bool {
        self.table.lock().complete_start(port, generation)
    }

    fn fail_start(&self, port: u16, generation: u64) {
        self.table
            .lock()
            .remove_if_current(port, generation, ListenerState::Starting);
    }

    fn complete_drain(&self, port: u16, generation: u64) {
        self.table
            .lock()
            .remove_if_current(port, generation, ListenerState::Draining);
    }

    fn worker_set_exited(&self, port: u16, generation: u64) -> bool {
        self.table
            .lock()
            .remove_if_current(port, generation, ListenerState::Active)
    }

    fn fail_operation(&self, fence: OperationFence) {
        fence.cancel.cancel();
        fence.drained.cancel();
        self.table
            .lock()
            .remove_if_current(fence.port, fence.generation, fence.state);
    }

    fn begin_shutdown(&self) -> Vec<(u16, CancellationToken)> {
        self.table.lock().begin_shutdown()
    }

    fn health_snapshot(
        &self,
        desired: &[IngressListener],
        affected_ports: Option<&HashSet<u16>>,
        failures: &BTreeMap<u16, (u64, String)>,
    ) -> BTreeMap<u16, crate::runtime::health::ListenerHealth> {
        let table = self.table.lock();
        desired
            .iter()
            .filter(|listener| affected_ports.is_none_or(|ports| ports.contains(&listener.port)))
            .map(|listener| {
                let entry = table.entries.get(&listener.port);
                let failed = failures.get(&listener.port);
                let desired_generation = entry
                    .map(|entry| entry.generation)
                    .or_else(|| failed.map(|(generation, _)| *generation))
                    .unwrap_or(0);
                let active_generation = entry
                    .filter(|entry| entry.state == ListenerState::Active)
                    .map(|entry| entry.generation);
                (
                    listener.port,
                    crate::runtime::health::ListenerHealth {
                        desired_generation,
                        active_generation,
                        error: failed.map(|(_, error)| error.clone()),
                    },
                )
            })
            .collect()
    }
}

impl ListenerKind {
    fn from_listener(listener: &IngressListener) -> Self {
        match &listener.mode {
            IngressMode::Http(_) => Self::Http,
            IngressMode::Tcp(cfg) => Self::Tcp {
                group_id: cfg.client_group.clone(),
                proxy_name: cfg.proxy_name.clone(),
            },
        }
    }
}

const LISTENER_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

async fn wait_listener_drained(port: u16, drained: &CancellationToken) {
    if tokio::time::timeout(LISTENER_DRAIN_TIMEOUT, drained.cancelled())
        .await
        .is_err()
    {
        warn!(
            port = %port,
            timeout_secs = LISTENER_DRAIN_TIMEOUT.as_secs(),
            "listener did not report drained in time; continuing lifecycle transition"
        );
    }
}

fn bind_listener_workers(
    port: u16,
    accept_workers: usize,
) -> anyhow::Result<Vec<Arc<TcpListener>>> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    bind_listener_workers_with(accept_workers, || {
        tunnel_lib::build_reuseport_listener(addr)
    })
}

fn bind_listener_workers_with<F>(
    accept_workers: usize,
    mut bind: F,
) -> anyhow::Result<Vec<Arc<TcpListener>>>
where
    F: FnMut() -> anyhow::Result<TcpListener>,
{
    let mut sockets = Vec::with_capacity(accept_workers);
    for _ in 0..accept_workers {
        sockets.push(Arc::new(bind()?));
    }
    Ok(sockets)
}

async fn run_start_operation(
    state: Arc<ServerState>,
    reservation: StartReservation,
    listener: IngressListener,
) -> anyhow::Result<()> {
    let StartReservation {
        port,
        generation,
        cancel,
        drained,
        predecessor_drained,
    } = reservation;

    if let Some(predecessor_drained) = predecessor_drained {
        wait_listener_drained(port, &predecessor_drained).await;
    }
    if cancel.is_cancelled() || !state.listeners().is_current_start(port, generation) {
        drained.cancel();
        return Ok(());
    }

    let sockets = match bind_listener_workers(port, state.accept_workers()) {
        Ok(sockets) if !sockets.is_empty() => sockets,
        Ok(_) => {
            error!(port = %port, generation, "listener has no accept workers");
            state.listeners().fail_start(port, generation);
            drained.cancel();
            anyhow::bail!("listener {port} has no accept workers");
        }
        Err(e) => {
            error!(port = %port, generation, error = %e, "failed to bind listener worker set");
            state.listeners().fail_start(port, generation);
            drained.cancel();
            return Err(e.context(format!(
                "failed to bind listener {port} generation {generation}"
            )));
        }
    };

    if cancel.is_cancelled() || !state.listeners().complete_start(port, generation) {
        drop(sockets);
        drained.cancel();
        return Ok(());
    }

    let remaining = Arc::new(AtomicUsize::new(sockets.len()));
    match listener.mode {
        IngressMode::Http(_) => {
            for listener_socket in sockets {
                let worker_state = state.clone();
                let worker_cancel = cancel.clone();
                let worker_drained = drained.clone();
                let worker_remaining = remaining.clone();
                state.proxy_handle().spawn(async move {
                    if let Err(e) = crate::ingress::handlers::http::run_http_accept_loop(
                        listener_socket,
                        worker_state.clone(),
                        port,
                        worker_cancel,
                    )
                    .await
                    {
                        error!(port = %port, generation, error = %e, "HTTP accept loop failed");
                    }
                    if worker_remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
                        worker_drained.cancel();
                        if worker_state.listeners().worker_set_exited(port, generation) {
                            worker_state
                                .health()
                                .listener_worker_exited(port, generation);
                        }
                    }
                });
            }
        }
        IngressMode::Tcp(cfg) => {
            for listener_socket in sockets {
                let worker_state = state.clone();
                let worker_cancel = cancel.clone();
                let worker_drained = drained.clone();
                let worker_remaining = remaining.clone();
                let group_id = cfg.client_group.clone();
                let proxy_name = cfg.proxy_name.clone();
                state.proxy_handle().spawn(async move {
                    if let Err(e) = crate::ingress::handlers::tcp::run_tcp_accept_loop(
                        listener_socket,
                        worker_state.clone(),
                        port,
                        proxy_name,
                        group_id,
                        worker_cancel,
                    )
                    .await
                    {
                        error!(port = %port, generation, error = %e, "TCP accept loop failed");
                    }
                    if worker_remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
                        worker_drained.cancel();
                        if worker_state.listeners().worker_set_exited(port, generation) {
                            worker_state
                                .health()
                                .listener_worker_exited(port, generation);
                        }
                    }
                });
            }
        }
    }

    info!(port = %port, generation, "listener active");
    Ok(())
}

async fn run_listener_operation(
    state: Arc<ServerState>,
    operation: ListenerOperation,
) -> anyhow::Result<()> {
    match operation {
        ListenerOperation::Start {
            reservation,
            listener,
        } => run_start_operation(state, reservation, listener).await,
        ListenerOperation::Drain(reservation) => {
            wait_listener_drained(reservation.port, &reservation.drained).await;
            state
                .listeners()
                .complete_drain(reservation.port, reservation.generation);
            Ok(())
        }
    }
}

async fn sync_listeners_inner(
    state: &Arc<ServerState>,
    desired: &[IngressListener],
    affected_ports: Option<&HashSet<u16>>,
) -> anyhow::Result<()> {
    let operations = state.listeners().plan_sync(desired, affected_ports);
    let mut handles = Vec::with_capacity(operations.len());
    for operation in operations {
        let fence = operation.fence();
        let operation_state = state.clone();
        handles.push((
            fence,
            state
                .proxy_handle()
                .spawn(run_listener_operation(operation_state, operation)),
        ));
    }
    let mut failures = BTreeMap::new();
    for (fence, handle) in handles {
        match handle.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                failures.insert(fence.port, (fence.generation, e.to_string()));
            }
            Err(e) => {
                let error = format!("listener lifecycle operation task failed: {e}");
                let port = fence.port;
                let generation = fence.generation;
                state.listeners().fail_operation(fence);
                failures.insert(port, (generation, error));
            }
        }
    }
    let facts = state
        .listeners()
        .health_snapshot(desired, affected_ports, &failures);
    state.health().replace_listener_facts(facts, affected_ports);
    if failures.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "listener apply failed: {}",
            failures
                .into_iter()
                .map(|(port, (_, error))| format!("{port}: {error}"))
                .collect::<Vec<_>>()
                .join("; ")
        )
    }
}

pub(crate) async fn sync_all_listeners(
    state: &Arc<ServerState>,
    desired: &[IngressListener],
) -> anyhow::Result<()> {
    sync_listeners_inner(state, desired, None).await
}

pub(crate) async fn sync_listener_subset(
    state: &Arc<ServerState>,
    desired: &[IngressListener],
    affected_ports: &HashSet<u16>,
) -> anyhow::Result<()> {
    if affected_ports.is_empty() {
        return Ok(());
    }
    sync_listeners_inner(state, desired, Some(affected_ports)).await
}

pub(crate) async fn sync_listeners(
    state: &Arc<ServerState>,
    desired: &[IngressListener],
) -> anyhow::Result<()> {
    sync_all_listeners(state, desired).await
}

pub(crate) async fn shutdown_all_listeners(state: &Arc<ServerState>) {
    let drained = state.listeners().begin_shutdown();
    for (port, drained) in drained {
        wait_listener_drained(port, &drained).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Barrier;

    #[test]
    fn a_to_b_to_c_only_c_can_become_active() {
        let mut table = ListenerTable::new();
        let a = table.begin_start(8080, ListenerKind::Http).unwrap();
        let b = table
            .begin_start(
                8080,
                ListenerKind::Tcp {
                    group_id: "group-b".into(),
                    proxy_name: "proxy-b".into(),
                },
            )
            .unwrap();
        let c = table
            .begin_start(
                8080,
                ListenerKind::Tcp {
                    group_id: "group-c".into(),
                    proxy_name: "proxy-c".into(),
                },
            )
            .unwrap();

        assert!(a.cancel.is_cancelled());
        assert!(b.cancel.is_cancelled());
        assert!(!table.complete_start(8080, a.generation));
        assert!(!table.complete_start(8080, b.generation));
        assert!(table.complete_start(8080, c.generation));

        let active = table.entries.get(&8080).unwrap();
        assert_eq!(active.generation, c.generation);
        assert_eq!(active.state, ListenerState::Active);
    }

    #[tokio::test]
    async fn stale_completion_loses_when_newer_apply_races() {
        let manager = Arc::new(ListenerManager::new());
        let a = manager
            .table
            .lock()
            .begin_start(8081, ListenerKind::Http)
            .unwrap();
        let barrier = Arc::new(Barrier::new(2));

        let stale_manager = manager.clone();
        let stale_barrier = barrier.clone();
        let stale = tokio::spawn(async move {
            stale_barrier.wait().await;
            tokio::task::yield_now().await;
            stale_manager.complete_start(a.port, a.generation)
        });

        manager.table.lock().begin_drain(8081).unwrap();
        let c = manager
            .table
            .lock()
            .begin_start(8081, ListenerKind::Http)
            .unwrap();
        barrier.wait().await;

        assert!(!stale.await.unwrap());
        assert!(manager.complete_start(c.port, c.generation));
        let table = manager.table.lock();
        let active = table.entries.get(&8081).unwrap();
        assert_eq!(active.generation, c.generation);
        assert_eq!(active.state, ListenerState::Active);
    }

    #[test]
    fn shutdown_fences_future_starts_and_completions() {
        let mut table = ListenerTable::new();
        let reservation = table.begin_start(8082, ListenerKind::Http).unwrap();

        let drained = table.begin_shutdown();

        assert!(reservation.cancel.is_cancelled());
        assert_eq!(drained.len(), 1);
        assert!(!table.complete_start(8082, reservation.generation));
        assert!(table.begin_start(8082, ListenerKind::Http).is_none());
        assert!(table.entries.is_empty());
    }

    #[test]
    fn bind_failure_is_propagated_to_listener_apply() {
        let error = bind_listener_workers_with(1, || {
            Err(std::io::Error::from(std::io::ErrorKind::AddrInUse).into())
        })
        .unwrap_err();

        assert!(
            error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|error| error.kind() == std::io::ErrorKind::AddrInUse),
            "unexpected bind error: {error}"
        );
    }
}
