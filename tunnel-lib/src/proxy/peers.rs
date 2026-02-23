use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use quinn::{RecvStream, SendStream};

#[async_trait]
pub trait UpstreamPeer: Send + Sync {
    async fn connect(
        &self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()>;
}
