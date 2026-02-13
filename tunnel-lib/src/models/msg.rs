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
}
