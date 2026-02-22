use serde::{Serialize, Deserialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Result, anyhow};

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
    pub client_id: String,
    pub group_id: Option<String>,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResp {
    pub success: bool,
    pub error: Option<String>,
    pub config: ClientConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientConfig {
    pub config_version: String,
    pub proxies: Vec<ProxyConfig>,
    pub upstreams: Vec<UpstreamConfig>,
    pub rules: Vec<RuleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    pub proxy_type: String,
    pub domains: Vec<String>,
    pub remote_port: Option<u16>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub rule_type: String,
    pub match_host: String,
    pub action_upstream: String,
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
    writer.write_u8(msg_type as u8).await?;
    writer.write_u32(payload.len() as u32).await?;
    writer.write_all(&payload).await?;
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
    fn test_message_serialize() {
        let login = Login {
            client_id: "test-client".to_string(),
            group_id: Some("group-a".to_string()),
            token: "secret".to_string(),
        };
        let encoded = bincode::serialize(&login).unwrap();
        let decoded: Login = bincode::deserialize(&encoded).unwrap();
        assert_eq!(login.client_id, decoded.client_id);
        assert_eq!(login.group_id, decoded.group_id);
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

    // ── MessageType::from_u8 ─────────────────────────────────────────────────

    #[test]
    fn test_message_type_from_u8_all_valid() {
        assert!(matches!(MessageType::from_u8(0x01), Ok(MessageType::Login)));
        assert!(matches!(MessageType::from_u8(0x02), Ok(MessageType::LoginResp)));
        assert!(matches!(MessageType::from_u8(0x10), Ok(MessageType::RoutingInfo)));
        assert!(matches!(MessageType::from_u8(0x04), Ok(MessageType::Ping)));
        assert!(matches!(MessageType::from_u8(0x05), Ok(MessageType::Pong)));
        assert!(matches!(MessageType::from_u8(0x06), Ok(MessageType::ConfigPush)));
    }

    #[test]
    fn test_message_type_from_u8_invalid_returns_error() {
        assert!(MessageType::from_u8(0x00).is_err());
        assert!(MessageType::from_u8(0xFF).is_err());
        assert!(MessageType::from_u8(0x03).is_err());
        assert!(MessageType::from_u8(0x07).is_err());
    }

    // ── Option<String> = None fields ─────────────────────────────────────────

    #[test]
    fn test_login_group_id_none() {
        let login = Login {
            client_id: "client-x".to_string(),
            group_id: None,
            token: "tok".to_string(),
        };
        let encoded = bincode::serialize(&login).unwrap();
        let decoded: Login = bincode::deserialize(&encoded).unwrap();
        assert_eq!(decoded.group_id, None);
        assert_eq!(decoded.client_id, "client-x");
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

    // ── Full wire-protocol round-trip via send_message / recv_message ────────

    #[tokio::test]
    async fn test_send_recv_login_full_frame() {
        let login = Login {
            client_id: "client-1".to_string(),
            group_id: Some("group-a".to_string()),
            token: "s3cr3t".to_string(),
        };

        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_message(&mut writer, MessageType::Login, &login).await.unwrap();
        drop(writer); // signal EOF

        let msg_type = recv_message_type(&mut reader).await.unwrap();
        assert_eq!(msg_type, MessageType::Login);

        let decoded: Login = recv_message(&mut reader).await.unwrap();
        assert_eq!(decoded.client_id, login.client_id);
        assert_eq!(decoded.group_id, login.group_id);
        assert_eq!(decoded.token, login.token);
    }

    #[tokio::test]
    async fn test_send_recv_login_resp_full_frame() {
        let resp = LoginResp {
            success: true,
            error: None,
            config: ClientConfig {
                config_version: "v1.0.0".to_string(),
                proxies: vec![],
                upstreams: vec![],
                rules: vec![],
            },
        };

        let (mut writer, mut reader) = tokio::io::duplex(4096);
        send_message(&mut writer, MessageType::LoginResp, &resp).await.unwrap();
        drop(writer);

        let msg_type = recv_message_type(&mut reader).await.unwrap();
        assert_eq!(msg_type, MessageType::LoginResp);

        let decoded: LoginResp = recv_message(&mut reader).await.unwrap();
        assert!(decoded.success);
        assert_eq!(decoded.error, None);
        assert_eq!(decoded.config.config_version, "v1.0.0");
    }

    #[tokio::test]
    async fn test_send_recv_login_resp_failure() {
        let resp = LoginResp {
            success: false,
            error: Some("auth failed".to_string()),
            config: ClientConfig::default(),
        };

        let (mut writer, mut reader) = tokio::io::duplex(4096);
        send_message(&mut writer, MessageType::LoginResp, &resp).await.unwrap();
        drop(writer);

        recv_message_type(&mut reader).await.unwrap(); // consume type byte
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
        // Send a Login message but try to recv_routing_info — should fail
        let login = Login {
            client_id: "c".to_string(),
            group_id: None,
            token: "t".to_string(),
        };

        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_message(&mut writer, MessageType::Login, &login).await.unwrap();
        drop(writer);

        let result = recv_routing_info(&mut reader).await;
        assert!(result.is_err(), "recv_routing_info on a Login message must fail");
    }

    #[tokio::test]
    async fn test_recv_message_size_limit() {
        // Construct a frame manually with len = 10MB + 1 (exceeds limit)
        use tokio::io::AsyncWriteExt;
        let (mut writer, mut reader) = tokio::io::duplex(64);
        // Write: [type=0x01][len=10MB+1]  — no payload bytes written (reader will error on EOF)
        let too_large: u32 = 10 * 1024 * 1024 + 1;
        writer.write_u32(too_large).await.unwrap();
        drop(writer);

        let result: Result<Login> = recv_message(&mut reader).await;
        assert!(result.is_err(), "message exceeding 10MB limit must be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("too large") || msg.contains("10"), "error should mention size: {}", msg);
    }

    // ── Ping / Pong wire round-trip (empty payload via ClientConfig::default) ─

    #[tokio::test]
    async fn test_multiple_messages_sequential_on_same_pipe() {
        // Verify framing: two messages sent back-to-back on the same stream
        // are decoded independently without bleed-over.
        let login = Login {
            client_id: "c1".to_string(),
            group_id: None,
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
        send_message(&mut writer, MessageType::Login, &login).await.unwrap();
        send_routing_info(&mut writer, &info).await.unwrap();
        drop(writer);

        // Read first message
        let t1 = recv_message_type(&mut reader).await.unwrap();
        assert_eq!(t1, MessageType::Login);
        let decoded_login: Login = recv_message(&mut reader).await.unwrap();
        assert_eq!(decoded_login.client_id, "c1");

        // Read second message — must be RoutingInfo, not garbled
        let decoded_info = recv_routing_info(&mut reader).await.unwrap();
        assert_eq!(decoded_info.proxy_name, "p");
        assert_eq!(decoded_info.host.as_deref(), Some("foo.com"));
    }
}
