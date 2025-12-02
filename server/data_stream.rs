use anyhow::Result;
use std::sync::Arc;
use tracing::warn;
use quinn::{SendStream, RecvStream};
use crate::types::ServerState;
// Forward tunnels from client are now handled using frame protocol
// This handler is kept for backward compatibility but forward tunnels
// should be initiated from server side (reverse tunnels)

/// Handle data stream (Forward Tunnels from Client)
/// Note: Forward tunnels are now handled using frame protocol
/// This is a placeholder for backward compatibility
pub async fn handle_data_stream(
    _send: SendStream,
    _recv: RecvStream,
    _state: Arc<ServerState>,
) -> Result<()> {
    // Forward tunnels (Client -> Server -> Target) are not currently implemented
    // The new frame protocol is designed for reverse tunnels (Server -> Client -> Target)
    // If forward tunnels are needed, they should be implemented using the frame protocol
    // similar to how reverse tunnels work
    
    warn!("Forward tunnel handler: not implemented with frame protocol yet");
    warn!("Forward tunnels should use reverse tunnel pattern (Server -> Client -> Target)");
    
    // TODO: If forward tunnels are needed, implement using frame protocol:
    // 1. Read routing frame from client
    // 2. Read request frames and reassemble
    // 3. Match rules and connect to upstream
    // 4. Send request to upstream
    // 5. Read response and send back as frames
    
    Ok(())
}
