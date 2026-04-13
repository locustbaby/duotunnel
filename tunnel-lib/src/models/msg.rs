use crate::proxy::core::Protocol;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use futures_util::stream::try_unfold;
use http_body_util::{BodyExt, StreamBody};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Login = 0x01,
    LoginResp = 0x02,
    RoutingInfo = 0x10,
    HttpRequestHead = 0x11,
    HttpResponseHead = 0x12,
    HttpBodyChunk = 0x13,
    HttpBodyEnd = 0x14,
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
            0x11 => Ok(MessageType::HttpRequestHead),
            0x12 => Ok(MessageType::HttpResponseHead),
            0x13 => Ok(MessageType::HttpBodyChunk),
            0x14 => Ok(MessageType::HttpBodyEnd),
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    pub proxy_name: String,
    pub protocol: Protocol,
    pub host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderField {
    pub name: String,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpRequestHead {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: Vec<HeaderField>,
    pub route_key: Option<String>,
    pub target_host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpResponseHead {
    pub status: u16,
    pub version: String,
    pub headers: Vec<HeaderField>,
}

fn http_version_to_string(version: http::Version) -> &'static str {
    match version {
        http::Version::HTTP_09 => "HTTP/0.9",
        http::Version::HTTP_10 => "HTTP/1.0",
        http::Version::HTTP_11 => "HTTP/1.1",
        http::Version::HTTP_2 => "HTTP/2",
        http::Version::HTTP_3 => "HTTP/3",
        _ => "HTTP/1.1",
    }
}

fn http_version_from_string(version: &str) -> Result<http::Version> {
    match version {
        "HTTP/0.9" => Ok(http::Version::HTTP_09),
        "HTTP/1.0" => Ok(http::Version::HTTP_10),
        "HTTP/1.1" => Ok(http::Version::HTTP_11),
        "HTTP/2" => Ok(http::Version::HTTP_2),
        "HTTP/3" => Ok(http::Version::HTTP_3),
        _ => Err(anyhow!("unsupported HTTP version: {}", version)),
    }
}

fn headers_to_wire(headers: &http::HeaderMap) -> Vec<HeaderField> {
    headers
        .iter()
        .map(|(name, value)| HeaderField {
            name: name.as_str().to_string(),
            value: value.as_bytes().to_vec(),
        })
        .collect()
}

fn headers_from_wire(headers: &[HeaderField]) -> Result<http::HeaderMap> {
    let mut header_map = http::HeaderMap::new();
    for field in headers {
        let name = http::header::HeaderName::from_bytes(field.name.as_bytes())
            .map_err(|e| anyhow!("invalid header name {}: {}", field.name, e))?;
        let value = http::header::HeaderValue::from_bytes(&field.value)
            .map_err(|e| anyhow!("invalid header value for {}: {}", field.name, e))?;
        header_map.append(name, value);
    }
    Ok(header_map)
}

impl HttpRequestHead {
    pub fn from_parts(
        parts: &http::request::Parts,
        route_key: Option<String>,
        target_host: Option<String>,
    ) -> Self {
        Self {
            method: parts.method.to_string(),
            uri: parts.uri.to_string(),
            version: http_version_to_string(parts.version).to_string(),
            headers: headers_to_wire(&parts.headers),
            route_key,
            target_host,
        }
    }

    pub fn into_parts(self) -> Result<http::request::Parts> {
        let mut req = http::Request::builder()
            .method(self.method.parse::<http::Method>()?)
            .uri(self.uri.parse::<http::Uri>()?)
            .version(http_version_from_string(&self.version)?)
            .body(())?;
        *req.headers_mut() = headers_from_wire(&self.headers)?;
        Ok(req.into_parts().0)
    }
}

impl HttpResponseHead {
    pub fn from_parts(parts: &http::response::Parts) -> Self {
        Self {
            status: parts.status.as_u16(),
            version: http_version_to_string(parts.version).to_string(),
            headers: headers_to_wire(&parts.headers),
        }
    }

    pub fn into_parts(self) -> Result<http::response::Parts> {
        let mut resp = http::Response::builder()
            .status(http::StatusCode::from_u16(self.status)?)
            .version(http_version_from_string(&self.version)?)
            .body(())?;
        *resp.headers_mut() = headers_from_wire(&self.headers)?;
        Ok(resp.into_parts().0)
    }
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

pub async fn send_raw_message<W>(writer: &mut W, msg_type: MessageType, payload: &[u8]) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    if payload.len() > 10 * 1024 * 1024 {
        return Err(anyhow!(
            "Message too large to send: {} bytes",
            payload.len()
        ));
    }
    writer.write_u8(msg_type as u8).await?;
    writer.write_u32(payload.len() as u32).await?;
    writer.write_all(payload).await?;
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

pub async fn recv_raw_message<R>(reader: &mut R) -> Result<Vec<u8>>
where
    R: AsyncReadExt + Unpin,
{
    let len = reader.read_u32().await? as usize;
    if len > 10 * 1024 * 1024 {
        return Err(anyhow!("Message too large: {} bytes", len));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}
pub async fn recv_typed_message<R, M>(reader: &mut R, expected: MessageType) -> Result<M>
where
    R: AsyncReadExt + Unpin,
    M: for<'de> Deserialize<'de>,
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

pub async fn send_http_request_head<W>(writer: &mut W, head: &HttpRequestHead) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    send_message(writer, MessageType::HttpRequestHead, head).await
}

pub async fn recv_http_request_head<R>(reader: &mut R) -> Result<HttpRequestHead>
where
    R: AsyncReadExt + Unpin,
{
    recv_typed_message(reader, MessageType::HttpRequestHead).await
}

pub async fn send_http_response_head<W>(writer: &mut W, head: &HttpResponseHead) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    send_message(writer, MessageType::HttpResponseHead, head).await
}

pub async fn recv_http_response_head<R>(reader: &mut R) -> Result<HttpResponseHead>
where
    R: AsyncReadExt + Unpin,
{
    recv_typed_message(reader, MessageType::HttpResponseHead).await
}

pub async fn send_http_body<W, B>(writer: &mut W, body: B) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
    B: hyper::body::Body + Send,
    B::Data: Into<Bytes>,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    futures_util::pin_mut!(body);
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|e| anyhow!("http body read failed: {}", e.into()))?;
        if let Ok(data) = frame.into_data() {
            send_raw_message(writer, MessageType::HttpBodyChunk, &data.into()).await?;
        }
    }
    send_raw_message(writer, MessageType::HttpBodyEnd, &[]).await
}

pub fn recv_http_body<R>(
    reader: R,
) -> http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>
where
    R: AsyncReadExt + Unpin + Send + 'static,
{
    let stream = try_unfold(reader, |mut reader| async move {
        let msg_type = recv_message_type(&mut reader)
            .await
            .map_err(std::io::Error::other)?;
        match msg_type {
            MessageType::HttpBodyChunk => {
                let payload = recv_raw_message(&mut reader)
                    .await
                    .map_err(std::io::Error::other)?;
                Ok(Some((
                    hyper::body::Frame::data(Bytes::from(payload)),
                    reader,
                )))
            }
            MessageType::HttpBodyEnd => {
                let _ = recv_raw_message(&mut reader)
                    .await
                    .map_err(std::io::Error::other)?;
                Ok(None)
            }
            other => Err(std::io::Error::other(format!(
                "unexpected body message type: {:?}",
                other
            ))),
        }
    });
    StreamBody::new(stream).boxed_unsync()
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
            protocol: Protocol::H1,
            host: Some("example.com".to_string()),
        };
        let encoded = bincode::serialize(&info).unwrap();
        let decoded: RoutingInfo = bincode::deserialize(&encoded).unwrap();
        assert_eq!(info.proxy_name, decoded.proxy_name);
        assert_eq!(info.protocol, decoded.protocol);
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
    fn test_routing_info_tcp() {
        let info = RoutingInfo {
            proxy_name: "tcp-proxy".to_string(),
            protocol: Protocol::Tcp,
            host: None,
        };
        let encoded = bincode::serialize(&info).unwrap();
        let decoded: RoutingInfo = bincode::deserialize(&encoded).unwrap();
        assert_eq!(decoded.proxy_name, "tcp-proxy");
        assert_eq!(decoded.protocol, Protocol::Tcp);
        assert_eq!(decoded.host, None);
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
            protocol: Protocol::H2,
            host: None,
        };
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_routing_info(&mut writer, &info).await.unwrap();
        drop(writer);
        let decoded = recv_routing_info(&mut reader).await.unwrap();
        assert_eq!(decoded.proxy_name, info.proxy_name);
        assert_eq!(decoded.protocol, Protocol::H2);
    }
    #[tokio::test]
    async fn test_send_recv_routing_info_tcp() {
        let info = RoutingInfo {
            proxy_name: "tcp-svc".to_string(),
            protocol: Protocol::Tcp,
            host: Some("target.example.com".to_string()),
        };
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        send_routing_info(&mut writer, &info).await.unwrap();
        drop(writer);
        let decoded = recv_routing_info(&mut reader).await.unwrap();
        assert_eq!(decoded.proxy_name, "tcp-svc");
        assert_eq!(decoded.protocol, Protocol::Tcp);
        assert_eq!(decoded.host.as_deref(), Some("target.example.com"));
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
            protocol: Protocol::H1,
            host: None,
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
        assert_eq!(decoded_info.protocol, Protocol::H1);
    }

    #[tokio::test]
    async fn test_send_recv_http_request_head_and_body() {
        use http_body_util::Full;

        let req = http::Request::builder()
            .method("POST")
            .uri("/v1/ping?x=1")
            .header("host", "api.example.com")
            .header("content-type", "text/plain")
            .body(Full::new(Bytes::from_static(b"hello world")))
            .unwrap();
        let (parts, body) = req.into_parts();
        let head = HttpRequestHead::from_parts(
            &parts,
            Some("route-a".to_string()),
            Some("backend.internal".to_string()),
        );

        let (mut writer, mut reader) = tokio::io::duplex(4096);
        send_http_request_head(&mut writer, &head).await.unwrap();
        send_http_body(&mut writer, body).await.unwrap();
        drop(writer);

        let decoded_head = recv_http_request_head(&mut reader).await.unwrap();
        assert_eq!(decoded_head.method, "POST");
        assert_eq!(decoded_head.uri, "/v1/ping?x=1");
        assert_eq!(decoded_head.route_key.as_deref(), Some("route-a"));
        assert_eq!(decoded_head.target_host.as_deref(), Some("backend.internal"));

        let body = recv_http_body(reader);
        let collected = body.collect().await.unwrap().to_bytes();
        assert_eq!(collected.as_ref(), b"hello world");
    }

    #[tokio::test]
    async fn test_send_recv_http_response_head_and_body() {
        use http_body_util::Full;

        let resp = http::Response::builder()
            .status(201)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from_static(br#"{"ok":true}"#)))
            .unwrap();
        let (parts, body) = resp.into_parts();
        let head = HttpResponseHead::from_parts(&parts);

        let (mut writer, mut reader) = tokio::io::duplex(4096);
        send_http_response_head(&mut writer, &head).await.unwrap();
        send_http_body(&mut writer, body).await.unwrap();
        drop(writer);

        let decoded_head = recv_http_response_head(&mut reader).await.unwrap();
        assert_eq!(decoded_head.status, 201);
        let body = recv_http_body(reader);
        let collected = body.collect().await.unwrap().to_bytes();
        assert_eq!(collected.as_ref(), br#"{"ok":true}"#);
    }
}
