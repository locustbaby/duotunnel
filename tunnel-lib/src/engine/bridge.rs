use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tracing::{debug, trace};

pub async fn relay<A, B>(stream_a: A, stream_b: B) -> std::io::Result<(u64, u64)>
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    let (mut a_read, mut a_write) = tokio::io::split(stream_a);
    let (mut b_read, mut b_write) = tokio::io::split(stream_b);

    let a_to_b = async {
        let bytes = tokio::io::copy(&mut a_read, &mut b_write).await?;
        let _ = b_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };

    let b_to_a = async {
        let bytes = tokio::io::copy(&mut b_read, &mut a_write).await?;
        let _ = a_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };

    let (sent, recv) = tokio::join!(a_to_b, b_to_a);
    
    debug!("relay completed: sent={:?}, recv={:?}", sent, recv);
    
    Ok((sent.unwrap_or(0), recv.unwrap_or(0)))
}

pub async fn relay_unidirectional<R, W>(mut reader: R, mut writer: W) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let bytes = tokio::io::copy(&mut reader, &mut writer).await?;
    let _ = writer.shutdown().await;
    trace!("unidirectional relay completed: {} bytes", bytes);
    Ok(bytes)
}

pub struct QuicBiStream {
    pub send: quinn::SendStream,
    pub recv: quinn::RecvStream,
}

impl QuicBiStream {
    pub fn new(send: quinn::SendStream, recv: quinn::RecvStream) -> Self {
        Self { send, recv }
    }

    pub fn split(self) -> (quinn::SendStream, quinn::RecvStream) {
        (self.send, self.recv)
    }
}

pub async fn relay_quic_to_tcp(
    mut quic_recv: quinn::RecvStream,
    mut quic_send: quinn::SendStream,
    tcp_stream: tokio::net::TcpStream,
) -> anyhow::Result<(u64, u64)> {
    let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();

    let quic_to_tcp = async {
        let bytes = tokio::io::copy(&mut quic_recv, &mut tcp_write).await?;
        let _ = tcp_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };

    let tcp_to_quic = async {
        let bytes = tokio::io::copy(&mut tcp_read, &mut quic_send).await?;
        let _ = quic_send.finish();
        Ok::<_, std::io::Error>(bytes)
    };

    let (sent, recv) = tokio::join!(quic_to_tcp, tcp_to_quic);
    
    debug!("quic-tcp relay: quic->tcp={:?}, tcp->quic={:?}", sent, recv);
    
    Ok((sent.unwrap_or(0), recv.unwrap_or(0)))
}

pub async fn relay_with_first_data(
    mut quic_recv: quinn::RecvStream,
    mut quic_send: quinn::SendStream,
    mut tcp_stream: tokio::net::TcpStream,
    first_data: Option<&[u8]>,
) -> anyhow::Result<(u64, u64)> {
    if let Some(data) = first_data {
        tcp_stream.write_all(data).await?;
    }

    let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();

    let quic_to_tcp = async {
        let bytes = tokio::io::copy(&mut quic_recv, &mut tcp_write).await?;
        let _ = tcp_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };

    let tcp_to_quic = async {
        let bytes = tokio::io::copy(&mut tcp_read, &mut quic_send).await?;
        let _ = quic_send.finish();
        Ok::<_, std::io::Error>(bytes)
    };

    let (sent, recv) = tokio::join!(quic_to_tcp, tcp_to_quic);
    
    Ok((sent.unwrap_or(0), recv.unwrap_or(0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_relay_unidirectional() {
        let data = b"hello world";
        let (mut client, mut server) = tokio::io::duplex(64);
        
        tokio::spawn(async move {
            client.write_all(data).await.unwrap();
            client.shutdown().await.unwrap();
        });

        let mut buf = Vec::new();
        server.read_to_end(&mut buf).await.unwrap();
        assert_eq!(&buf, data);
    }
}
