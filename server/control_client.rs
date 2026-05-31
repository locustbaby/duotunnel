/// ControlClient: connects to tunnel-ctld and maintains the list-watch stream.
///
/// On connect:
///   1. Sends WatchRequest { resource_version: last_known } over ConfigPush framing
///   2. Receives WatchEvent::Snapshot → applies full state (routing + token cache)
///   3. Loops receiving WatchEvent::Patch → applies incremental updates
///   4. On disconnect: exponential back-off, then reconnect
use std::net::SocketAddr;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tracing::{error, info, warn};
use tunnel_lib::ctld_proto::{
    send_watch_request, ConfigPatch, ConfigSnapshot, ProtoClientGroup, ProtoEgressUpstreamDef,
    ProtoEgressVhostRule, ProtoIngressListener, ProtoIngressListenerMode, ResourceOp, WatchEvent,
    WatchRequest,
};
use tunnel_lib::models::msg::{recv_typed_message, MessageType};

use crate::config::{
    ClientConfigs, EgressHttpRule, EgressRules, GroupConfig, HttpListenerConfig, IngressListener,
    IngressMode, IngressRouting, ServerDef, ServerEgressUpstream, TcpListenerConfig,
    TunnelManagement, UpstreamDef, VhostRule,
};
use crate::local_auth::CacheEntry;
use crate::service::BackgroundService;
use crate::{build_routing_snapshot, ServerState};
use tokio_util::sync::CancellationToken;

pub struct ControlClientService {
    pub ctld_addr: SocketAddr,
    pub auth_token: Option<String>,
}

impl BackgroundService for ControlClientService {
    fn name(&self) -> &'static str {
        "control-client"
    }

    fn run(
        self: Box<Self>,
        state: Arc<ServerState>,
        shutdown: CancellationToken,
        _proxy_handle: tokio::runtime::Handle,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            watch_loop(self.ctld_addr, self.auth_token, state, shutdown).await;
            Ok(())
        })
    }
}

async fn watch_loop(
    ctld_addr: SocketAddr,
    auth_token: Option<String>,
    state: Arc<ServerState>,
    shutdown: CancellationToken,
) {
    let mut backoff = Duration::from_secs(1);
    let mut last_version: u64 = 0;
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => return,
            result = connect_and_watch(ctld_addr, auth_token.as_deref(), &state, &mut last_version) => {
                match result {
                    Ok(()) => {
                        backoff = Duration::from_secs(1);
                    }
                    Err(e) => {
                        error!(error = %e, addr = %ctld_addr, "ctld watch connection failed");
                        let jittered = {
                            let min_ms = backoff.as_millis() as u64 / 2;
                            let max_ms = backoff.as_millis() as u64;
                            let ms = if min_ms >= max_ms {
                                min_ms
                            } else {
                                min_ms + (fastrand::u64(..) % (max_ms - min_ms + 1))
                            };
                            Duration::from_millis(ms)
                        };
                        tokio::select! {
                            _ = shutdown.cancelled() => return,
                            _ = tokio::time::sleep(jittered) => {}
                        }
                        backoff = (backoff * 2).min(Duration::from_secs(30));
                    }
                }
            }
        }
    }
}

async fn connect_and_watch(
    addr: SocketAddr,
    auth_token: Option<&str>,
    state: &Arc<ServerState>,
    last_version: &mut u64,
) -> anyhow::Result<()> {
    info!(addr = %addr, "connecting to tunnel-ctld");
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut current_snapshot: Option<ConfigSnapshot> = None;

    // Step 1: send WatchRequest
    let req = WatchRequest {
        resource_version: *last_version,
        token: auth_token.map(str::to_string),
    };
    send_watch_request(&mut writer, &req).await?;
    info!(addr = %addr, resource_version = *last_version, "sent WatchRequest");

    // Step 2+3: receive Snapshot then stream Patches until error/disconnect.
    // Returns the last seen resource_version so the caller can reconnect with it.
    let err = loop {
        let event: WatchEvent = match recv_typed_message(&mut reader, MessageType::ConfigPush).await
        {
            Ok(e) => e,
            Err(e) => break e,
        };
        match event {
            WatchEvent::Snapshot(snap) => {
                let v = snap.resource_version;
                info!(resource_version = v, "received Snapshot from ctld");
                apply_snapshot(&snap, state).await;
                current_snapshot = Some(snap);
                *last_version = v;
            }
            WatchEvent::Patch(patch) => {
                let v = patch.resource_version;
                info!(resource_version = v, "received Patch from ctld");
                if let Some(snapshot) = current_snapshot.as_mut() {
                    let affected_ports = apply_patch_to_snapshot(snapshot, &patch);
                    apply_patch_to_runtime(snapshot, &patch, &affected_ports, state).await;
                } else {
                    warn!(resource_version = v, "received Patch before Snapshot; patch dropped, routing may be stale");
                }
                *last_version = v;
            }
        }
    };
    // Return the last version we successfully applied so the reconnect sends
    // resource_version=N instead of 0, allowing future delta optimisation.
    Err(err.context(format!("ctld disconnected at version {}", *last_version)))
}

/// Apply a ConfigSnapshot to both the routing ArcSwap and the token cache.
async fn apply_snapshot(snap: &ConfigSnapshot, state: &Arc<ServerState>) {
    update_token_cache(&snap.token_cache, state);
    let (listeners, routing_snapshot) = build_runtime_snapshot(snap, state);
    crate::sync_all_listeners(state, &listeners).await;
    state.routing.store(Arc::new(routing_snapshot));
}

fn update_token_cache(entries: &[tunnel_lib::shared::TokenCacheEntryDef], state: &Arc<ServerState>) {
    if let Some(cache) = state.local_token_cache.as_ref() {
        let entries: Vec<CacheEntry> = entries
            .iter()
            .filter_map(|e| {
                let bytes = match hex::decode(&e.hash_hex) {
                    Ok(b) if b.len() == 32 => {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&b);
                        arr
                    }
                    _ => {
                        warn!(hash = %e.hash_hex, "ignoring token cache entry with invalid hash");
                        return None;
                    }
                };
                Some(CacheEntry {
                    hash_bytes: bytes,
                    client_group: e.client_group.clone(),
                    client_status: e.client_status,
                    token_status: e.token_status,
                })
            })
            .collect();
        cache.update(entries);
    }
}

fn build_runtime_snapshot(
    snap: &ConfigSnapshot,
    state: &Arc<ServerState>,
) -> (Vec<IngressListener>, crate::RoutingSnapshot) {
    let tm = proto_to_tunnel_management(&snap.ingress_listeners, &snap.client_groups);
    let egress = proto_to_server_egress(&snap.egress_upstreams, &snap.egress_vhost_rules);
    let http_params = tunnel_lib::HttpClientParams::from(&state.config.server.http_pool);
    let routing_snapshot = build_routing_snapshot(&tm, &egress, &http_params);
    let listeners = tm.server_ingress_routing.listeners.clone();
    (listeners, routing_snapshot)
}

async fn apply_patch_to_runtime(
    snapshot: &ConfigSnapshot,
    patch: &ConfigPatch,
    affected_ports: &HashSet<u16>,
    state: &Arc<ServerState>,
) {
    update_token_cache(&snapshot.token_cache, state);
    let touches_routing = !patch.ingress_listeners.is_empty()
        || !patch.client_groups.is_empty()
        || !patch.egress_upstreams.is_empty()
        || !patch.egress_vhost_rules.is_empty();
    if !touches_routing {
        return;
    }
    let (listeners, routing_snapshot) = build_runtime_snapshot(snapshot, state);
    if !patch.ingress_listeners.is_empty() {
        crate::sync_listener_subset(state, &listeners, affected_ports).await;
    }
    state.routing.store(Arc::new(routing_snapshot));
}

fn apply_patch_to_snapshot(snapshot: &mut ConfigSnapshot, patch: &ConfigPatch) -> HashSet<u16> {
    snapshot.resource_version = patch.resource_version;
    let affected_ports = patch
        .ingress_listeners
        .iter()
        .filter_map(|op| match op {
            ResourceOp::Upsert(item) => Some(item.port),
            ResourceOp::Delete { key } => key.parse().ok(),
        })
        .collect();
    apply_ops(
        &mut snapshot.ingress_listeners,
        &patch.ingress_listeners,
        |item| item.port.to_string(),
    );
    apply_ops(&mut snapshot.client_groups, &patch.client_groups, |item| {
        item.group_id.clone()
    });
    apply_ops(
        &mut snapshot.egress_upstreams,
        &patch.egress_upstreams,
        |item| item.name.clone(),
    );
    apply_ops(
        &mut snapshot.egress_vhost_rules,
        &patch.egress_vhost_rules,
        |item| item.match_host.clone(),
    );
    apply_ops(&mut snapshot.token_cache, &patch.token_cache, |item| {
        item.hash_hex.clone()
    });
    affected_ports
}

fn apply_ops<T, F>(items: &mut Vec<T>, ops: &[ResourceOp<T>], key_of: F)
where
    T: Clone,
    F: Fn(&T) -> String,
{
    let mut map: std::collections::BTreeMap<String, T> =
        items.drain(..).map(|item| (key_of(&item), item)).collect();
    for op in ops {
        match op {
            ResourceOp::Upsert(item) => {
                map.insert(key_of(item), item.clone());
            }
            ResourceOp::Delete { key } => {
                map.remove(key);
            }
        }
    }
    *items = map.into_values().collect();
}

// ── Type conversions: tunnel_store routing types → server config types ────────

fn proto_to_tunnel_management(
    listeners: &[ProtoIngressListener],
    groups: &[ProtoClientGroup],
) -> TunnelManagement {
    let ingress = listeners
        .iter()
        .map(|l| IngressListener {
            port: l.port,
            mode: match &l.mode {
                ProtoIngressListenerMode::Http { vhost } => IngressMode::Http(HttpListenerConfig {
                    vhost: vhost
                        .iter()
                        .map(|r| VhostRule {
                            match_host: r.match_host.clone(),
                            client_group: r.group_id.clone(),
                            proxy_name: r.proxy_name.clone(),
                        })
                        .collect(),
                }),
                ProtoIngressListenerMode::Tcp {
                    group_id,
                    proxy_name,
                } => IngressMode::Tcp(TcpListenerConfig {
                    client_group: group_id.clone(),
                    proxy_name: proxy_name.clone(),
                }),
            },
        })
        .collect();

    let groups_map = groups
        .iter()
        .map(|g| {
            let upstreams = g
                .upstreams
                .iter()
                .map(|u| {
                    (
                        u.name.clone(),
                        UpstreamDef {
                            servers: u
                                .servers
                                .iter()
                                .map(|s| ServerDef {
                                    address: s.address.clone(),
                                    resolve: s.resolve,
                                })
                                .collect(),
                            lb_policy: u.lb_policy.clone(),
                        },
                    )
                })
                .collect();
            (
                g.group_id.clone(),
                GroupConfig {
                    config_version: g.config_version.clone(),
                    upstreams,
                },
            )
        })
        .collect();

    TunnelManagement {
        server_ingress_routing: IngressRouting { listeners: ingress },
        client_configs: ClientConfigs { groups: groups_map },
    }
}

fn proto_to_server_egress(
    upstreams: &[ProtoEgressUpstreamDef],
    vhost_rules: &[ProtoEgressVhostRule],
) -> ServerEgressUpstream {
    let upstream_map = upstreams
        .iter()
        .map(|u| {
            (
                u.name.clone(),
                UpstreamDef {
                    servers: u
                        .servers
                        .iter()
                        .map(|s| ServerDef {
                            address: s.address.clone(),
                            resolve: s.resolve,
                        })
                        .collect(),
                    lb_policy: u.lb_policy.clone(),
                },
            )
        })
        .collect();

    let vhost = vhost_rules
        .iter()
        .map(|r| EgressHttpRule {
            match_host: r.match_host.clone(),
            action_upstream: r.action_upstream.clone(),
        })
        .collect();

    ServerEgressUpstream {
        upstreams: upstream_map,
        rules: EgressRules { vhost },
    }
}
