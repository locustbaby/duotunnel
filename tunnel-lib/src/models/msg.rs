//! Client<->server wire protocol messages (rkyv-archived payloads).
//!
//! Evolution rules — rkyv binary layout is bound to the field definitions, so:
//! - Only append fields at the end of a struct; never reorder, remove, or
//!   change the type of an existing field.
//! - Every appended field must be gated by a capability bit negotiated in the
//!   login handshake; peers that did not negotiate the bit must not depend on it.
//! - Breaking layout changes require a new ALPN generation
//!   (see [`crate::transport::quic::TUNNEL_ALPN`]) so incompatible peers fail
//!   at the QUIC handshake instead of choking on rkyv validation after connect.

use crate::models::id::{GroupId, ProxyName};
use crate::proxy::core::Protocol;
use anyhow::{anyhow, Result};
use rkyv::{
    api::high::{HighDeserializer, HighSerializer, HighValidator},
    bytecheck::CheckBytes,
    rancor,
    ser::allocator::ArenaHandle,
    util::AlignedVec,
    Archive, Deserialize, Serialize,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(crate) const MAX_MESSAGE_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_DATAGRAM_BYTES: usize = 1200;
/// Ceiling for the `Login` frame, which is read before the peer is
/// authenticated. A real Login is a token plus two integers; the generous
/// headroom covers token length growth without letting an unauthenticated peer
/// dictate a large allocation.
pub const MAX_LOGIN_BYTES: usize = 64 * 1024;
pub const MAX_ROUTING_INFO_BYTES: usize = 8 * 1024;

/// Highest wire-protocol version this build speaks.
pub const PROTOCOL_VERSION: u16 = 1;
/// Oldest client version this build still accepts at login.
pub const MIN_SUPPORTED_VERSION: u16 = 1;
/// Capability bits exchanged in the login handshake. Plain u64 masks with
/// named constants (bitflags semantics without the dependency); no bits are
/// defined yet.
pub const CAP_NONE: u64 = 0;
/// Every capability this build implements — advertised in `Login`, intersected
/// with the peer's set for `LoginResp`.
pub const SUPPORTED_CAPABILITIES: u64 = CAP_NONE;

/// Outcome of the login handshake, kept in per-connection session state so
/// future features can be gated on what the peer actually negotiated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegotiatedProtocol {
    pub version: u16,
    pub capabilities: u64,
}

/// Server-side negotiation. A client reporting a version newer than ours is
/// not an error — it still speaks everything up to our version, so we answer
/// with min(ours, theirs) (forward compatibility for client-first upgrades).
/// Only clients below `MIN_SUPPORTED_VERSION` are refused, and the caller must
/// say so explicitly in the `LoginResp` failure.
pub fn negotiate_protocol(
    client_version: u16,
    client_capabilities: u64,
) -> Option<NegotiatedProtocol> {
    if client_version < MIN_SUPPORTED_VERSION {
        return None;
    }
    Some(NegotiatedProtocol {
        version: PROTOCOL_VERSION.min(client_version),
        capabilities: SUPPORTED_CAPABILITIES & client_capabilities,
    })
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Login = 0x01,
    LoginResp = 0x02,
    RoutingInfo = 0x10,
    Ping = 0x04,
    Pong = 0x05,
    ConfigPush = 0x06,
}
impl MessageType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(MessageType::Login),
            0x02 => Ok(MessageType::LoginResp),
            0x10 => Ok(MessageType::RoutingInfo),
            0x04 => Ok(MessageType::Ping),
            0x05 => Ok(MessageType::Pong),
            0x06 => Ok(MessageType::ConfigPush),
            _ => Err(anyhow!("Unknown message type: 0x{:02x}", value)),
        }
    }
}
#[derive(Clone, Archive, Serialize, Deserialize)]
pub struct Login {
    pub token: String,
    pub protocol_version: u16,
    pub capabilities: u64,
}

impl std::fmt::Debug for Login {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let masked = if self.token.starts_with("dt_") && self.token.len() >= 10 {
            "dt_masked_...".to_string()
        } else {
            "***".to_string()
        };
        f.debug_struct("Login")
            .field("token", &masked)
            .field("protocol_version", &self.protocol_version)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct LoginResp {
    pub success: bool,
    pub error: Option<String>,
    pub config: ClientConfig,
    pub client_group: GroupId,
    pub negotiated_version: u16,
    pub capabilities: u64,
    /// Whether the client should retry. Machine-readable on purpose: error
    /// strings sent to unauthenticated peers are deliberately generic, so the
    /// client cannot tell a transient server-side fault from a rejected token
    /// by inspecting text.
    pub retryable: bool,
}
impl LoginResp {
    pub fn success(
        config: ClientConfig,
        client_group: GroupId,
        negotiated: NegotiatedProtocol,
    ) -> Self {
        Self {
            success: true,
            error: None,
            config,
            client_group,
            negotiated_version: negotiated.version,
            capabilities: negotiated.capabilities,
            retryable: false,
        }
    }
    /// Permanent rejection: the client must not retry with the same input.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(error.into()),
            config: ClientConfig::default(),
            client_group: GroupId::default(),
            // 0 = no agreement reached; valid versions start at 1.
            negotiated_version: 0,
            capabilities: CAP_NONE,
            retryable: false,
        }
    }
    /// Transient rejection: the same client may succeed on a later attempt
    /// (server still starting, backing store unavailable, capacity exhausted).
    pub fn failure_retryable(error: impl Into<String>) -> Self {
        Self {
            retryable: true,
            ..Self::failure(error)
        }
    }
}
#[derive(Debug, Clone, Default, Archive, Serialize, Deserialize)]
pub struct ClientConfig {
    pub config_version: String,
    pub upstreams: Vec<UpstreamConfig>,
    pub egress_rules: Vec<crate::EgressVhostRuleDef>,
}
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct UpstreamConfig {
    pub name: String,
    pub servers: Vec<UpstreamServer>,
    pub lb_policy: String,
}
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct UpstreamServer {
    pub address: String,
    pub resolve: bool,
}
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct RoutingInfo {
    pub proxy_name: ProxyName,
    pub src_addr: String,
    pub src_port: u16,
    pub protocol: Protocol,
    pub host: Option<String>,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UdpSessionKey {
    pub proxy_name: ProxyName,
    pub client_addr: std::net::IpAddr,
    pub client_port: u16,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub struct UdpDatagramEnvelope {
    pub session: UdpSessionKey,
    pub payload: Vec<u8>,
}

pub async fn send_message<W, M>(writer: &mut W, msg_type: MessageType, msg: &M) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
    M: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rancor::Error>>,
{
    let payload =
        rkyv::to_bytes::<rancor::Error>(msg).map_err(|e| anyhow!("rkyv serialize failed: {e}"))?;
    if payload.len() > MAX_MESSAGE_BYTES {
        return Err(anyhow!(
            "Message too large to send: {} bytes",
            payload.len()
        ));
    }
    let mut frame = Vec::with_capacity(5 + payload.len());
    frame.push(msg_type as u8);
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(&payload);
    writer.write_all(&frame).await?;
    Ok(())
}
pub async fn recv_message_type<R>(reader: &mut R) -> Result<MessageType>
where
    R: AsyncReadExt + Unpin,
{
    let type_byte = reader.read_u8().await?;
    MessageType::from_u8(type_byte)
}
pub async fn recv_message<R, M>(reader: &mut R) -> Result<M>
where
    R: AsyncReadExt + Unpin,
    M: Archive,
    M::Archived: for<'a> CheckBytes<HighValidator<'a, rancor::Error>>
        + Deserialize<M, HighDeserializer<rancor::Error>>,
{
    recv_message_bounded(reader, MAX_MESSAGE_BYTES).await
}

/// Like [`recv_message`] but with a caller-supplied ceiling. The buffer is
/// allocated from the peer-declared length before any payload arrives, so paths
/// that read from an unauthenticated peer must pass a bound that fits the
/// message they expect rather than the generic 10 MiB frame limit.
pub async fn recv_message_bounded<R, M>(reader: &mut R, max_bytes: usize) -> Result<M>
where
    R: AsyncReadExt + Unpin,
    M: Archive,
    M::Archived: for<'a> CheckBytes<HighValidator<'a, rancor::Error>>
        + Deserialize<M, HighDeserializer<rancor::Error>>,
{
    let max_bytes = max_bytes.min(MAX_MESSAGE_BYTES);
    let len = reader.read_u32().await? as usize;
    if len > max_bytes {
        return Err(anyhow!("Message too large: {} bytes", len));
    }
    let mut buf = AlignedVec::<16>::with_capacity(len);
    buf.resize(len, 0);
    reader.read_exact(&mut buf[..]).await?;
    let archived = rkyv::access::<M::Archived, rancor::Error>(&buf[..])
        .map_err(|e| anyhow!("rkyv access failed: {e}"))?;
    let value = rkyv::deserialize::<M, rancor::Error>(archived)
        .map_err(|e| anyhow!("rkyv deserialize failed: {e}"))?;
    Ok(value)
}
pub async fn recv_typed_message<R, M>(reader: &mut R, expected: MessageType) -> Result<M>
where
    R: AsyncReadExt + Unpin,
    M: Archive,
    M::Archived: for<'a> CheckBytes<HighValidator<'a, rancor::Error>>
        + Deserialize<M, HighDeserializer<rancor::Error>>,
{
    let msg_type = recv_message_type(reader).await?;
    if msg_type != expected {
        return Err(anyhow!("expected {:?}, got {:?}", expected, msg_type));
    }
    recv_message(reader).await
}
pub async fn send_routing_info<W>(writer: &mut W, info: &RoutingInfo) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    send_message(writer, MessageType::RoutingInfo, info).await
}
pub async fn recv_routing_info<R>(reader: &mut R) -> Result<RoutingInfo>
where
    R: AsyncReadExt + Unpin,
{
    recv_typed_message(reader, MessageType::RoutingInfo).await
}

pub async fn recv_routing_info_bounded<R>(reader: &mut R) -> Result<RoutingInfo>
where
    R: AsyncReadExt + Unpin,
{
    let msg_type = recv_message_type(reader).await?;
    if msg_type != MessageType::RoutingInfo {
        return Err(anyhow!(
            "expected {:?}, got {:?}",
            MessageType::RoutingInfo,
            msg_type
        ));
    }
    recv_message_bounded(reader, MAX_ROUTING_INFO_BYTES).await
}

pub fn encode_udp_datagram_envelope(envelope: &UdpDatagramEnvelope) -> Result<AlignedVec<16>> {
    let payload = rkyv::to_bytes::<rancor::Error>(envelope)
        .map_err(|e| anyhow!("rkyv serialize failed: {e}"))?;
    if payload.len() > MAX_DATAGRAM_BYTES {
        return Err(anyhow!(
            "Datagram too large to send: {} bytes",
            payload.len()
        ));
    }
    Ok(payload)
}

pub fn decode_udp_datagram_envelope(buf: &[u8]) -> Result<UdpDatagramEnvelope> {
    if buf.len() > MAX_DATAGRAM_BYTES {
        return Err(anyhow!("Datagram too large: {} bytes", buf.len()));
    }
    let archived = rkyv::access::<ArchivedUdpDatagramEnvelope, rancor::Error>(buf)
        .map_err(|e| anyhow!("rkyv access failed: {e}"))?;
    rkyv::deserialize::<UdpDatagramEnvelope, rancor::Error>(archived)
        .map_err(|e| anyhow!("rkyv deserialize failed: {e}"))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn encode<T>(value: &T) -> AlignedVec
    where
        T: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rancor::Error>>,
    {
        rkyv::to_bytes::<rancor::Error>(value).unwrap()
    }
    fn decode<T>(bytes: &[u8]) -> T
    where
        T: Archive,
        T::Archived: for<'a> CheckBytes<HighValidator<'a, rancor::Error>>
            + Deserialize<T, HighDeserializer<rancor::Error>>,
    {
        rkyv::from_bytes::<T, rancor::Error>(bytes).unwrap()
    }
    #[test]
    fn test_login_serialize() {
        let login = Login {
            token: "dt_test123".to_string(),
            protocol_version: PROTOCOL_VERSION,
            capabilities: SUPPORTED_CAPABILITIES,
        };
        let encoded = encode(&login);
        let decoded: Login = decode(&encoded);
        assert_eq!(login.token, decoded.token);
    }
    #[test]
    fn test_routing_info_serialize() {
        let info = RoutingInfo {
            proxy_name: "web".into(),
            src_addr: "192.168.1.1".to_string(),
            src_port: 12345,
            protocol: Protocol::H1,
            host: Some("example.com".to_string()),
        };
        let encoded = encode(&info);
        let decoded: RoutingInfo = decode(&encoded);
        assert_eq!(info.proxy_name, decoded.proxy_name);
        assert_eq!(info.host, decoded.host);
    }
    #[test]
    fn test_message_type_from_u8_all_valid() {
        assert!(matches!(MessageType::from_u8(0x01), Ok(MessageType::Login)));
        assert!(matches!(
            MessageType::from_u8(0x02),
            Ok(MessageType::LoginResp)
        ));
        assert!(matches!(
            MessageType::from_u8(0x10),
            Ok(MessageType::RoutingInfo)
        ));
        assert!(matches!(MessageType::from_u8(0x04), Ok(MessageType::Ping)));
        assert!(matches!(MessageType::from_u8(0x05), Ok(MessageType::Pong)));
        assert!(matches!(
            MessageType::from_u8(0x06),
            Ok(MessageType::ConfigPush)
        ));
    }
    #[test]
    fn test_message_type_from_u8_invalid_returns_error() {
        assert!(MessageType::from_u8(0x00).is_err());
        assert!(MessageType::from_u8(0xFF).is_err());
        assert!(MessageType::from_u8(0x03).is_err());
        assert!(MessageType::from_u8(0x07).is_err());
    }
    #[test]
    fn test_routing_info_host_none() {
        let info = RoutingInfo {
            proxy_name: "tcp-proxy".into(),
            src_addr: "10.0.0.1".to_string(),
            src_port: 9000,
            protocol: Protocol::Tcp,
            host: None,
        };
        let encoded = encode(&info);
        let decoded: RoutingInfo = decode(&encoded);
        assert_eq!(decoded.host, None);
        assert_eq!(decoded.protocol, Protocol::Tcp);
    }
    #[tokio::test]
    async fn test_send_recv_login_full_frame() {
        let login = Login {
            token: "dt_s3cr3t".to_string(),
            protocol_version: PROTOCOL_VERSION,
            capabilities: SUPPORTED_CAPABILITIES,
        };
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_message(&mut writer, MessageType::Login, &login)
            .await
            .unwrap();
        drop(writer);
        let msg_type = recv_message_type(&mut reader).await.unwrap();
        assert_eq!(msg_type, MessageType::Login);
        let decoded: Login = recv_message(&mut reader).await.unwrap();
        assert_eq!(decoded.token, login.token);
    }
    #[tokio::test]
    async fn test_send_recv_login_resp_full_frame() {
        let resp = LoginResp {
            success: true,
            error: None,
            config: ClientConfig {
                config_version: "v1.0.0".to_string(),
                upstreams: vec![],
                egress_rules: vec![],
            },
            client_group: "test-client".into(),
            negotiated_version: PROTOCOL_VERSION,
            capabilities: CAP_NONE,
            retryable: false,
        };
        let (mut writer, mut reader) = tokio::io::duplex(4096);
        send_message(&mut writer, MessageType::LoginResp, &resp)
            .await
            .unwrap();
        drop(writer);
        let msg_type = recv_message_type(&mut reader).await.unwrap();
        assert_eq!(msg_type, MessageType::LoginResp);
        let decoded: LoginResp = recv_message(&mut reader).await.unwrap();
        assert!(decoded.success);
        assert_eq!(decoded.error, None);
        assert_eq!(decoded.config.config_version, "v1.0.0");
        assert_eq!(decoded.client_group, "test-client");
    }
    #[tokio::test]
    async fn test_send_recv_login_resp_failure() {
        let resp = LoginResp {
            success: false,
            error: Some("auth failed".to_string()),
            config: ClientConfig::default(),
            client_group: "".into(),
            negotiated_version: 0,
            capabilities: CAP_NONE,
            retryable: false,
        };
        let (mut writer, mut reader) = tokio::io::duplex(4096);
        send_message(&mut writer, MessageType::LoginResp, &resp)
            .await
            .unwrap();
        drop(writer);
        recv_message_type(&mut reader).await.unwrap();
        let decoded: LoginResp = recv_message(&mut reader).await.unwrap();
        assert!(!decoded.success);
        assert_eq!(decoded.error.as_deref(), Some("auth failed"));
    }
    #[tokio::test]
    async fn test_send_recv_routing_info_full_frame() {
        let info = RoutingInfo {
            proxy_name: "web".into(),
            src_addr: "192.168.0.1".to_string(),
            src_port: 54321,
            protocol: Protocol::H2,
            host: Some("example.com".to_string()),
        };
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_routing_info(&mut writer, &info).await.unwrap();
        drop(writer);
        let decoded = recv_routing_info(&mut reader).await.unwrap();
        assert_eq!(decoded.proxy_name, info.proxy_name);
        assert_eq!(decoded.src_addr, info.src_addr);
        assert_eq!(decoded.src_port, info.src_port);
        assert_eq!(decoded.protocol, Protocol::H2);
        assert_eq!(decoded.host, info.host);
    }
    #[tokio::test]
    async fn test_send_recv_routing_info_no_host() {
        let info = RoutingInfo {
            proxy_name: "tcp-svc".into(),
            src_addr: "::1".to_string(),
            src_port: 22,
            protocol: Protocol::Tcp,
            host: None,
        };
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_routing_info(&mut writer, &info).await.unwrap();
        drop(writer);
        let decoded = recv_routing_info(&mut reader).await.unwrap();
        assert_eq!(decoded.host, None);
        assert_eq!(decoded.protocol, Protocol::Tcp);
    }
    #[tokio::test]
    async fn test_recv_routing_info_wrong_type_returns_error() {
        let login = Login {
            token: "t".to_string(),
            protocol_version: PROTOCOL_VERSION,
            capabilities: SUPPORTED_CAPABILITIES,
        };
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_message(&mut writer, MessageType::Login, &login)
            .await
            .unwrap();
        drop(writer);
        let result = recv_routing_info(&mut reader).await;
        assert!(
            result.is_err(),
            "recv_routing_info on a Login message must fail"
        );
    }
    #[tokio::test]
    async fn test_recv_message_size_limit() {
        use tokio::io::AsyncWriteExt;
        let (mut writer, mut reader) = tokio::io::duplex(64);
        let too_large: u32 = 10 * 1024 * 1024 + 1;
        writer.write_u32(too_large).await.unwrap();
        drop(writer);
        let result: Result<Login> = recv_message(&mut reader).await;
        assert!(
            result.is_err(),
            "message exceeding 10MB limit must be rejected"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("too large") || msg.contains("10"),
            "error should mention size: {}",
            msg
        );
    }
    #[tokio::test]
    async fn test_multiple_messages_sequential_on_same_pipe() {
        let login = Login {
            token: "tok1".to_string(),
            protocol_version: PROTOCOL_VERSION,
            capabilities: SUPPORTED_CAPABILITIES,
        };
        let info = RoutingInfo {
            proxy_name: "p".into(),
            src_addr: "1.2.3.4".to_string(),
            src_port: 80,
            protocol: Protocol::H1,
            host: Some("foo.com".to_string()),
        };
        let (mut writer, mut reader) = tokio::io::duplex(4096);
        send_message(&mut writer, MessageType::Login, &login)
            .await
            .unwrap();
        send_routing_info(&mut writer, &info).await.unwrap();
        drop(writer);
        let t1 = recv_message_type(&mut reader).await.unwrap();
        assert_eq!(t1, MessageType::Login);
        let decoded_login: Login = recv_message(&mut reader).await.unwrap();
        assert_eq!(decoded_login.token, "tok1");
        let decoded_info = recv_routing_info(&mut reader).await.unwrap();
        assert_eq!(decoded_info.proxy_name, "p");
        assert_eq!(decoded_info.host.as_deref(), Some("foo.com"));
    }

    #[tokio::test]
    async fn routing_info_rejects_oversized_frame_before_allocation() {
        let (mut writer, mut reader) = tokio::io::duplex(16);
        writer
            .write_u8(MessageType::RoutingInfo as u8)
            .await
            .unwrap();
        writer
            .write_u32((MAX_ROUTING_INFO_BYTES + 1) as u32)
            .await
            .unwrap();

        let error = recv_routing_info_bounded(&mut reader)
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("Message too large"));
    }

    #[test]
    fn test_udp_datagram_envelope_round_trip() {
        let envelope = UdpDatagramEnvelope {
            session: UdpSessionKey {
                proxy_name: "dns".into(),
                client_addr: "127.0.0.1".parse().unwrap(),
                client_port: 5353,
            },
            payload: vec![1, 2, 3, 4, 5],
        };

        let encoded = encode_udp_datagram_envelope(&envelope).unwrap();
        let decoded = decode_udp_datagram_envelope(&encoded).unwrap();
        assert_eq!(decoded, envelope);
    }
}
