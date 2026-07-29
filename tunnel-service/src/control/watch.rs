use crate::control::proto::{diff_snapshots, ConfigDelta, ConfigEvent, ConfigSnapshot};
use crate::control::service::ControlService;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};
use tunnel_lib::ctld_proto::{
    recv_apply_response, recv_watch_request, snapshot_content_hash, ApplyStatus, ControlRevision,
    VersionedConfigSnapshot, WatchRequest,
};
use tunnel_lib::models::msg::{send_message, MessageType};

pub struct WatchServer {
    svc: Arc<ControlService>,
    bind_addr: SocketAddr,
    auth_token: Option<String>,
}

struct WatchConnectionState {
    last_acked_snapshot: Arc<ConfigSnapshot>,
    last_acked_revision: ControlRevision,
    last_acked_hash: String,
}

impl WatchServer {
    pub fn new(
        svc: Arc<ControlService>,
        bind_addr: SocketAddr,
        auth_token: Option<String>,
    ) -> Self {
        let auth_token = auth_token.and_then(|t| {
            let t = t.trim().to_string();
            (!t.is_empty()).then_some(t)
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
                        if let Err(error) =
                            handle_watch_connection(stream, peer, svc, auth_token).await
                        {
                            debug!(peer = %peer, error = %error, "watch connection ended");
                        }
                    });
                }
                Err(error) => error!(error = %error, "WatchServer accept error"),
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

    let request = recv_watch_request(&mut reader).await?;
    authorize(&request, auth_token.as_ref())?;
    debug!(
        peer = %peer,
        last_applied_revision = ?request.last_applied_revision,
        "received canonical WatchRequest"
    );

    let mut changes = svc.subscribe();
    let initial = svc.snapshot();
    let initial_revision = svc.revision_for_snapshot(initial.as_ref());
    let initial_hash = snapshot_content_hash(initial.as_ref())?;
    let mut state = WatchConnectionState {
        last_acked_snapshot: initial.clone(),
        last_acked_revision: initial_revision.clone(),
        last_acked_hash: initial_hash.clone(),
    };

    let initial_event = ConfigEvent::Snapshot(versioned_snapshot(
        initial_revision,
        initial_hash,
        initial.as_ref().clone(),
    ));
    if matches!(
        send_and_ack(&mut reader, &mut writer, &initial_event, &mut state).await?,
        AckOutcome::Resync
    ) {
        anyhow::bail!("server requested resync for initial snapshot");
    }

    loop {
        if changes.changed().await.is_err() {
            warn!(peer = %peer, "ControlService watch channel closed");
            return Ok(());
        }
        let current = svc.snapshot();
        let current_revision = svc.revision_for_snapshot(current.as_ref());
        let current_hash = snapshot_content_hash(current.as_ref())?;
        if current_hash == state.last_acked_hash {
            continue;
        }

        let operations = diff_snapshots(state.last_acked_snapshot.as_ref(), current.as_ref());
        let delta = ConfigDelta {
            base_revision: state.last_acked_revision.clone(),
            base_hash: state.last_acked_hash.clone(),
            target_revision: current_revision.clone(),
            target_hash: current_hash.clone(),
            operations,
        };
        let delta_size = serde_json::to_vec(&delta)?.len();
        let snapshot_event = ConfigEvent::Snapshot(versioned_snapshot(
            current_revision.clone(),
            current_hash.clone(),
            current.as_ref().clone(),
        ));
        let snapshot_size = serde_json::to_vec(&snapshot_event)?.len();
        let event = if delta_size < snapshot_size {
            ConfigEvent::Delta(delta)
        } else {
            snapshot_event
        };

        if matches!(
            send_and_ack(&mut reader, &mut writer, &event, &mut state).await?,
            AckOutcome::Resync
        ) {
            let snapshot_event = ConfigEvent::Snapshot(versioned_snapshot(
                current_revision,
                current_hash,
                current.as_ref().clone(),
            ));
            send_and_ack(&mut reader, &mut writer, &snapshot_event, &mut state).await?;
        }
    }
}

fn authorize(request: &WatchRequest, auth_token: &Option<String>) -> Result<()> {
    if let Some(expected) = auth_token {
        let provided = request.token.as_deref().unwrap_or("");
        if !tokens_equal(provided, expected) {
            anyhow::bail!("unauthorized watch request");
        }
    }
    Ok(())
}

fn versioned_snapshot(
    revision: ControlRevision,
    content_hash: String,
    snapshot: ConfigSnapshot,
) -> VersionedConfigSnapshot {
    VersionedConfigSnapshot {
        revision,
        content_hash,
        generated_at_unix_ms: unix_time_ms(),
        snapshot,
    }
}

enum AckOutcome {
    Applied,
    Resync,
}

async fn send_and_ack<R, W>(
    reader: &mut R,
    writer: &mut W,
    event: &ConfigEvent,
    state: &mut WatchConnectionState,
) -> Result<AckOutcome>
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let (revision, hash) = event_target(event);
    send_message(writer, MessageType::ConfigPush, event).await?;
    writer.flush().await?;
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        recv_apply_response(reader),
    )
    .await
    .map_err(|_| anyhow::anyhow!("timed out waiting for config apply response"))??;

    if response.status == ApplyStatus::ResyncRequired {
        return Ok(AckOutcome::Resync);
    }
    if response.revision != *revision || response.content_hash != hash {
        anyhow::bail!("config apply response does not match the sent event");
    }
    match response.status {
        ApplyStatus::Applied | ApplyStatus::Duplicate => {
            let target = match event {
                ConfigEvent::Snapshot(snapshot) => &snapshot.snapshot,
                ConfigEvent::Delta(delta) => {
                    let mut candidate = state.last_acked_snapshot.as_ref().clone();
                    tunnel_lib::ctld_proto::apply_config_operations(
                        &mut candidate,
                        &delta.operations,
                    )?;
                    candidate.resource_version = delta.target_revision.sequence;
                    state.last_acked_snapshot = Arc::new(candidate);
                    state.last_acked_revision = delta.target_revision.clone();
                    state.last_acked_hash = delta.target_hash.clone();
                    return Ok(AckOutcome::Applied);
                }
            };
            state.last_acked_snapshot = Arc::new(target.clone());
            state.last_acked_revision = revision.clone();
            state.last_acked_hash = hash.to_string();
            Ok(AckOutcome::Applied)
        }
        ApplyStatus::Rejected => Err(anyhow::anyhow!(
            "server rejected config: {}",
            response.reason.as_deref().unwrap_or("unspecified")
        )),
        ApplyStatus::ResyncRequired => unreachable!(),
    }
}

fn event_target(event: &ConfigEvent) -> (&ControlRevision, &str) {
    match event {
        ConfigEvent::Snapshot(snapshot) => (&snapshot.revision, &snapshot.content_hash),
        ConfigEvent::Delta(delta) => (&delta.target_revision, &delta.target_hash),
    }
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn tokens_equal(provided: &str, expected: &str) -> bool {
    use subtle::ConstantTimeEq;
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}
