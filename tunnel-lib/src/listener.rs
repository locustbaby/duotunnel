use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error, debug};

#[async_trait::async_trait]
pub trait ConnectionHandler: Send + Sync {
    async fn handle_connection(&self, socket: tokio::net::TcpStream) -> Result<()>;
}

pub struct ListenerManager {
    http_port: Option<u16>,
    grpc_port: Option<u16>,
    wss_port: Option<u16>,
}

impl ListenerManager {

    pub fn new(
        http_port: Option<u16>,
        grpc_port: Option<u16>,
        wss_port: Option<u16>,
    ) -> Self {
        Self {
            http_port,
            grpc_port,
            wss_port,
        }
    }



    pub async fn start_all<H1, H2, H3>(
        &self,
        http_handler: Option<Arc<H1>>,
        grpc_handler: Option<Arc<H2>>,
        wss_handler: Option<Arc<H3>>,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>>
    where
        H1: ConnectionHandler + 'static,
        H2: ConnectionHandler + 'static,
        H3: ConnectionHandler + 'static,
    {
        let mut handles = Vec::new();

        if let (Some(port), Some(handler)) = (self.http_port, http_handler) {
            info!("Starting HTTP listener on port {}", port);
            let handle = tokio::spawn(async move {
                if let Err(e) = start_tcp_listener(port, handler, "HTTP".to_string()).await {
                    error!("HTTP listener error: {}", e);
                }
            });
            handles.push(handle);
        }

        if let (Some(port), Some(handler)) = (self.grpc_port, grpc_handler) {
            info!("Starting gRPC listener on port {}", port);
            let handle = tokio::spawn(async move {
                if let Err(e) = start_tcp_listener(port, handler, "gRPC".to_string()).await {
                    error!("gRPC listener error: {}", e);
                }
            });
            handles.push(handle);
        }

        if let (Some(port), Some(handler)) = (self.wss_port, wss_handler) {
            info!("Starting WebSocket listener on port {}", port);
            let handle = tokio::spawn(async move {
                if let Err(e) = start_tcp_listener(port, handler, "WebSocket".to_string()).await {
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

async fn start_tcp_listener<H>(
    port: u16,
    handler: Arc<H>,
    protocol_name: String,
) -> Result<()>
where
    H: ConnectionHandler + 'static,
{
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("{} listener listening on 0.0.0.0:{}", protocol_name, port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        debug!("Accepted {} connection from {}", protocol_name, peer_addr);
        let handler = handler.clone();
        let protocol_name = protocol_name.clone();

        tokio::spawn(async move {
            if let Err(e) = handler.handle_connection(socket).await {
                error!("{} connection handling error: {}", protocol_name, e);
            }
        });
    }
}
