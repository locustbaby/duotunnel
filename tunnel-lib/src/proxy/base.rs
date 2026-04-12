use anyhow::Result;
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncWriteExt, BufReader};
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
    let (tcp_read, mut tcp_write) = external_stream.into_split();
    let mut quic_recv = BufReader::with_capacity(relay_buf_size, recv);
    let mut tcp_read = BufReader::with_capacity(relay_buf_size, tcp_read);
    let quic_to_tcp = async {
        let bytes = tokio::io::copy_buf(&mut quic_recv, &mut tcp_write).await?;
        let _ = tcp_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };
    let tcp_to_quic = async {
        let bytes = tokio::io::copy_buf(&mut tcp_read, &mut send).await?;
        let _ = send.finish();
        Ok::<_, std::io::Error>(bytes)
    };
    let (r1, r2) = tokio::join!(quic_to_tcp, tcp_to_quic);
    match (r1, r2) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
    }
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
    forward_inner(send, recv, external_stream, Some(initial_data), relay_buf_size).await
}
