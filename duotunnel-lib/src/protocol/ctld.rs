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
use std::collections::BTreeMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const CURRENT_CONTROL_PROTOCOL_VERSION: u16 = 3;

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
    send_message(writer, MessageType::ConfigPush, req).await
}

pub async fn recv_watch_request<R>(reader: &mut R) -> Result<WatchRequest>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader).await?;
    decode_rkyv::<WatchRequest>(&buf)
}

pub async fn recv_config_event<R>(reader: &mut R) -> Result<ConfigEvent>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader).await?;
    decode_rkyv::<ConfigEvent>(&buf)
}

pub async fn recv_apply_response<R>(reader: &mut R) -> Result<ApplyResponse>
where
    R: AsyncReadExt + Unpin,
{
    let buf = recv_control_payload(reader).await?;
    decode_rkyv::<ApplyResponse>(&buf)
}

pub fn apply_config_operations(
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
}
