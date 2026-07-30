use crate::models::defs::{
    ClientGroupDef, ClientUpstreamDef, EgressUpstreamDef, EgressVhostRuleDef, IngressListenerDef,
    IngressListenerModeDef, IngressVhostRuleDef, TokenCacheEntryDef, UpstreamServerDef,
};
use crate::models::msg::{recv_message_type, MessageType, MAX_MESSAGE_BYTES};
use anyhow::{anyhow, Result};
use rkyv::{
    api::high::{HighDeserializer, HighSerializer, HighValidator},
    bytecheck::CheckBytes,
    rancor,
    util::AlignedVec,
    Archive, Deserialize, Serialize,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const CURRENT_CONTROL_PROTOCOL_VERSION: u16 = 3;
const CONTROL_WIRE_MAGIC: [u8; 4] = *b"DTCP";
const CONTROL_ENVELOPE_HEADER_BYTES: usize = 4 + 2 + 1 + 4;

pub type ProtoIngressListener = IngressListenerDef;
pub type ProtoIngressListenerMode = IngressListenerModeDef;
pub type ProtoIngressVhostRule = IngressVhostRuleDef;
pub type ProtoClientGroup = ClientGroupDef;
pub type ProtoClientUpstream = ClientUpstreamDef;
pub type ProtoEgressUpstreamDef = EgressUpstreamDef;
pub type ProtoEgressVhostRule = EgressVhostRuleDef;
pub type ProtoUpstreamServer = UpstreamServerDef;
pub type TokenCacheEntry = TokenCacheEntryDef;

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchRequest {
    pub token: Option<String>,
    pub last_applied_revision: Option<ControlRevision>,
    pub last_applied_hash: Option<String>,
}

#[derive(
    Debug,
    Clone,
    Archive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ConfigOperation {
    UpsertIngressListener(ProtoIngressListener),
    DeleteIngressListener { port: u16 },
    UpsertClientGroup(ProtoClientGroup),
    DeleteClientGroup { group_id: String },
    UpsertEgressUpstream(ProtoEgressUpstreamDef),
    DeleteEgressUpstream { name: String },
    UpsertEgressVhostRule(ProtoEgressVhostRule),
    DeleteEgressVhostRule { match_host: String },
    UpsertToken(TokenCacheEntry),
    DeleteToken { hash_hex: String },
}

#[derive(
    Debug,
    Clone,
    Archive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ConfigSnapshot {
    pub resource_version: u64,
    pub ingress_listeners: Vec<ProtoIngressListener>,
    pub client_groups: Vec<ProtoClientGroup>,
    pub egress_upstreams: Vec<ProtoEgressUpstreamDef>,
    pub egress_vhost_rules: Vec<ProtoEgressVhostRule>,
    pub token_cache: Vec<TokenCacheEntry>,
}

#[derive(
    Debug,
    Clone,
    Archive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ConfigDelta {
    pub base_revision: ControlRevision,
    pub base_hash: String,
    pub target_revision: ControlRevision,
    pub target_hash: String,
    pub operations: Vec<ConfigOperation>,
}

#[derive(
    Debug,
    Clone,
    Archive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ConfigEvent {
    Snapshot(VersionedConfigSnapshot),
    Delta(ConfigDelta),
}

#[derive(
    Debug,
    Clone,
    Archive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ControlRevision {
    pub epoch: String,
    pub sequence: u64,
}

#[derive(
    Debug,
    Clone,
    Archive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct VersionedConfigSnapshot {
    pub revision: ControlRevision,
    pub content_hash: String,
    pub generated_at_unix_ms: u64,
    pub snapshot: ConfigSnapshot,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplyStatus {
    Applied,
    Duplicate,
    Rejected,
    ResyncRequired,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyResponse {
    pub revision: ControlRevision,
    pub content_hash: String,
    pub status: ApplyStatus,
    pub reason: Option<String>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlMessageKind {
    WatchRequest = 1,
    ConfigEvent = 2,
    ApplyResponse = 3,
}

impl ControlMessageKind {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::WatchRequest),
            2 => Ok(Self::ConfigEvent),
            3 => Ok(Self::ApplyResponse),
            _ => Err(anyhow!("unknown control message kind: {value}")),
        }
    }
}

fn decode_rkyv<T>(buf: &[u8]) -> Result<T>
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, rancor::Error>>
        + Deserialize<T, HighDeserializer<rancor::Error>>,
{
    let archived = rkyv::access::<T::Archived, rancor::Error>(buf)
        .map_err(|e| anyhow!("rkyv access failed: {e}"))?;
    rkyv::deserialize::<T, rancor::Error>(archived)
        .map_err(|e| anyhow!("rkyv deserialize failed: {e}"))
}

fn encode_control_payload<T>(kind: ControlMessageKind, payload: &T) -> Result<Vec<u8>>
where
    T: for<'a> Serialize<
        HighSerializer<AlignedVec, rkyv::ser::allocator::ArenaHandle<'a>, rancor::Error>,
    >,
{
    let payload = rkyv::to_bytes::<rancor::Error>(payload)
        .map_err(|e| anyhow!("rkyv serialize failed: {e}"))?;
    let payload_len = u32::try_from(payload.len())
        .map_err(|_| anyhow!("control payload is too large: {} bytes", payload.len()))?;
    let frame_len = CONTROL_ENVELOPE_HEADER_BYTES
        .checked_add(payload.len())
        .ok_or_else(|| anyhow!("control envelope length overflow"))?;
    if frame_len > MAX_MESSAGE_BYTES {
        return Err(anyhow!("control message too large: {frame_len} bytes"));
    }

    let mut frame = Vec::with_capacity(frame_len);
    frame.extend_from_slice(&CONTROL_WIRE_MAGIC);
    frame.extend_from_slice(&CURRENT_CONTROL_PROTOCOL_VERSION.to_be_bytes());
    frame.push(kind as u8);
    frame.extend_from_slice(&payload_len.to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

async fn send_control_payload<W, T>(
    writer: &mut W,
    kind: ControlMessageKind,
    payload: &T,
) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
    T: for<'a> Serialize<
        HighSerializer<AlignedVec, rkyv::ser::allocator::ArenaHandle<'a>, rancor::Error>,
    >,
{
    let payload = encode_control_payload(kind, payload)?;
    let frame_len = u32::try_from(payload.len())
        .map_err(|_| anyhow!("control message is too large: {} bytes", payload.len()))?;
    writer.write_u8(MessageType::ConfigPush as u8).await?;
    writer.write_u32(frame_len).await?;
    writer.write_all(&payload).await?;
    Ok(())
}

async fn recv_control_payload<R>(
    reader: &mut R,
    expected_kind: ControlMessageKind,
) -> Result<AlignedVec<16>>
where
    R: AsyncReadExt + Unpin,
{
    let msg_type = recv_message_type(reader).await?;
    if msg_type != MessageType::ConfigPush {
        return Err(anyhow!(
            "expected {:?}, got {:?}",
            MessageType::ConfigPush,
            msg_type
        ));
    }
    let frame_len = reader.read_u32().await? as usize;
    if frame_len > MAX_MESSAGE_BYTES {
        return Err(anyhow!("Message too large: {} bytes", frame_len));
    }
    if frame_len < CONTROL_ENVELOPE_HEADER_BYTES {
        return Err(anyhow!("control envelope is truncated: {frame_len} bytes"));
    }

    let mut frame = vec![0; frame_len];
    reader.read_exact(&mut frame).await?;

    if frame[..4] != CONTROL_WIRE_MAGIC {
        return Err(anyhow!("invalid control envelope magic"));
    }
    let wire_version = u16::from_be_bytes([frame[4], frame[5]]);
    if wire_version != CURRENT_CONTROL_PROTOCOL_VERSION {
        return Err(anyhow!("unsupported control wire version: {wire_version}"));
    }
    let kind = ControlMessageKind::from_u8(frame[6])?;
    if kind != expected_kind {
        return Err(anyhow!(
            "expected control message kind {:?}, got {:?}",
            expected_kind,
            kind
        ));
    }
    let payload_len = u32::from_be_bytes([frame[7], frame[8], frame[9], frame[10]]) as usize;
    let actual_payload_len = frame_len - CONTROL_ENVELOPE_HEADER_BYTES;
    if payload_len != actual_payload_len || payload_len > MAX_MESSAGE_BYTES {
        return Err(anyhow!(
            "control payload length mismatch: declared {payload_len}, actual {actual_payload_len}"
        ));
    }

    let mut payload = AlignedVec::<16>::with_capacity(payload_len);
    payload.extend_from_slice(&frame[CONTROL_ENVELOPE_HEADER_BYTES..]);
    Ok(payload)
}

pub async fn send_watch_request<W>(writer: &mut W, req: &WatchRequest) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    send_control_payload(writer, ControlMessageKind::WatchRequest, req).await
}

pub async fn send_config_event<W>(writer: &mut W, event: &ConfigEvent) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    send_control_payload(writer, ControlMessageKind::ConfigEvent, event).await
}

pub fn config_event_wire_size(event: &ConfigEvent) -> Result<usize> {
    let encoded = encode_control_payload(ControlMessageKind::ConfigEvent, event)?;
    5usize
        .checked_add(encoded.len())
        .ok_or_else(|| anyhow!("control event wire size overflow"))
}

pub async fn send_apply_response<W>(writer: &mut W, response: &ApplyResponse) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    send_control_payload(writer, ControlMessageKind::ApplyResponse, response).await
}

pub async fn recv_watch_request<R>(reader: &mut R) -> Result<WatchRequest>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader, ControlMessageKind::WatchRequest).await?;
    decode_rkyv::<WatchRequest>(&buf)
}

pub async fn recv_config_event<R>(reader: &mut R) -> Result<ConfigEvent>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader, ControlMessageKind::ConfigEvent).await?;
    decode_rkyv::<ConfigEvent>(&buf)
}

pub async fn recv_apply_response<R>(reader: &mut R) -> Result<ApplyResponse>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader, ControlMessageKind::ApplyResponse).await?;
    decode_rkyv::<ApplyResponse>(&buf)
}

pub fn apply_config_operations(
    snapshot: &mut ConfigSnapshot,
    operations: &[ConfigOperation],
) -> Result<()> {
    let mut candidate = snapshot.clone();
    apply_config_operations_in_place(&mut candidate, operations)?;
    *snapshot = candidate;
    Ok(())
}

fn apply_config_operations_in_place(
    snapshot: &mut ConfigSnapshot,
    operations: &[ConfigOperation],
) -> Result<()> {
    for operation in operations {
        match operation {
            ConfigOperation::UpsertIngressListener(item) => {
                upsert_by_key(&mut snapshot.ingress_listeners, item.clone(), |value| {
                    value.port
                })
            }
            ConfigOperation::DeleteIngressListener { port } => {
                snapshot.ingress_listeners.retain(|item| item.port != *port)
            }
            ConfigOperation::UpsertClientGroup(item) => {
                upsert_by_key(&mut snapshot.client_groups, item.clone(), |value| {
                    value.group_id.as_str().to_owned()
                })
            }
            ConfigOperation::DeleteClientGroup { group_id } => snapshot
                .client_groups
                .retain(|item| item.group_id.as_str() != group_id),
            ConfigOperation::UpsertEgressUpstream(item) => {
                upsert_by_key(&mut snapshot.egress_upstreams, item.clone(), |value| {
                    value.name.clone()
                })
            }
            ConfigOperation::DeleteEgressUpstream { name } => {
                snapshot.egress_upstreams.retain(|item| item.name != *name)
            }
            ConfigOperation::UpsertEgressVhostRule(item) => {
                upsert_by_key(&mut snapshot.egress_vhost_rules, item.clone(), |value| {
                    value.match_host.clone()
                })
            }
            ConfigOperation::DeleteEgressVhostRule { match_host } => snapshot
                .egress_vhost_rules
                .retain(|item| item.match_host != *match_host),
            ConfigOperation::UpsertToken(item) => {
                upsert_by_key(&mut snapshot.token_cache, item.clone(), |value| {
                    value.hash_hex.clone()
                })
            }
            ConfigOperation::DeleteToken { hash_hex } => snapshot
                .token_cache
                .retain(|item| item.hash_hex != *hash_hex),
        }
    }
    validate_snapshot(snapshot)
}

fn upsert_by_key<T, K, F>(items: &mut Vec<T>, item: T, key_of: F)
where
    K: Ord,
    F: Fn(&T) -> K,
{
    let key = key_of(&item);
    if let Some(existing) = items.iter_mut().find(|existing| key_of(existing) == key) {
        *existing = item;
    } else {
        items.push(item);
    }
}

fn validate_snapshot(snapshot: &ConfigSnapshot) -> Result<()> {
    ensure_unique(
        snapshot.ingress_listeners.iter().map(|item| item.port),
        "ingress port",
    )?;
    ensure_unique(
        snapshot
            .client_groups
            .iter()
            .map(|item| item.group_id.as_str().to_owned()),
        "client group",
    )?;
    ensure_unique(
        snapshot
            .egress_upstreams
            .iter()
            .map(|item| item.name.clone()),
        "egress upstream",
    )?;
    ensure_unique(
        snapshot
            .egress_vhost_rules
            .iter()
            .map(|item| item.match_host.clone()),
        "egress vhost",
    )?;
    ensure_unique(
        snapshot
            .token_cache
            .iter()
            .map(|item| item.hash_hex.clone()),
        "token hash",
    )?;
    Ok(())
}

fn ensure_unique<T, I>(values: I, label: &str) -> Result<()>
where
    T: Ord,
    I: IntoIterator<Item = T>,
{
    let mut seen = BTreeMap::new();
    for value in values {
        if seen.insert(value, ()).is_some() {
            return Err(anyhow!("duplicate {label}"));
        }
    }
    Ok(())
}

pub fn snapshot_content_hash(snapshot: &ConfigSnapshot) -> Result<String> {
    let mut canonical = snapshot.clone();
    canonical.resource_version = 0;
    canonical
        .ingress_listeners
        .sort_by_key(|listener| listener.port);
    for listener in &mut canonical.ingress_listeners {
        if let ProtoIngressListenerMode::Http { vhost } = &mut listener.mode {
            vhost.sort_by(|left, right| left.match_host.cmp(&right.match_host));
        }
    }
    canonical
        .client_groups
        .sort_by(|left, right| left.group_id.cmp(&right.group_id));
    for group in &mut canonical.client_groups {
        group
            .upstreams
            .sort_by(|left, right| left.name.cmp(&right.name));
        for upstream in &mut group.upstreams {
            upstream.servers.sort_by(|left, right| {
                left.address
                    .cmp(&right.address)
                    .then(left.resolve.cmp(&right.resolve))
            });
        }
    }
    canonical
        .egress_upstreams
        .sort_by(|left, right| left.name.cmp(&right.name));
    for upstream in &mut canonical.egress_upstreams {
        upstream.servers.sort_by(|left, right| {
            left.address
                .cmp(&right.address)
                .then(left.resolve.cmp(&right.resolve))
        });
    }
    canonical
        .egress_vhost_rules
        .sort_by(|left, right| left.match_host.cmp(&right.match_host));
    canonical
        .token_cache
        .sort_by(|left, right| left.hash_hex.cmp(&right.hash_hex));

    let bytes = serde_json::to_vec(&canonical)?;
    let mut hasher = Sha256::new();
    hasher.update(b"duotunnel-control-protocol-v3\0");
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hash = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut hash, "{byte:02x}")?;
    }
    Ok(hash)
}

pub fn validate_config_snapshot(snapshot: &ConfigSnapshot) -> Result<()> {
    let group_ids = snapshot
        .client_groups
        .iter()
        .map(|group| group.group_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    if group_ids.len() != snapshot.client_groups.len()
        || group_ids.iter().any(|group_id| group_id.trim().is_empty())
    {
        anyhow::bail!("snapshot contains duplicate or empty client group ids");
    }

    let mut listener_ports = std::collections::HashSet::new();
    for listener in &snapshot.ingress_listeners {
        if !listener_ports.insert(listener.port) {
            anyhow::bail!(
                "snapshot contains duplicate ingress listener port {}",
                listener.port
            );
        }
        match &listener.mode {
            IngressListenerModeDef::Http { vhost } => {
                let mut hosts = std::collections::HashSet::new();
                for rule in vhost {
                    if !group_ids.contains(rule.group_id.as_str())
                        || rule.match_host.trim().is_empty()
                        || !hosts.insert(rule.match_host.as_str())
                    {
                        anyhow::bail!("snapshot contains invalid ingress vhost rule");
                    }
                }
            }
            IngressListenerModeDef::Tcp { group_id, .. } => {
                if !group_ids.contains(group_id.as_str()) {
                    anyhow::bail!("snapshot ingress listener references unknown client group");
                }
            }
        }
    }

    for group in &snapshot.client_groups {
        let mut upstream_names = std::collections::HashSet::new();
        for upstream in &group.upstreams {
            if upstream.name.trim().is_empty() || !upstream_names.insert(upstream.name.as_str()) {
                anyhow::bail!("snapshot contains duplicate or empty client upstream names");
            }
            if upstream
                .servers
                .iter()
                .any(|server| server.address.trim().is_empty())
            {
                anyhow::bail!("snapshot contains an empty client upstream server address");
            }
        }
    }

    let upstream_names = snapshot
        .egress_upstreams
        .iter()
        .map(|upstream| upstream.name.as_str())
        .collect::<std::collections::HashSet<_>>();
    if upstream_names.len() != snapshot.egress_upstreams.len()
        || upstream_names.iter().any(|name| name.trim().is_empty())
        || snapshot.egress_upstreams.iter().any(|upstream| {
            upstream
                .servers
                .iter()
                .any(|server| server.address.trim().is_empty())
        })
    {
        anyhow::bail!("snapshot contains invalid egress upstreams");
    }

    let mut vhost_names = std::collections::HashSet::new();
    for rule in &snapshot.egress_vhost_rules {
        if rule.match_host.trim().is_empty()
            || !vhost_names.insert(rule.match_host.as_str())
            || !upstream_names.contains(rule.action_upstream.as_str())
        {
            anyhow::bail!("snapshot contains invalid egress vhost rule");
        }
    }

    let mut token_hashes = std::collections::HashSet::new();
    for token in &snapshot.token_cache {
        if token.hash_hex.trim().is_empty() || !token_hashes.insert(token.hash_hex.as_str()) {
            anyhow::bail!("snapshot contains invalid token cache entries");
        }
    }
    Ok(())
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

    #[tokio::test]
    async fn watch_request_always_uses_canonical_wire_format() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let req = WatchRequest {
            last_applied_revision: Some(ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 42,
            }),
            last_applied_hash: Some("hash".to_string()),
            token: None,
        };

        send_watch_request(&mut client, &req).await.unwrap();
        assert_eq!(recv_watch_request(&mut server).await.unwrap(), req);
    }

    #[tokio::test]
    async fn control_event_and_response_helpers_use_distinct_message_kinds() {
        let (mut event_client, mut event_server) = tokio::io::duplex(4096);
        let event = ConfigEvent::Snapshot(VersionedConfigSnapshot {
            revision: ControlRevision {
                epoch: "epoch-a".into(),
                sequence: 1,
            },
            content_hash: "hash".into(),
            generated_at_unix_ms: 0,
            snapshot: empty_snapshot(1),
        });
        send_config_event(&mut event_client, &event).await.unwrap();
        assert_eq!(recv_config_event(&mut event_server).await.unwrap(), event);

        let (mut response_client, mut response_server) = tokio::io::duplex(4096);
        let response = ApplyResponse {
            revision: ControlRevision {
                epoch: "epoch-a".into(),
                sequence: 1,
            },
            content_hash: "hash".into(),
            status: ApplyStatus::Applied,
            reason: None,
        };
        send_apply_response(&mut response_client, &response)
            .await
            .unwrap();
        assert_eq!(
            recv_apply_response(&mut response_server).await.unwrap(),
            response
        );
    }

    #[tokio::test]
    async fn control_envelope_rejects_unsupported_wire_version_before_decode() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let frame = [
            CONTROL_WIRE_MAGIC.as_slice(),
            2u16.to_be_bytes().as_slice(),
            &[ControlMessageKind::WatchRequest as u8],
            0u32.to_be_bytes().as_slice(),
        ]
        .concat();
        client
            .write_u8(MessageType::ConfigPush as u8)
            .await
            .unwrap();
        client.write_u32(frame.len() as u32).await.unwrap();
        client.write_all(&frame).await.unwrap();

        let error = recv_watch_request(&mut server).await.unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported control wire version"));
    }

    #[tokio::test]
    async fn control_envelope_rejects_payload_length_mismatch_before_decode() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let mut frame = Vec::from(CONTROL_WIRE_MAGIC);
        frame.extend_from_slice(&CURRENT_CONTROL_PROTOCOL_VERSION.to_be_bytes());
        frame.push(ControlMessageKind::WatchRequest as u8);
        frame.extend_from_slice(&1u32.to_be_bytes());
        client
            .write_u8(MessageType::ConfigPush as u8)
            .await
            .unwrap();
        client.write_u32(frame.len() as u32).await.unwrap();
        client.write_all(&frame).await.unwrap();

        let error = recv_watch_request(&mut server).await.unwrap_err();
        assert!(error
            .to_string()
            .contains("control payload length mismatch"));
    }

    #[test]
    fn content_hash_excludes_transport_revision_and_is_order_independent() {
        let mut first = empty_snapshot(1);
        let mut second = empty_snapshot(99);
        first.egress_vhost_rules = vec![
            EgressVhostRuleDef {
                match_host: "b.example".into(),
                action_upstream: "b".into(),
            },
            EgressVhostRuleDef {
                match_host: "a.example".into(),
                action_upstream: "a".into(),
            },
        ];
        second.egress_vhost_rules = first.egress_vhost_rules.iter().rev().cloned().collect();
        assert_eq!(
            snapshot_content_hash(&first).unwrap(),
            snapshot_content_hash(&second).unwrap()
        );
    }

    #[test]
    fn operations_are_applied_as_one_validated_batch() {
        let mut snapshot = empty_snapshot(1);
        apply_config_operations(
            &mut snapshot,
            &[ConfigOperation::UpsertEgressUpstream(EgressUpstreamDef {
                name: "api".into(),
                lb_policy: "round_robin".into(),
                servers: vec![],
            })],
        )
        .unwrap();
        assert_eq!(snapshot.egress_upstreams[0].name, "api");
    }

    #[test]
    fn invalid_operation_batch_does_not_mutate_input() {
        let mut snapshot = empty_snapshot(1);
        snapshot.egress_upstreams = vec![
            EgressUpstreamDef {
                name: "api".into(),
                lb_policy: "round_robin".into(),
                servers: vec![],
            },
            EgressUpstreamDef {
                name: "api".into(),
                lb_policy: "least_conn".into(),
                servers: vec![],
            },
        ];
        let original = snapshot.clone();
        let result = apply_config_operations(
            &mut snapshot,
            &[ConfigOperation::DeleteEgressUpstream {
                name: "missing".into(),
            }],
        );
        assert!(result.is_err());
        assert_eq!(snapshot, original);
    }
}
