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
use tunnel_lib::ctld_proto::{
    recv_apply_response, recv_watch_request, snapshot_content_hash, ApplyStatus,
    VersionedConfigSnapshot, WatchEventV2,
};
use tunnel_lib::models::msg::{send_message, MessageType};

const SNAPSHOT_HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

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
    let use_v2 = req.supports_v2();
    debug!(
        peer = %peer,
        resource_version = req.legacy_resource_version(),
        protocol = if use_v2 { "v2" } else { "legacy" },
        "received WatchRequest"
    );

    // Subscribe before reading current state so a concurrent publish either
    // appears in this snapshot or leaves a pending change signal.
    let mut rx = svc.subscribe();

    let current = svc.snapshot();
    send_snapshot(&mut reader, &mut writer, current.as_ref(), &svc, use_v2).await?;
    info!(
        peer = %peer,
        resource_version = current.resource_version,
        "sent initial Snapshot"
    );
    let mut heartbeat = tokio::time::interval(SNAPSHOT_HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    heartbeat.tick().await;
    loop {
        tokio::select! {
            changed = rx.changed() => {
                if changed.is_err() {
                    warn!(peer = %peer, "ControlService watch channel closed, dropping connection");
                    break;
                }
            }
            _ = heartbeat.tick() => {}
        }
        let current = svc.snapshot();
        if let Err(e) =
            send_snapshot(&mut reader, &mut writer, current.as_ref(), &svc, use_v2).await
        {
            debug!(peer = %peer, error = %e, "failed to send Snapshot, closing connection");
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

async fn send_snapshot<R, W>(
    reader: &mut R,
    writer: &mut W,
    snapshot: &crate::control::proto::ConfigSnapshot,
    svc: &ControlService,
    use_v2: bool,
) -> Result<()>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    if !use_v2 {
        send_message(
            writer,
            MessageType::ConfigPush,
            &WatchEvent::Snapshot(snapshot.clone()),
        )
        .await?;
        writer.flush().await?;
        return Ok(());
    }

    let revision = svc.revision_for_snapshot(snapshot);
    let content_hash = snapshot_content_hash(snapshot)?;
    let generated_at_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX);
    let event = WatchEventV2::Snapshot(VersionedConfigSnapshot {
        revision: revision.clone(),
        content_hash: content_hash.clone(),
        generated_at_unix_ms,
        snapshot: snapshot.clone(),
    });
    send_message(writer, MessageType::ConfigPush, &event).await?;
    writer.flush().await?;

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        recv_apply_response(reader),
    )
    .await
    .map_err(|_| anyhow::anyhow!("timed out waiting for config apply response"))??;
    if response.revision != revision || response.content_hash != content_hash {
        anyhow::bail!("config apply response does not match the sent revision");
    }
    match response.status {
        ApplyStatus::Applied | ApplyStatus::Duplicate => Ok(()),
        ApplyStatus::Rejected => anyhow::bail!(
            "server rejected revision: {}",
            response.reason.as_deref().unwrap_or("unspecified")
        ),
    }
}

fn tokens_equal(provided: &str, expected: &str) -> bool {
    use subtle::ConstantTimeEq;

    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}
