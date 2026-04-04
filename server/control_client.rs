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
use tunnel_lib::models::msg::{recv_message, recv_message_type, send_message, MessageType};
use tunnel_lib::ctld_proto::{
    ConfigSnapshot, ProtoClientGroup, ProtoEgressUpstreamDef, ProtoEgressVhostRule,
    ProtoIngressListener, ProtoIngressListenerMode, WatchEvent, WatchRequest,
};

use crate::config::{
    ClientConfigs, EgressHttpRule, EgressRules, GroupConfig, HttpListenerConfig, IngressListener,
    IngressMode, IngressRouting, ServerDef, ServerEgressUpstream, TcpListenerConfig,
    TunnelManagement, UpstreamDef, VhostRule,
};
use crate::local_auth::CacheEntry;
use crate::{build_routing_snapshot, ServerState};

/// Spawn the control client watch loop as a background task.
pub fn spawn_control_client(ctld_addr: SocketAddr, state: Arc<ServerState>) {
    tokio::spawn(async move {
        let mut backoff = Duration::from_secs(1);
        let mut last_version: u64 = 0;
        loop {
            match connect_and_watch(ctld_addr, &state, last_version).await {
                Ok(version) => {
                    last_version = version;
                    backoff = Duration::from_secs(1);
                }
                Err(e) => {
                    error!(error = %e, addr = %ctld_addr, "ctld watch connection failed");
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(Duration::from_secs(30));
                }
            }
        }
    });
}

async fn connect_and_watch(
    addr: SocketAddr,
    state: &Arc<ServerState>,
    last_version: u64,
) -> anyhow::Result<u64> {
    info!(addr = %addr, "connecting to tunnel-ctld");
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Step 1: send WatchRequest
    let req = WatchRequest { resource_version: last_version };
    send_message(&mut writer, MessageType::ConfigPush, &req).await?;
    info!(addr = %addr, resource_version = last_version, "sent WatchRequest");

    // Step 2+3: receive Snapshot then stream Patches until error/disconnect.
    // Returns the last seen resource_version so the caller can reconnect with it.
    let mut last_seen: u64 = 0;
    let err = loop {
        let msg_type = match recv_message_type(&mut reader).await {
            Ok(t) => t,
            Err(e) => break e,
        };
        if msg_type != MessageType::ConfigPush {
            break anyhow::anyhow!("expected ConfigPush, got {:?}", msg_type);
        }
        let event: WatchEvent = match recv_message(&mut reader).await {
            Ok(e) => e,
            Err(e) => break e,
        };
        last_seen = match event {
            WatchEvent::Snapshot(snap) => {
                let v = snap.resource_version;
                info!(resource_version = v, "received Snapshot from ctld");
                apply_snapshot(snap, state);
                v
            }
            WatchEvent::Patch(snap) => {
                let v = snap.resource_version;
                info!(resource_version = v, "received Patch from ctld");
                apply_snapshot(snap, state);
                v
            }
        };
    };
    // Return the last version we successfully applied so the reconnect sends
    // resource_version=N instead of 0, allowing future delta optimisation.
    Err(err.context(format!("ctld disconnected at version {}", last_seen)))
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
                ProtoIngressListenerMode::Tcp { group_id, proxy_name } => {
                    IngressMode::Tcp(TcpListenerConfig {
                        client_group: group_id.clone(),
                        proxy_name: proxy_name.clone(),
                    })
                }
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
                                .map(|s| ServerDef { address: s.address.clone(), resolve: s.resolve })
                                .collect(),
                            lb_policy: u.lb_policy.clone(),
                        },
                    )
                })
                .collect();
            (
                g.group_id.clone(),
                GroupConfig { config_version: g.config_version.clone(), upstreams },
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
                        .map(|s| ServerDef { address: s.address.clone(), resolve: s.resolve })
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

    ServerEgressUpstream { upstreams: upstream_map, rules: EgressRules { vhost } }
}
