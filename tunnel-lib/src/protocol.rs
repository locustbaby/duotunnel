use anyhow::Result;
use prost::Message;
use quinn::{RecvStream, SendStream};


pub async fn write_protobuf_message<M: Message>(
    send: &mut SendStream,
    message: &M,
) -> Result<()> {
    let mut buf = Vec::new();
    message.encode(&mut buf)?;
    

    let len = buf.len() as u32;
    send.write_all(&len.to_be_bytes()).await?;
    

    send.write_all(&buf).await?;
    
    Ok(())
}

pub async fn read_protobuf_message<M: Message + Default>(
    recv: &mut RecvStream,
) -> Result<M> {

    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    

    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await?;
    

    let message = M::decode(&buf[..])?;
    Ok(message)
}

pub async fn write_control_message(
    send: &mut SendStream,
    message: &crate::proto::tunnel::ControlMessage,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

pub async fn read_control_message(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::ControlMessage> {
    read_protobuf_message(recv).await
}


#[derive(Debug, Clone)]
pub struct RoutingInfo {
    pub r#type: String,  // "http", "grpc", "wss"
    pub host: String,
    pub method: String,
    pub path: String,
}

impl RoutingInfo {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.r#type.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.host.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.method.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.path.as_bytes());
        buf.push(0);
        buf
    }


    pub fn decode(data: &[u8]) -> Result<Self> {
        let parts: Vec<&[u8]> = data.split(|&b| b == 0).collect();
        if parts.len() < 4 {

            if parts.len() >= 2 {
                let r#type = String::from_utf8(parts[0].to_vec())?;
                let host = String::from_utf8(parts[1].to_vec())?;
                return Ok(Self {
                    r#type,
                    host,
                    method: String::new(),
                    path: String::new(),
                });
            }
            return Err(anyhow::anyhow!("Invalid routing info format"));
        }
        let r#type = String::from_utf8(parts[0].to_vec())?;
        let host = String::from_utf8(parts[1].to_vec())?;
        let method = String::from_utf8(parts[2].to_vec())?;
        let path = String::from_utf8(parts[3].to_vec())?;
        Ok(Self { r#type, host, method, path })
    }
}

pub fn create_routing_frame(session_id: u64, routing_info: &RoutingInfo) -> TunnelFrame {
    let protocol_type = match routing_info.r#type.as_str() {
        "http" => ProtocolType::Http11,
        "grpc" => ProtocolType::Grpc,
        "wss" => ProtocolType::WssFrame,
        _ => ProtocolType::Http11,
    };
    TunnelFrame::new(
        session_id,
        protocol_type,
        false,
        routing_info.encode(),
    )
}

pub async fn write_heartbeat(
    send: &mut SendStream,
    message: &crate::proto::tunnel::Heartbeat,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

pub async fn read_heartbeat(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::Heartbeat> {
    read_protobuf_message(recv).await
}

pub async fn write_config_push(
    send: &mut SendStream,
    message: &crate::proto::tunnel::ConfigPushNotification,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

pub async fn read_config_push(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::ConfigPushNotification> {
    read_protobuf_message(recv).await
}

pub async fn write_hash_request(
    send: &mut SendStream,
    message: &crate::proto::tunnel::ConfigHashRequest,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

pub async fn read_hash_request(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::ConfigHashRequest> {
    read_protobuf_message(recv).await
}

pub async fn write_hash_response(
    send: &mut SendStream,
    message: &crate::proto::tunnel::ConfigHashResponse,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

pub async fn read_hash_response(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::ConfigHashResponse> {
    read_protobuf_message(recv).await
}

pub async fn write_incremental_update(
    send: &mut SendStream,
    message: &crate::proto::tunnel::IncrementalConfigUpdate,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

pub async fn read_incremental_update(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::IncrementalConfigUpdate> {
    read_protobuf_message(recv).await
}

pub use crate::frame::{TunnelFrame, ProtocolType};
pub use crate::frame::{read_frame, write_frame};

pub use crate::proto::tunnel::{
    ControlMessage, ConfigSyncRequest, ConfigSyncResponse, ErrorMessage,
    Rule, Upstream, UpstreamServer, Heartbeat, ConfigPushNotification,
    ConfigHashRequest, ConfigHashResponse, IncrementalConfigUpdate,
};
