use anyhow::{Result, Context};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, warn};
use hyper::{Body, Client};
use hyper_rustls::HttpsConnector;
use hyper::client::HttpConnector;

/// Forward WSS request using tokio-tungstenite
/// This handles WebSocket connections with bidirectional streaming
pub async fn forward_wss_request(
    _https_client: &Client<HttpsConnector<HttpConnector>, Body>,
    request_bytes: &[u8],
    target_uri: &str,
    is_ssl: bool,
) -> Result<Vec<u8>> {
    if !is_ssl {
        anyhow::bail!("WSS requires HTTPS");
    }

    // Parse HTTP Upgrade request
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    
    let parse_result = req.parse(request_bytes)?;
    if !matches!(parse_result, httparse::Status::Complete(_)) {
        anyhow::bail!("Incomplete HTTP headers");
    }

    // Build WebSocket URL
    let ws_url = if target_uri.starts_with("wss://") || target_uri.starts_with("ws://") {
        target_uri.to_string()
    } else {
        format!("wss://{}", target_uri.trim_start_matches("https://"))
    };

    debug!("Connecting to WebSocket: {}", ws_url);

    // Connect to WebSocket server
    let (_ws_stream, _) = connect_async(&ws_url)
        .await
        .with_context(|| format!("Failed to connect to WebSocket: {}", ws_url))?;

    debug!("WebSocket connected");

    // For WSS, we need bidirectional streaming
    // This is a simplified implementation - in practice, you'd need to:
    // 1. Keep the WebSocket connection alive
    // 2. Forward frames bidirectionally
    // 3. Handle WebSocket close frames
    
    if !request_bytes.is_empty() {
        warn!("WSS forwarding: Initial message received but bidirectional streaming not fully implemented");
    }
    
    // Return empty response for now (WSS requires streaming)
    Ok(Vec::new())
}

/// Handle bidirectional WSS streaming
pub async fn forward_wss_stream(
    _https_client: &Client<HttpsConnector<HttpConnector>, Body>,
    target_uri: &str,
    is_ssl: bool,
    mut _tunnel_recv: impl futures::Stream<Item = Result<Vec<u8>, anyhow::Error>> + Unpin,
    mut _tunnel_send: impl futures::Sink<Vec<u8>, Error = anyhow::Error> + Unpin,
) -> Result<()> {
    if !is_ssl {
        anyhow::bail!("WSS requires HTTPS");
    }

    let ws_url = if target_uri.starts_with("wss://") || target_uri.starts_with("ws://") {
        target_uri.to_string()
    } else {
        format!("wss://{}", target_uri.trim_start_matches("https://"))
    };

    debug!("Connecting to WebSocket for streaming: {}", ws_url);

    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .with_context(|| format!("Failed to connect to WebSocket: {}", ws_url))?;

    debug!("WebSocket connected, starting bidirectional forwarding");

    // Split WebSocket stream into sink and stream
    let (mut ws_sink, mut ws_stream_recv) = ws_stream.split();

    // Forward tunnel -> WebSocket
    let tunnel_to_ws = async {
        while let Some(Ok(data)) = _tunnel_recv.next().await {
            let message = if let Ok(text) = std::str::from_utf8(&data) {
                Message::Text(text.to_string())
            } else {
                Message::Binary(data)
            };
            
            if let Err(e) = ws_sink.send(message).await {
                warn!("Failed to send to WebSocket: {}", e);
                break;
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    // Forward WebSocket -> tunnel
    let ws_to_tunnel = async {
        while let Some(msg_result) = ws_stream_recv.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Err(e) = _tunnel_send.send(text.into_bytes()).await {
                        warn!("Failed to send to tunnel: {}", e);
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Err(e) = _tunnel_send.send(data).await {
                        warn!("Failed to send to tunnel: {}", e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    debug!("WebSocket closed");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    warn!("WebSocket stream error: {}", e);
                    break;
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    // Run both directions concurrently
    tokio::select! {
        result = tunnel_to_ws => result,
        result = ws_to_tunnel => result,
    }
}

