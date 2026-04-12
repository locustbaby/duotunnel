use crate::proxy::buffer_params::DEFAULT_RELAY_BUF_SIZE;
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::debug;

use DEFAULT_RELAY_BUF_SIZE as RELAY_BUF;

/// Core relay for QUIC↔generic-stream (e.g. TLS).
/// Writes `initial_data` to the stream before the copy loop.
/// Uses copy_buf with shared RELAY_BUF for consistent buffer sizing.
async fn relay_inner<S>(
    quic_recv: RecvStream,
    mut quic_send: SendStream,
    stream: S,
    initial_data: &[u8],
) -> Result<(u64, u64)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (stream_read, mut stream_write) = tokio::io::split(stream);
    if !initial_data.is_empty() {
        stream_write.write_all(initial_data).await?;
    }
    let mut quic_recv = BufReader::with_capacity(RELAY_BUF, quic_recv);
    let mut stream_read = BufReader::with_capacity(RELAY_BUF, stream_read);
    let quic_to_stream = async {
        let bytes = tokio::io::copy_buf(&mut quic_recv, &mut stream_write).await?;
        let _ = stream_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };
    let stream_to_quic = async {
        let bytes = tokio::io::copy_buf(&mut stream_read, &mut quic_send).await?;
        let _ = quic_send.finish();
        Ok::<_, std::io::Error>(bytes)
    };
    let (r1, r2) = tokio::join!(quic_to_stream, stream_to_quic);
    debug!(quic_to_stream = ?r1, stream_to_quic = ?r2, "relay completed");
    match (r1, r2) {
        (Ok(a), Ok(b)) => Ok((a, b)),
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
    }
}

pub async fn relay_bidirectional<S>(
    quic_recv: RecvStream,
    quic_send: SendStream,
    stream: S,
) -> Result<(u64, u64)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    relay_inner(quic_recv, quic_send, stream, &[]).await
}

pub async fn relay_with_initial<S>(
    quic_recv: RecvStream,
    quic_send: SendStream,
    stream: S,
    initial_data: &[u8],
) -> Result<(u64, u64)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    relay_inner(quic_recv, quic_send, stream, initial_data).await
}
