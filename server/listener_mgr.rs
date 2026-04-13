use crate::config::{IngressListener, IngressMode};
use crate::ServerState;
use parking_lot::Mutex as ParkingMutex;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct ListenerEntry {
    pub id: u64,
    pub kind: ListenerKind,
    pub cancel: CancellationToken,
}

pub enum ListenerKind {
    Http,
    Tcp {
        group_id: String,
        proxy_name: String,
    },
}

/// Tracks all running ingress listener tasks.
pub struct ListenerManager {
    next_id: AtomicU64,
    pub map: ParkingMutex<HashMap<u16, ListenerEntry>>,
}

impl ListenerManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            map: ParkingMutex::new(HashMap::new()),
        }
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

/// Reconcile the running listeners against a desired set.
/// Cancels removed/changed listeners and spawns new ones.
/// All map mutations happen under a single lock acquisition.
pub fn sync_listeners(
    state: &Arc<ServerState>,
    desired: &[IngressListener],
    proxy_handle: &tokio::runtime::Handle,
    accept_workers: usize,
) {
    let desired_ports: HashSet<u16> = desired.iter().map(|l| l.port).collect();
    let desired_by_port: HashMap<u16, &IngressListener> =
        desired.iter().map(|l| (l.port, l)).collect();

    struct PrepareDesc {
        port: u16,
        listener_id: u64,
        cancel: CancellationToken,
        mode: IngressMode,
        tcp_group: Option<String>,
        tcp_proxy: Option<String>,
    }

    struct SpawnDesc {
        port: u16,
        listener_id: u64,
        cancel: CancellationToken,
        mode: IngressMode,
        tcp_group: Option<String>,
        tcp_proxy: Option<String>,
        listeners: Vec<TcpListener>,
    }

    let to_prepare: Vec<PrepareDesc> = {
        let mut map = state.listeners.map.lock();

        // 1. Prune removed/changed listeners and cancel them.
        let mut to_cancel = Vec::new();
        map.retain(|port, entry| {
            if !desired_ports.contains(port) {
                info!(port = %port, "listener removed (hot-reload)");
                to_cancel.push(entry.cancel.clone());
                return false;
            }
            let listener = desired_by_port[port];
            let changed = match (&entry.kind, &listener.mode) {
                (ListenerKind::Http, IngressMode::Http(_)) => false,
                (
                    ListenerKind::Tcp {
                        group_id,
                        proxy_name,
                    },
                    IngressMode::Tcp(cfg),
                ) => group_id != &cfg.client_group || proxy_name != &cfg.proxy_name,
                _ => true,
            };
            if changed {
                info!(port = %port, "listener config changed (hot-reload), restarting");
                to_cancel.push(entry.cancel.clone());
                false
            } else {
                true
            }
        });
        for cancel in to_cancel {
            cancel.cancel();
        }

        // 2. Collect spawn descriptors for listeners that need to be started.
        let mut spawns = Vec::new();
        for (port, listener) in &desired_by_port {
            if map.contains_key(port) {
                continue;
            }
            let listener_id = state.listeners.next_id();
            let cancel = CancellationToken::new();
            let (tcp_group, tcp_proxy) = match &listener.mode {
                IngressMode::Http(_) => (None, None),
                IngressMode::Tcp(cfg) => (
                    Some(cfg.client_group.clone()),
                    Some(cfg.proxy_name.clone()),
                ),
            };
            spawns.push(PrepareDesc {
                port: *port,
                listener_id,
                cancel,
                mode: listener.mode.clone(),
                tcp_group,
                tcp_proxy,
            });
        }
        spawns
    }; // lock released here

    let mut prepared = Vec::new();
    for desc in to_prepare {
        let addr: std::net::SocketAddr = format!("0.0.0.0:{}", desc.port).parse().unwrap();
        let listeners = match bind_listener_workers(addr, accept_workers) {
            Ok(listeners) => listeners,
            Err(e) => {
                error!(port = %desc.port, error = %e, "failed to bind listener");
                continue;
            }
        };
        prepared.push(SpawnDesc {
            port: desc.port,
            listener_id: desc.listener_id,
            cancel: desc.cancel,
            mode: desc.mode,
            tcp_group: desc.tcp_group,
            tcp_proxy: desc.tcp_proxy,
            listeners,
        });
    }

    let to_spawn: Vec<SpawnDesc> = {
        let mut map = state.listeners.map.lock();
        let mut spawns = Vec::new();
        for desc in prepared {
            if map.contains_key(&desc.port) {
                continue;
            }
            let kind = match &desc.mode {
                IngressMode::Http(_) => ListenerKind::Http,
                IngressMode::Tcp(_) => ListenerKind::Tcp {
                    group_id: desc.tcp_group.clone().unwrap_or_default(),
                    proxy_name: desc.tcp_proxy.clone().unwrap_or_default(),
                },
            };
            map.insert(
                desc.port,
                ListenerEntry {
                    id: desc.listener_id,
                    kind,
                    cancel: desc.cancel.clone(),
                },
            );
            spawns.push(desc);
        }
        spawns
    };

    // 3. Spawn tasks outside the lock.
    for desc in to_spawn {
        let SpawnDesc {
            port,
            listener_id,
            cancel,
            mode,
            tcp_group,
            tcp_proxy,
            listeners,
        } = desc;
        match mode {
            IngressMode::Http(_) => {
                let worker_count = listeners.len();
                for (worker_idx, listener) in listeners.into_iter().enumerate() {
                    let s = state.clone();
                    let cancel = cancel.clone();
                    let is_last = worker_idx + 1 == worker_count;
                    proxy_handle.spawn(async move {
                        if let Err(e) =
                            crate::handlers::http::run_http_listener(s.clone(), listener, port, cancel).await
                        {
                            error!(port = %port, error = %e, "HTTP listener failed");
                        }
                        if is_last {
                            let mut map = s.listeners.map.lock();
                            if map.get(&port).map(|e| e.id == listener_id).unwrap_or(false) {
                                map.remove(&port);
                            }
                        }
                    });
                }
            }
            IngressMode::Tcp(_) => {
                let group_id = tcp_group.unwrap_or_default();
                let proxy_name = tcp_proxy.unwrap_or_default();
                let worker_count = listeners.len();
                for (worker_idx, listener) in listeners.into_iter().enumerate() {
                    let s = state.clone();
                    let cancel = cancel.clone();
                    let group_id = group_id.clone();
                    let proxy_name = proxy_name.clone();
                    let is_last = worker_idx + 1 == worker_count;
                    proxy_handle.spawn(async move {
                        if let Err(e) = crate::handlers::tcp::run_tcp_listener(
                            s.clone(),
                            listener,
                            port,
                            proxy_name,
                            group_id,
                            cancel,
                        )
                        .await
                        {
                            error!(port = %port, error = %e, "TCP listener failed");
                        }
                        if is_last {
                            let mut map = s.listeners.map.lock();
                            if map.get(&port).map(|e| e.id == listener_id).unwrap_or(false) {
                                map.remove(&port);
                            }
                        }
                    });
                }
            }
        }
    }
}

fn bind_listener_workers(
    addr: std::net::SocketAddr,
    accept_workers: usize,
) -> anyhow::Result<Vec<TcpListener>> {
    let mut listeners = Vec::with_capacity(accept_workers);
    for _ in 0..accept_workers {
        listeners.push(tunnel_lib::build_reuseport_listener(addr)?);
    }
    Ok(listeners)
}
