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

    match (sent, recv) {
        (Ok(a), Ok(b)) => Ok((a, b)),
        (Err(e1), Err(e2)) => {
            debug!("relay: both directions failed; suppressed: {}", e2);
            Err(e1)
        }
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
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

    match (sent, recv) {
        (Ok(a), Ok(b)) => Ok((a, b)),
        (Err(e1), Err(e2)) => {
            debug!("relay_quic_to_tcp: both directions failed; suppressed: {}", e2);
            Err(e1.into())
        }
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
    }
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

    match (sent, recv) {
        (Ok(a), Ok(b)) => Ok((a, b)),
        (Err(e1), Err(e2)) => {
            debug!("relay_with_first_data: both directions failed; suppressed: {}", e2);
            Err(e1.into())
        }
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
    }
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

    // ── relay_unidirectional: byte count ─────────────────────────────────────

    #[tokio::test]
    async fn test_relay_unidirectional_returns_byte_count() {
        let data = b"count me";
        let (mut src, dst) = tokio::io::duplex(64);

        tokio::spawn(async move {
            src.write_all(data).await.unwrap();
            src.shutdown().await.unwrap();
        });

        // relay_unidirectional reads from `dst` (acting as reader) into a sink.
        // Use a duplex as the writer so we can discard bytes.
        let (mut sink_read, sink_write) = tokio::io::duplex(64);
        let count = relay_unidirectional(dst, sink_write).await.unwrap();
        assert_eq!(count, data.len() as u64);

        // Confirm the data actually arrived at the sink
        let mut received = Vec::new();
        sink_read.read_to_end(&mut received).await.unwrap();
        assert_eq!(&received, data);
    }

    #[tokio::test]
    async fn test_relay_unidirectional_empty_stream() {
        let (mut src, dst) = tokio::io::duplex(64);
        src.shutdown().await.unwrap(); // EOF immediately

        let (_, sink_write) = tokio::io::duplex(64);
        let count = relay_unidirectional(dst, sink_write).await.unwrap();
        assert_eq!(count, 0, "empty stream must transfer 0 bytes");
    }

    // ── relay: bidirectional ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_relay_bidirectional_both_directions() {
        // side_a writes A_DATA → should arrive at side_b.
        // side_b writes B_DATA → should arrive at side_a.
        let a_data = b"from-a-to-b";
        let b_data = b"from-b-to-a";

        let (side_a, relay_a) = tokio::io::duplex(256);
        let (side_b, relay_b) = tokio::io::duplex(256);

        let (mut side_a_read, mut side_a_write) = tokio::io::split(side_a);
        let (mut side_b_read, mut side_b_write) = tokio::io::split(side_b);

        // Writer task for side_a
        tokio::spawn(async move {
            side_a_write.write_all(a_data).await.unwrap();
            side_a_write.shutdown().await.unwrap();
        });

        // Writer task for side_b
        tokio::spawn(async move {
            side_b_write.write_all(b_data).await.unwrap();
            side_b_write.shutdown().await.unwrap();
        });

        // Run the relay between the two relay ends
        let (sent, recv) = relay(relay_a, relay_b).await.unwrap();

        // sent = bytes from relay_a → relay_b (i.e. a_data)
        // recv = bytes from relay_b → relay_a (i.e. b_data)
        assert_eq!(sent, a_data.len() as u64, "sent byte count mismatch");
        assert_eq!(recv, b_data.len() as u64, "recv byte count mismatch");

        // Verify data arrived on the correct side
        let mut a_received = Vec::new();
        side_a_read.read_to_end(&mut a_received).await.unwrap();
        assert_eq!(&a_received, b_data);

        let mut b_received = Vec::new();
        side_b_read.read_to_end(&mut b_received).await.unwrap();
        assert_eq!(&b_received, a_data);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_relay_bidirectional_large_payload() {
        // 256 KB in each direction — exercises internal buffering.
        //
        // Topology (4 duplex endpoints, all independent):
        //   side_a (writer + reader) ←→ relay_a ←[relay]→ relay_b ←→ side_b (writer + reader)
        //
        // Writers and readers run in separate tasks so they don't block each other:
        //   side_a_write --256KB--> relay_a --relay--> relay_b --> side_b_read
        //   side_b_write --256KB--> relay_b --relay--> relay_a --> side_a_read
        let size = 256 * 1024usize;
        let a_data: Vec<u8> = (0u8..=255).cycle().take(size).collect();
        let b_data: Vec<u8> = (0u8..=255).cycle().take(size).collect();

        let (side_a, relay_a) = tokio::io::duplex(64 * 1024);
        let (side_b, relay_b) = tokio::io::duplex(64 * 1024);

        let (mut side_a_read, mut side_a_write) = tokio::io::split(side_a);
        let (mut side_b_read, mut side_b_write) = tokio::io::split(side_b);

        // Spawn writers and readers as separate tasks — they must run concurrently with
        // relay() to avoid blocking on full duplex buffers.
        let a_clone = a_data.clone();
        let a_writer = tokio::spawn(async move {
            side_a_write.write_all(&a_clone).await.unwrap();
            side_a_write.shutdown().await.unwrap();
        });

        let b_clone = b_data.clone();
        let b_writer = tokio::spawn(async move {
            side_b_write.write_all(&b_clone).await.unwrap();
            side_b_write.shutdown().await.unwrap();
        });

        // Readers drain what relay delivers; without this the relay's output buffers fill up.
        let a_reader = tokio::spawn(async move {
            let mut buf = Vec::new();
            side_a_read.read_to_end(&mut buf).await.unwrap();
            buf
        });

        let b_reader = tokio::spawn(async move {
            let mut buf = Vec::new();
            side_b_read.read_to_end(&mut buf).await.unwrap();
            buf
        });

        let (sent, recv) = relay(relay_a, relay_b).await.unwrap();

        a_writer.await.unwrap();
        b_writer.await.unwrap();
        let a_received = a_reader.await.unwrap();
        let b_received = b_reader.await.unwrap();

        assert_eq!(sent, size as u64, "a→b byte count");
        assert_eq!(recv, size as u64, "b→a byte count");
        assert_eq!(b_received, a_data, "side_b received a_data");
        assert_eq!(a_received, b_data, "side_a received b_data");
    }

    #[tokio::test]
    async fn test_relay_bidirectional_empty_both_sides() {
        let (side_a, relay_a) = tokio::io::duplex(64);
        let (side_b, relay_b) = tokio::io::duplex(64);

        // Immediately shut down both sides — relay sees EOF on first read
        let (_, mut side_a_write) = tokio::io::split(side_a);
        let (_, mut side_b_write) = tokio::io::split(side_b);
        side_a_write.shutdown().await.unwrap();
        side_b_write.shutdown().await.unwrap();

        let (sent, recv) = relay(relay_a, relay_b).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(recv, 0);
    }
}
