use anyhow::Result;
use prost::Message;
use quinn::{RecvStream, SendStream};
use tokio::io::AsyncWriteExt;

/// Helper functions for reading/writing Protobuf messages over QUIC streams

/// Write a Protobuf message to a QUIC stream with length prefix
pub async fn write_protobuf_message<M: Message>(
    send: &mut SendStream,
    message: &M,
) -> Result<()> {
    let mut buf = Vec::new();
    message.encode(&mut buf)?;
    
    // Write length prefix (4 bytes, big-endian)
    let len = buf.len() as u32;
    send.write_all(&len.to_be_bytes()).await?;
    
    // Write message
    send.write_all(&buf).await?;
    
    Ok(())
}

/// Read a Protobuf message from a QUIC stream with length prefix
pub async fn read_protobuf_message<M: Message + Default>(
    recv: &mut RecvStream,
) -> Result<M> {
    // Read length prefix (4 bytes, big-endian)
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    
    // Read message
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await?;
    
    // Decode protobuf message
    let message = M::decode(&buf[..])?;
    Ok(message)
}

/// Write ControlMessage to a control stream
pub async fn write_control_message(
    send: &mut SendStream,
    message: &crate::proto::tunnel::ControlMessage,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

/// Read ControlMessage from a control stream
pub async fn read_control_message(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::ControlMessage> {
    read_protobuf_message(recv).await
}

/// Write DataStreamHeader to a data stream
pub async fn write_data_stream_header(
    send: &mut SendStream,
    header: &crate::proto::tunnel::DataStreamHeader,
) -> Result<()> {
    write_protobuf_message(send, header).await
}

/// Read DataStreamHeader from a data stream
pub async fn read_data_stream_header(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::DataStreamHeader> {
    read_protobuf_message(recv).await
}

/// Write Heartbeat to a control stream
pub async fn write_heartbeat(
    send: &mut SendStream,
    message: &crate::proto::tunnel::Heartbeat,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

/// Read Heartbeat from a control stream
pub async fn read_heartbeat(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::Heartbeat> {
    read_protobuf_message(recv).await
}

/// Write ConfigPushNotification to a control stream
pub async fn write_config_push(
    send: &mut SendStream,
    message: &crate::proto::tunnel::ConfigPushNotification,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

/// Read ConfigPushNotification from a control stream
pub async fn read_config_push(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::ConfigPushNotification> {
    read_protobuf_message(recv).await
}

/// Write ConfigHashRequest to a control stream
pub async fn write_hash_request(
    send: &mut SendStream,
    message: &crate::proto::tunnel::ConfigHashRequest,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

/// Read ConfigHashRequest from a control stream
pub async fn read_hash_request(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::ConfigHashRequest> {
    read_protobuf_message(recv).await
}

/// Write ConfigHashResponse to a control stream
pub async fn write_hash_response(
    send: &mut SendStream,
    message: &crate::proto::tunnel::ConfigHashResponse,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

/// Read ConfigHashResponse from a control stream
pub async fn read_hash_response(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::ConfigHashResponse> {
    read_protobuf_message(recv).await
}

/// Write IncrementalConfigUpdate to a control stream
pub async fn write_incremental_update(
    send: &mut SendStream,
    message: &crate::proto::tunnel::IncrementalConfigUpdate,
) -> Result<()> {
    write_protobuf_message(send, message).await
}

/// Read IncrementalConfigUpdate from a control stream
pub async fn read_incremental_update(
    recv: &mut RecvStream,
) -> Result<crate::proto::tunnel::IncrementalConfigUpdate> {
    read_protobuf_message(recv).await
}

// Re-export protobuf types for convenience (used by client and server)
pub use crate::proto::tunnel::{
    ControlMessage, ConfigSyncRequest, ConfigSyncResponse, DataStreamHeader, ErrorMessage,
    Rule, Upstream, UpstreamServer, Heartbeat, ConfigPushNotification,
    ConfigHashRequest, ConfigHashResponse, IncrementalConfigUpdate,
};
