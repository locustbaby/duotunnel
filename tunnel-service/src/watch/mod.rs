/// WatchServer: TCP listener that implements the ctld-side of the list-watch protocol.
///
/// Protocol flow per connection:
///   1. Read WatchRequest from the server
///   2. Send WatchEvent::Snapshot (full current state)
///   3. Loop: await ControlService watch channel changes → send WatchEvent::Patch
///   4. On peer disconnect or error, drop the connection silently
use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, BufWriter};
use tracing::{debug, error, info, warn};
use tunnel_lib::models::msg::{MessageType, recv_message_type, recv_message, send_message};
use crate::proto::{WatchRequest, WatchEvent};
use crate::service::ControlService;

pub struct WatchServer {
    svc: Arc<ControlService>,
    bind_addr: SocketAddr,
}

impl WatchServer {
    pub fn new(svc: Arc<ControlService>, bind_addr: SocketAddr) -> Self {
        Self { svc, bind_addr }
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!(addr = %self.bind_addr, "WatchServer listening");
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let svc = Arc::clone(&self.svc);
                    tokio::spawn(async move {
                        if let Err(e) = handle_watch_connection(stream, peer, svc).await {
                            debug!(peer = %peer, error = %e, "watch connection ended");
                        }
                    });
                }
                Err(e) => {
                    error!(error = %e, "WatchServer accept error");
                }
            }
        }
    }
}

async fn handle_watch_connection(
    stream: TcpStream,
    peer: SocketAddr,
    svc: Arc<ControlService>,
) -> Result<()> {
    info!(peer = %peer, "watch connection accepted");
    let (reader, writer) = stream.into_split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    // Step 1: read the WatchRequest
    let msg_type = recv_message_type(&mut reader).await?;
    if msg_type != MessageType::ConfigPush {
        anyhow::bail!("expected ConfigPush/WatchRequest, got {:?}", msg_type);
    }
    let req: WatchRequest = recv_message(&mut reader).await?;
    debug!(peer = %peer, resource_version = req.resource_version, "received WatchRequest");

    // Step 2: subscribe BEFORE reading current state to avoid a race where a
    // Patch is published between reading the snapshot and starting to watch.
    // tokio::sync::watch guarantees that borrow_and_update() after changed()
    // will deliver every value published after subscribe() was called.
    let mut rx = svc.subscribe();

    // Send the full snapshot that was current at subscribe time.
    let current = rx.borrow_and_update().clone();
    send_message(&mut writer, MessageType::ConfigPush, &*current).await?;
    writer.flush().await?;
    info!(
        peer = %peer,
        resource_version = svc.current_version(),
        "sent initial Snapshot"
    );
    loop {
        // Wait for the next mutation
        match rx.changed().await {
            Ok(()) => {}
            Err(_) => {
                // Sender dropped — service shutting down
                warn!(peer = %peer, "ControlService watch channel closed, dropping connection");
                break;
            }
        }
        let event: Arc<WatchEvent> = rx.borrow_and_update().clone();
        if let Err(e) = send_message(&mut writer, MessageType::ConfigPush, &*event).await {
            debug!(peer = %peer, error = %e, "failed to send Patch, closing connection");
            break;
        }
        if let Err(e) = writer.flush().await {
            debug!(peer = %peer, error = %e, "flush failed, closing connection");
            break;
        }
        debug!(peer = %peer, "sent Patch event");
    }

    Ok(())
}
