use crate::engine::copy::{copy_buffered_then_finish, copy_quic_to_shutdown};
use crate::proxy::buffer_params::DEFAULT_RELAY_BUF_SIZE;
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

/// Core relay for QUIC↔generic-stream (e.g. TLS).
/// Writes `initial_data` to the stream before the copy loop.
/// Uses copy_buf with shared RELAY_BUF for consistent buffer sizing.
async fn relay_inner<S>(
    quic_recv: RecvStream,
    quic_send: SendStream,
    stream: S,
    initial_data: &[u8],
    buffer_size: usize,
) -> Result<(u64, u64)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (stream_read, mut stream_write) = tokio::io::split(stream);
    if !initial_data.is_empty() {
        stream_write.write_all(initial_data).await?;
    }
    let quic_to_stream = copy_quic_to_shutdown(quic_recv, stream_write);
    let stream_to_quic = copy_buffered_then_finish(stream_read, quic_send, buffer_size);
    match tokio::try_join!(quic_to_stream, stream_to_quic) {
        Ok((a, b)) => {
            debug!(quic_to_stream = a, stream_to_quic = b, "relay completed");
            Ok((a, b))
        }
        Err(e) => Err(e.into()),
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
    relay_inner(quic_recv, quic_send, stream, &[], DEFAULT_RELAY_BUF_SIZE).await
}

pub async fn relay_bidirectional_with_buffer_size<S>(
    quic_recv: RecvStream,
    quic_send: SendStream,
    stream: S,
    buffer_size: usize,
) -> Result<(u64, u64)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    relay_inner(quic_recv, quic_send, stream, &[], buffer_size).await
}

pub async fn relay_tcp_bidirectional(
    quic_recv: RecvStream,
    quic_send: SendStream,
    stream: TcpStream,
) -> Result<(u64, u64)> {
    relay_tcp_with_initial_and_buffer_size(
        quic_recv,
        quic_send,
        stream,
        &[],
        DEFAULT_RELAY_BUF_SIZE,
    )
    .await
}

pub async fn relay_tcp_bidirectional_with_buffer_size(
    quic_recv: RecvStream,
    quic_send: SendStream,
    stream: TcpStream,
    buffer_size: usize,
) -> Result<(u64, u64)> {
    relay_tcp_with_initial_and_buffer_size(quic_recv, quic_send, stream, &[], buffer_size).await
}

pub async fn relay_tcp_with_initial(
    quic_recv: RecvStream,
    quic_send: SendStream,
    stream: TcpStream,
    initial_data: &[u8],
) -> Result<(u64, u64)> {
    relay_tcp_with_initial_and_buffer_size(
        quic_recv,
        quic_send,
        stream,
        initial_data,
        DEFAULT_RELAY_BUF_SIZE,
    )
    .await
}

pub async fn relay_tcp_with_initial_and_buffer_size(
    quic_recv: RecvStream,
    quic_send: SendStream,
    mut stream: TcpStream,
    initial_data: &[u8],
    buffer_size: usize,
) -> Result<(u64, u64)> {
    if !initial_data.is_empty() {
        stream.write_all(initial_data).await?;
    }
    let (stream_read, stream_write) = stream.into_split();
    let quic_to_stream = copy_quic_to_shutdown(quic_recv, stream_write);
    let stream_to_quic = copy_buffered_then_finish(stream_read, quic_send, buffer_size);
    let (a, b) = tokio::try_join!(quic_to_stream, stream_to_quic)?;
    debug!(quic_to_stream = a, stream_to_quic = b, "relay completed");
    Ok((a, b))
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
    relay_with_initial_and_buffer_size(
        quic_recv,
        quic_send,
        stream,
        initial_data,
        DEFAULT_RELAY_BUF_SIZE,
    )
    .await
}

pub async fn relay_with_initial_and_buffer_size<S>(
    quic_recv: RecvStream,
    quic_send: SendStream,
    stream: S,
    initial_data: &[u8],
    buffer_size: usize,
) -> Result<(u64, u64)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    relay_inner(quic_recv, quic_send, stream, initial_data, buffer_size).await
}
