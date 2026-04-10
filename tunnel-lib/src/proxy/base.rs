use anyhow::Result;
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub async fn forward_to_client(
    mut send: SendStream,
    recv: RecvStream,
    external_stream: TcpStream,
    relay_buf_size: usize,
) -> Result<()> {
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

pub async fn forward_prefixed<S>(
    mut send: SendStream,
    recv: RecvStream,
    external_stream: S,
    relay_buf_size: usize,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (ext_read, mut ext_write) = tokio::io::split(external_stream);
    let mut quic_recv = BufReader::with_capacity(relay_buf_size, recv);
    let mut ext_read = BufReader::with_capacity(relay_buf_size, ext_read);
    let quic_to_ext = async {
        let bytes = tokio::io::copy_buf(&mut quic_recv, &mut ext_write).await?;
        let _ = ext_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };
    let ext_to_quic = async {
        let bytes = tokio::io::copy_buf(&mut ext_read, &mut send).await?;
        let _ = send.finish();
        Ok::<_, std::io::Error>(bytes)
    };
    let (r1, r2) = tokio::join!(quic_to_ext, ext_to_quic);
    match (r1, r2) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
    }
}

pub async fn forward_with_initial_data(
    mut send: SendStream,
    recv: RecvStream,
    external_stream: TcpStream,
    initial_data: &[u8],
    relay_buf_size: usize,
) -> Result<()> {
    send.write_all(initial_data).await?;
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
