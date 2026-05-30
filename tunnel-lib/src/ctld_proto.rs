use crate::models::msg::{recv_message_type, send_message, MessageType, MAX_MESSAGE_BYTES};
use crate::shared::{
    ClientGroupDef, ClientUpstreamDef, EgressUpstreamDef, EgressVhostRuleDef,
    IngressListenerDef, IngressListenerModeDef, IngressVhostRuleDef, TokenCacheEntryDef,
    UpstreamServerDef,
};
use anyhow::{anyhow, Result};
use rkyv::{
    api::high::{HighDeserializer, HighValidator},
    bytecheck::CheckBytes,
    rancor,
    util::AlignedVec,
    Archive, Deserialize, Serialize,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
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
    let msg_type = recv_message_type(reader).await?;
    if msg_type != MessageType::ConfigPush {
        return Err(anyhow!("expected {:?}, got {:?}", MessageType::ConfigPush, msg_type));
    }
    let len = reader.read_u32().await? as usize;
    if len > MAX_MESSAGE_BYTES {
        return Err(anyhow!("Message too large: {} bytes", len));
    }
    let mut buf = AlignedVec::<16>::with_capacity(len);
    unsafe {
        buf.set_len(len);
    }
    reader.read_exact(&mut buf[..]).await?;
    decode_rkyv::<WatchRequest>(&buf)
        .or_else(|_| decode_rkyv::<WatchRequestV1>(&buf).map(Into::into))
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
}
