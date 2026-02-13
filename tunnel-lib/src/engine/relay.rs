use anyhow::Result;
use quinn::{SendStream, RecvStream};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tracing::debug;

pub async fn relay_bidirectional<S>(
    mut quic_recv: RecvStream,
    mut quic_send: SendStream,
    stream: S,
) -> Result<(u64, u64)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (mut stream_read, mut stream_write) = tokio::io::split(stream);

    let quic_to_stream = async {
        let bytes = tokio::io::copy(&mut quic_recv, &mut stream_write).await?;
        let _ = stream_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };

    let stream_to_quic = async {
        let bytes = tokio::io::copy(&mut stream_read, &mut quic_send).await?;
        let _ = quic_send.finish();
        Ok::<_, std::io::Error>(bytes)
    };

    let (r1, r2) = tokio::join!(quic_to_stream, stream_to_quic);

    debug!(quic_to_stream = ?r1, stream_to_quic = ?r2, "relay completed");

    Ok((r1.unwrap_or(0), r2.unwrap_or(0)))
}

pub async fn relay_with_initial<S>(
    mut quic_recv: RecvStream,
    mut quic_send: SendStream,
    stream: S,
    initial_data: &[u8],
) -> Result<(u64, u64)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (mut stream_read, mut stream_write) = tokio::io::split(stream);

    stream_write.write_all(initial_data).await?;

    let quic_to_stream = async {
        let bytes = tokio::io::copy(&mut quic_recv, &mut stream_write).await?;
        let _ = stream_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };

    let stream_to_quic = async {
        let bytes = tokio::io::copy(&mut stream_read, &mut quic_send).await?;
        let _ = quic_send.finish();
        Ok::<_, std::io::Error>(bytes)
    };

    let (r1, r2) = tokio::join!(quic_to_stream, stream_to_quic);

    Ok((r1.unwrap_or(0), r2.unwrap_or(0)))
}
