use crate::models::defs::{
    ClientGroupDef, ClientUpstreamDef, EgressUpstreamDef, EgressVhostRuleDef, IngressListenerDef,
    IngressListenerModeDef, IngressVhostRuleDef, TokenCacheEntryDef, UpstreamServerDef,
};
use crate::models::msg::{recv_message_type, send_message, MessageType, MAX_MESSAGE_BYTES};
use anyhow::{anyhow, Result};
use rkyv::{
    api::high::{HighDeserializer, HighValidator},
    bytecheck::CheckBytes,
    rancor,
    util::AlignedVec,
    Archive, Deserialize, Serialize,
};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// The high bit is reserved as a capability flag so the legacy rkyv request
// layout remains unchanged. Legacy ctld instances ignore the requested version
// and still send a full v1 Snapshot; V2 peers mask the bit before comparison.
pub const CONTROL_PROTOCOL_V2_CAPABILITY: u64 = 1 << 63;
pub const CONTROL_PROTOCOL_LEGACY_VERSION_MASK: u64 = !CONTROL_PROTOCOL_V2_CAPABILITY;

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
    pub resource_version: u64,
    pub token: Option<String>,
}

impl WatchRequest {
    pub fn advertise_v2(mut self) -> Self {
        self.resource_version = (self.resource_version & CONTROL_PROTOCOL_LEGACY_VERSION_MASK)
            | CONTROL_PROTOCOL_V2_CAPABILITY;
        self
    }

    pub fn supports_v2(&self) -> bool {
        self.resource_version & CONTROL_PROTOCOL_V2_CAPABILITY != 0
    }

    pub fn legacy_resource_version(&self) -> u64 {
        self.resource_version & CONTROL_PROTOCOL_LEGACY_VERSION_MASK
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
struct WatchRequestV1 {
    resource_version: u64,
}

impl From<WatchRequestV1> for WatchRequest {
    fn from(value: WatchRequestV1) -> Self {
        Self {
            resource_version: value.resource_version,
            token: None,
        }
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceOp<T> {
    Upsert(T),
    Delete { key: String },
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

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigPatch {
    pub resource_version: u64,
    pub ingress_listeners: Vec<ResourceOp<ProtoIngressListener>>,
    pub client_groups: Vec<ResourceOp<ProtoClientGroup>>,
    pub egress_upstreams: Vec<ResourceOp<ProtoEgressUpstreamDef>>,
    pub egress_vhost_rules: Vec<ResourceOp<ProtoEgressVhostRule>>,
    pub token_cache: Vec<ResourceOp<TokenCacheEntry>>,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub enum WatchEvent {
    Snapshot(ConfigSnapshot),
    Patch(ConfigPatch),
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

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionedConfigSnapshot {
    pub revision: ControlRevision,
    pub content_hash: String,
    pub generated_at_unix_ms: u64,
    pub snapshot: ConfigSnapshot,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub enum WatchEventV2 {
    Snapshot(VersionedConfigSnapshot),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceivedWatchEvent {
    Legacy(WatchEvent),
    V2(WatchEventV2),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplyStatus {
    Applied,
    Duplicate,
    Rejected,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyResponse {
    pub revision: ControlRevision,
    pub content_hash: String,
    pub status: ApplyStatus,
    pub reason: Option<String>,
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

async fn recv_control_payload<R>(reader: &mut R) -> Result<AlignedVec<16>>
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
    let len = reader.read_u32().await? as usize;
    if len > MAX_MESSAGE_BYTES {
        return Err(anyhow!("Message too large: {} bytes", len));
    }
    let mut buf = AlignedVec::<16>::with_capacity(len);
    buf.resize(len, 0);
    reader.read_exact(&mut buf[..]).await?;
    Ok(buf)
}

pub async fn send_watch_request<W>(writer: &mut W, req: &WatchRequest) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    if req.token.is_some() {
        send_message(writer, MessageType::ConfigPush, req).await
    } else {
        send_message(
            writer,
            MessageType::ConfigPush,
            &WatchRequestV1 {
                resource_version: req.resource_version,
            },
        )
        .await
    }
}

pub async fn recv_watch_request<R>(reader: &mut R) -> Result<WatchRequest>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader).await?;
    decode_rkyv::<WatchRequest>(&buf)
        .or_else(|_| decode_rkyv::<WatchRequestV1>(&buf).map(Into::into))
}

pub async fn recv_watch_event<R>(reader: &mut R) -> Result<ReceivedWatchEvent>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader).await?;
    decode_rkyv::<WatchEventV2>(&buf)
        .map(ReceivedWatchEvent::V2)
        .or_else(|_| decode_rkyv::<WatchEvent>(&buf).map(ReceivedWatchEvent::Legacy))
}

pub async fn recv_apply_response<R>(reader: &mut R) -> Result<ApplyResponse>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader).await?;
    decode_rkyv::<ApplyResponse>(&buf)
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
    hasher.update(b"duotunnel-control-protocol-v2\0");
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hash = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut hash, "{byte:02x}")?;
    }
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn watch_request_without_token_uses_legacy_wire_format() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let req = WatchRequest {
            resource_version: 42,
            token: None,
        };

        send_watch_request(&mut client, &req).await.unwrap();
        let decoded = recv_watch_request(&mut server).await.unwrap();

        assert_eq!(decoded.resource_version, 42);
        assert!(decoded.token.is_none());
    }

    #[tokio::test]
    async fn watch_request_with_token_uses_current_wire_format() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let req = WatchRequest {
            resource_version: 7,
            token: Some("secret".to_string()),
        };

        send_watch_request(&mut client, &req).await.unwrap();
        let decoded = recv_watch_request(&mut server).await.unwrap();

        assert_eq!(decoded.resource_version, 7);
        assert_eq!(decoded.token.as_deref(), Some("secret"));
    }

    #[test]
    fn v2_capability_preserves_legacy_resource_version() {
        let req = WatchRequest {
            resource_version: 42,
            token: None,
        }
        .advertise_v2();

        assert!(req.supports_v2());
        assert_eq!(req.legacy_resource_version(), 42);
    }

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
    fn content_hash_excludes_transport_revision() {
        assert_eq!(
            snapshot_content_hash(&empty_snapshot(1)).unwrap(),
            snapshot_content_hash(&empty_snapshot(99)).unwrap()
        );
    }

    #[tokio::test]
    async fn v2_event_is_distinct_from_legacy_wire_event() {
        let (mut sender, mut receiver) = tokio::io::duplex(2048);
        let snapshot = empty_snapshot(3);
        let event = WatchEventV2::Snapshot(VersionedConfigSnapshot {
            revision: ControlRevision {
                epoch: "epoch-a".to_string(),
                sequence: 3,
            },
            content_hash: snapshot_content_hash(&snapshot).unwrap(),
            generated_at_unix_ms: 10,
            snapshot,
        });

        send_message(&mut sender, MessageType::ConfigPush, &event)
            .await
            .unwrap();

        assert!(matches!(
            recv_watch_event(&mut receiver).await.unwrap(),
            ReceivedWatchEvent::V2(WatchEventV2::Snapshot(_))
        ));
    }

    #[tokio::test]
    async fn legacy_event_remains_decodable_by_dual_stack_receiver() {
        let (mut sender, mut receiver) = tokio::io::duplex(2048);
        let event = WatchEvent::Snapshot(empty_snapshot(2));

        send_message(&mut sender, MessageType::ConfigPush, &event)
            .await
            .unwrap();

        assert!(matches!(
            recv_watch_event(&mut receiver).await.unwrap(),
            ReceivedWatchEvent::Legacy(WatchEvent::Snapshot(_))
        ));
    }
}
