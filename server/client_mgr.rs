use anyhow::Result;
use std::net::SocketAddr;
use tracing::{info, debug, warn};
use crate::types::ServerState;

/// Find client_id by remote address
pub fn find_client_id_by_addr(state: &ServerState, remote_addr: &SocketAddr) -> Option<String> {
    state.clients.iter()
        .find(|entry| entry.value().remote_address() == *remote_addr)
        .map(|entry| entry.key().clone())
}

/// Clean up client registration when connection closes
pub fn cleanup_client_registration(state: &ServerState, client_id: &str) {
    info!("Cleaning up client registration: client_id='{}'", client_id);
    
    // Remove from clients map and explicitly close the connection
    if let Some((_, conn)) = state.clients.remove(client_id) {
        info!("Closing connection for client '{}'", client_id);
        conn.close(0u32.into(), b"server cleanup");
    }
    
    // Get client group before removing (DashMap::remove returns Option<(K, V)>)
    if let Some((_, client_group)) = state.client_groups.remove(client_id) {
        // Remove from group_clients mapping
        if let Some(mut clients) = state.group_clients.get_mut(&client_group) {
            clients.retain(|id| id != client_id);
            if clients.is_empty() {
                drop(clients); // Release the reference before removing
                state.group_clients.remove(&client_group);
            }
        }
        info!("Client '{}' removed from group '{}'", client_id, client_group);
    }
}

/// Select a client from a group (simple round-robin)
pub fn select_client_from_group(
    state: &ServerState,
    group: &str,
) -> Result<String> {
    // Debug: log all available groups
    let available_groups: Vec<String> = state.group_clients.iter().map(|e| e.key().clone()).collect();
    debug!("Available client groups: {:?}", available_groups);
    debug!("Looking for group: {}", group);
    
    let clients = state.group_clients.get(group)
        .ok_or_else(|| {
            let available = available_groups.join(", ");
            anyhow::anyhow!("Client group '{}' not found. Available groups: [{}]", group, available)
        })?;
    
    if clients.is_empty() {
        anyhow::bail!("Client group {} has no clients", group);
    }
    
    // Simple round-robin: just pick the first available client
    // TODO: Implement proper load balancing
    for client_id in clients.iter() {
        if state.clients.contains_key(client_id) {
            return Ok(client_id.clone());
        }
    }
    
    anyhow::bail!("No available clients in group {}", group)
}

/// Probe a client to check if it's still alive
/// If dead, cleans up registration
pub async fn probe_client(state: &ServerState, client_id: &str) -> bool {
    if let Some(conn) = state.clients.get(client_id) {
        // Try to open a bidirectional stream with short timeout
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            conn.open_bi()
        ).await {
            Ok(Ok((mut send, _recv))) => {
                // Immediately close, just probing
                // We finish the stream to be polite, but client might see error reading header
                // This is acceptable for a probe
                let _ = send.finish().await;
                info!("Client {} probe successful", client_id);
                true
            }
            Ok(Err(e)) => {
                warn!("Client {} probe failed (connection error): {}, cleaning up", client_id, e);
                // Drop the reference to conn before cleaning up to avoid deadlock if cleanup needs to lock something
                // (though DashMap handles this fine usually)
                drop(conn); 
                cleanup_client_registration(state, client_id);
                false
            }
            Err(_) => {
                warn!("Client {} probe failed (timeout), cleaning up", client_id);
                drop(conn);
                cleanup_client_registration(state, client_id);
                false
            }
        }
    } else {
        warn!("Client {} not found for probing", client_id);
        false
    }
}

