use anyhow::Result;
use quinn::Connection;
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

    let (mut send, recv) = client_conn.open_bi().await?;
    
    send_routing_info(&mut send, &routing_info).await?;
    
    debug!("routing info sent, starting relay");
    
    let (tcp_read, tcp_write) = external_stream.into_split();
    
    let quic_to_tcp = async {
        tokio::io::copy(&mut tokio::io::BufReader::new(recv), &mut tokio::io::BufWriter::new(tcp_write)).await
    };
    
    let tcp_to_quic = async {
        let bytes = tokio::io::copy(&mut tokio::io::BufReader::new(tcp_read), &mut send).await?;
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

    let (mut send, recv) = client_conn.open_bi().await?;
    
    send_routing_info(&mut send, &routing_info).await?;
    
    send.write_all(initial_data).await?;
    
    debug!("routing info and initial data sent, starting relay");
    
    let (tcp_read, tcp_write) = external_stream.into_split();
    
    let quic_to_tcp = async {
        tokio::io::copy(&mut tokio::io::BufReader::new(recv), &mut tokio::io::BufWriter::new(tcp_write)).await
    };
    
    let tcp_to_quic = async {
        let bytes = tokio::io::copy(&mut tokio::io::BufReader::new(tcp_read), &mut send).await?;
        let _ = send.finish();
        Ok::<_, std::io::Error>(bytes)
    };
    
    let (r1, r2) = tokio::join!(quic_to_tcp, tcp_to_quic);
    
    debug!("relay completed: quic->tcp={:?}, tcp->quic={:?}", r1, r2);
    
    Ok(())
}


