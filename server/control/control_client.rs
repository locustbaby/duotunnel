use anyhow::Context;
use serde::{Deserialize, Serialize};
/// Maintains the canonical ctld watch stream and applies Snapshot/Delta events.
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tracing::{error, info, warn};
use tunnel_lib::ctld_proto::{
    apply_config_operations, recv_config_event, send_watch_request, snapshot_content_hash,
    ApplyResponse, ApplyStatus, ConfigEvent, ConfigSnapshot, ControlRevision, ProtoClientGroup,
    ProtoEgressUpstreamDef, ProtoEgressVhostRule, ProtoIngressListener, ProtoIngressListenerMode,
    VersionedConfigSnapshot, WatchRequest, CURRENT_CONTROL_PROTOCOL_VERSION,
};
use tunnel_lib::models::msg::{send_message, MessageType};

use crate::bootstrap::config::{
    ClientConfigs, EgressHttpRule, EgressRules, GroupConfig, HttpListenerConfig, IngressListener,
    IngressMode, IngressRouting, ServerDef, ServerEgressUpstream, TcpListenerConfig,
    TunnelManagement, UpstreamDef, VhostRule,
};
use crate::control::local_auth::{CacheEntry, TokenMap};
use crate::control::service::BackgroundService;
use crate::{RuntimeGeneration, ServerState};
use tokio_util::sync::CancellationToken;

pub struct ControlClientService {
    pub ctld_addr: SocketAddr,
    pub auth_token: Option<String>,
    pub config_path: String,
}

impl BackgroundService for ControlClientService {
    fn run(
        self: Box<Self>,
        state: Arc<ServerState>,
        shutdown: CancellationToken,
        _proxy_handle: tokio::runtime::Handle,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            watch_loop(
                self.ctld_addr,
                self.auth_token,
                self.config_path,
                state,
                shutdown,
            )
            .await;
            Ok(())
        })
    }
}

const LKG_FORMAT_VERSION: u32 = 3;
const LKG_CONTROL_PROTOCOL_VERSION: u32 = CURRENT_CONTROL_PROTOCOL_VERSION as u32;
const MAX_LKG_BYTES: u64 = 16 * 1024 * 1024;
const MAX_LKG_AGE: Duration = crate::runtime::health::CONTROL_SECURITY_STALE_AFTER;
const MAX_LKG_FUTURE_SKEW: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LastKnownGood {
    format_version: u32,
    control_protocol_version: u32,
    payload_length: u64,
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
    current_snapshot: Option<ConfigSnapshot>,
    applied: Option<AppliedControlState>,
}

fn get_snapshot_path(config_path: &str) -> PathBuf {
    let mut p = std::path::PathBuf::from(config_path);
    p.set_file_name("local_snapshot.json");
    p
}

async fn save_snapshot_to_disk(path: &Path, lkg: &LastKnownGood) -> anyhow::Result<()> {
    let bytes = encode_validated_lkg(lkg)?;
    let owned_path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Ok(previous) = std::fs::read(&owned_path) {
            if decode_lkg_bytes(&previous).is_ok() {
                atomic_write(&previous_snapshot_path(&owned_path), &previous)?;
            }
        }
        atomic_write(&owned_path, &bytes)?;
        anyhow::Ok(())
    })
    .await
    .context("LKG writer task failed")??;
    tracing::debug!(
        path = ?path,
        version = lkg.snapshot.resource_version,
        "saved snapshot to disk"
    );
    Ok(())
}

fn previous_snapshot_path(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("local_snapshot.json");
    parent.join(format!("{file_name}.previous"))
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
    let previous_path = previous_snapshot_path(path);
    let primary = load_snapshot_file(path).await;
    let previous = load_snapshot_file(&previous_path).await;
    match (primary, previous) {
        (Ok(primary), Ok(previous)) => Ok(select_newer_lkg(primary, previous)),
        (Ok(primary), Err(_)) => Ok(primary),
        (Err(_), Ok(previous)) => Ok(previous),
        (Err(primary), Err(previous)) => {
            anyhow::bail!("primary LKG invalid ({primary}); previous LKG invalid ({previous})")
        }
    }
}

async fn load_snapshot_file(path: &Path) -> anyhow::Result<LastKnownGood> {
    let metadata = tokio::fs::metadata(path).await?;
    if metadata.len() > MAX_LKG_BYTES {
        anyhow::bail!(
            "LKG exceeds maximum size: {} > {} bytes",
            metadata.len(),
            MAX_LKG_BYTES
        );
    }
    let content = tokio::fs::read(path).await?;
    if content.len() as u64 > MAX_LKG_BYTES {
        anyhow::bail!(
            "LKG exceeds maximum size: {} > {} bytes",
            content.len(),
            MAX_LKG_BYTES
        );
    }
    decode_lkg_bytes(&content)
}

fn decode_lkg_bytes(content: &[u8]) -> anyhow::Result<LastKnownGood> {
    if content.len() as u64 > MAX_LKG_BYTES {
        anyhow::bail!(
            "LKG exceeds maximum size: {} > {} bytes",
            content.len(),
            MAX_LKG_BYTES
        );
    }
    let lkg: LastKnownGood = serde_json::from_slice(content).context("failed to parse LKG")?;
    validate_lkg_envelope(&lkg)?;
    Ok(lkg)
}

fn validate_lkg_envelope(lkg: &LastKnownGood) -> anyhow::Result<()> {
    if lkg.format_version != LKG_FORMAT_VERSION {
        anyhow::bail!("unsupported LKG format version {}", lkg.format_version);
    }
    if lkg.control_protocol_version != LKG_CONTROL_PROTOCOL_VERSION {
        anyhow::bail!(
            "unsupported LKG control protocol version {}",
            lkg.control_protocol_version
        );
    }
    let payload_length = serde_json::to_vec(&lkg.snapshot)?.len() as u64;
    if payload_length != lkg.payload_length {
        anyhow::bail!("LKG payload length mismatch");
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
    validate_lkg_timestamp(lkg.generated_at_unix_ms)?;
    Ok(())
}

fn encode_validated_lkg(lkg: &LastKnownGood) -> anyhow::Result<Vec<u8>> {
    validate_lkg_envelope(lkg)?;
    let bytes = serde_json::to_vec(lkg)?;
    if bytes.len() as u64 > MAX_LKG_BYTES {
        anyhow::bail!(
            "LKG exceeds maximum size: {} > {} bytes",
            bytes.len(),
            MAX_LKG_BYTES
        );
    }
    Ok(bytes)
}

fn select_newer_lkg(primary: LastKnownGood, previous: LastKnownGood) -> LastKnownGood {
    use std::cmp::Ordering;

    let ordering = match (&primary.revision, &previous.revision) {
        (Some(left), Some(right)) if left.epoch == right.epoch => {
            left.sequence.cmp(&right.sequence)
        }
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (Some(_), Some(_)) => Ordering::Equal,
        _ => primary
            .snapshot
            .resource_version
            .cmp(&previous.snapshot.resource_version),
    }
    .then_with(|| {
        primary
            .generated_at_unix_ms
            .cmp(&previous.generated_at_unix_ms)
    });

    if ordering == Ordering::Less {
        previous
    } else {
        primary
    }
}

fn make_lkg(
    revision: Option<ControlRevision>,
    content_hash: String,
    generated_at_unix_ms: u64,
    snapshot: ConfigSnapshot,
) -> anyhow::Result<LastKnownGood> {
    let payload_length = serde_json::to_vec(&snapshot)?.len() as u64;
    Ok(LastKnownGood {
        format_version: LKG_FORMAT_VERSION,
        control_protocol_version: LKG_CONTROL_PROTOCOL_VERSION,
        payload_length,
        revision,
        content_hash,
        generated_at_unix_ms,
        snapshot,
    })
}

fn validate_lkg_timestamp(generated_at_unix_ms: u64) -> anyhow::Result<Duration> {
    let now = unix_time_ms();
    if generated_at_unix_ms > now {
        let future = Duration::from_millis(generated_at_unix_ms - now);
        if future > MAX_LKG_FUTURE_SKEW {
            anyhow::bail!("LKG timestamp is too far in the future");
        }
        return Ok(Duration::ZERO);
    }
    let age = Duration::from_millis(now - generated_at_unix_ms);
    if age >= MAX_LKG_AGE {
        anyhow::bail!("LKG exceeds maximum security age");
    }
    Ok(age)
}

async fn watch_loop(
    ctld_addr: SocketAddr,
    auth_token: Option<String>,
    config_path: String,
    state: Arc<ServerState>,
    shutdown: CancellationToken,
) {
    let snapshot_path = get_snapshot_path(&config_path);
    let mut watch_state = WatchState::default();

    if snapshot_path.exists() || previous_snapshot_path(&snapshot_path).exists() {
        match load_snapshot_from_disk(&snapshot_path).await {
            Ok(lkg) => {
                info!(
                    path = ?snapshot_path,
                    resource_version = lkg.snapshot.resource_version,
                    "loaded local snapshot fallback"
                );
                match apply_snapshot(&lkg.snapshot, &lkg.content_hash, &state).await {
                    Ok(()) => {
                        let age =
                            validate_lkg_timestamp(lkg.generated_at_unix_ms).unwrap_or(MAX_LKG_AGE);
                        state.health().restore_config_applied(age);
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
    watch_state: &mut WatchState,
    snapshot_path: &Path,
) -> anyhow::Result<()> {
    info!(addr = %addr, "connecting to tunnel-ctld");
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let applied = watch_state.applied.as_ref();
    let req = WatchRequest {
        token: auth_token.map(str::to_string),
        last_applied_revision: applied.and_then(|state| state.revision.clone()),
        last_applied_hash: applied.map(|state| state.content_hash.clone()),
    };
    send_watch_request(&mut writer, &req).await?;
    info!(addr = %addr, revision = ?req.last_applied_revision, "sent WatchRequest");

    let err = loop {
        let event = match recv_config_event(&mut reader).await {
            Ok(e) => e,
            Err(e) => break e,
        };
        match event {
            ConfigEvent::Snapshot(versioned) => {
                handle_snapshot(versioned, state, watch_state, snapshot_path, &mut writer).await?;
            }
            ConfigEvent::Delta(delta) => {
                handle_delta(delta, state, watch_state, snapshot_path, &mut writer).await?;
            }
        }
    };
    Err(err.context(format!(
        "ctld disconnected at revision {:?}",
        watch_state
            .applied
            .as_ref()
            .and_then(|state| state.revision.as_ref())
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
        return RevisionDecision::Apply;
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

async fn handle_snapshot<W>(
    versioned: VersionedConfigSnapshot,
    state: &Arc<ServerState>,
    watch_state: &mut WatchState,
    snapshot_path: &Path,
    writer: &mut W,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let actual_hash = snapshot_content_hash(&versioned.snapshot)?;
    let reject = if versioned.revision.sequence != versioned.snapshot.resource_version {
        Some("revision and snapshot version diverged".to_string())
    } else {
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
            state.health().confirm_control_freshness();
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

    state.health().begin_config_apply();
    if let Err(error) = apply_snapshot(&versioned.snapshot, &actual_hash, state).await {
        state.health().fail_config_apply();
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
    state.health().finish_config_apply();
    watch_state.current_snapshot = Some(versioned.snapshot.clone());
    watch_state.applied = Some(AppliedControlState {
        revision: Some(versioned.revision.clone()),
        content_hash: versioned.content_hash.clone(),
    });

    let lkg = make_lkg(
        Some(versioned.revision.clone()),
        versioned.content_hash.clone(),
        versioned.generated_at_unix_ms,
        versioned.snapshot.clone(),
    )?;
    if let Err(error) = save_snapshot_to_disk(snapshot_path, &lkg).await {
        warn!(error = %error, "live snapshot applied but LKG persistence degraded");
        metrics::counter!("duotunnel_control_lkg_persist_failures_total").increment(1);
    }

    send_apply_response(
        writer,
        &versioned.revision,
        &versioned.content_hash,
        ApplyStatus::Applied,
        None,
    )
    .await
}

async fn handle_delta<W>(
    delta: tunnel_lib::ctld_proto::ConfigDelta,
    state: &Arc<ServerState>,
    watch_state: &mut WatchState,
    snapshot_path: &Path,
    writer: &mut W,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let Some(current) = watch_state.current_snapshot.as_ref() else {
        send_apply_response(
            writer,
            &delta.target_revision,
            &delta.target_hash,
            ApplyStatus::ResyncRequired,
            Some("no base snapshot is applied".to_string()),
        )
        .await?;
        return Ok(());
    };
    let Some(applied) = watch_state.applied.as_ref() else {
        anyhow::bail!("current snapshot has no applied control state");
    };
    let Some(base_revision) = applied.revision.as_ref() else {
        send_apply_response(
            writer,
            &delta.target_revision,
            &delta.target_hash,
            ApplyStatus::ResyncRequired,
            Some("base snapshot has no control revision".to_string()),
        )
        .await?;
        return Ok(());
    };
    if base_revision != &delta.base_revision || applied.content_hash != delta.base_hash {
        send_apply_response(
            writer,
            &delta.target_revision,
            &delta.target_hash,
            ApplyStatus::ResyncRequired,
            Some("delta base revision or hash does not match".to_string()),
        )
        .await?;
        return Ok(());
    }
    if delta.target_revision.epoch != delta.base_revision.epoch
        || delta.target_revision.sequence <= delta.base_revision.sequence
    {
        send_apply_response(
            writer,
            &delta.target_revision,
            &delta.target_hash,
            ApplyStatus::ResyncRequired,
            Some("delta target revision is not newer than its base".to_string()),
        )
        .await?;
        return Ok(());
    }

    let mut candidate = current.clone();
    apply_config_operations(&mut candidate, &delta.operations)?;
    candidate.resource_version = delta.target_revision.sequence;
    let actual_hash = snapshot_content_hash(&candidate)?;
    if actual_hash != delta.target_hash {
        send_apply_response(
            writer,
            &delta.target_revision,
            &delta.target_hash,
            ApplyStatus::Rejected,
            Some("delta target content hash mismatch".to_string()),
        )
        .await?;
        anyhow::bail!("delta target content hash mismatch");
    }

    state.health().begin_config_apply();
    if let Err(error) = apply_snapshot(&candidate, &actual_hash, state).await {
        state.health().fail_config_apply();
        send_apply_response(
            writer,
            &delta.target_revision,
            &delta.target_hash,
            ApplyStatus::Rejected,
            Some(format!("runtime apply failed: {error}")),
        )
        .await?;
        return Err(error);
    }
    state.health().finish_config_apply();
    watch_state.current_snapshot = Some(candidate.clone());
    watch_state.applied = Some(AppliedControlState {
        revision: Some(delta.target_revision.clone()),
        content_hash: delta.target_hash.clone(),
    });
    let lkg = make_lkg(
        Some(delta.target_revision.clone()),
        delta.target_hash.clone(),
        unix_time_ms(),
        candidate,
    )?;
    if let Err(error) = save_snapshot_to_disk(snapshot_path, &lkg).await {
        warn!(error = %error, "live delta applied but LKG persistence degraded");
        metrics::counter!("duotunnel_control_lkg_persist_failures_total").increment(1);
    }
    send_apply_response(
        writer,
        &delta.target_revision,
        &delta.target_hash,
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

/// Build and publish one immutable runtime generation.
async fn apply_snapshot(
    snap: &ConfigSnapshot,
    content_hash: &str,
    state: &Arc<ServerState>,
) -> anyhow::Result<()> {
    let (listeners, generation) = build_runtime_generation(snap, content_hash, state)?;
    let previous = state.runtime_generation();
    let previous_listeners = previous.routing().ingress_listeners().to_vec();
    let _security_commit = state.security_apply_gate().write().await;
    if let Err(error) = crate::ingress::sync_all_listeners(state, &listeners).await {
        state.health().hold_config_apply_fence();
        return Err(error);
    }
    if let Err(error) =
        fence_revoked_sessions(state, previous.token_map(), generation.token_map()).await
    {
        state.health().hold_config_apply_fence();
        if let Err(rollback_error) =
            crate::ingress::sync_all_listeners(state, &previous_listeners).await
        {
            tracing::error!(error = %rollback_error, "failed to roll back listeners after token fence failure");
        }
        return Err(error);
    }
    state.publish_generation(generation);
    Ok(())
}

fn build_token_map(
    entries: &[tunnel_lib::shared::TokenCacheEntryDef],
) -> anyhow::Result<Arc<TokenMap>> {
    let mut map = TokenMap::with_capacity(entries.len());
    for entry in entries {
        let decoded = hex::decode(&entry.hash_hex)
            .with_context(|| format!("invalid token hash {}", entry.hash_hex))?;
        let hash_bytes: [u8; 32] = decoded
            .try_into()
            .map_err(|_| anyhow::anyhow!("token hash must contain exactly 32 bytes"))?;
        let cache_entry = CacheEntry {
            client_group: entry.client_group.clone(),
            client_status: entry.client_status,
            token_status: entry.token_status,
        };
        if map.insert(hash_bytes, cache_entry).is_some() {
            anyhow::bail!("duplicate token hash in runtime generation");
        }
    }
    Ok(Arc::new(map))
}

fn build_runtime_generation(
    snap: &ConfigSnapshot,
    content_hash: &str,
    state: &Arc<ServerState>,
) -> anyhow::Result<(Vec<IngressListener>, Arc<RuntimeGeneration>)> {
    let tm = proto_to_tunnel_management(&snap.ingress_listeners, &snap.client_groups);
    let egress = proto_to_server_egress(&snap.egress_upstreams, &snap.egress_vhost_rules);
    let http_params = state.http_client_params();
    let routing_snapshot = crate::build_routing_snapshot_with_health(
        &tm,
        &egress,
        &http_params,
        state.upstream_health(),
    )?;
    let listeners = tm.server_ingress_routing.listeners.clone();
    let token_map = build_token_map(&snap.token_cache)?;
    Ok((
        listeners,
        Arc::new(RuntimeGeneration::managed(
            snap.resource_version,
            Arc::<str>::from(content_hash),
            routing_snapshot,
            token_map,
        )),
    ))
}

fn token_is_active(entry: &CacheEntry) -> bool {
    entry.client_status == tunnel_store::ClientStatus::Active
        && entry.token_status == tunnel_store::TokenStatus::Active
}

async fn fence_revoked_sessions(
    state: &Arc<ServerState>,
    previous: &TokenMap,
    next: &TokenMap,
) -> anyhow::Result<()> {
    let revoked = previous
        .iter()
        .filter(|(hash, entry)| {
            token_is_active(entry)
                && next
                    .get(*hash)
                    .is_none_or(|next_entry| !token_is_active(next_entry))
        })
        .map(|(hash, _)| *hash)
        .collect();
    let fenced = state
        .registry()
        .revoke_tokens(revoked)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;
    if fenced > 0 {
        info!(fenced, "fenced sessions authenticated by revoked tokens");
    }
    Ok(())
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
            // A full Snapshot from a new control epoch is an authority reset;
            // only Delta events require the existing epoch as their base.
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
            RevisionDecision::Apply
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
        let lkg = make_lkg(
            Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 4,
            }),
            snapshot_content_hash(&snapshot).unwrap(),
            unix_time_ms(),
            snapshot,
        )
        .unwrap();

        save_snapshot_to_disk(&path, &lkg).await.unwrap();
        let loaded = load_snapshot_from_disk(&path).await.unwrap();

        assert_eq!(loaded.format_version, LKG_FORMAT_VERSION);
        assert_eq!(loaded.revision, lkg.revision);
        assert_eq!(loaded.content_hash, lkg.content_hash);
        assert_eq!(loaded.generated_at_unix_ms, lkg.generated_at_unix_ms);
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
        let mut lkg = make_lkg(
            Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 5,
            }),
            snapshot_content_hash(&snapshot).unwrap(),
            unix_time_ms(),
            snapshot,
        )
        .unwrap();
        lkg.content_hash = "not-the-content-hash".to_string();

        assert!(save_snapshot_to_disk(&path, &lkg).await.is_err());
        atomic_write(&path, &serde_json::to_vec(&lkg).unwrap()).unwrap();
        assert!(load_snapshot_from_disk(&path).await.is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn lkg_load_selects_highest_valid_revision() {
        let dir = std::env::temp_dir().join(format!(
            "duotunnel-lkg-select-test-{}-{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("local_snapshot.json");
        let primary_snapshot = empty_snapshot(6);
        let primary = make_lkg(
            Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 6,
            }),
            snapshot_content_hash(&primary_snapshot).unwrap(),
            unix_time_ms(),
            primary_snapshot,
        )
        .unwrap();
        let previous_snapshot = empty_snapshot(7);
        let previous = make_lkg(
            Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 7,
            }),
            snapshot_content_hash(&previous_snapshot).unwrap(),
            unix_time_ms(),
            previous_snapshot,
        )
        .unwrap();
        atomic_write(&path, &encode_validated_lkg(&primary).unwrap()).unwrap();
        atomic_write(
            &previous_snapshot_path(&path),
            &encode_validated_lkg(&previous).unwrap(),
        )
        .unwrap();

        let loaded = load_snapshot_from_disk(&path).await.unwrap();

        assert_eq!(loaded.revision.unwrap().sequence, 7);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn lkg_rotation_keeps_immediate_predecessor() {
        let dir = std::env::temp_dir().join(format!(
            "duotunnel-lkg-rotation-test-{}-{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("local_snapshot.json");
        let first_snapshot = empty_snapshot(8);
        let first = make_lkg(
            Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 8,
            }),
            snapshot_content_hash(&first_snapshot).unwrap(),
            unix_time_ms(),
            first_snapshot,
        )
        .unwrap();
        save_snapshot_to_disk(&path, &first).await.unwrap();
        let second_snapshot = empty_snapshot(9);
        let second = make_lkg(
            Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 9,
            }),
            snapshot_content_hash(&second_snapshot).unwrap(),
            unix_time_ms(),
            second_snapshot,
        )
        .unwrap();

        save_snapshot_to_disk(&path, &second).await.unwrap();

        let primary = load_snapshot_file(&path).await.unwrap();
        let previous = load_snapshot_file(&previous_snapshot_path(&path))
            .await
            .unwrap();
        assert_eq!(primary.revision.unwrap().sequence, 9);
        assert_eq!(previous.revision.unwrap().sequence, 8);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
