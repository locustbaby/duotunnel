use crate::config::{IngressListener, IngressMode};
use crate::ServerState;
use parking_lot::Mutex as ParkingMutex;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
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
pub fn sync_listeners(state: &Arc<ServerState>, desired: &[IngressListener]) {
    let desired_ports: HashSet<u16> = desired.iter().map(|l| l.port).collect();
    let desired_by_port: HashMap<u16, &IngressListener> =
        desired.iter().map(|l| (l.port, l)).collect();

    struct SpawnDesc {
        port: u16,
        listener_id: u64,
        cancel: CancellationToken,
        mode: IngressMode,
        tcp_group: Option<String>,
        tcp_proxy: Option<String>,
    }

    let to_spawn: Vec<SpawnDesc> = {
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

        // 2. Insert placeholders and collect spawn descriptors.
        let mut spawns = Vec::new();
        for (port, listener) in &desired_by_port {
            if map.contains_key(port) {
                continue;
            }
            let listener_id = state.listeners.next_id();
            let cancel = CancellationToken::new();
            let (kind, tcp_group, tcp_proxy) = match &listener.mode {
                IngressMode::Http(_) => (ListenerKind::Http, None, None),
                IngressMode::Tcp(cfg) => (
                    ListenerKind::Tcp {
                        group_id: cfg.client_group.clone(),
                        proxy_name: cfg.proxy_name.clone(),
                    },
                    Some(cfg.client_group.clone()),
                    Some(cfg.proxy_name.clone()),
                ),
            };
            map.insert(
                *port,
                ListenerEntry {
                    id: listener_id,
                    kind,
                    cancel: cancel.clone(),
                },
            );
            spawns.push(SpawnDesc {
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

    // 3. Spawn N accept loops per listener outside the lock.
    let accept_workers = state
        .config
        .server
        .accept_workers
        .unwrap_or(4)
        .max(1);

    for desc in to_spawn {
        let SpawnDesc {
            port,
            listener_id,
            cancel,
            mode,
            tcp_group,
            tcp_proxy,
        } = desc;

        let addr = format!("0.0.0.0:{}", port);
        let listener = match tunnel_lib::build_reuseport_listener(addr.parse().unwrap()) {
            Ok(l) => Arc::new(l),
            Err(e) => {
                error!(port = %port, error = %e, "failed to bind listener");
                continue;
            }
        };

        match mode {
            IngressMode::Http(_) => {
                for i in 0..accept_workers {
                    let s = state.clone();
                    let listener = listener.clone();
                    let cancel = cancel.clone();
                    let is_last = i == accept_workers - 1;
                    tokio::task::spawn(async move {
                        if let Err(e) =
                            crate::handlers::http::run_http_accept_loop(listener, s.clone(), port, cancel).await
                        {
                            error!(port = %port, error = %e, "HTTP accept loop failed");
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
                for i in 0..accept_workers {
                    let s = state.clone();
                    let listener = listener.clone();
                    let cancel = cancel.clone();
                    let proxy_name = proxy_name.clone();
                    let group_id = group_id.clone();
                    let is_last = i == accept_workers - 1;
                    tokio::task::spawn(async move {
                        if let Err(e) = crate::handlers::tcp::run_tcp_accept_loop(
                            listener,
                            s.clone(),
                            port,
                            proxy_name,
                            group_id,
                            cancel,
                        )
                        .await
                        {
                            error!(port = %port, error = %e, "TCP accept loop failed");
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
