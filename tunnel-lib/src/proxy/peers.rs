use async_trait::async_trait;
use anyhow::Result;
use bytes::Bytes;
use quinn::{SendStream, RecvStream};

#[async_trait]
pub trait UpstreamPeer: Send + Sync {
    /// Connect to the upstream and relay traffic
    async fn connect(
        &self, 
        send: SendStream, 
        recv: RecvStream, 
        initial_data: Option<Bytes>
    ) -> Result<()>;
}
