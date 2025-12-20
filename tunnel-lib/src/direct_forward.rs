use anyhow::Result;
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncRead, AsyncWrite};
use tracing::{debug, error, warn, info};
use crate::frame::read_frame;
use crate::protocol::RoutingInfo;
use std::time::Duration;

/// Forward raw data bidirectionally between QUIC stream and a stream that implements AsyncRead + AsyncWrite
/// This function forwards raw data without encapsulation
pub async fn forward_bidirectional_raw<S>(
    mut quic_recv: RecvStream,
    mut quic_send: SendStream,
    stream: S,
    request_id: &str,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    const FORWARD_TIMEOUT: Duration = Duration::from_secs(300);
    
    // Clone request_id for use in spawned tasks
    let request_id1 = request_id.to_string();
    let request_id2 = request_id.to_string();
    
    // Split stream for bidirectional forwarding
    let (mut stream_recv, mut stream_send) = tokio::io::split(stream);
    
    // Task 1: Forward from QUIC to TCP
    let quic_to_tcp_task = tokio::spawn(async move {
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
        let mut total_bytes = 0u64;
        
        loop {
            match tokio::time::timeout(FORWARD_TIMEOUT, quic_recv.read(&mut buffer)).await {
                Ok(Ok(Some(n))) => {
                    if n == 0 {
                        debug!("[{}] QUIC stream ended", request_id1);
                        break;
                    }
                    
                    total_bytes += n as u64;
                    debug!("[{}] Read {} bytes from QUIC (total: {}), forwarding to TCP", request_id1, n, total_bytes);
                    
                    if let Err(e) = stream_send.write_all(&buffer[..n]).await {
                        error!("[{}] Error writing to stream: {}", request_id1, e);
                        break;
                    }
                    
                    if let Err(e) = stream_send.flush().await {
                        error!("[{}] Error flushing stream: {}", request_id1, e);
                        break;
                    }
                }
                Ok(Ok(None)) => {
                    debug!("[{}] QUIC stream closed (total bytes forwarded: {})", request_id1, total_bytes);
                    break;
                }
                Ok(Err(e)) => {
                    error!("[{}] Error reading from QUIC: {}", request_id1, e);
                    break;
                }
                Err(_) => {
                    warn!("[{}] QUIC to TCP forward timeout (total bytes forwarded: {})", request_id1, total_bytes);
                    break;
                }
            }
        }
        
        info!("[{}] QUIC to TCP forwarding completed (total: {} bytes), closing TCP send side", request_id1, total_bytes);
        // Flush and close stream send side to signal request completion
        // This tells upstream server that request is complete
        let _ = stream_send.flush().await;
        let _ = stream_send.shutdown().await;
    });
    
    // Task 2: Forward from TCP to QUIC
    let tcp_to_quic_task = tokio::spawn(async move {
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
        let mut total_bytes = 0u64;
        
        info!("[{}] TCP to QUIC task started, waiting for response from upstream", request_id2);
        
        loop {
            match tokio::time::timeout(FORWARD_TIMEOUT, stream_recv.read(&mut buffer)).await {
                Ok(Ok(n)) => {
                    if n == 0 {
                        debug!("[{}] Stream ended (total bytes forwarded: {})", request_id2, total_bytes);
                        break;
                    }
                    
                    total_bytes += n as u64;
                    debug!("[{}] Read {} bytes from TCP (total: {}), forwarding to QUIC", request_id2, n, total_bytes);
                    
                    if let Err(e) = quic_send.write_all(&buffer[..n]).await {
                        error!("[{}] Error writing to QUIC: {}", request_id2, e);
                        break;
                    }
                }
                Ok(Err(e)) => {
                    error!("[{}] Error reading from stream: {}", request_id2, e);
                    break;
                }
                Err(_) => {
                    warn!("[{}] TCP to QUIC forward timeout (total bytes forwarded: {})", request_id2, total_bytes);
                    break;
                }
            }
        }
        
        info!("[{}] TCP to QUIC forwarding completed (total: {} bytes), finishing QUIC send stream", request_id2, total_bytes);
        // Finish QUIC send stream
        let _ = quic_send.finish();
    });
    
    // Wait for quic_to_tcp_task to complete first (request sending phase)
    // This ensures TCP send side is closed, signaling upstream that request is complete
    let quic_to_tcp_result = quic_to_tcp_task.await;
    if let Err(e) = quic_to_tcp_result {
        error!("[{}] QUIC to TCP task error: {}", request_id, e);
        // Even if quic_to_tcp_task failed, we should still wait for response
    }
    
    info!("[{}] Request forwarding completed, waiting for response", request_id);
    
    // Then wait for tcp_to_quic_task to complete (response receiving phase)
    // This ensures response is forwarded back to client
    let tcp_to_quic_result = tcp_to_quic_task.await;
    if let Err(e) = tcp_to_quic_result {
        error!("[{}] TCP to QUIC task error: {}", request_id, e);
    }
    
    Ok(())
}

/// Read RoutingFrame and return RoutingInfo
pub async fn read_routing_frame(
    recv: &mut RecvStream,
) -> Result<(RoutingInfo, u64)> {
    const ROUTING_TIMEOUT: Duration = Duration::from_secs(10);
    
    let routing_frame = tokio::time::timeout(ROUTING_TIMEOUT, read_frame(recv)).await??;
    let routing_info = RoutingInfo::decode(&routing_frame.payload)?;
    
    Ok((routing_info, routing_frame.session_id))
}

