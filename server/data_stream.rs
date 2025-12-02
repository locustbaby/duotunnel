use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, warn};
use quinn::{SendStream, RecvStream};
use crate::types::ServerState;
use tunnel_lib::protocol::read_data_stream_header;

/// Handle data stream (Forward Tunnels from Client)
pub async fn handle_data_stream(
    mut send: SendStream,
    mut recv: RecvStream,
    _state: Arc<ServerState>,
) -> Result<()> {
    // 1. Read DataStreamHeader
    let header = read_data_stream_header(&mut recv).await?;
    debug!(
        "Received DataStreamHeader: request_id={}, type={}, host={}",
        header.request_id, header.r#type, header.host
    );
    
    // Note: This is a forward tunnel (Client -> Server -> Target)
    // In the new design, Client sends type and host, Server needs to:
    // 1. Match rules based on type and host to find upstream
    // 2. Connect to upstream
    // 3. Forward data
    
    // For now, this is a placeholder - forward tunnels need rule matching on server side
    warn!("Forward tunnel handler: rule matching on server side not yet implemented");
    
    // TODO: Implement server-side rule matching for forward tunnels
    // This would require server to have rules configured for matching client requests
    // and routing them to upstreams
    
    Ok(())
}

