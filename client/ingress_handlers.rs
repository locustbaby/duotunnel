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
        // ✅ Optimized: Use load_full() for lock-free atomic read
        let connection = self.state.quic_connection.load_full();

        if let Some(conn) = connection {
            handle_http_forward_connection(socket, conn).await
        } else {
            warn!("No active QUIC connection, dropping HTTP request");
            Ok(())
        }
    }
}

// Note: gRPC and WSS (Client → Server) are no longer supported.
// Only HTTP bidirectional forwarding is supported on the client side.
