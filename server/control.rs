use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, error, debug, warn};
use quinn::{Connection, SendStream, RecvStream};
use crate::types::ServerState;
use tunnel_lib::protocol::{write_control_message, read_control_message};
use tunnel_lib::proto::tunnel::control_message::Payload;
use tunnel_lib::proto::tunnel::ConfigSyncResponse;


pub fn find_client_id_by_addr(state: &ServerState, remote_addr: &SocketAddr) -> Option<String> {
    state.addr_to_client.get(remote_addr).map(|r| r.value().clone())
}

pub fn cleanup_client_registration(state: &ServerState, client_id: &str) {
    info!("Cleaning up client registration: client_id='{}'", client_id);
    
    if let Some((_, conn)) = state.clients.remove(client_id) {
        let remote_addr = conn.remote_address();
        state.addr_to_client.remove(&remote_addr);
        info!("Closing connection for client '{}'", client_id);
        conn.close(0u32.into(), b"server cleanup");
    }
    
    if let Some((_, client_group)) = state.client_groups.remove(client_id) {
        if let Some(mut clients) = state.group_clients.get_mut(&client_group) {
            clients.retain(|id| id != client_id);
            if clients.is_empty() {
                drop(clients);
                state.group_clients.remove(&client_group);
            }
        }
        info!("Client '{}' removed from group '{}'", client_id, client_group);
    }
}

pub fn select_client_from_group(
    state: &ServerState,
    group: &str,
) -> Result<String> {
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
    
    for client_id in clients.iter() {
        if state.clients.contains_key(client_id) {
            return Ok(client_id.clone());
        }
    }
    
    anyhow::bail!("No available clients in group {}", group)
}

pub async fn probe_client(state: &ServerState, client_id: &str) -> bool {
    if let Some(conn) = state.clients.get(client_id) {
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            conn.open_bi()
        ).await {
            Ok(Ok((mut send, _recv))) => {
                let _ = send.finish();
                info!("Client {} probe successful", client_id);
                true
            }
            Ok(Err(e)) => {
                warn!("Client {} probe failed (connection error): {}, cleaning up", client_id, e);
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



pub async fn handle_control_stream(
    mut control_send: SendStream,
    mut control_recv: RecvStream,
    conn: Connection,
    remote_addr: std::net::SocketAddr,
    state: Arc<ServerState>,
) -> Result<()> {
    info!("Control stream handler started");
    

    let control_msg = match read_control_message(&mut control_recv).await {
        Ok(msg) => msg,
        Err(e) => {
            error!("Failed to read initial ConfigSyncRequest: {}", e);
            return Err(e.into());
        }
    };
    
    let (requested_client_id, client_group) = match control_msg.payload {
        Some(Payload::ConfigSync(req)) => {
            info!("Received ConfigSyncRequest from client_id: {}, group: {}, version: {}", 
                req.client_id, req.group, req.config_version);
            (req.client_id.clone(), req.group.clone())
        }
        _ => {
            warn!("Expected ConfigSyncRequest, got something else");
            return Err(anyhow::anyhow!("Expected ConfigSyncRequest"));
        }
    };
    

    let client_id = requested_client_id;
    

    let display_name = format!("{}-{}-{}", client_group, remote_addr.ip(), &client_id[..8.min(client_id.len())]);
    

    info!("Registering client: client_id='{}' (display: '{}'), group='{}'", 
        client_id, display_name, client_group);
    

    if let Some(old_conn) = state.clients.get(&client_id) {

        if old_conn.close_reason().is_none() {
            warn!("Client ID '{}' already exists with active connection, closing old connection", client_id);
            old_conn.close(0u32.into(), b"replaced by new connection");
        }

        cleanup_client_registration(&state, &client_id);
        info!("Cleaned up old registration for client '{}'", client_id);
    }
    

    state.clients.insert(client_id.clone(), conn.clone());
    state.client_groups.insert(client_id.clone(), client_group.clone());
    state.addr_to_client.insert(remote_addr, client_id.clone());
    
    state.group_clients
        .entry(client_group.clone())
        .or_insert_with(Vec::new)
        .retain(|id| id != &client_id);
    state.group_clients
        .get_mut(&client_group)
        .unwrap()
        .push(client_id.clone());
    
    info!("Client registered successfully. Total clients in group '{}': {}", 
        client_group, state.group_clients.get(&client_group).map(|v| v.len()).unwrap_or(0));
    

    let (client_rules_init, client_upstreams_init, client_config_version_init) = state.client_configs
        .get(&client_group)
        .map(|(r, u, v)| (r.clone(), u.clone(), v.clone()))
        .unwrap_or_else(|| {
            warn!("No client config found for group '{}', sending empty config", client_group);
            (Vec::new(), Vec::new(), state.config_version.clone())
        });
    
    info!("Sending config to client group '{}': {} rules, {} upstreams, version: {}", 
        client_group, client_rules_init.len(), client_upstreams_init.len(), client_config_version_init);
    
    let config_hash_init = tunnel_lib::hash::calculate_config_hash(&client_rules_init, &client_upstreams_init);
    
    let response_init = ConfigSyncResponse {
        config_version: client_config_version_init.clone(),
        config_hash: config_hash_init,
        rules: client_rules_init.clone(),
        upstreams: client_upstreams_init.clone(),
    };
    
    let response_msg_init = tunnel_lib::proto::tunnel::ControlMessage {
        payload: Some(Payload::ConfigSyncResponse(response_init)),
    };
    
    if let Err(e) = write_control_message(&mut control_send, &response_msg_init).await {
        error!("Failed to send initial ConfigSyncResponse: {}", e);
        return Err(e.into());
    }
    info!("Sent initial ConfigSyncResponse to client_id: {}", client_id);
    

    loop {
        match read_control_message(&mut control_recv).await {
            Ok(msg) => {
                if let Some(payload) = msg.payload {
                    match payload {
                        Payload::HashRequest(req) => {
                            debug!("Received ConfigHashRequest from client_id: {}", req.client_id);
                            

                            let client_group_for_hash = state.client_groups
                                .get(&req.client_id)
                                .map(|g| g.value().clone())
                                .unwrap_or_else(|| req.group.clone());
                            

                            let (client_rules, client_upstreams, _) = state.client_configs
                                .get(&client_group_for_hash)
                                .map(|(r, u, v)| (r.clone(), u.clone(), v.clone()))
                                .unwrap_or_else(|| {
                                    warn!("No client config found for group '{}'", client_group_for_hash);
                                    (Vec::new(), Vec::new(), state.config_version.clone())
                                });
                            
                            let server_hash = tunnel_lib::hash::calculate_config_hash(&client_rules, &client_upstreams);
                            let needs_update = req.current_hash != server_hash;
                            
                            if needs_update {
                                debug!("Client {} hash mismatch: client='{}', server='{}'", 
                                    req.client_id, &req.current_hash[..8.min(req.current_hash.len())], &server_hash[..8]);
                            } else {
                                debug!("Client {} hash matches: {}", req.client_id, &server_hash[..8]);
                            }
                            
                            let hash_response = tunnel_lib::proto::tunnel::ConfigHashResponse {
                                config_hash: server_hash,
                                needs_update,
                            };
                            
                            let response_msg = tunnel_lib::proto::tunnel::ControlMessage {
                                payload: Some(Payload::HashResponse(hash_response)),
                            };
                            
                            if let Err(e) = write_control_message(&mut control_send, &response_msg).await {
                                error!("Failed to send ConfigHashResponse: {}", e);
                                if conn.close_reason().is_some() {
                                    info!("Connection closed, exiting control stream handler");
                                    break;
                                }
                                continue;
                            }
                        }
                        Payload::ConfigSync(req) => {
                            debug!("Received ConfigSyncRequest from client_id: {}, version: {}", 
                                req.client_id, req.config_version);
                            

                            let client_group_for_sync = state.client_groups
                                .get(&req.client_id)
                                .map(|g| g.value().clone())
                                .unwrap_or_else(|| {
                                    warn!("Client '{}' not found in groups, using requested group", req.client_id);
                                    req.group.clone()
                                });
                            

                            let (client_rules, client_upstreams, client_config_version) = state.client_configs
                                .get(&client_group_for_sync)
                                .map(|(r, u, v)| (r.clone(), u.clone(), v.clone()))
                                .unwrap_or_else(|| {
                                    warn!("No client config found for group '{}'", client_group_for_sync);
                                    (Vec::new(), Vec::new(), state.config_version.clone())
                                });
                            

                            if req.config_version != client_config_version {
                                info!("Client {} config version mismatch: client has '{}', server has '{}', sending update",
                                    req.client_id, req.config_version, client_config_version);
                                

                                let config_hash = tunnel_lib::hash::calculate_config_hash(&client_rules, &client_upstreams);
                                
                                let response = ConfigSyncResponse {
                                    config_version: client_config_version,
                                    config_hash,
                                    rules: client_rules,
                                    upstreams: client_upstreams,
                                };
                                
                                let response_msg = tunnel_lib::proto::tunnel::ControlMessage {
                                    payload: Some(Payload::ConfigSyncResponse(response)),
                                };
                                
                                if let Err(e) = write_control_message(&mut control_send, &response_msg).await {
                                    error!("Failed to send ConfigSyncResponse: {}", e);

                                    if conn.close_reason().is_some() {
                                        info!("Connection closed, exiting control stream handler");
                                        break;
                                    }

                                    continue;
                                }
                                debug!("Sent ConfigSyncResponse to client_id: {}", req.client_id);
                            } else {
                                debug!("Client {} already has latest config version: {}", req.client_id, client_config_version);
                            }
                        }
                        Payload::Unregister(req) => {
                            info!("Client {} requested graceful unregister: {}", req.client_id, req.reason);
                            cleanup_client_registration(&state, &req.client_id);

                            break;
                        }
                        _ => {
                            warn!("Received unexpected control message on control stream");
                        }
                    }
                }
            }
            Err(e) => {

                if conn.close_reason().is_some() {
                    info!("Control stream closed because connection closed for client {}", client_id);
                    break;
                }

                warn!("Control stream read error for client {}: {} (connection still alive, will retry)", 
                    client_id, e);

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                if conn.close_reason().is_some() {
                    info!("Connection closed during retry, exiting control stream handler");
                    break;
                }

            }
        }
    }
    

    warn!("Control stream handler exited for client {}", client_id);
    

    if state.clients.contains_key(&client_id) {

        if let Some(conn_ref) = state.clients.get(&client_id) {
            if conn_ref.close_reason().is_some() {
                info!("Cleaning up client {} after control stream closed", client_id);
                cleanup_client_registration(&state, &client_id);
            } else {
                debug!("Client {} connection still alive, not cleaning up", client_id);
            }
        }
    } else {
        debug!("Client {} already cleaned up, skipping", client_id);
    }
    
    Ok(())
}

