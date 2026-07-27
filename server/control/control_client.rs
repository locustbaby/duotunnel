use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
/// ControlClient: connects to tunnel-ctld and maintains the list-watch stream.
///
/// On connect:
///   1. Sends WatchRequest { resource_version: last_known } over ConfigPush framing
///   2. Receives WatchEvent::Snapshot → applies full state (routing + token cache)
///   3. Loops receiving WatchEvent::Patch → applies incremental updates
///   4. On disconnect: exponential back-off, then reconnect
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tracing::{error, info, warn};
use tunnel_lib::ctld_proto::{
    recv_watch_event, send_watch_request, snapshot_content_hash, ApplyResponse, ApplyStatus,
    ConfigPatch, ConfigSnapshot, ControlRevision, ProtoClientGroup, ProtoEgressUpstreamDef,
    ProtoEgressVhostRule, ProtoIngressListener, ProtoIngressListenerMode, ReceivedWatchEvent,
    ResourceOp, VersionedConfigSnapshot, WatchEvent, WatchEventV2, WatchRequest,
};
use tunnel_lib::models::msg::{send_message, MessageType};

use crate::bootstrap::config::{
    ClientConfigs, EgressHttpRule, EgressRules, GroupConfig, HttpListenerConfig, IngressListener,
    IngressMode, IngressRouting, ServerDef, ServerEgressUpstream, TcpListenerConfig,
    TunnelManagement, UpstreamDef, VhostRule,
};
use crate::control::local_auth::CacheEntry;
use crate::control::service::BackgroundService;
use crate::{build_routing_snapshot, ServerState};
use tokio_util::sync::CancellationToken;

pub struct ControlClientService {
    pub ctld_addr: SocketAddr,
    pub auth_token: Option<String>,
    pub config_path: String,
}

impl BackgroundService for ControlClientService {
    fn name(&self) -> &'static str {
        "control-client"
    }

    fn run(
        self: Box<Self>,
        state: Arc<ServerState>,
        ready: Arc<std::sync::atomic::AtomicBool>,
        shutdown: CancellationToken,
        _proxy_handle: tokio::runtime::Handle,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            watch_loop(
                self.ctld_addr,
                self.auth_token,
                self.config_path,
                state,
                ready,
                shutdown,
            )
            .await;
            Ok(())
        })
    }
}

const LKG_FORMAT_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LastKnownGood {
    format_version: u32,
    revision: Option<ControlRevision>,
    content_hash: String,
    generated_at_unix_ms: u64,
    snapshot: ConfigSnapshot,
}

#[derive(Debug, Clone)]
struct AppliedControlState {
    revision: Option<ControlRevision>,
    content_hash: String,
}

#[derive(Default)]
struct WatchState {
    last_version: u64,
    current_snapshot: Option<ConfigSnapshot>,
    applied: Option<AppliedControlState>,
}

fn get_snapshot_path(config_path: &str) -> PathBuf {
    let mut p = std::path::PathBuf::from(config_path);
    p.set_file_name("local_snapshot.json");
    p
}

async fn save_snapshot_to_disk(path: &Path, lkg: &LastKnownGood) -> anyhow::Result<()> {
    let bytes = serde_json::to_vec(lkg)?;
    let owned_path = path.to_path_buf();
    tokio::task::spawn_blocking(move || atomic_write(&owned_path, &bytes))
        .await
        .context("LKG writer task failed")??;
    tracing::debug!(
        path = ?path,
        version = lkg.snapshot.resource_version,
        "saved snapshot to disk"
    );
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    use std::io::Write;

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("local_snapshot.json");
    let temp_path = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        fastrand::u64(..)
    ));
    let result = (|| -> anyhow::Result<()> {
        #[cfg(unix)]
        use std::os::unix::fs::OpenOptionsExt;

        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options.open(&temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temp_path, path)?;
        #[cfg(unix)]
        std::fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }
    result
}

async fn load_snapshot_from_disk(path: &Path) -> anyhow::Result<LastKnownGood> {
    let content = tokio::fs::read(path).await?;
    if let Ok(lkg) = serde_json::from_slice::<LastKnownGood>(&content) {
        if lkg.format_version != LKG_FORMAT_VERSION {
            anyhow::bail!("unsupported LKG format version {}", lkg.format_version);
        }
        let actual_hash = snapshot_content_hash(&lkg.snapshot)?;
        if actual_hash != lkg.content_hash {
            anyhow::bail!("LKG content hash mismatch");
        }
        if let Some(revision) = &lkg.revision {
            if revision.sequence != lkg.snapshot.resource_version {
                anyhow::bail!("LKG revision and snapshot version diverged");
            }
        }
        return Ok(lkg);
    }

    let snapshot: ConfigSnapshot =
        serde_json::from_slice(&content).context("failed to parse LKG")?;
    Ok(LastKnownGood {
        format_version: LKG_FORMAT_VERSION,
        revision: None,
        content_hash: snapshot_content_hash(&snapshot)?,
        generated_at_unix_ms: 0,
        snapshot,
    })
}

async fn watch_loop(
    ctld_addr: SocketAddr,
    auth_token: Option<String>,
    config_path: String,
    state: Arc<ServerState>,
    ready: Arc<std::sync::atomic::AtomicBool>,
    shutdown: CancellationToken,
) {
    let snapshot_path = get_snapshot_path(&config_path);
    let mut watch_state = WatchState::default();

    if snapshot_path.exists() {
        match load_snapshot_from_disk(&snapshot_path).await {
            Ok(lkg) => {
                info!(
                    path = ?snapshot_path,
                    resource_version = lkg.snapshot.resource_version,
                    "loaded local snapshot fallback"
                );
                match apply_snapshot(&lkg.snapshot, &state).await {
                    Ok(()) => {
                        ready.store(true, Ordering::Release);
                        watch_state.last_version = lkg.snapshot.resource_version;
                        watch_state.applied = Some(AppliedControlState {
                            revision: lkg.revision,
                            content_hash: lkg.content_hash,
                        });
                        watch_state.current_snapshot = Some(lkg.snapshot);
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to apply local snapshot fallback");
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "failed to load local snapshot fallback");
            }
        }
    }

    let mut backoff = Duration::from_secs(1);
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => return,
            result = connect_and_watch(
                ctld_addr,
                auth_token.as_deref(),
                &state,
                &ready,
                &mut watch_state,
                &snapshot_path,
            ) => {
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
    ready: &Arc<std::sync::atomic::AtomicBool>,
    watch_state: &mut WatchState,
    snapshot_path: &Path,
) -> anyhow::Result<()> {
    info!(addr = %addr, "connecting to tunnel-ctld");
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Step 1: send WatchRequest
    let req = WatchRequest {
        resource_version: watch_state.last_version,
        token: auth_token.map(str::to_string),
    }
    .advertise_v2();
    send_watch_request(&mut writer, &req).await?;
    info!(addr = %addr, resource_version = watch_state.last_version, "sent WatchRequest");

    // Step 2+3: receive Snapshot then stream Patches until error/disconnect.
    // Returns the last seen resource_version so the caller can reconnect with it.
    let err = loop {
        let event = match recv_watch_event(&mut reader).await {
            Ok(e) => e,
            Err(e) => break e,
        };
        match event {
            ReceivedWatchEvent::Legacy(WatchEvent::Snapshot(snap)) => {
                let v = snap.resource_version;
                info!(resource_version = v, "received Snapshot from ctld");
                apply_snapshot(&snap, state).await?;
                ready.store(true, Ordering::Release);
                watch_state.last_version = v;
                let lkg = LastKnownGood {
                    format_version: LKG_FORMAT_VERSION,
                    revision: None,
                    content_hash: snapshot_content_hash(&snap)?,
                    generated_at_unix_ms: unix_time_ms(),
                    snapshot: snap.clone(),
                };
                save_snapshot_to_disk(snapshot_path, &lkg).await?;
                watch_state.applied = Some(AppliedControlState {
                    revision: None,
                    content_hash: lkg.content_hash,
                });
                watch_state.current_snapshot = Some(snap);
            }
            ReceivedWatchEvent::Legacy(WatchEvent::Patch(patch)) => {
                let v = patch.resource_version;
                info!(resource_version = v, "received Patch from ctld");
                if let Some(snapshot) = watch_state.current_snapshot.as_mut() {
                    let affected_ports = apply_patch_to_snapshot(snapshot, &patch);
                    apply_patch_to_runtime(snapshot, &patch, &affected_ports, state).await?;
                    watch_state.last_version = v;
                    let lkg = LastKnownGood {
                        format_version: LKG_FORMAT_VERSION,
                        revision: None,
                        content_hash: snapshot_content_hash(snapshot)?,
                        generated_at_unix_ms: unix_time_ms(),
                        snapshot: snapshot.clone(),
                    };
                    save_snapshot_to_disk(snapshot_path, &lkg).await?;
                    watch_state.applied = Some(AppliedControlState {
                        revision: None,
                        content_hash: lkg.content_hash,
                    });
                } else {
                    anyhow::bail!("received Patch before Snapshot at version {v}");
                }
            }
            ReceivedWatchEvent::V2(WatchEventV2::Snapshot(versioned)) => {
                handle_v2_snapshot(
                    versioned,
                    state,
                    ready,
                    watch_state,
                    snapshot_path,
                    &mut writer,
                )
                .await?;
            }
        }
    };
    // Return the last version we successfully applied so the reconnect sends
    // resource_version=N instead of 0, allowing future delta optimisation.
    Err(err.context(format!(
        "ctld disconnected at version {}",
        watch_state.last_version
    )))
}

enum RevisionDecision {
    Apply,
    Duplicate,
    Reject(String),
}

fn classify_revision(
    applied: Option<&AppliedControlState>,
    incoming: &ControlRevision,
    content_hash: &str,
) -> RevisionDecision {
    let Some(applied) = applied else {
        return RevisionDecision::Apply;
    };
    let Some(current) = applied.revision.as_ref() else {
        return RevisionDecision::Apply;
    };
    if current.epoch != incoming.epoch {
        return if incoming.epoch > current.epoch {
            RevisionDecision::Apply
        } else {
            RevisionDecision::Reject(format!(
                "epoch rollback: incoming={} current={}",
                incoming.epoch, current.epoch
            ))
        };
    }
    if incoming.sequence < current.sequence {
        return RevisionDecision::Reject(format!(
            "revision rollback: incoming={} current={}",
            incoming.sequence, current.sequence
        ));
    }
    if incoming.sequence == current.sequence {
        if applied.content_hash == content_hash {
            return RevisionDecision::Duplicate;
        }
        return RevisionDecision::Reject(
            "equal revision carries different content hash".to_string(),
        );
    }
    RevisionDecision::Apply
}

async fn handle_v2_snapshot<W>(
    versioned: VersionedConfigSnapshot,
    state: &Arc<ServerState>,
    ready: &Arc<std::sync::atomic::AtomicBool>,
    watch_state: &mut WatchState,
    snapshot_path: &Path,
    writer: &mut W,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let reject = if versioned.revision.sequence != versioned.snapshot.resource_version {
        Some("revision and snapshot version diverged".to_string())
    } else {
        let actual_hash = snapshot_content_hash(&versioned.snapshot)?;
        (actual_hash != versioned.content_hash).then(|| "content hash mismatch".to_string())
    };
    if let Some(reason) = reject {
        send_apply_response(
            writer,
            &versioned.revision,
            &versioned.content_hash,
            ApplyStatus::Rejected,
            Some(reason.clone()),
        )
        .await?;
        anyhow::bail!(reason);
    }

    match classify_revision(
        watch_state.applied.as_ref(),
        &versioned.revision,
        &versioned.content_hash,
    ) {
        RevisionDecision::Duplicate => {
            watch_state.last_version = versioned.revision.sequence;
            send_apply_response(
                writer,
                &versioned.revision,
                &versioned.content_hash,
                ApplyStatus::Duplicate,
                None,
            )
            .await?;
            return Ok(());
        }
        RevisionDecision::Reject(reason) => {
            send_apply_response(
                writer,
                &versioned.revision,
                &versioned.content_hash,
                ApplyStatus::Rejected,
                Some(reason.clone()),
            )
            .await?;
            anyhow::bail!(reason);
        }
        RevisionDecision::Apply => {}
    }

    if let Err(error) = apply_snapshot(&versioned.snapshot, state).await {
        let reason = format!("runtime apply failed: {error}");
        send_apply_response(
            writer,
            &versioned.revision,
            &versioned.content_hash,
            ApplyStatus::Rejected,
            Some(reason),
        )
        .await?;
        return Err(error);
    }

    let lkg = LastKnownGood {
        format_version: LKG_FORMAT_VERSION,
        revision: Some(versioned.revision.clone()),
        content_hash: versioned.content_hash.clone(),
        generated_at_unix_ms: versioned.generated_at_unix_ms,
        snapshot: versioned.snapshot.clone(),
    };
    if let Err(error) = save_snapshot_to_disk(snapshot_path, &lkg).await {
        let reason = format!("LKG persist failed: {error}");
        send_apply_response(
            writer,
            &versioned.revision,
            &versioned.content_hash,
            ApplyStatus::Rejected,
            Some(reason),
        )
        .await?;
        return Err(error);
    }

    ready.store(true, Ordering::Release);
    watch_state.last_version = versioned.revision.sequence;
    watch_state.current_snapshot = Some(versioned.snapshot);
    watch_state.applied = Some(AppliedControlState {
        revision: Some(versioned.revision.clone()),
        content_hash: versioned.content_hash.clone(),
    });
    send_apply_response(
        writer,
        &versioned.revision,
        &versioned.content_hash,
        ApplyStatus::Applied,
        None,
    )
    .await
}

async fn send_apply_response<W>(
    writer: &mut W,
    revision: &ControlRevision,
    content_hash: &str,
    status: ApplyStatus,
    reason: Option<String>,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let response = ApplyResponse {
        revision: revision.clone(),
        content_hash: content_hash.to_string(),
        status,
        reason,
    };
    send_message(writer, MessageType::ConfigPush, &response).await?;
    writer.flush().await?;
    Ok(())
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// Apply a ConfigSnapshot to both the routing ArcSwap and the token cache.
async fn apply_snapshot(snap: &ConfigSnapshot, state: &Arc<ServerState>) -> anyhow::Result<()> {
    let (listeners, routing_snapshot) = build_runtime_snapshot(snap, state)?;
    update_token_cache(&snap.token_cache, state);
    state.replace_routing(routing_snapshot);
    crate::ingress::sync_all_listeners(state, &listeners).await;
    Ok(())
}

fn update_token_cache(
    entries: &[tunnel_lib::shared::TokenCacheEntryDef],
    state: &Arc<ServerState>,
) {
    if let Some(cache) = state.local_token_cache() {
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
) -> anyhow::Result<(Vec<IngressListener>, crate::RoutingSnapshot)> {
    let tm = proto_to_tunnel_management(&snap.ingress_listeners, &snap.client_groups);
    let egress = proto_to_server_egress(&snap.egress_upstreams, &snap.egress_vhost_rules);
    let http_params = state.http_client_params();
    let routing_snapshot = build_routing_snapshot(&tm, &egress, &http_params)?;
    let listeners = tm.server_ingress_routing.listeners.clone();
    Ok((listeners, routing_snapshot))
}

async fn apply_patch_to_runtime(
    snapshot: &ConfigSnapshot,
    patch: &ConfigPatch,
    affected_ports: &HashSet<u16>,
    state: &Arc<ServerState>,
) -> anyhow::Result<()> {
    let touches_routing = !patch.ingress_listeners.is_empty()
        || !patch.client_groups.is_empty()
        || !patch.egress_upstreams.is_empty()
        || !patch.egress_vhost_rules.is_empty();
    if !touches_routing {
        update_token_cache(&snapshot.token_cache, state);
        return Ok(());
    }
    let (listeners, routing_snapshot) = build_runtime_snapshot(snapshot, state)?;
    update_token_cache(&snapshot.token_cache, state);
    state.replace_routing(routing_snapshot);
    if !patch.ingress_listeners.is_empty() {
        crate::ingress::sync_listener_subset(state, &listeners, affected_ports).await;
    }
    Ok(())
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
        item.group_id.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_snapshot(resource_version: u64) -> ConfigSnapshot {
        ConfigSnapshot {
            resource_version,
            ingress_listeners: Vec::new(),
            client_groups: Vec::new(),
            egress_upstreams: Vec::new(),
            egress_vhost_rules: Vec::new(),
            token_cache: Vec::new(),
        }
    }

    #[test]
    fn revision_validation_rejects_rollback_and_split_brain() {
        let current = AppliedControlState {
            revision: Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 8,
            }),
            content_hash: "hash-a".to_string(),
        };

        assert!(matches!(
            classify_revision(
                Some(&current),
                &ControlRevision {
                    epoch: "epoch-a".to_string(),
                    sequence: 7,
                },
                "hash-a"
            ),
            RevisionDecision::Reject(_)
        ));
        assert!(matches!(
            classify_revision(
                Some(&current),
                &ControlRevision {
                    epoch: "epoch-a".to_string(),
                    sequence: 8,
                },
                "hash-b"
            ),
            RevisionDecision::Reject(_)
        ));
        assert!(matches!(
            classify_revision(
                Some(&current),
                &ControlRevision {
                    epoch: "epoch-a".to_string(),
                    sequence: 8,
                },
                "hash-a"
            ),
            RevisionDecision::Duplicate
        ));
        assert!(matches!(
            classify_revision(
                Some(&current),
                &ControlRevision {
                    epoch: "epoch-0".to_string(),
                    sequence: 999,
                },
                "hash-a"
            ),
            RevisionDecision::Reject(_)
        ));
        assert!(matches!(
            classify_revision(
                Some(&current),
                &ControlRevision {
                    epoch: "epoch-z".to_string(),
                    sequence: 1,
                },
                "hash-b"
            ),
            RevisionDecision::Apply
        ));
    }

    #[tokio::test]
    async fn lkg_round_trip_verifies_hash_and_uses_envelope() {
        let dir = std::env::temp_dir().join(format!(
            "duotunnel-lkg-test-{}-{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("local_snapshot.json");
        let snapshot = empty_snapshot(4);
        let lkg = LastKnownGood {
            format_version: LKG_FORMAT_VERSION,
            revision: Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 4,
            }),
            content_hash: snapshot_content_hash(&snapshot).unwrap(),
            generated_at_unix_ms: 123,
            snapshot,
        };

        save_snapshot_to_disk(&path, &lkg).await.unwrap();
        let loaded = load_snapshot_from_disk(&path).await.unwrap();

        assert_eq!(loaded.format_version, LKG_FORMAT_VERSION);
        assert_eq!(loaded.revision, lkg.revision);
        assert_eq!(loaded.content_hash, lkg.content_hash);
        assert_eq!(loaded.generated_at_unix_ms, 123);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn lkg_hash_corruption_is_rejected() {
        let dir = std::env::temp_dir().join(format!(
            "duotunnel-lkg-corrupt-test-{}-{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("local_snapshot.json");
        let snapshot = empty_snapshot(5);
        let lkg = LastKnownGood {
            format_version: LKG_FORMAT_VERSION,
            revision: Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 5,
            }),
            content_hash: "not-the-content-hash".to_string(),
            generated_at_unix_ms: 123,
            snapshot,
        };

        save_snapshot_to_disk(&path, &lkg).await.unwrap();
        assert!(load_snapshot_from_disk(&path).await.is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
