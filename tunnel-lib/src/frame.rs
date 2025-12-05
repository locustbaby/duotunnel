use anyhow::{Result, anyhow};
use bytes::{BytesMut, BufMut, Buf};
use quinn::{RecvStream, SendStream};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    Http11 = 0,
    Grpc = 1,
    WssFrame = 2,
}

impl ProtocolType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value & 0x0F {
            0 => Ok(ProtocolType::Http11),
            1 => Ok(ProtocolType::Grpc),
            2 => Ok(ProtocolType::WssFrame),
            _ => Err(anyhow!("Invalid protocol type: {}", value & 0x0F)),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone)]
pub struct TunnelFrame {

    pub session_id: u64,

    pub protocol_type: ProtocolType,

    pub end_of_stream: bool,

    pub payload: Vec<u8>,
}

impl TunnelFrame {

    pub fn new(
        session_id: u64,
        protocol_type: ProtocolType,
        end_of_stream: bool,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            session_id,
            protocol_type,
            end_of_stream,
            payload,
        }
    }


    pub fn session_id_from_uuid(uuid_str: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        uuid_str.hash(&mut hasher);
        hasher.finish()
    }


    pub fn session_id_from_uuid_obj(uuid: &Uuid) -> u64 {
        Self::session_id_from_uuid(&uuid.to_string())
    }


    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(13 + self.payload.len());
        

        buf.put_u64(self.session_id);
        

        let mut flags = self.protocol_type.to_u8();
        if self.end_of_stream {
            flags |= 0x10;
        }
        buf.put_u8(flags);
        

        buf.put_u32(self.payload.len() as u32);
        

        buf.put_slice(&self.payload);
        
        buf.to_vec()
    }


    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 13 {
            return Err(anyhow!("Frame too short: {} bytes", data.len()));
        }

        let mut buf = &data[..];
        

        let session_id = buf.get_u64();
        

        let flags = buf.get_u8();
        let protocol_type = ProtocolType::from_u8(flags)?;
        let end_of_stream = (flags & 0x10) != 0;
        

        let data_length = buf.get_u32() as usize;
        

        if buf.len() < data_length {
            return Err(anyhow!(
                "Incomplete frame: expected {} bytes, got {}",
                data_length,
                buf.len()
            ));
        }
        

        let payload = buf[..data_length].to_vec();
        
        Ok(Self {
            session_id,
            protocol_type,
            end_of_stream,
            payload,
        })
    }
}

pub async fn read_frame(recv: &mut RecvStream) -> Result<TunnelFrame> {
    read_frame_with_timeout(recv, None).await
}

pub async fn read_frame_with_timeout(
    recv: &mut RecvStream,
    timeout: Option<std::time::Duration>,
) -> Result<TunnelFrame> {
    let read_future = async {

        let mut header = vec![0u8; 13];
        recv.read_exact(&mut header).await?;
        

        let mut buf = &header[..];
        let _session_id = buf.get_u64();
        buf.get_u8();
        let data_length = buf.get_u32() as usize;
        

        if data_length > 100 * 1024 * 1024 {

            return Err(anyhow::anyhow!("Frame payload too large: {} bytes", data_length));
        }
        

        let mut payload = vec![0u8; data_length];
        if data_length > 0 {
            recv.read_exact(&mut payload).await?;
        }
        

        let mut full_frame = header;
        full_frame.extend_from_slice(&payload);
        TunnelFrame::decode(&full_frame)
    };
    
    match timeout {
        Some(duration) => {
            tokio::time::timeout(duration, read_future)
                .await
                .map_err(|_| anyhow::anyhow!("Frame read timeout after {:?}", duration))?
        }
        None => read_future.await,
    }
}

pub async fn write_frame(send: &mut SendStream, frame: &TunnelFrame) -> Result<()> {
    let encoded = frame.encode();
    send.write_all(&encoded).await?;
    Ok(())
}

pub async fn write_frames(send: &mut SendStream, frames: &[TunnelFrame]) -> Result<()> {
    for frame in frames {
        write_frame(send, frame).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_encode_decode() {
        let original = TunnelFrame::new(
            12345678901234567890,
            ProtocolType::Http11,
            true,
            b"Hello, World!".to_vec(),
        );
        
        let encoded = original.encode();
        let decoded = TunnelFrame::decode(&encoded).unwrap();
        
        assert_eq!(original.session_id, decoded.session_id);
        assert_eq!(original.protocol_type, decoded.protocol_type);
        assert_eq!(original.end_of_stream, decoded.end_of_stream);
        assert_eq!(original.payload, decoded.payload);
    }

    #[test]
    fn test_session_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let session_id1 = TunnelFrame::session_id_from_uuid(&uuid.to_string());
        let session_id2 = TunnelFrame::session_id_from_uuid(&uuid.to_string());
        assert_eq!(session_id1, session_id2);
    }
}

