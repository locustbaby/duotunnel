use crate::bootstrap::config::{IngressListener, IngressMode};
use crate::ServerState;
use anyhow::Context;
use parking_lot::Mutex as ParkingMutex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

#[derive(Clone)]
struct ListenerEntry {
    generation: u64,
    kind: ListenerKind,
    state: ListenerState,
    cancel: CancellationToken,
    drained: CancellationToken,
    abort_handles: Vec<tokio::task::AbortHandle>,
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
    Tcp,
}

struct StartReservation {
    port: u16,
    generation: u64,
    cancel: CancellationToken,
    drained: CancellationToken,
    predecessor: Option<ListenerEntry>,
}

struct DrainReservation {
    port: u16,
    generation: u64,
    drained: CancellationToken,
    abort_handles: Vec<tokio::task::AbortHandle>,
}

enum ListenerOperation {
    Start {
        reservation: StartReservation,
        listener: IngressListener,
        sockets: Vec<Arc<TcpListener>>,
    },
    Drain(DrainReservation),
}

struct OperationFence {
    port: u16,
    generation: u64,
    state: ListenerState,
    cancel: CancellationToken,
    drained: CancellationToken,
    predecessor: Option<ListenerEntry>,
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
                predecessor: reservation.predecessor.clone(),
            },
            Self::Drain(reservation) => OperationFence {
                port: reservation.port,
                generation: reservation.generation,
                state: ListenerState::Draining,
                cancel: CancellationToken::new(),
                drained: reservation.drained.clone(),
                predecessor: None,
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
                abort_handles: Vec::new(),
            },
        );
        Some(StartReservation {
            port,
            generation,
            cancel,
            drained,
            predecessor,
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
        let abort_handles = entry.abort_handles.clone();
        self.entries.insert(
            port,
            ListenerEntry {
                generation,
                kind: entry.kind,
                state: ListenerState::Draining,
                cancel: entry.cancel,
                drained: drained.clone(),
                abort_handles: entry.abort_handles,
            },
        );
        Some(DrainReservation {
            port,
            generation,
            drained,
            abort_handles,
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

    fn restore_predecessor(
        &mut self,
        port: u16,
        generation: u64,
        predecessor: Option<ListenerEntry>,
    ) {
        let can_restore = self.remove_if_current(port, generation, ListenerState::Starting)
            && !self.shutting_down;
        if let Some(predecessor) = predecessor {
            if can_restore && predecessor.is_live_for_restore() {
                self.entries.insert(port, predecessor);
            } else {
                predecessor.cancel.cancel();
            }
        }
    }

    fn set_worker_abort_handles(
        &mut self,
        port: u16,
        generation: u64,
        abort_handles: Vec<tokio::task::AbortHandle>,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(&port) else {
            return false;
        };
        if entry.generation != generation || entry.state != ListenerState::Active {
            return false;
        }
        entry.abort_handles = abort_handles;
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

    fn begin_shutdown(&mut self) -> Vec<(u16, CancellationToken, Vec<tokio::task::AbortHandle>)> {
        self.shutting_down = true;
        self.entries
            .drain()
            .map(|(port, entry)| {
                entry.cancel.cancel();
                (port, entry.drained, entry.abort_handles)
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
        accept_workers: usize,
    ) -> anyhow::Result<Vec<ListenerOperation>> {
        self.plan_sync_with(desired, affected_ports, accept_workers, |port, workers| {
            bind_listener_workers(port, workers)
        })
    }

    fn plan_sync_with<F>(
        &self,
        desired: &[IngressListener],
        affected_ports: Option<&HashSet<u16>>,
        accept_workers: usize,
        mut bind: F,
    ) -> anyhow::Result<Vec<ListenerOperation>>
    where
        F: FnMut(u16, usize) -> anyhow::Result<Vec<Arc<TcpListener>>>,
    {
        let desired_by_port: HashMap<u16, &IngressListener> = desired
            .iter()
            .filter(|listener| affected_ports.is_none_or(|ports| ports.contains(&listener.port)))
            .map(|listener| (listener.port, listener))
            .collect();

        let mut table = self.table.lock();
        if table.shutting_down {
            return Ok(Vec::new());
        }

        let existing_ports: Vec<u16> = table
            .entries
            .keys()
            .copied()
            .filter(|port| affected_ports.is_none_or(|ports| ports.contains(port)))
            .collect();
        let drain_ports = existing_ports
            .into_iter()
            .filter(|port| !desired_by_port.contains_key(port))
            .collect::<Vec<_>>();
        let start_specs = desired_by_port
            .into_iter()
            .filter_map(|(port, listener)| {
                let kind = ListenerKind::from_listener(listener);
                let already_active = table.entries.get(&port).is_some_and(|entry| {
                    entry.state != ListenerState::Draining && entry.kind == kind
                });
                (!already_active).then(|| (port, kind, listener.clone()))
            })
            .collect::<Vec<_>>();

        // Keep the listener table locked through preparation. Worker exits and
        // shutdown cannot invalidate the predecessor set between the last bind
        // and the single logical commit below.
        let mut prepared = Vec::with_capacity(start_specs.len());
        for (port, kind, listener) in start_specs {
            let sockets = bind(port, accept_workers)
                .with_context(|| format!("failed to pre-bind listener {port}"))?;
            if sockets.is_empty() {
                anyhow::bail!("listener {port} has no accept workers");
            }
            prepared.push((port, kind, listener, sockets));
        }

        let mut operations = Vec::with_capacity(drain_ports.len() + prepared.len());
        for port in drain_ports {
            if let Some(reservation) = table.begin_drain(port) {
                operations.push(ListenerOperation::Drain(reservation));
            }
        }
        for (port, kind, listener, sockets) in prepared {
            let reservation = table
                .begin_start(port, kind)
                .expect("validated listener start changed while table lock was held");
            operations.push(ListenerOperation::Start {
                reservation,
                listener,
                sockets,
            });
        }
        Ok(operations)
    }

    fn is_current_start(&self, port: u16, generation: u64) -> bool {
        self.table.lock().is_current_start(port, generation)
    }

    fn complete_start(&self, port: u16, generation: u64) -> bool {
        self.table.lock().complete_start(port, generation)
    }

    fn fail_start(&self, port: u16, generation: u64, predecessor: Option<ListenerEntry>) {
        self.table
            .lock()
            .restore_predecessor(port, generation, predecessor);
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

    fn set_worker_abort_handles(
        &self,
        port: u16,
        generation: u64,
        abort_handles: Vec<tokio::task::AbortHandle>,
    ) -> bool {
        self.table
            .lock()
            .set_worker_abort_handles(port, generation, abort_handles)
    }

    fn fail_operation(&self, fence: OperationFence) {
        fence.cancel.cancel();
        fence.drained.cancel();
        let mut predecessor_to_stop = None;
        {
            let mut table = self.table.lock();
            if fence.state == ListenerState::Starting {
                let current_allows_restore = table.entries.get(&fence.port).is_none_or(|entry| {
                    entry.generation == fence.generation
                        && matches!(entry.state, ListenerState::Starting | ListenerState::Active)
                });
                let can_restore = current_allows_restore
                    && !table.shutting_down
                    && fence
                        .predecessor
                        .as_ref()
                        .is_some_and(ListenerEntry::is_live_for_restore);
                table.remove_if_current(fence.port, fence.generation, ListenerState::Starting);
                table.remove_if_current(fence.port, fence.generation, ListenerState::Active);
                if can_restore {
                    if let Some(predecessor) = fence.predecessor {
                        table.entries.insert(fence.port, predecessor);
                    }
                } else {
                    predecessor_to_stop = fence.predecessor;
                }
            } else {
                table.remove_if_current(fence.port, fence.generation, fence.state);
            }
        }
        if let Some(predecessor) = predecessor_to_stop {
            predecessor.cancel.cancel();
            for handle in predecessor.abort_handles {
                handle.abort();
            }
        }
    }

    fn begin_shutdown(&self) -> Vec<(u16, CancellationToken, Vec<tokio::task::AbortHandle>)> {
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

impl ListenerEntry {
    fn is_live_for_restore(&self) -> bool {
        self.state == ListenerState::Active
            && !self.cancel.is_cancelled()
            && !self.drained.is_cancelled()
            && self
                .abort_handles
                .iter()
                .all(|handle| !handle.is_finished())
    }
}

impl ListenerKind {
    fn from_listener(listener: &IngressListener) -> Self {
        match &listener.mode {
            IngressMode::Http(_) => Self::Http,
            IngressMode::Tcp(_) => Self::Tcp,
        }
    }
}

const LISTENER_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const LISTENER_ABORT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

async fn wait_listener_drained(
    port: u16,
    drained: &CancellationToken,
    abort_handles: &[tokio::task::AbortHandle],
) -> bool {
    if tokio::time::timeout(LISTENER_DRAIN_TIMEOUT, drained.cancelled())
        .await
        .is_ok()
    {
        return true;
    }
    {
        warn!(
            port = %port,
            timeout_secs = LISTENER_DRAIN_TIMEOUT.as_secs(),
            workers = abort_handles.len(),
            "listener did not report drained in time; aborting worker set"
        );
        for handle in abort_handles {
            handle.abort();
        }
        if tokio::time::timeout(LISTENER_ABORT_TIMEOUT, drained.cancelled())
            .await
            .is_ok()
        {
            return true;
        } else {
            error!(
                port = %port,
                timeout_secs = LISTENER_ABORT_TIMEOUT.as_secs(),
                "listener worker set did not finalize after abort"
            );
        }
    }
    false
}

struct ListenerWorkerExit {
    state: Arc<ServerState>,
    port: u16,
    generation: u64,
    drained: CancellationToken,
    remaining: Arc<AtomicUsize>,
}

impl Drop for ListenerWorkerExit {
    fn drop(&mut self) {
        if self.remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.drained.cancel();
            if self
                .state
                .listeners()
                .worker_set_exited(self.port, self.generation)
            {
                self.state
                    .health()
                    .listener_worker_exited(self.port, self.generation);
            }
        }
    }
}

fn bind_listener_workers(
    port: u16,
    accept_workers: usize,
) -> anyhow::Result<Vec<Arc<TcpListener>>> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    bind_listener_workers_with(accept_workers, || {
        duotunnel_lib::build_reuseport_listener(addr)
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
    sockets: Vec<Arc<TcpListener>>,
) -> anyhow::Result<()> {
    let StartReservation {
        port,
        generation,
        cancel,
        drained,
        predecessor,
    } = reservation;

    if cancel.is_cancelled() {
        state.listeners().fail_start(port, generation, predecessor);
        drained.cancel();
        return Ok(());
    }
    if !state.listeners().is_current_start(port, generation) {
        if let Some(predecessor) = predecessor {
            predecessor.cancel.cancel();
            if !wait_listener_drained(port, &predecessor.drained, &predecessor.abort_handles).await
            {
                anyhow::bail!("listener {port} predecessor did not drain after replacement race");
            }
        }
        drained.cancel();
        return Ok(());
    }

    if cancel.is_cancelled() || !state.listeners().complete_start(port, generation) {
        drop(sockets);
        state.listeners().fail_start(port, generation, predecessor);
        drained.cancel();
        return Ok(());
    }

    let remaining = Arc::new(AtomicUsize::new(sockets.len()));
    let mut abort_handles = Vec::with_capacity(sockets.len());
    let start_gate = CancellationToken::new();
    match listener.mode {
        IngressMode::Http(_) => {
            for listener_socket in sockets {
                let worker_state = state.clone();
                let worker_cancel = cancel.clone();
                let worker_drained = drained.clone();
                let worker_remaining = remaining.clone();
                let worker_start_gate = start_gate.clone();
                let handle = state.proxy_handle().spawn(async move {
                    worker_start_gate.cancelled().await;
                    let _exit = ListenerWorkerExit {
                        state: worker_state.clone(),
                        port,
                        generation,
                        drained: worker_drained,
                        remaining: worker_remaining,
                    };
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
                });
                abort_handles.push(handle.abort_handle());
            }
        }
        IngressMode::Tcp(_) => {
            for listener_socket in sockets {
                let worker_state = state.clone();
                let worker_cancel = cancel.clone();
                let worker_drained = drained.clone();
                let worker_remaining = remaining.clone();
                let worker_start_gate = start_gate.clone();
                let handle = state.proxy_handle().spawn(async move {
                    worker_start_gate.cancelled().await;
                    let _exit = ListenerWorkerExit {
                        state: worker_state.clone(),
                        port,
                        generation,
                        drained: worker_drained,
                        remaining: worker_remaining,
                    };
                    if let Err(e) = crate::ingress::handlers::tcp::run_tcp_accept_loop(
                        listener_socket,
                        worker_state.clone(),
                        port,
                        worker_cancel,
                    )
                    .await
                    {
                        error!(port = %port, generation, error = %e, "TCP accept loop failed");
                    }
                });
                abort_handles.push(handle.abort_handle());
            }
        }
    }
    let workers_registered =
        state
            .listeners()
            .set_worker_abort_handles(port, generation, abort_handles.clone());
    if !workers_registered {
        for handle in abort_handles {
            handle.abort();
        }
        state.listeners().fail_operation(OperationFence {
            port,
            generation,
            state: ListenerState::Starting,
            cancel,
            drained,
            predecessor,
        });
        anyhow::bail!("listener {port} activation lost its lifecycle reservation");
    }
    start_gate.cancel();
    if let Some(predecessor) = predecessor {
        predecessor.cancel.cancel();
        if !wait_listener_drained(port, &predecessor.drained, &predecessor.abort_handles).await {
            anyhow::bail!("listener {port} predecessor did not drain after replacement");
        }
    }

    if workers_registered {
        info!(port = %port, generation, "listener active");
    }
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
            sockets,
        } => run_start_operation(state, reservation, listener, sockets).await,
        ListenerOperation::Drain(reservation) => {
            if !wait_listener_drained(
                reservation.port,
                &reservation.drained,
                &reservation.abort_handles,
            )
            .await
            {
                anyhow::bail!("listener {} did not drain before timeout", reservation.port);
            }
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
    let plan_state = state.clone();
    let plan_desired = desired.to_vec();
    let plan_affected_ports = affected_ports.cloned();
    let operations = state
        .proxy_handle()
        .spawn(async move {
            plan_state.listeners().plan_sync(
                &plan_desired,
                plan_affected_ports.as_ref(),
                plan_state.accept_workers(),
            )
        })
        .await
        .map_err(|error| anyhow::anyhow!("listener prepare task failed: {error}"))??;
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
                let port = fence.port;
                let generation = fence.generation;
                state.listeners().fail_operation(fence);
                failures.insert(port, (generation, e.to_string()));
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
    let _guard = state.listener_apply_gate().lock().await;
    sync_listeners_inner(state, desired, None).await
}

pub(crate) async fn sync_current_listeners(state: &Arc<ServerState>) -> anyhow::Result<()> {
    let _guard = state.listener_apply_gate().lock().await;
    let desired = state.ingress_listeners();
    sync_listeners_inner(state, &desired, None).await
}

pub(crate) async fn shutdown_all_listeners(state: &Arc<ServerState>) {
    let _guard = state.listener_apply_gate().lock().await;
    let drained = state.listeners().begin_shutdown();
    for (port, drained, abort_handles) in drained {
        let _ = wait_listener_drained(port, &drained, &abort_handles).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::config::TcpListenerConfig;
    use tokio::sync::Barrier;

    fn tcp_listener(port: u16) -> IngressListener {
        IngressListener {
            port,
            mode: IngressMode::Tcp(TcpListenerConfig {
                client_group: "test-group".into(),
                proxy_name: "test-proxy".into(),
            }),
        }
    }

    #[test]
    fn replacement_prepare_keeps_predecessor_active_until_commit() {
        let mut table = ListenerTable::new();
        let a = table.begin_start(8080, ListenerKind::Http).unwrap();
        assert!(table.complete_start(8080, a.generation));
        let b = table.begin_start(8080, ListenerKind::Tcp).unwrap();

        assert!(!a.cancel.is_cancelled());
        assert!(!table.complete_start(8080, a.generation));
        assert_eq!(
            b.predecessor.as_ref().map(|entry| entry.generation),
            Some(a.generation)
        );

        table.restore_predecessor(8080, b.generation, b.predecessor);
        let restored = table.entries.get(&8080).unwrap();
        assert_eq!(restored.generation, a.generation);
        assert_eq!(restored.state, ListenerState::Active);
        assert!(!restored.cancel.is_cancelled());
    }

    #[test]
    fn rollback_does_not_restore_a_drained_predecessor() {
        let mut table = ListenerTable::new();
        let active = table.begin_start(8083, ListenerKind::Http).unwrap();
        assert!(table.complete_start(8083, active.generation));
        let replacement = table.begin_start(8083, ListenerKind::Tcp).unwrap();
        replacement.predecessor.as_ref().unwrap().drained.cancel();

        table.restore_predecessor(
            replacement.port,
            replacement.generation,
            replacement.predecessor,
        );

        assert!(!table.entries.contains_key(&8083));
    }

    #[test]
    fn activation_failure_restores_live_predecessor() {
        let manager = ListenerManager::new();
        let fence = {
            let mut table = manager.table.lock();
            let active = table.begin_start(8084, ListenerKind::Http).unwrap();
            assert!(table.complete_start(8084, active.generation));
            let replacement = table.begin_start(8084, ListenerKind::Tcp).unwrap();
            assert!(table.complete_start(8084, replacement.generation));
            OperationFence {
                port: replacement.port,
                generation: replacement.generation,
                state: ListenerState::Starting,
                cancel: replacement.cancel,
                drained: replacement.drained,
                predecessor: replacement.predecessor,
            }
        };

        manager.fail_operation(fence);

        let table = manager.table.lock();
        let restored = table.entries.get(&8084).unwrap();
        assert_eq!(restored.kind, ListenerKind::Http);
        assert_eq!(restored.state, ListenerState::Active);
        assert!(!restored.cancel.is_cancelled());
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

    #[tokio::test]
    async fn multi_port_bind_failure_commits_no_listener_changes() {
        let manager = ListenerManager::new();
        let first = manager
            .table
            .lock()
            .begin_start(8084, ListenerKind::Http)
            .unwrap();
        assert!(manager.complete_start(first.port, first.generation));
        let second = manager
            .table
            .lock()
            .begin_start(8085, ListenerKind::Http)
            .unwrap();
        assert!(manager.complete_start(second.port, second.generation));
        let first_cancel = first.cancel.clone();
        let second_cancel = second.cancel.clone();
        let desired = vec![tcp_listener(8084), tcp_listener(8085)];
        let mut binds = 0;

        let result = manager.plan_sync_with(&desired, None, 1, |_port, _workers| {
            binds += 1;
            if binds == 2 {
                return Err(std::io::Error::from(std::io::ErrorKind::AddrInUse).into());
            }
            let socket = std::net::TcpListener::bind("127.0.0.1:0")?;
            socket.set_nonblocking(true)?;
            Ok(vec![Arc::new(TcpListener::from_std(socket)?)])
        });

        assert!(result.is_err());
        assert!(!first_cancel.is_cancelled());
        assert!(!second_cancel.is_cancelled());
        let table = manager.table.lock();
        assert_eq!(
            table.entries.get(&8084).map(|entry| entry.generation),
            Some(first.generation)
        );
        assert_eq!(
            table.entries.get(&8085).map(|entry| entry.generation),
            Some(second.generation)
        );
        assert!(table
            .entries
            .values()
            .all(|entry| entry.state == ListenerState::Active));
    }
}
