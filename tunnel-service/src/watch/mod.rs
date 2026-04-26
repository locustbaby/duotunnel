use crate::proto::{WatchEvent, WatchRequest};
use crate::service::ControlService;
use anyhow::Result;
/// WatchServer: TCP listener that implements the ctld-side of the list-watch protocol.
///
/// Protocol flow per connection:
///   1. Read WatchRequest from the server
///   2. Send WatchEvent::Snapshot (full current state)
///   3. Loop: await ControlService watch channel changes → send WatchEvent::Patch
///   4. On peer disconnect or error, drop the connection silently
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};
use tunnel_lib::models::msg::{recv_message, recv_message_type, send_message, MessageType};

pub struct WatchServer {
    svc: Arc<ControlService>,
    bind_addr: SocketAddr,
    auth_token: Option<String>,
}

impl WatchServer {
    pub fn new(
        svc: Arc<ControlService>,
        bind_addr: SocketAddr,
        auth_token: Option<String>,
    ) -> Self {
        let auth_token = auth_token.and_then(|t| {
            let t = t.trim().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        });
        Self {
            svc,
            bind_addr,
            auth_token,
        }
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!(
            addr = %self.bind_addr,
            auth_enabled = self.auth_token.is_some(),
            "WatchServer listening"
        );
        let auth_token = Arc::new(self.auth_token);
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let svc = Arc::clone(&self.svc);
                    let auth_token = auth_token.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_watch_connection(stream, peer, svc, auth_token).await
                        {
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
    auth_token: Arc<Option<String>>,
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
    if let Some(expected) = auth_token.as_ref() {
        let provided = req.token.as_deref().unwrap_or("");
        if !tokens_equal(provided, expected) {
            warn!(peer = %peer, "unauthorized watch request");
            anyhow::bail!("unauthorized watch request");
        }
    }
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

fn tokens_equal(provided: &str, expected: &str) -> bool {
    use subtle::ConstantTimeEq;

    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}
