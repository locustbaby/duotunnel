use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, debug, warn};
use quinn::Connection;
use crate::types::ClientState;
use tunnel_lib::protocol::{write_control_message, read_control_message, write_hash_request};
use tunnel_lib::proto::tunnel::control_message::Payload;

const HASH_CHECK_INTERVAL: Duration = Duration::from_secs(15);  // 15s lightweight hash check
const FULL_SYNC_INTERVAL: Duration = Duration::from_secs(300);  // 5min full config sync
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Handle Control Stream with auto-reconnection, hash checks, and periodic full sync
pub async fn handle_control_stream(
    connection: Arc<Connection>,
    client_id: String, // This is the group_id from config
    client_group_id: String,
    instance_id: String, // This is the unique UUID
    state: Arc<ClientState>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    let mut backoff = INITIAL_BACKOFF;
    
    loop {
        info!("Establishing control stream...");
        
        // Clone receiver for the session
        let session_shutdown_rx = shutdown_rx.resubscribe();
        
        match run_control_stream_session(
            connection.clone(),
            client_id.clone(),
            client_group_id.clone(),
            instance_id.clone(),
            state.clone(),
            session_shutdown_rx,
        ).await {
            Ok(_) => {
                // Check if we shut down because of signal
                if shutdown_rx.try_recv().is_ok() || shutdown_rx.try_recv() == Err(tokio::sync::broadcast::error::TryRecvError::Closed) {
                    info!("Control stream shutdown complete");
                    return Ok(());
                }
                info!("Control stream session ended normally");
                backoff = INITIAL_BACKOFF;
            }
            Err(e) => {
                error!("Control stream session error: {}", e);
            }
        }
        
        // Check shutdown signal again
        if shutdown_rx.try_recv().is_ok() || shutdown_rx.try_recv() == Err(tokio::sync::broadcast::error::TryRecvError::Closed) {
             info!("Shutting down control stream handler");
             return Ok(());
        }
        
        if connection.close_reason().is_some() {
            warn!("QUIC connection closed, attempting to reconnect...");
            info!("Waiting {:?} before reconnection attempt", backoff);
            tokio::time::sleep(backoff).await;
            backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
            continue;
        }
        
        warn!("Control stream closed but connection alive, retrying...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Run a single control stream session with dual-timer sync (hash + full)
async fn run_control_stream_session(
    connection: Arc<Connection>,
    _client_id: String, // Unused, we use instance_id as the protocol client_id
    client_group_id: String,
    instance_id: String,
    state: Arc<ClientState>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    let (mut send, mut recv) = connection.open_bi().await?;
    info!("Control stream opened");
    
    // Send initial full config request
    // Use instance_id as the client_id in the protocol to ensure uniqueness
    send_full_config_request(&mut send, &instance_id, &client_group_id, &state).await?;
    
    // Create dual timers
    let mut hash_check_timer = tokio::time::interval(HASH_CHECK_INTERVAL);
    let mut full_sync_timer = tokio::time::interval(FULL_SYNC_INTERVAL);
    hash_check_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    full_sync_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    
    // Skip first tick (we just sent initial request)
    hash_check_timer.tick().await;
    full_sync_timer.tick().await;
    
    loop {
        tokio::select! {
            // Shutdown signal
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, sending unregister request...");
                send_unregister_request(&mut send, &instance_id).await?;
                // Give it a moment to send
                tokio::time::sleep(Duration::from_millis(100)).await;
                // Finish the stream
                let _ = send.finish().await;
                return Ok(());
            }

            // Every 15s: lightweight hash check
            _ = hash_check_timer.tick() => {
                debug!("Periodic hash check triggered");
                if let Err(e) = send_hash_check_request(&mut send, &instance_id, &client_group_id, &state).await {
                    error!("Failed to send hash check request: {}", e);
                    return Err(e);
                }
            }
            
            // Every 5min: full config sync
            _ = full_sync_timer.tick() => {
                info!("Periodic full config sync triggered (5min)");
                if let Err(e) = send_full_config_request(&mut send, &instance_id, &client_group_id, &state).await {
                    error!("Failed to send full config request: {}", e);
                    return Err(e);
                }
            }
            
            // Receive messages from server
            result = read_control_message(&mut recv) => {
                match result {
                    Ok(msg) => {
                        if let Some(payload) = msg.payload {
                            match payload {
                                Payload::ConfigSyncResponse(resp) => {
                                    handle_config_sync_response(resp, &state).await;
                                }
                                Payload::HashResponse(hash_resp) => {
                                    handle_hash_response(hash_resp, &mut send, &instance_id, &client_group_id, &state).await?;
                                }
                                Payload::IncrementalUpdate(update) => {
                                    handle_incremental_update(update, &state).await;
                                }
                                Payload::ConfigPush(push) => {
                                    debug!("Received ConfigPushNotification for version: {}", push.config_version);
                                    send_full_config_request(&mut send, &instance_id, &client_group_id, &state).await?;
                                }
                                Payload::Heartbeat(_) => {
                                    debug!("Received heartbeat from server");
                                }
                                Payload::ErrorMessage(err) => {
                                    error!("Received error from server: {} - {}", err.code, err.message);
                                }
                                _ => {
                                    warn!("Received unexpected control message");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if connection.close_reason().is_some() {
                            error!("Connection closed, exiting control stream session");
                            return Err(e.into());
                        }
                        warn!("Control stream read error (connection still alive): {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        if connection.close_reason().is_some() {
                            error!("Connection closed during retry, exiting control stream session");
                            return Err(e.into());
                        }
                        return Err(e.into());
                    }
                }
            }
        }
    }
}

/// Send lightweight hash check request (15s interval)
async fn send_hash_check_request(
    send: &mut quinn::SendStream,
    client_id: &str,
    client_group_id: &str,
    state: &ClientState,
) -> Result<()> {
    let current_hash = state.config_hash.read().await.clone();
    
    let hash_request = tunnel_lib::proto::tunnel::ConfigHashRequest {
        client_id: client_id.to_string(),
        group: client_group_id.to_string(),
        current_hash,
    };
    
    let control_msg = tunnel_lib::proto::tunnel::ControlMessage {
        payload: Some(Payload::HashRequest(hash_request)),
    };
    
    write_control_message(send, &control_msg).await?;
    debug!("Sent ConfigHashRequest");
    Ok(())
}

/// Send full config request (5min interval or on-demand)
async fn send_full_config_request(
    send: &mut quinn::SendStream,
    client_id: &str,
    client_group_id: &str,
    state: &ClientState,
) -> Result<()> {
    let current_version = state.config_version.read().await.clone();
    let current_hash = state.config_hash.read().await.clone();
    
    let config_sync = tunnel_lib::proto::tunnel::ConfigSyncRequest {
        client_id: client_id.to_string(),
        group: client_group_id.to_string(),
        config_version: current_version,
        current_hash,
        request_full: true,
    };
    
    let control_msg = tunnel_lib::proto::tunnel::ControlMessage {
        payload: Some(Payload::ConfigSync(config_sync)),
    };
    
    write_control_message(send, &control_msg).await?;
    debug!("Sent full ConfigSyncRequest");
    Ok(())
}

/// Handle hash response from server
async fn handle_hash_response(
    resp: tunnel_lib::proto::tunnel::ConfigHashResponse,
    send: &mut quinn::SendStream,
    client_id: &str,
    client_group_id: &str,
    state: &ClientState,
) -> Result<()> {
    if resp.needs_update {
        info!("Hash mismatch detected, requesting full config update");
        send_full_config_request(send, client_id, client_group_id, state).await?;
    } else {
        debug!("Config hash matches, no update needed");
    }
    Ok(())
}

/// Handle full config sync response (only update if hash changed)
async fn handle_config_sync_response(
    resp: tunnel_lib::proto::tunnel::ConfigSyncResponse,
    state: &ClientState,
) {
    let current_hash = state.config_hash.read().await.clone();
    
    // Only update if hash changed
    if resp.config_hash != current_hash {
        info!("Config hash changed: {} -> {}", 
            if current_hash.is_empty() { "empty" } else { &current_hash }, 
            &resp.config_hash);
        
        // Update rules
        state.rules.clear();
        for rule in &resp.rules {
            state.rules.insert(rule.rule_id.clone(), rule.clone());
        }
        
        // Update upstreams
        state.upstreams.clear();
        for upstream in &resp.upstreams {
            state.upstreams.insert(upstream.name.clone(), upstream.clone());
        }
        
        // Update config version and hash
        *state.config_version.write().await = resp.config_version.clone();
        *state.config_hash.write().await = resp.config_hash.clone();
        
        info!("Updated config: {} rules, {} upstreams (version: {}, hash: {})", 
            resp.rules.len(), resp.upstreams.len(), resp.config_version, &resp.config_hash[..8]);
        
        // Warmup connection pools for all upstream targets
        let rules: Vec<_> = state.rules.iter().map(|r| r.value().clone()).collect();
        let upstreams: Vec<_> = state.upstreams.iter().map(|u| u.value().clone()).collect();
        let http_client = state.http_client.clone();
        let https_client = state.https_client.clone();
        
        tokio::spawn(async move {
            crate::warmup::warmup_connection_pools(&http_client, &https_client, &rules, &upstreams).await;
        });
    } else {
        debug!("Config hash unchanged: {}", if current_hash.is_empty() { "empty" } else { &current_hash[..8] });
    }
}

/// Handle incremental config update from server
async fn handle_incremental_update(
    update: tunnel_lib::proto::tunnel::IncrementalConfigUpdate,
    state: &ClientState,
) {
    info!("Received incremental config update");
    
    // Delete rules
    for rule_id in &update.deleted_rule_ids {
        state.rules.remove(rule_id);
        debug!("Deleted rule: {}", rule_id);
    }
    
    // Add/update rules
    for rule in &update.added_rules {
        state.rules.insert(rule.rule_id.clone(), rule.clone());
        debug!("Added rule: {}", rule.rule_id);
    }
    for rule in &update.updated_rules {
        state.rules.insert(rule.rule_id.clone(), rule.clone());
        debug!("Updated rule: {}", rule.rule_id);
    }
    
    // Delete upstreams
    for upstream_name in &update.deleted_upstream_names {
        state.upstreams.remove(upstream_name);
        debug!("Deleted upstream: {}", upstream_name);
    }
    
    // Add/update upstreams
    for upstream in &update.added_upstreams {
        state.upstreams.insert(upstream.name.clone(), upstream.clone());
        debug!("Added upstream: {}", upstream.name);
    }
    for upstream in &update.updated_upstreams {
        state.upstreams.insert(upstream.name.clone(), upstream.clone());
        debug!("Updated upstream: {}", upstream.name);
    }
    
    // Update hash
    *state.config_hash.write().await = update.config_hash.clone();
    
    info!("Applied incremental update: +{} rules, ~{} rules, -{} rules, +{} upstreams, ~{} upstreams, -{} upstreams (new hash: {})",
        update.added_rules.len(), update.updated_rules.len(), update.deleted_rule_ids.len(),
        update.added_upstreams.len(), update.updated_upstreams.len(), update.deleted_upstream_names.len(),
        &update.config_hash[..8]);
    
    // Warmup connection pools for updated/added upstream targets
    let rules: Vec<_> = state.rules.iter().map(|r| r.value().clone()).collect();
    let upstreams: Vec<_> = state.upstreams.iter().map(|u| u.value().clone()).collect();
    let http_client = state.http_client.clone();
    let https_client = state.https_client.clone();
    
    tokio::spawn(async move {
        crate::warmup::warmup_connection_pools(&http_client, &https_client, &rules, &upstreams).await;
    });
}

/// Send unregister request for graceful shutdown
async fn send_unregister_request(
    send: &mut quinn::SendStream,
    client_id: &str,
) -> Result<()> {
    let unregister = tunnel_lib::proto::tunnel::UnregisterRequest {
        client_id: client_id.to_string(),
        reason: "graceful shutdown".to_string(),
    };
    
    let control_msg = tunnel_lib::proto::tunnel::ControlMessage {
        payload: Some(Payload::Unregister(unregister)),
    };
    
    write_control_message(send, &control_msg).await?;
    info!("Sent UnregisterRequest");
    Ok(())
}

