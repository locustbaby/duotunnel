use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login {
    pub token: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResp {
    pub success: bool,
    pub error: Option<String>,
    pub config: ClientConfig,
    pub client_group: String,
}
impl LoginResp {
    pub fn success(config: ClientConfig, client_group: String) -> Self {
        Self {
            success: true,
            error: None,
            config,
            client_group,
        }
    }
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(error.into()),
            config: ClientConfig::default(),
            client_group: String::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientConfig {
    pub config_version: String,
    pub upstreams: Vec<UpstreamConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    pub name: String,
    pub servers: Vec<UpstreamServer>,
    pub lb_policy: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamServer {
    pub address: String,
    pub resolve: bool,
}
/// Server → Client heartbeat. Also carries the server's current config hash for this
/// client group so the client can detect stale config without a separate round-trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
    pub seq: u64,
    pub timestamp_ms: u64,
    /// Hash of the config currently active for this client's group on the server.
    pub config_hash: u64,
}
/// Client → Server heartbeat reply. Echoes seq and carries the client's own config hash
/// so the server can push a ConfigPush if the two hashes differ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {
    pub seq: u64,
    /// Hash of the config currently applied on the client side.
    pub config_hash: u64,
}

/// FNV-1a 64-bit hash of a `ClientConfig`'s canonical binary encoding.
///
/// Uses a zero-dependency, endian-stable algorithm:
///  1. bincode-serialize the config (deterministic across runs on same arch; we control both sides)
///  2. fold bytes through FNV-1a
///
/// This is NOT cryptographic — it is a fast change-detection fingerprint.
pub fn config_hash(config: &ClientConfig) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    let bytes = bincode::serialize(config).unwrap_or_default();
    let mut h = FNV_OFFSET;
    for b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    pub proxy_name: String,
    pub src_addr: String,
    pub src_port: u16,
    pub protocol: String,
    pub host: Option<String>,
}
pub async fn send_message<W, M>(writer: &mut W, msg_type: MessageType, msg: &M) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
    M: Serialize,
{
    let payload = bincode::serialize(msg)?;
    if payload.len() > 10 * 1024 * 1024 {
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
    M: for<'de> Deserialize<'de>,
{
    let len = reader.read_u32().await? as usize;
    if len > 10 * 1024 * 1024 {
        return Err(anyhow!("Message too large: {} bytes", len));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(bincode::deserialize(&buf)?)
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
    let msg_type = recv_message_type(reader).await?;
    if msg_type != MessageType::RoutingInfo {
        return Err(anyhow!("Expected RoutingInfo, got {:?}", msg_type));
    }
    recv_message(reader).await
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_login_serialize() {
        let login = Login {
            token: "dt_test123".to_string(),
        };
        let encoded = bincode::serialize(&login).unwrap();
        let decoded: Login = bincode::deserialize(&encoded).unwrap();
        assert_eq!(login.token, decoded.token);
    }
    #[test]
    fn test_routing_info_serialize() {
        let info = RoutingInfo {
            proxy_name: "web".to_string(),
            src_addr: "192.168.1.1".to_string(),
            src_port: 12345,
            protocol: "http".to_string(),
            host: Some("example.com".to_string()),
        };
        let encoded = bincode::serialize(&info).unwrap();
        let decoded: RoutingInfo = bincode::deserialize(&encoded).unwrap();
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
            proxy_name: "tcp-proxy".to_string(),
            src_addr: "10.0.0.1".to_string(),
            src_port: 9000,
            protocol: "tcp".to_string(),
            host: None,
        };
        let encoded = bincode::serialize(&info).unwrap();
        let decoded: RoutingInfo = bincode::deserialize(&encoded).unwrap();
        assert_eq!(decoded.host, None);
        assert_eq!(decoded.protocol, "tcp");
    }
    #[tokio::test]
    async fn test_send_recv_login_full_frame() {
        let login = Login {
            token: "dt_s3cr3t".to_string(),
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
            },
            client_group: "test-client".to_string(),
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
            client_group: String::new(),
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
            proxy_name: "web".to_string(),
            src_addr: "192.168.0.1".to_string(),
            src_port: 54321,
            protocol: "https".to_string(),
            host: Some("example.com".to_string()),
        };
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_routing_info(&mut writer, &info).await.unwrap();
        drop(writer);
        let decoded = recv_routing_info(&mut reader).await.unwrap();
        assert_eq!(decoded.proxy_name, info.proxy_name);
        assert_eq!(decoded.src_addr, info.src_addr);
        assert_eq!(decoded.src_port, info.src_port);
        assert_eq!(decoded.protocol, info.protocol);
        assert_eq!(decoded.host, info.host);
    }
    #[tokio::test]
    async fn test_send_recv_routing_info_no_host() {
        let info = RoutingInfo {
            proxy_name: "tcp-svc".to_string(),
            src_addr: "::1".to_string(),
            src_port: 22,
            protocol: "tcp".to_string(),
            host: None,
        };
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_routing_info(&mut writer, &info).await.unwrap();
        drop(writer);
        let decoded = recv_routing_info(&mut reader).await.unwrap();
        assert_eq!(decoded.host, None);
        assert_eq!(decoded.protocol, "tcp");
    }
    #[tokio::test]
    async fn test_recv_routing_info_wrong_type_returns_error() {
        let login = Login {
            token: "t".to_string(),
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
        };
        let info = RoutingInfo {
            proxy_name: "p".to_string(),
            src_addr: "1.2.3.4".to_string(),
            src_port: 80,
            protocol: "http".to_string(),
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
}
