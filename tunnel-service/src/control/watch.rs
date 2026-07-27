use crate::control::proto::WatchEvent;
use crate::control::service::ControlService;
use anyhow::Result;
/// WatchServer: TCP listener that implements the ctld-side of the list-watch protocol.
///
/// Protocol flow per connection:
///   1. Read WatchRequest from the server
///   2. Send WatchEvent::Snapshot (full current state)
///   3. Loop: await ControlService change signal → send the latest full Snapshot
///   4. On peer disconnect or error, drop the connection silently
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};
use tunnel_lib::ctld_proto::recv_watch_request;
use tunnel_lib::models::msg::{send_message, MessageType};

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
    let req = recv_watch_request(&mut reader).await?;
    if let Some(expected) = auth_token.as_ref() {
        let provided = req.token.as_deref().unwrap_or("");
        if !tokens_equal(provided, expected) {
            warn!(peer = %peer, "unauthorized watch request");
            anyhow::bail!("unauthorized watch request");
        }
    }
    debug!(peer = %peer, resource_version = req.resource_version, "received WatchRequest");

    // Subscribe before reading current state so a concurrent publish either
    // appears in this snapshot or leaves a pending change signal.
    let mut rx = svc.subscribe();

    let current = svc.snapshot();
    send_message(
        &mut writer,
        MessageType::ConfigPush,
        &WatchEvent::Snapshot(current.as_ref().clone()),
    )
    .await?;
    writer.flush().await?;
    info!(
        peer = %peer,
        resource_version = current.resource_version,
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
        let current = svc.snapshot();
        let event = WatchEvent::Snapshot(current.as_ref().clone());
        if let Err(e) = send_message(&mut writer, MessageType::ConfigPush, &event).await {
            debug!(peer = %peer, error = %e, "failed to send Snapshot, closing connection");
            break;
        }
        if let Err(e) = writer.flush().await {
            debug!(peer = %peer, error = %e, "flush failed, closing connection");
            break;
        }
        debug!(
            peer = %peer,
            resource_version = current.resource_version,
            "sent Snapshot event"
        );
    }

    Ok(())
}

fn tokens_equal(provided: &str, expected: &str) -> bool {
    use subtle::ConstantTimeEq;

    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}
