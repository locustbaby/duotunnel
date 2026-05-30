use crate::config::{IngressListener, IngressMode};
use crate::ServerState;
use parking_lot::Mutex as ParkingMutex;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct ListenerEntry {
    pub generation: u64,
    pub kind: ListenerKind,
    pub state: ListenerState,
    pub cancel: CancellationToken,
    pub drained: Arc<Notify>,
    pub handles: Vec<JoinHandle<()>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ListenerState {
    Starting,
    Active,
    Draining,
}

pub enum ListenerKind {
    Http,
    Tcp {
        group_id: String,
        proxy_name: String,
    },
}

pub struct ListenerManager {
    next_generation: AtomicU64,
    pub map: ParkingMutex<HashMap<u16, ListenerEntry>>,
}

impl ListenerManager {
    pub fn new() -> Self {
        Self {
            next_generation: AtomicU64::new(1),
            map: ParkingMutex::new(HashMap::new()),
        }
    }

    fn next_generation(&self) -> u64 {
        self.next_generation.fetch_add(1, Ordering::Relaxed)
    }
}

async fn spawn_single_listener(
    state: Arc<ServerState>,
    port: u16,
    listener: IngressListener,
) {
    let accept_workers = state
        .config
        .server
        .accept_workers
        .unwrap_or(tunnel_lib::DEFAULT_ACCEPT_WORKERS)
        .max(1);

    let addr = format!("0.0.0.0:{port}");
    let generation = state.listeners.next_generation();
    let cancel = CancellationToken::new();
    let drained = Arc::new(Notify::new());
    let remaining = Arc::new(AtomicUsize::new(accept_workers));
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
                let listener_socket = match tunnel_lib::build_reuseport_listener(addr.parse().unwrap()) {
                    Ok(listener_socket) => Arc::new(listener_socket),
                    Err(e) => {
                        error!(port = %port, error = %e, "failed to bind http worker listener");
                        break;
                    }
                };
                let cancel = cancel.clone();
                let drained = drained.clone();
                let remaining = remaining.clone();
                handles.push(tokio::spawn(async move {
                    if let Err(e) = crate::handlers::http::run_http_accept_loop(
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
                        drained.notify_waiters();
                    }
                }));
            }
        }
        IngressMode::Tcp(cfg) => {
            for _ in 0..accept_workers {
                let s = state.clone();
                let listener_socket = match tunnel_lib::build_reuseport_listener(addr.parse().unwrap()) {
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
                handles.push(tokio::spawn(async move {
                    if let Err(e) = crate::handlers::tcp::run_tcp_accept_loop(
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
                        drained.notify_waiters();
                    }
                }));
            }
        }
    }

    if handles.is_empty() {
        cancel.cancel();
        return;
    }

    let entry = ListenerEntry {
        generation,
        kind,
        state: ListenerState::Active,
        cancel,
        drained,
        handles,
    };
    info!(port = %port, generation, "listener active");
    state.listeners.map.lock().insert(port, entry);
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

    let mut map = state.listeners.map.lock();
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
                    ListenerKind::Tcp { group_id, proxy_name },
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

pub async fn sync_all_listeners(state: &Arc<ServerState>, desired: &[IngressListener]) {
    sync_listeners_inner(state, desired, None).await;
}

pub async fn sync_listener_subset(
    state: &Arc<ServerState>,
    desired: &[IngressListener],
    affected_ports: &HashSet<u16>,
) {
    if affected_ports.is_empty() {
        return;
    }
    sync_listeners_inner(state, desired, Some(affected_ports)).await;
}

pub async fn sync_listeners(state: &Arc<ServerState>, desired: &[IngressListener]) {
    sync_all_listeners(state, desired).await;
}
