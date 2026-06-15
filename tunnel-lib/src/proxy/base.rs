use crate::engine::copy::{copy_buffered_then_finish, copy_quic_to_shutdown};
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use tokio::net::TcpStream;

/// Core ingress relay: optionally writes `initial_data` to QUIC send (pushing
/// already-peeked bytes to the client) before entering the bidirectional copy loop.
async fn forward_inner(
    mut send: SendStream,
    recv: RecvStream,
    external_stream: TcpStream,
    initial_data: Option<&[u8]>,
    relay_buf_size: usize,
) -> Result<()> {
    if let Some(data) = initial_data {
        send.write_all(data).await?;
    }
    let (tcp_read, tcp_write) = external_stream.into_split();
    let quic_to_tcp = copy_quic_to_shutdown(recv, tcp_write);
    let tcp_to_quic = copy_buffered_then_finish(tcp_read, send, relay_buf_size);
    tokio::try_join!(quic_to_tcp, tcp_to_quic)?;
    Ok(())
}

pub async fn forward_to_client(
    send: SendStream,
    recv: RecvStream,
    external_stream: TcpStream,
    relay_buf_size: usize,
) -> Result<()> {
    forward_inner(send, recv, external_stream, None, relay_buf_size).await
}

pub async fn forward_with_initial_data(
    send: SendStream,
    recv: RecvStream,
    external_stream: TcpStream,
    initial_data: &[u8],
    relay_buf_size: usize,
) -> Result<()> {
    forward_inner(
        send,
        recv,
        external_stream,
        Some(initial_data),
        relay_buf_size,
    )
    .await
}

pub async fn forward_prefixed_to_client(
    send: SendStream,
    recv: RecvStream,
    external_stream: crate::PrefixedReadWrite<TcpStream>,
    relay_buf_size: usize,
) -> Result<()> {
    let (tcp_read, tcp_write) = external_stream.into_split();
    let quic_to_tcp = copy_quic_to_shutdown(recv, tcp_write);
    let tcp_to_quic = copy_buffered_then_finish(tcp_read, send, relay_buf_size);
    tokio::try_join!(quic_to_tcp, tcp_to_quic)?;
    Ok(())
}
