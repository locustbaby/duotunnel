use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::warn;
use crate::types::ClientState;
use crate::forwarder::http::handle_http_forward_connection;

pub struct HttpIngressHandler {
    state: Arc<ClientState>,
}

impl HttpIngressHandler {
    pub fn new(state: Arc<ClientState>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl tunnel_lib::listener::ConnectionHandler for HttpIngressHandler {
    async fn handle_connection(&self, socket: TcpStream) -> Result<()> {
        let connection = {
            let lock = self.state.quic_connection.read().await;
            lock.clone()
        };

        if let Some(conn) = connection {
            handle_http_forward_connection(socket, conn).await
        } else {
            warn!("No active QUIC connection, dropping HTTP request");
            Ok(())
        }
    }
}

pub struct GrpcIngressHandler {
    _state: Arc<ClientState>,
}

impl GrpcIngressHandler {
    pub fn new(state: Arc<ClientState>) -> Self {
        Self { _state: state }
    }
}

#[async_trait::async_trait]
impl tunnel_lib::listener::ConnectionHandler for GrpcIngressHandler {
    async fn handle_connection(&self, _socket: TcpStream) -> Result<()> {
        warn!("gRPC forwarding not yet implemented");
        Ok(())
    }
}

pub struct WssIngressHandler {
    _state: Arc<ClientState>,
}

impl WssIngressHandler {
    pub fn new(state: Arc<ClientState>) -> Self {
        Self { _state: state }
    }
}

#[async_trait::async_trait]
impl tunnel_lib::listener::ConnectionHandler for WssIngressHandler {
    async fn handle_connection(&self, _socket: TcpStream) -> Result<()> {
        warn!("WebSocket forwarding not yet implemented");
        Ok(())
    }
}
