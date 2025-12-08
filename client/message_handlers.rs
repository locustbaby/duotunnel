use anyhow::Result;
use async_trait::async_trait;
use quinn::SendStream;
use std::sync::Arc;
use tunnel_lib::proto::tunnel::control_message::Payload;
use crate::types::ClientState;
use tunnel_lib::egress_pool::EgressPool;

#[async_trait]
pub trait MessageHandler: Send + Sync {
    fn can_handle(&self, payload: &Payload) -> bool;
    async fn handle(&self, payload: Payload, send: &mut SendStream, state: &ClientState, egress_pool: &Arc<EgressPool>) -> Result<()>;
}

pub struct ConfigSyncHandler;

#[async_trait]
impl MessageHandler for ConfigSyncHandler {
    fn can_handle(&self, payload: &Payload) -> bool {
        matches!(payload, Payload::ConfigSyncResponse(_))
    }

    async fn handle(&self, payload: Payload, _send: &mut SendStream, state: &ClientState, egress_pool: &Arc<EgressPool>) -> Result<()> {
        if let Payload::ConfigSyncResponse(resp) = payload {
            handle_config_sync_response(resp, state, egress_pool).await;
        }
        Ok(())
    }
}

pub struct HashResponseHandler {
    identity: crate::types::ClientIdentity,
}

impl HashResponseHandler {
    pub fn new(identity: crate::types::ClientIdentity) -> Self {
        Self { identity }
    }
}

#[async_trait]
impl MessageHandler for HashResponseHandler {
    fn can_handle(&self, payload: &Payload) -> bool {
        matches!(payload, Payload::HashResponse(_))
    }

    async fn handle(&self, payload: Payload, send: &mut SendStream, state: &ClientState, _egress_pool: &Arc<EgressPool>) -> Result<()> {
        if let Payload::HashResponse(hash_resp) = payload {
            if hash_resp.needs_update {
                tracing::info!("Hash mismatch detected, requesting full config update");
                send_full_config_request(send, &self.identity, state).await?;
            } else {
                tracing::debug!("Config hash matches, no update needed");
            }
        }
        Ok(())
    }
}

pub struct IncrementalUpdateHandler;

#[async_trait]
impl MessageHandler for IncrementalUpdateHandler {
    fn can_handle(&self, payload: &Payload) -> bool {
        matches!(payload, Payload::IncrementalUpdate(_))
    }

    async fn handle(&self, payload: Payload, _send: &mut SendStream, state: &ClientState, egress_pool: &Arc<EgressPool>) -> Result<()> {
        if let Payload::IncrementalUpdate(update) = payload {
            handle_incremental_update(update, state, egress_pool).await;
        }
        Ok(())
    }
}

pub struct ConfigPushHandler {
    identity: crate::types::ClientIdentity,
}

impl ConfigPushHandler {
    pub fn new(identity: crate::types::ClientIdentity) -> Self {
        Self { identity }
    }
}

#[async_trait]
impl MessageHandler for ConfigPushHandler {
    fn can_handle(&self, payload: &Payload) -> bool {
        matches!(payload, Payload::ConfigPush(_))
    }

    async fn handle(&self, payload: Payload, send: &mut SendStream, state: &ClientState, _egress_pool: &Arc<EgressPool>) -> Result<()> {
        if let Payload::ConfigPush(push) = payload {
            tracing::debug!("Received ConfigPushNotification for version: {}", push.config_version);
            send_full_config_request(send, &self.identity, state).await?;
        }
        Ok(())
    }
}

pub struct HeartbeatHandler;

#[async_trait]
impl MessageHandler for HeartbeatHandler {
    fn can_handle(&self, payload: &Payload) -> bool {
        matches!(payload, Payload::Heartbeat(_))
    }

    async fn handle(&self, _payload: Payload, _send: &mut SendStream, _state: &ClientState, _egress_pool: &Arc<EgressPool>) -> Result<()> {
        tracing::debug!("Received heartbeat from server");
        Ok(())
    }
}

pub struct ErrorMessageHandler;

#[async_trait]
impl MessageHandler for ErrorMessageHandler {
    fn can_handle(&self, payload: &Payload) -> bool {
        matches!(payload, Payload::ErrorMessage(_))
    }

    async fn handle(&self, payload: Payload, _send: &mut SendStream, _state: &ClientState, _egress_pool: &Arc<EgressPool>) -> Result<()> {
        if let Payload::ErrorMessage(err) = payload {
            tracing::error!("Received error from server: {} - {}", err.code, err.message);
        }
        Ok(())
    }
}

pub struct MessageHandlerChain {
    handlers: Vec<Box<dyn MessageHandler>>,
}

impl MessageHandlerChain {
    pub fn new(identity: crate::types::ClientIdentity) -> Self {
        let mut handlers: Vec<Box<dyn MessageHandler>> = Vec::new();
        
        handlers.push(Box::new(ConfigSyncHandler));
        handlers.push(Box::new(HashResponseHandler::new(identity.clone())));
        handlers.push(Box::new(IncrementalUpdateHandler));
        handlers.push(Box::new(ConfigPushHandler::new(identity)));
        handlers.push(Box::new(HeartbeatHandler));
        handlers.push(Box::new(ErrorMessageHandler));
        
        Self { handlers }
    }

    pub     async fn handle(
        &self,
        payload: Payload,
        send: &mut SendStream,
        state: &ClientState,
        egress_pool: &Arc<EgressPool>,
    ) -> Result<()> {
        for handler in &self.handlers {
            if handler.can_handle(&payload) {
                return handler.handle(payload, send, state, egress_pool).await;
            }
        }
        
        tracing::warn!("Received unexpected control message");
        Ok(())
    }
}

// Helper function for sending full config request
async fn send_full_config_request(
    send: &mut SendStream,
    identity: &crate::types::ClientIdentity,
    state: &ClientState,
) -> Result<()> {
    use tunnel_lib::protocol::write_control_message;
    use tunnel_lib::proto::tunnel::control_message::Payload;
    
    let current_version = state.config_version.read().await.clone();
    let current_hash = state.config_hash.read().await.clone();

    let config_sync = tunnel_lib::proto::tunnel::ConfigSyncRequest {
        client_id: identity.instance_id.clone(),
        group: identity.group_id.clone(),
        config_version: current_version,
        current_hash,
        request_full: true,
    };

    let control_msg = tunnel_lib::proto::tunnel::ControlMessage {
        payload: Some(Payload::ConfigSync(config_sync)),
    };

    write_control_message(send, &control_msg).await?;
    tracing::debug!("Sent full ConfigSyncRequest");
    Ok(())
}

// Helper functions (extracted from control.rs)
async fn handle_config_sync_response(
    resp: tunnel_lib::proto::tunnel::ConfigSyncResponse,
    state: &ClientState,
    egress_pool: &Arc<EgressPool>,
) {
    use tunnel_lib::proto::tunnel::Rule;
    let current_hash = state.config_hash.read().await.clone();

    if resp.config_hash != current_hash {
        tracing::info!("Config hash changed: {} -> {}", 
            if current_hash.is_empty() { "empty" } else { &current_hash }, 
            &resp.config_hash);

        let new_rule_ids: std::collections::HashSet<String> = resp.rules.iter()
            .map(|r| r.rule_id.clone())
            .collect();
        
        state.rules.retain(|k, _| new_rule_ids.contains(k));
        
        for rule in &resp.rules {
            state.rules.insert(rule.rule_id.clone(), rule.clone());
        }

        // Update RuleMatcher with new rules
        {
            let rules: Vec<Rule> = state.rules.iter().map(|r| r.value().clone()).collect();
            let mut matcher = state.rule_matcher.write().await;
            matcher.update_rules(rules);
        }

        let new_upstream_names: std::collections::HashSet<String> = resp.upstreams.iter()
            .map(|u| u.name.clone())
            .collect();
        
        state.upstreams.retain(|k, _| new_upstream_names.contains(k));
        
        for upstream in &resp.upstreams {
            state.upstreams.insert(upstream.name.clone(), upstream.clone());
        }

        *state.config_version.write().await = resp.config_version.clone();
        *state.config_hash.write().await = resp.config_hash.clone();

        tracing::info!("Updated config: {} rules, {} upstreams (version: {}, hash: {})", 
            resp.rules.len(), resp.upstreams.len(), resp.config_version, &resp.config_hash[..8]);

        let upstreams: Vec<_> = resp.upstreams.clone();
        let egress_pool_clone = egress_pool.clone();
        tokio::spawn(async move {
            tracing::info!("Starting egress pool warmup for {} upstreams...", upstreams.len());
            egress_pool_clone.warmup_upstreams(&upstreams).await;
            tracing::info!("Egress pool warmup completed");
        });
    } else {
        tracing::debug!("Config hash unchanged: {}", if current_hash.is_empty() { "empty" } else { &current_hash[..8] });
    }
}

async fn handle_hash_response(
    resp: tunnel_lib::proto::tunnel::ConfigHashResponse,
    send: &mut SendStream,
) -> Result<()> {
    use tunnel_lib::protocol::write_control_message;
    use tunnel_lib::proto::tunnel::control_message::Payload;
    
    if resp.needs_update {
        tracing::info!("Hash mismatch detected, requesting full config update");
        
        // Note: This requires access to identity, which we don't have here
        // For now, we'll just log. In a full implementation, we'd pass identity through the chain.
        tracing::warn!("Full config request needs identity - not implemented in handler chain");
    } else {
        tracing::debug!("Config hash matches, no update needed");
    }
    Ok(())
}

async fn handle_incremental_update(
    update: tunnel_lib::proto::tunnel::IncrementalConfigUpdate,
    state: &ClientState,
    egress_pool: &Arc<EgressPool>,
) {
    use tunnel_lib::proto::tunnel::Rule;
    tracing::info!("Received incremental config update");

    for rule_id in &update.deleted_rule_ids {
        state.rules.remove(rule_id);
        tracing::debug!("Deleted rule: {}", rule_id);
    }

    for rule in &update.added_rules {
        state.rules.insert(rule.rule_id.clone(), rule.clone());
        tracing::debug!("Added rule: {}", rule.rule_id);
    }
    for rule in &update.updated_rules {
        state.rules.insert(rule.rule_id.clone(), rule.clone());
        tracing::debug!("Updated rule: {}", rule.rule_id);
    }

    // Update RuleMatcher with updated rules
    {
        let rules: Vec<Rule> = state.rules.iter().map(|r| r.value().clone()).collect();
        let mut matcher = state.rule_matcher.write().await;
        matcher.update_rules(rules);
    }

    for upstream_name in &update.deleted_upstream_names {
        state.upstreams.remove(upstream_name);
        tracing::debug!("Deleted upstream: {}", upstream_name);
    }

    for upstream in &update.added_upstreams {
        state.upstreams.insert(upstream.name.clone(), upstream.clone());
        tracing::debug!("Added upstream: {}", upstream.name);
    }
    for upstream in &update.updated_upstreams {
        state.upstreams.insert(upstream.name.clone(), upstream.clone());
        tracing::debug!("Updated upstream: {}", upstream.name);
    }

    *state.config_hash.write().await = update.config_hash.clone();

    tracing::info!("Applied incremental update: +{} rules, ~{} rules, -{} rules, +{} upstreams, ~{} upstreams, -{} upstreams (new hash: {})",
        update.added_rules.len(), update.updated_rules.len(), update.deleted_rule_ids.len(),
        update.added_upstreams.len(), update.updated_upstreams.len(), update.deleted_upstream_names.len(),
        &update.config_hash[..8]);

    let mut upstreams = update.added_upstreams;
    upstreams.extend(update.updated_upstreams);

    if !upstreams.is_empty() {
        let egress_pool_clone = egress_pool.clone();
        tokio::spawn(async move {
            tracing::info!("Starting egress pool warmup for {} updated upstreams...", upstreams.len());
            egress_pool_clone.warmup_upstreams(&upstreams).await;
            tracing::info!("Egress pool warmup completed");
        });
    }
}


