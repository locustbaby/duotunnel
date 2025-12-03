use std::sync::Arc;
use tracing::{info, error};
use anyhow::Result;
use crate::types::ClientState;
use crate::forwarder::Forwarder;
use crate::client_listener;

/// Listener Manager
/// Manages all protocol listeners (HTTP, gRPC, WebSocket)
/// Receives requests and forwards them through QUIC tunnel
pub struct ListenerManager {
    http_port: Option<u16>,
    grpc_port: Option<u16>,
    wss_port: Option<u16>,
    state: Arc<ClientState>,
    forwarder: Arc<Forwarder>,
}

impl ListenerManager {
    /// Create a new listener manager
    pub fn new(
        http_port: Option<u16>,
        grpc_port: Option<u16>,
        wss_port: Option<u16>,
        state: Arc<ClientState>,
        forwarder: Arc<Forwarder>,
    ) -> Self {
        Self {
            http_port,
            grpc_port,
            wss_port,
            state,
            forwarder,
        }
    }

    /// Start all configured listeners
    /// Returns a vector of join handles for all spawned listeners
    pub async fn start_all(&self) -> Result<Vec<tokio::task::JoinHandle<()>>> {
        let mut handles = Vec::new();

        // Start HTTP listener
        if let Some(port) = self.http_port {
            info!("Starting HTTP listener on port {}", port);
            let state = self.state.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = client_listener::start_http_listener(port, state).await {
                    error!("HTTP listener error: {}", e);
                }
            });
            handles.push(handle);
        }

        // Start gRPC listener
        if let Some(port) = self.grpc_port {
            info!("Starting gRPC listener on port {}", port);
            let state = self.state.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = client_listener::start_grpc_listener(port, state).await {
                    error!("gRPC listener error: {}", e);
                }
            });
            handles.push(handle);
        }

        // Start WebSocket listener
        if let Some(port) = self.wss_port {
            info!("Starting WebSocket listener on port {}", port);
            let state = self.state.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = client_listener::start_wss_listener(port, state).await {
                    error!("WebSocket listener error: {}", e);
                }
            });
            handles.push(handle);
        }

        if handles.is_empty() {
            info!("No listeners configured");
        } else {
            info!("Started {} listener(s)", handles.len());
        }

        Ok(handles)
    }
}
