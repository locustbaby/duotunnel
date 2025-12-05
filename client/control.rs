use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, debug, warn};
use quinn::{Connection, SendStream, RecvStream};
use dashmap::DashMap;
use tunnel_lib::protocol::{write_control_message, read_control_message};
use tunnel_lib::proto::tunnel::control_message::Payload;
use crate::types::{ClientState, ClientIdentity};
use tunnel_lib::egress_pool::EgressPool;
use crate::register::RegisterManager;

const HASH_CHECK_INTERVAL: Duration = Duration::from_secs(15);
const FULL_SYNC_INTERVAL: Duration = Duration::from_secs(300);
const CONNECTION_CHECK_INTERVAL: Duration = Duration::from_millis(500);
const STREAM_RECONNECT_BACKOFF: Duration = Duration::from_secs(1);
const CONNECTION_STATUS_CHECK_INTERVAL: Duration = Duration::from_secs(1);

pub struct ConfigManager {
    identity: ClientIdentity,
    state: Arc<ClientState>,
    egress_pool: Arc<EgressPool>,
}

impl ConfigManager {
    pub fn new(
        identity: ClientIdentity,
        state: Arc<ClientState>,
        egress_pool: Arc<EgressPool>,
    ) -> Self {
        Self {
            identity,
            state,
            egress_pool,
        }
    }

    pub async fn run(
        &self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        info!("Control stream manager started, waiting for QUIC connection...");

        loop {

            if shutdown_rx.try_recv().is_ok() {
                info!("Control stream manager received shutdown signal");
                return Ok(());
            }


            let connection = self.wait_for_connection().await?;
            info!("QUIC connection available, starting control stream loop");


            if let Err(e) = self.run_control_stream_loop(connection, &mut shutdown_rx).await {
                error!("Control stream loop error: {}, will reconnect", e);
                tokio::time::sleep(STREAM_RECONNECT_BACKOFF).await;
                continue;
            }
        }
    }


    async fn run_control_stream_loop(
        &self,
        connection: Arc<Connection>,
        shutdown_rx: &mut tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        loop {

            if shutdown_rx.try_recv().is_ok() {
                info!("Control stream loop received shutdown signal");
                return Ok(());
            }


            self.check_connection_alive(&connection)?;


            let (mut send, mut recv) = match self.open_control_stream(connection.clone()).await {
                Ok(streams) => {
                    info!("Control stream opened successfully");
                    streams
                }
                Err(e) => {
                    self.check_connection_alive(&connection)?;
                    warn!("Failed to open control stream: {}, retrying in {:?}", e, STREAM_RECONNECT_BACKOFF);
                    tokio::time::sleep(STREAM_RECONNECT_BACKOFF).await;
                    continue;
                }
            };


            if let Err(e) = self.send_full_config_request(&mut send).await {
                warn!("Failed to send initial config request: {}, will retry", e);
                let _ = send.finish();
                tokio::time::sleep(STREAM_RECONNECT_BACKOFF).await;
                continue;
            }


            if let Err(e) = self.process_control_messages(
                &mut send, 
                &mut recv, 
                &connection, 
                shutdown_rx
            ).await {
                warn!("Control message processing error: {}, will reopen stream", e);
                let _ = send.finish();
                tokio::time::sleep(STREAM_RECONNECT_BACKOFF).await;
                continue;
            }


            warn!("Control stream closed, reopening");
            tokio::time::sleep(STREAM_RECONNECT_BACKOFF).await;
        }
    }


    async fn process_control_messages(
        &self,
        send: &mut SendStream,
        recv: &mut RecvStream,
        connection: &Arc<Connection>,
        shutdown_rx: &mut tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        let mut hash_check_timer = tokio::time::interval(HASH_CHECK_INTERVAL);
        let mut full_sync_timer = tokio::time::interval(FULL_SYNC_INTERVAL);
        let mut connection_status_timer = tokio::time::interval(CONNECTION_STATUS_CHECK_INTERVAL);
        

        hash_check_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        full_sync_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        connection_status_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);


        hash_check_timer.tick().await;
        full_sync_timer.tick().await;
        connection_status_timer.tick().await;

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Control message loop received shutdown signal");
                    let _ = send.finish();
                    return Ok(());
                }

                _ = connection_status_timer.tick() => {
                    self.check_connection_alive(connection)?;
                }

                _ = hash_check_timer.tick() => {
                    self.check_connection_alive(connection)?;
                    debug!("Periodic hash check triggered");
                    self.send_hash_check_request(send).await?;
                }

                _ = full_sync_timer.tick() => {
                    self.check_connection_alive(connection)?;
                    info!("Periodic full config sync triggered");
                    self.send_full_config_request(send).await?;
                }

                result = read_control_message(recv) => {
                    self.check_connection_alive(connection)?;
                    let msg = result?;
                    
                    if let Some(payload) = msg.payload {
                        self.handle_control_message(payload, send).await?;
                    }
                }
            }
        }
    }


    fn check_connection_alive(&self, connection: &Arc<Connection>) -> Result<()> {
        if let Some(reason) = connection.close_reason() {
            Err(anyhow::anyhow!("Connection closed: {:?}", reason))
        } else {
            Ok(())
        }
    }

    async fn wait_for_connection(&self) -> Result<Arc<Connection>> {
        loop {
            let connection = {
                let lock = self.state.quic_connection.read().await;
                lock.clone()
            };

            if let Some(conn) = connection {
                if conn.close_reason().is_none() {
                    return Ok(conn);
                }
            }

            tokio::time::sleep(CONNECTION_CHECK_INTERVAL).await;
        }
    }

    async fn open_control_stream(
        &self,
        connection: Arc<Connection>,
    ) -> Result<(SendStream, RecvStream)> {
        let register_manager = RegisterManager::new(self.identity.clone());
        register_manager.register(connection).await
    }

    async fn handle_control_message(
        &self,
        payload: Payload,
        send: &mut SendStream,
    ) -> Result<()> {
        match payload {
            Payload::ConfigSyncResponse(resp) => {
                self.handle_config_sync_response(resp).await;
            }
            Payload::HashResponse(hash_resp) => {
                self.handle_hash_response(hash_resp, send).await?;
            }
            Payload::IncrementalUpdate(update) => {
                self.handle_incremental_update(update).await;
            }
            Payload::ConfigPush(push) => {
                debug!("Received ConfigPushNotification for version: {}", push.config_version);
                self.send_full_config_request(send).await?;
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
        Ok(())
    }

    async fn send_hash_check_request(&self, send: &mut SendStream) -> Result<()> {
        let current_hash = self.state.config_hash.read().await.clone();

        let hash_request = tunnel_lib::proto::tunnel::ConfigHashRequest {
            client_id: self.identity.instance_id.clone(),
            group: self.identity.group_id.clone(),
            current_hash,
        };

        let control_msg = tunnel_lib::proto::tunnel::ControlMessage {
            payload: Some(Payload::HashRequest(hash_request)),
        };

        write_control_message(send, &control_msg).await?;
        debug!("Sent ConfigHashRequest");
        Ok(())
    }

    async fn send_full_config_request(&self, send: &mut SendStream) -> Result<()> {
        let current_version = self.state.config_version.read().await.clone();
        let current_hash = self.state.config_hash.read().await.clone();

        let config_sync = tunnel_lib::proto::tunnel::ConfigSyncRequest {
            client_id: self.identity.instance_id.clone(),
            group: self.identity.group_id.clone(),
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

    async fn handle_hash_response(
        &self,
        resp: tunnel_lib::proto::tunnel::ConfigHashResponse,
        send: &mut SendStream,
    ) -> Result<()> {
        if resp.needs_update {
            info!("Hash mismatch detected, requesting full config update");
            self.send_full_config_request(send).await?;
        } else {
            debug!("Config hash matches, no update needed");
        }
        Ok(())
    }

    async fn handle_config_sync_response(
        &self,
        resp: tunnel_lib::proto::tunnel::ConfigSyncResponse,
    ) {
        let current_hash = self.state.config_hash.read().await.clone();

        if resp.config_hash != current_hash {
            info!("Config hash changed: {} -> {}", 
                if current_hash.is_empty() { "empty" } else { &current_hash }, 
                &resp.config_hash);

            let new_rule_ids: std::collections::HashSet<String> = resp.rules.iter()
                .map(|r| r.rule_id.clone())
                .collect();
            
            self.state.rules.retain(|k, _| new_rule_ids.contains(k));
            
            for rule in &resp.rules {
                self.state.rules.insert(rule.rule_id.clone(), rule.clone());
            }

            let new_upstream_names: std::collections::HashSet<String> = resp.upstreams.iter()
                .map(|u| u.name.clone())
                .collect();
            
            self.state.upstreams.retain(|k, _| new_upstream_names.contains(k));
            
            for upstream in &resp.upstreams {
                self.state.upstreams.insert(upstream.name.clone(), upstream.clone());
            }

            *self.state.config_version.write().await = resp.config_version.clone();
            *self.state.config_hash.write().await = resp.config_hash.clone();

            info!("Updated config: {} rules, {} upstreams (version: {}, hash: {})", 
                resp.rules.len(), resp.upstreams.len(), resp.config_version, &resp.config_hash[..8]);

            let upstreams: Vec<_> = resp.upstreams.clone();
            let egress_pool = self.egress_pool.clone();
            tokio::spawn(async move {
                info!("Starting egress pool warmup for {} upstreams...", upstreams.len());
                egress_pool.warmup_upstreams(&upstreams).await;
                info!("Egress pool warmup completed");
            });
        } else {
            debug!("Config hash unchanged: {}", if current_hash.is_empty() { "empty" } else { &current_hash[..8] });
        }
    }

    async fn handle_incremental_update(
        &self,
        update: tunnel_lib::proto::tunnel::IncrementalConfigUpdate,
    ) {
        info!("Received incremental config update");

        for rule_id in &update.deleted_rule_ids {
            self.state.rules.remove(rule_id);
            debug!("Deleted rule: {}", rule_id);
        }

        for rule in &update.added_rules {
            self.state.rules.insert(rule.rule_id.clone(), rule.clone());
            debug!("Added rule: {}", rule.rule_id);
        }
        for rule in &update.updated_rules {
            self.state.rules.insert(rule.rule_id.clone(), rule.clone());
            debug!("Updated rule: {}", rule.rule_id);
        }

        for upstream_name in &update.deleted_upstream_names {
            self.state.upstreams.remove(upstream_name);
            debug!("Deleted upstream: {}", upstream_name);
        }

        for upstream in &update.added_upstreams {
            self.state.upstreams.insert(upstream.name.clone(), upstream.clone());
            debug!("Added upstream: {}", upstream.name);
        }
        for upstream in &update.updated_upstreams {
            self.state.upstreams.insert(upstream.name.clone(), upstream.clone());
            debug!("Updated upstream: {}", upstream.name);
        }

        *self.state.config_hash.write().await = update.config_hash.clone();

        info!("Applied incremental update: +{} rules, ~{} rules, -{} rules, +{} upstreams, ~{} upstreams, -{} upstreams (new hash: {})",
            update.added_rules.len(), update.updated_rules.len(), update.deleted_rule_ids.len(),
            update.added_upstreams.len(), update.updated_upstreams.len(), update.deleted_upstream_names.len(),
            &update.config_hash[..8]);

        let mut upstreams = update.added_upstreams;
        upstreams.extend(update.updated_upstreams);

        if !upstreams.is_empty() {
            let egress_pool = self.egress_pool.clone();
            tokio::spawn(async move {
                info!("Starting egress pool warmup for {} updated upstreams...", upstreams.len());
                egress_pool.warmup_upstreams(&upstreams).await;
                info!("Egress pool warmup completed");
            });
        }
    }
}
