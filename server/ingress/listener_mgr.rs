use crate::ServerState;
use crate::bootstrap::config::{IngressListener, IngressMode};
use parking_lot::Mutex as ParkingMutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

struct ListenerEntry {
    kind: ListenerKind,
    state: ListenerState,
    cancel: CancellationToken,
    drained: Arc<Notify>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListenerState {
    Active,
    Draining,
}

use tunnel_lib::{GroupId, ProxyName};

enum ListenerKind {
    Http,
    Tcp {
        group_id: GroupId,
        proxy_name: ProxyName,
    },
}

pub(crate) struct ListenerManager {
    next_generation: AtomicU64,
    map: ParkingMutex<HashMap<u16, ListenerEntry>>,
}

impl ListenerManager {
    pub(crate) fn new() -> Self {
        Self {
            next_generation: AtomicU64::new(1),
            map: ParkingMutex::new(HashMap::new()),
        }
    }

    pub(crate) fn next_generation(&self) -> u64 {
        self.next_generation.fetch_add(1, Ordering::Relaxed)
    }

    fn lock_map(&self) -> parking_lot::MutexGuard<'_, HashMap<u16, ListenerEntry>> {
        self.map.lock()
    }

    fn activate(
        &self,
        port: u16,
        kind: ListenerKind,
        cancel: CancellationToken,
        drained: Arc<Notify>,
    ) {
        self.map.lock().insert(
            port,
            ListenerEntry {
                kind,
                state: ListenerState::Active,
                cancel,
                drained,
            },
        );
    }
}

fn notify_listener_drained(drained: &Notify) {
    drained.notify_waiters();
    drained.notify_one();
}

async fn spawn_single_listener(state: Arc<ServerState>, port: u16, listener: IngressListener) {
    let accept_workers = state.accept_workers();
    let generation = state.listeners().next_generation();
    let cancel = CancellationToken::new();
    let drained = Arc::new(Notify::new());
    let remaining = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(accept_workers);
    let kind = match &listener.mode {
        IngressMode::Http(_) => ListenerKind::Http,
        IngressMode::Tcp(cfg) => ListenerKind::Tcp {
            group_id: cfg.client_group.clone(),
            proxy_name: cfg.proxy_name.clone(),
        },
    };

    match listener.mode {
        IngressMode::Http(_) => {
            for _ in 0..accept_workers {
                let s = state.clone();
                let listener_socket = match tunnel_lib::build_reuseport_listener(
                    std::net::SocketAddr::from(([0, 0, 0, 0], port)),
                ) {
                    Ok(listener_socket) => Arc::new(listener_socket),
                    Err(e) => {
                        error!(port = %port, error = %e, "failed to bind http worker listener");
                        break;
                    }
                };
                let cancel = cancel.clone();
                let drained = drained.clone();
                let remaining = remaining.clone();
                remaining.fetch_add(1, Ordering::Release);
                handles.push(tokio::spawn(async move {
                    if let Err(e) = crate::ingress::handlers::http::run_http_accept_loop(
                        listener_socket,
                        s,
                        port,
                        cancel,
                    )
                    .await
                    {
                        error!(port = %port, error = %e, "HTTP accept loop failed");
                    }
                    if remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
                        notify_listener_drained(&drained);
                    }
                }));
            }
        }
        IngressMode::Tcp(cfg) => {
            for _ in 0..accept_workers {
                let s = state.clone();
                let listener_socket = match tunnel_lib::build_reuseport_listener(
                    std::net::SocketAddr::from(([0, 0, 0, 0], port)),
                ) {
                    Ok(listener_socket) => Arc::new(listener_socket),
                    Err(e) => {
                        error!(port = %port, error = %e, "failed to bind tcp worker listener");
                        break;
                    }
                };
                let cancel = cancel.clone();
                let drained = drained.clone();
                let remaining = remaining.clone();
                let group_id = cfg.client_group.clone();
                let proxy_name = cfg.proxy_name.clone();
                remaining.fetch_add(1, Ordering::Release);
                handles.push(tokio::spawn(async move {
                    if let Err(e) = crate::ingress::handlers::tcp::run_tcp_accept_loop(
                        listener_socket,
                        s,
                        port,
                        proxy_name,
                        group_id,
                        cancel,
                    )
                    .await
                    {
                        error!(port = %port, error = %e, "TCP accept loop failed");
                    }
                    if remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
                        notify_listener_drained(&drained);
                    }
                }));
            }
        }
    }

    if handles.is_empty() {
        cancel.cancel();
        notify_listener_drained(&drained);
        return;
    }

    info!(port = %port, generation, "listener active");
    state.listeners().activate(port, kind, cancel, drained);
}

async fn sync_listeners_inner(
    state: &Arc<ServerState>,
    desired: &[IngressListener],
    affected_ports: Option<&HashSet<u16>>,
) {
    let desired_by_port: HashMap<u16, &IngressListener> = desired
        .iter()
        .filter(|listener| affected_ports.is_none_or(|ports| ports.contains(&listener.port)))
        .map(|l| (l.port, l))
        .collect();

    let mut map = state.listeners().lock_map();
    let existing_ports: Vec<u16> = map
        .keys()
        .copied()
        .filter(|port| affected_ports.is_none_or(|ports| ports.contains(port)))
        .collect();

    for port in existing_ports {
        let should_remove = match desired_by_port.get(&port) {
            None => true,
            Some(listener) => match (&map[&port].kind, &listener.mode) {
                (ListenerKind::Http, IngressMode::Http(_)) => false,
                (
                    ListenerKind::Tcp {
                        group_id,
                        proxy_name,
                    },
                    IngressMode::Tcp(cfg),
                ) => group_id != &cfg.client_group || proxy_name != &cfg.proxy_name,
                _ => true,
            },
        };
        if should_remove {
            if let Some(mut entry) = map.remove(&port) {
                entry.state = ListenerState::Draining;
                entry.cancel.cancel();
                let desired_opt = desired_by_port.get(&port).cloned().cloned();
                let state_clone = state.clone();
                tokio::spawn(async move {
                    entry.drained.notified().await;
                    if let Some(desired_listener) = desired_opt {
                        spawn_single_listener(state_clone, port, desired_listener).await;
                    }
                });
            }
        }
    }

    for (port, listener) in &desired_by_port {
        if !map.contains_key(port) {
            let state_clone = state.clone();
            let listener_clone = (*listener).clone();
            let p = *port;
            tokio::spawn(async move {
                spawn_single_listener(state_clone, p, listener_clone).await;
            });
        }
    }
}

pub(crate) async fn sync_all_listeners(state: &Arc<ServerState>, desired: &[IngressListener]) {
    sync_listeners_inner(state, desired, None).await;
}

pub(crate) async fn sync_listener_subset(
    state: &Arc<ServerState>,
    desired: &[IngressListener],
    affected_ports: &HashSet<u16>,
) {
    if affected_ports.is_empty() {
        return;
    }
    sync_listeners_inner(state, desired, Some(affected_ports)).await;
}

pub(crate) async fn sync_listeners(state: &Arc<ServerState>, desired: &[IngressListener]) {
    sync_all_listeners(state, desired).await;
}

pub(crate) async fn shutdown_all_listeners(state: &Arc<ServerState>) {
    let drained = {
        let mut map = state.listeners().lock_map();
        map.drain()
            .map(|(_, mut entry)| {
                entry.state = ListenerState::Draining;
                entry.cancel.cancel();
                entry.drained
            })
            .collect::<Vec<_>>()
    };
    for notify in drained {
        notify.notified().await;
    }
}
