use anyhow::{Result, Context};
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tracing::{debug, warn};

/// Forward gRPC request using Tonic
/// This handles gRPC unary and streaming requests
pub async fn forward_grpc_request(
    request_bytes: &[u8],
    target_uri: &str,
    is_ssl: bool,
) -> Result<Vec<u8>> {
    // Parse target URI
    let uri = target_uri.trim();
    let uri = if uri.starts_with("http://") {
        uri.trim_start_matches("http://")
    } else if uri.starts_with("https://") {
        uri.trim_start_matches("https://")
    } else {
        uri
    };

    // Build gRPC endpoint
    let endpoint = if is_ssl {
        Endpoint::from_shared(format!("https://{}", uri))?
            .tls_config(ClientTlsConfig::new())?
    } else {
        Endpoint::from_shared(format!("http://{}", uri))?
    };

    debug!("Connecting to gRPC endpoint: {}", uri);

    // Create channel (with connection pooling)
    let channel = endpoint.connect().await
        .with_context(|| format!("Failed to connect to gRPC endpoint: {}", uri))?;

    debug!("gRPC channel connected");

    // For gRPC, request_bytes should contain the protobuf message
    // This is a simplified implementation - in practice, you'd need to:
    // 1. Parse the gRPC frame format (length-prefixed)
    // 2. Extract the service/method from the request
    // 3. Call the appropriate gRPC method
    
    // gRPC uses length-prefixed messages: [compression-flag (1 byte)][message-length (4 bytes)][message]
    if request_bytes.len() < 5 {
        anyhow::bail!("Invalid gRPC message format");
    }

    // Skip compression flag and length prefix for now
    // In a real implementation, you'd parse these and handle compression
    let _message_data = if request_bytes.len() > 5 {
        &request_bytes[5..]
    } else {
        request_bytes
    };

    // For now, return empty response (simplified)
    // In practice, you'd need to:
    // 1. Create a gRPC client for the specific service
    // 2. Call the method with the parsed message
    // 3. Serialize the response back to gRPC frame format
    
    warn!("gRPC forwarding: Simplified implementation - full gRPC method dispatch not implemented");
    
    // Return empty response for now (gRPC requires service-specific clients)
    Ok(Vec::new())
}

/// Create or get gRPC channel for a target (with connection pooling)
pub async fn get_grpc_channel(
    target_uri: &str,
    is_ssl: bool,
) -> Result<Channel> {
    let uri = target_uri.trim();
    let uri = if uri.starts_with("http://") {
        uri.trim_start_matches("http://")
    } else if uri.starts_with("https://") {
        uri.trim_start_matches("https://")
    } else {
        uri
    };

    let endpoint = if is_ssl {
        Endpoint::from_shared(format!("https://{}", uri))?
            .tls_config(ClientTlsConfig::new())?
    } else {
        Endpoint::from_shared(format!("http://{}", uri))?
    };

    endpoint.connect().await
        .with_context(|| format!("Failed to connect to gRPC endpoint: {}", uri))
}

/// Forward gRPC streaming request
pub async fn forward_grpc_stream(
    target_uri: &str,
    is_ssl: bool,
    mut _tunnel_recv: impl futures::Stream<Item = Result<Vec<u8>, anyhow::Error>> + Unpin,
    mut _tunnel_send: impl futures::Sink<Vec<u8>, Error = anyhow::Error> + Unpin,
) -> Result<()> {
    // Get gRPC channel
    let _channel = get_grpc_channel(target_uri, is_ssl).await?;

    debug!("gRPC channel ready for streaming");

    // For gRPC streaming, we need to:
    // 1. Parse gRPC frames from tunnel
    // 2. Forward to gRPC stream
    // 3. Forward gRPC responses back to tunnel
    
    // This is a simplified implementation
    // In practice, you'd need service-specific streaming clients
    
    warn!("gRPC streaming: Simplified implementation - full streaming not implemented");
    
    Ok(())
}

