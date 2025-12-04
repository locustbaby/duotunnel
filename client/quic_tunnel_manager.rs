use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};
use quinn::Connection;
use tunnel_lib::quic_transport::QuicClient;
use crate::types::{ClientState, ClientIdentity};
use tunnel_lib::egress_pool::EgressPool;
use crate::forwarder::Forwarder;

const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

pub struct QuicTunnelManager {
    server_addr: SocketAddr,
    server_name: String,
    identity: ClientIdentity,
    state: Arc<ClientState>,
    egress_pool: Arc<EgressPool>,
    forwarder: Arc<Forwarder>,
}

impl QuicTunnelManager {
    pub fn new(
        server_addr: SocketAddr,
        server_name: String,
        identity: ClientIdentity,
        state: Arc<ClientState>,
        egress_pool: Arc<EgressPool>,
        forwarder: Arc<Forwarder>,
    ) -> Self {
        Self {
            server_addr,
            server_name,
            identity,
            state,
            egress_pool,
            forwarder,
        }
    }

    pub async fn run(
        self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        let mut backoff = INITIAL_BACKOFF;

        loop {
            if shutdown_rx.try_recv().is_ok() {
                info!("QUIC tunnel manager received shutdown signal");
                break;
            }

            info!("Connecting to QUIC server at {}...", self.server_addr);

            let client = match QuicClient::new() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create QUIC client: {}", e);
                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
                    continue;
                }
            };

            match client.connect(self.server_addr, &self.server_name).await {
                Ok(conn) => {
                    info!("Successfully connected to QUIC server");
                    let conn = Arc::new(conn);

                    {
                        let mut lock = self.state.quic_connection.write().await;
                        *lock = Some(conn.clone());
                    }

                    backoff = INITIAL_BACKOFF;

                    let session_result = self.run_connection_session(conn.clone(), shutdown_rx.resubscribe()).await;
                    
                    {
                        let mut lock = self.state.quic_connection.write().await;
                        *lock = None;
                    }

                    match session_result {
                        Ok(_) => {
                            info!("Connection session ended normally");
                        }
                        Err(e) => {
                            warn!("Connection session error: {}, will reconnect", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to QUIC server: {}", e);
                }
            }

            if shutdown_rx.try_recv().is_ok() {
                info!("QUIC tunnel manager shutting down");
                break;
            }

            info!("Reconnecting in {:?}...", backoff);
            tokio::time::sleep(backoff).await;
            backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
        }

        Ok(())
    }

    async fn run_connection_session(
        &self,
        connection: Arc<Connection>,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        info!("Connection session started, monitoring connection status");

        let mut connection_check_interval = tokio::time::interval(Duration::from_secs(1));
        connection_check_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                reason = connection.closed() => {
                    let error_msg = format!("QUIC connection closed: {:?}", reason);
                    error!("{}", error_msg);
                    return Err(anyhow::anyhow!("{}", error_msg));
                }
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, closing QUIC connection");
                    connection.close(0u32.into(), b"graceful shutdown");
                    return Ok(());
                }
                _ = connection_check_interval.tick() => {
                    if let Some(reason) = connection.close_reason() {
                        let error_msg = format!("Connection closed detected via periodic check: {:?}", reason);
                        error!("{}", error_msg);
                        return Err(anyhow::anyhow!("{}", error_msg));
                    }
                }
            }
        }
    }
}
