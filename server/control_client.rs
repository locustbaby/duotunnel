/// ControlClient: connects to tunnel-ctld and maintains the list-watch stream.
///
/// On connect:
///   1. Sends WatchRequest { resource_version: last_known } over ConfigPush framing
///   2. Receives WatchEvent::Snapshot → applies full state (routing + token cache)
///   3. Loops receiving WatchEvent::Patch → applies incremental updates
///   4. On disconnect: exponential back-off, then reconnect
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tracing::{error, info, warn};
use tunnel_lib::ctld_proto::{
    ConfigSnapshot, ProtoClientGroup, ProtoEgressUpstreamDef, ProtoEgressVhostRule,
    ProtoIngressListener, ProtoIngressListenerMode, WatchEvent, WatchRequest,
};
use tunnel_lib::models::msg::{recv_typed_message, send_message, MessageType};

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
                        tokio::select! {
                            _ = shutdown.cancelled() => return,
                            _ = tokio::time::sleep(backoff) => {}
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

    // Step 1: send WatchRequest
    let req = WatchRequest {
        resource_version: *last_version,
        token: auth_token.map(str::to_string),
    };
    send_message(&mut writer, MessageType::ConfigPush, &req).await?;
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
                apply_snapshot(snap, state);
                *last_version = v;
            }
            WatchEvent::Patch(snap) => {
                let v = snap.resource_version;
                info!(resource_version = v, "received Patch from ctld");
                apply_snapshot(snap, state);
                *last_version = v;
            }
        }
    };
    // Return the last version we successfully applied so the reconnect sends
    // resource_version=N instead of 0, allowing future delta optimisation.
    Err(err.context(format!("ctld disconnected at version {}", *last_version)))
}

/// Apply a ConfigSnapshot to both the routing ArcSwap and the token cache.
fn apply_snapshot(snap: ConfigSnapshot, state: &Arc<ServerState>) {
    // Update token cache
    if let Some(cache) = state.local_token_cache.as_ref() {
        let entries: Vec<CacheEntry> = snap
            .token_cache
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
                    client_status: e.client_status.clone(),
                    token_status: e.token_status.clone(),
                })
            })
            .collect();
        cache.update(entries);
    }

    // Convert proto routing types → server config types, then build snapshot
    let tm = proto_to_tunnel_management(&snap.ingress_listeners, &snap.client_groups);
    let egress = proto_to_server_egress(&snap.egress_upstreams, &snap.egress_vhost_rules);
    let http_params = tunnel_lib::HttpClientParams::from(&state.config.server.http_pool);
    let routing_snapshot = build_routing_snapshot(&tm, &egress, &http_params);

    // Sync listeners BEFORE swapping the routing table: a listener that hasn't
    // started yet returns 503 (recoverable), while a route pointing to a port
    // with no listener silently drops the connection (unrecoverable from client POV).
    let listeners = tm.server_ingress_routing.listeners.clone();
    crate::sync_listeners(state, &listeners);
    state.routing.store(Arc::new(routing_snapshot));
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
