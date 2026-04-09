use crate::{send_routing_info, RoutingInfo};
use anyhow::Result;
use quinn::Connection;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::info;

const RELAY_BUF: usize = 65536;

pub async fn forward_to_client(
    client_conn: &Connection,
    routing_info: RoutingInfo,
    external_stream: TcpStream,
) -> Result<()> {
    info!(
        proxy_name = % routing_info.proxy_name, src_addr = % routing_info.src_addr,
        "forwarding to client via QUIC"
    );
    let (mut send, recv) = client_conn.open_bi().await?;
    send_routing_info(&mut send, &routing_info).await?;
    let (tcp_read, mut tcp_write) = external_stream.into_split();
    let mut quic_recv = BufReader::with_capacity(RELAY_BUF, recv);
    let mut tcp_read = BufReader::with_capacity(RELAY_BUF, tcp_read);
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

pub async fn forward_with_initial_data(
    client_conn: &Connection,
    routing_info: RoutingInfo,
    external_stream: TcpStream,
    initial_data: &[u8],
) -> Result<()> {
    info!(
        proxy_name = % routing_info.proxy_name, initial_bytes = initial_data.len(),
        "forwarding to client with initial data"
    );
    let (mut send, recv) = client_conn.open_bi().await?;
    send_routing_info(&mut send, &routing_info).await?;
    // initial_data is the peeked bytes from the TCP stream; send them to the
    // QUIC client first, then relay both directions with 64 KiB buffering.
    send.write_all(initial_data).await?;
    let (tcp_read, mut tcp_write) = external_stream.into_split();
    let mut quic_recv = BufReader::with_capacity(RELAY_BUF, recv);
    let mut tcp_read = BufReader::with_capacity(RELAY_BUF, tcp_read);
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
