use anyhow::Result;
use quinn::Connection;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{info, debug};
use crate::{RoutingInfo, send_routing_info};

pub async fn forward_to_client(
    client_conn: &Connection,
    routing_info: RoutingInfo,
    external_stream: TcpStream,
) -> Result<()> {
    info!(
        proxy_name = %routing_info.proxy_name,
        src_addr = %routing_info.src_addr,
        "forwarding to client via QUIC"
    );

    let (mut send, mut recv) = client_conn.open_bi().await?;

    send_routing_info(&mut send, &routing_info).await?;

    debug!("routing info sent, starting relay");

    let (mut tcp_read, mut tcp_write) = external_stream.into_split();

    // No BufReader/BufWriter: QUIC already has internal stream buffering,
    // and TcpStream uses kernel socket buffers. An extra userspace layer
    // only adds memory and an unnecessary copy.
    let quic_to_tcp = async {
        tokio::io::copy(&mut recv, &mut tcp_write).await
    };

    let tcp_to_quic = async {
        let bytes = tokio::io::copy(&mut tcp_read, &mut send).await?;
        let _ = send.finish();
        Ok::<_, std::io::Error>(bytes)
    };

    let (r1, r2) = tokio::join!(quic_to_tcp, tcp_to_quic);

    debug!("relay completed: quic->tcp={:?}, tcp->quic={:?}", r1, r2);

    Ok(())
}

pub async fn forward_with_initial_data(
    client_conn: &Connection,
    routing_info: RoutingInfo,
    external_stream: TcpStream,
    initial_data: &[u8],
) -> Result<()> {
    info!(
        proxy_name = %routing_info.proxy_name,
        initial_bytes = initial_data.len(),
        "forwarding to client with initial data"
    );

    let (mut send, mut recv) = client_conn.open_bi().await?;

    send_routing_info(&mut send, &routing_info).await?;

    send.write_all(initial_data).await?;

    debug!("routing info and initial data sent, starting relay");

    let (mut tcp_read, mut tcp_write) = external_stream.into_split();

    let quic_to_tcp = async {
        tokio::io::copy(&mut recv, &mut tcp_write).await
    };

    let tcp_to_quic = async {
        let bytes = tokio::io::copy(&mut tcp_read, &mut send).await?;
        let _ = send.finish();
        Ok::<_, std::io::Error>(bytes)
    };

    let (r1, r2) = tokio::join!(quic_to_tcp, tcp_to_quic);

    debug!("relay completed: quic->tcp={:?}, tcp->quic={:?}", r1, r2);

    Ok(())
}
