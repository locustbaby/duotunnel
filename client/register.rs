use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};
use quinn::{Connection, SendStream, RecvStream};
use tunnel_lib::protocol::{write_control_message, read_control_message};
use tunnel_lib::proto::tunnel::control_message::Payload;
use crate::types::ClientIdentity;

const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);
const MAX_RETRIES: usize = 5;

pub struct RegisterManager {
    identity: ClientIdentity,
}

impl RegisterManager {
    pub fn new(identity: ClientIdentity) -> Self {
        Self { identity }
    }

    pub async fn register(
        &self,
        connection: Arc<Connection>,
    ) -> Result<(SendStream, RecvStream)> {
        let mut backoff = INITIAL_BACKOFF;
        let mut retries = 0;

        loop {
            match self.try_register(connection.clone()).await {
                Ok(streams) => {
                    info!("Client registered successfully: client_id={}, group={}", 
                        self.identity.instance_id, self.identity.group_id);
                    return Ok(streams);
                }
                Err(e) => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        error!("Registration failed after {} retries: {}", retries, e);
                        return Err(e);
                    }

                    warn!("Registration attempt {} failed: {}, retrying in {:?}...", 
                        retries, e, backoff);

                    if connection.close_reason().is_some() {
                        return Err(anyhow::anyhow!("Connection closed during registration"));
                    }

                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
                }
            }
        }
    }

    async fn try_register(
        &self,
        connection: Arc<Connection>,
    ) -> Result<(SendStream, RecvStream)> {
        let (mut send, mut recv) = connection.open_bi().await?;
        info!("Opened control stream for registration");

        let config_sync = tunnel_lib::proto::tunnel::ConfigSyncRequest {
            client_id: self.identity.instance_id.clone(),
            group: self.identity.group_id.clone(),
            config_version: "0".to_string(),
            current_hash: String::new(),
            request_full: true,
        };

        let control_msg = tunnel_lib::proto::tunnel::ControlMessage {
            payload: Some(Payload::ConfigSync(config_sync)),
        };

        write_control_message(&mut send, &control_msg).await?;
        info!("Sent ConfigSyncRequest (registration): client_id={}, group={}", 
            self.identity.instance_id, self.identity.group_id);

        let msg = read_control_message(&mut recv).await?;
        match msg.payload {
            Some(Payload::ConfigSyncResponse(_)) => {
                info!("Registration successful via ConfigSyncResponse");
                Ok((send, recv))
            }
            Some(Payload::ErrorMessage(err)) => {
                Err(anyhow::anyhow!("Registration error: {} - {}", err.code, err.message))
            }
            _ => Err(anyhow::anyhow!("Unexpected response during registration")),
        }
    }

    pub async fn unregister(
        &self,
        mut send: SendStream,
    ) -> Result<()> {
        let unregister = tunnel_lib::proto::tunnel::UnregisterRequest {
            client_id: self.identity.instance_id.clone(),
            reason: "graceful shutdown".to_string(),
        };

        let control_msg = tunnel_lib::proto::tunnel::ControlMessage {
            payload: Some(Payload::Unregister(unregister)),
        };

        write_control_message(&mut send, &control_msg).await?;
        info!("Sent UnregisterRequest");
        
        let _ = send.finish().await;
        Ok(())
    }
}

