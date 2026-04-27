/// List-Watch protocol between tunnel-ctld and server.
///
/// Modelled after the Kubernetes list-watch pattern:
///   1. Server connects to ctld and sends a WatchRequest with resource_version=0
///      (meaning "give me the full snapshot first").
///   2. ctld responds with a WatchEvent::Snapshot (full state, sets the baseline version).
///   3. ctld keeps the TCP connection open and streams WatchEvent::Patch for every
///      subsequent mutation (token create/revoke/rotate, routing save).
///   4. If the server reconnects, it sends resource_version=N (last known version).
///      ctld responds with a full Snapshot again (simplest correct behaviour; diffs are
///      a future optimisation).
///
/// Wire framing: reuses the existing tunnel-lib send_message/recv_message rkyv framing.
///   [msg_type: u8 = 0x06 ConfigPush][len: u32 BE][rkyv(WatchEvent)]
///
/// All types here are self-contained (no tunnel_store dependency) so this module
/// can live in tunnel-lib and be used by both tunnel-service and server.
use anyhow::{anyhow, Result};
use rkyv::{
    api::high::{HighDeserializer, HighValidator},
    bytecheck::CheckBytes,
    rancor,
    util::AlignedVec,
    Archive, Deserialize, Serialize,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::models::msg::{send_message, MessageType, MAX_MESSAGE_BYTES};

// ── Ingress routing ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ProtoIngressVhostRule {
    pub match_host: String,
    pub group_id: String,
    pub proxy_name: String,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum ProtoIngressListenerMode {
    Http {
        vhost: Vec<ProtoIngressVhostRule>,
    },
    Tcp {
        group_id: String,
        proxy_name: String,
    },
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ProtoIngressListener {
    pub port: u16,
    pub mode: ProtoIngressListenerMode,
}

// ── Client groups & upstreams ─────────────────────────────────────────────────

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ProtoUpstreamServer {
    pub address: String,
    pub resolve: bool,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ProtoClientUpstream {
    pub name: String,
    pub lb_policy: String,
    pub servers: Vec<ProtoUpstreamServer>,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ProtoClientGroup {
    pub group_id: String,
    pub config_version: String,
    pub upstreams: Vec<ProtoClientUpstream>,
}

// ── Egress ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ProtoEgressVhostRule {
    pub match_host: String,
    pub action_upstream: String,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ProtoEgressUpstreamDef {
    pub name: String,
    pub lb_policy: String,
    pub servers: Vec<ProtoUpstreamServer>,
}

// ── Token cache ───────────────────────────────────────────────────────────────

/// A single token entry in the in-memory cache sent to server.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct TokenCacheEntry {
    /// Full 64-char hex SHA-256 of the raw token.
    pub hash_hex: String,
    /// The client group this token belongs to.
    pub client_group: String,
    /// "active" | "disabled"
    pub client_status: String,
    /// "active" | "revoked"
    pub token_status: String,
}

// ── Watch protocol ────────────────────────────────────────────────────────────

/// Sent by the server to initiate (or resume) a watch session.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct WatchRequest {
    /// The last resource_version the server has seen.
    /// 0 means "I have nothing; send me the full snapshot".
    pub resource_version: u64,
    /// Optional bearer token used to authenticate the watch client.
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
    let len = reader.read_u32().await? as usize;
    if len > MAX_MESSAGE_BYTES {
        return Err(anyhow!("Message too large: {} bytes", len));
    }
    let mut buf = AlignedVec::<16>::with_capacity(len);
    buf.resize(len, 0);
    reader.read_exact(&mut buf[..]).await?;

    decode_rkyv::<WatchRequest>(&buf)
        .or_else(|_| decode_rkyv::<WatchRequestV1>(&buf).map(Into::into))
}

/// The full config snapshot pushed by ctld as the list response,
/// and also after each mutation (no delta in v1 for simplicity).
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Monotonically increasing. Incremented on every write to ctld.
    pub resource_version: u64,
    pub ingress_listeners: Vec<ProtoIngressListener>,
    pub client_groups: Vec<ProtoClientGroup>,
    pub egress_upstreams: Vec<ProtoEgressUpstreamDef>,
    pub egress_vhost_rules: Vec<ProtoEgressVhostRule>,
    /// Flattened token table for server-side in-memory auth cache.
    pub token_cache: Vec<TokenCacheEntry>,
}

/// Message envelope pushed from ctld → server over the watch stream.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum WatchEvent {
    /// Full state snapshot (response to WatchRequest or post-reconnect).
    Snapshot(ConfigSnapshot),
    /// Incremental patch — currently always a full Snapshot re-send.
    /// Reserved for future delta optimisation.
    Patch(ConfigSnapshot),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::msg::recv_message_type;

    #[tokio::test]
    async fn watch_request_without_token_uses_legacy_wire_format() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let req = WatchRequest {
            resource_version: 42,
            token: None,
        };

        send_watch_request(&mut client, &req).await.unwrap();
        assert_eq!(
            recv_message_type(&mut server).await.unwrap(),
            MessageType::ConfigPush
        );
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
        assert_eq!(
            recv_message_type(&mut server).await.unwrap(),
            MessageType::ConfigPush
        );
        let decoded = recv_watch_request(&mut server).await.unwrap();

        assert_eq!(decoded.resource_version, 7);
        assert_eq!(decoded.token.as_deref(), Some("secret"));
    }
}
