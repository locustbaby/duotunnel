use crate::control::proto::{diff_snapshots, ConfigDelta, ConfigEvent, ConfigSnapshot};
use crate::control::service::ControlService;
use anyhow::Result;
use duotunnel_lib::ctld_proto::{
    config_event_wire_size, recv_apply_response, recv_watch_request, send_config_event,
    snapshot_content_hash, ApplyStatus, ControlRevision, VersionedConfigSnapshot, WatchRequest,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

const WATCH_HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const MAX_WATCH_CONNECTIONS: usize = 256;

pub struct WatchServer {
    svc: Arc<ControlService>,
    bind_addr: SocketAddr,
    auth_token: Option<String>,
    connection_budget: Arc<Semaphore>,
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
            connection_budget: Arc::new(Semaphore::new(MAX_WATCH_CONNECTIONS)),
        }
    }

    pub async fn run_with_listener(self, listener: TcpListener) -> Result<()> {
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
                    let permit = match self.connection_budget.clone().try_acquire_owned() {
                        Ok(permit) => permit,
                        Err(_) => {
                            debug!(peer = %peer, "watch connection budget exhausted");
                            continue;
                        }
                    };
                    tokio::spawn(async move {
                        let _permit = permit;
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

    let request = tokio::time::timeout(WATCH_HANDSHAKE_TIMEOUT, recv_watch_request(&mut reader))
        .await
        .map_err(|_| anyhow::anyhow!("watch handshake timed out"))??;
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
        send_and_ack(
            &mut reader,
            &mut writer,
            &initial_event,
            &mut state,
            &initial,
        )
        .await?,
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
        let snapshot_event = ConfigEvent::Snapshot(versioned_snapshot(
            current_revision.clone(),
            current_hash.clone(),
            current.as_ref().clone(),
        ));
        let delta_event = ConfigEvent::Delta(delta);
        let event =
            if config_event_wire_size(&delta_event)? < config_event_wire_size(&snapshot_event)? {
                delta_event
            } else {
                snapshot_event
            };

        if matches!(
            send_and_ack(&mut reader, &mut writer, &event, &mut state, &current).await?,
            AckOutcome::Resync
        ) {
            let snapshot_event = ConfigEvent::Snapshot(versioned_snapshot(
                current_revision,
                current_hash,
                current.as_ref().clone(),
            ));
            send_and_ack(
                &mut reader,
                &mut writer,
                &snapshot_event,
                &mut state,
                &current,
            )
            .await?;
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

#[derive(Debug)]
enum AckOutcome {
    Applied,
    Resync,
}

async fn send_and_ack<R, W>(
    reader: &mut R,
    writer: &mut W,
    event: &ConfigEvent,
    state: &mut WatchConnectionState,
    target_snapshot: &Arc<ConfigSnapshot>,
) -> Result<AckOutcome>
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let (revision, hash) = event_target(event);
    send_config_event(writer, event).await?;
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
            match event {
                ConfigEvent::Snapshot(_) => {}
                ConfigEvent::Delta(delta) => {
                    let mut candidate = state.last_acked_snapshot.as_ref().clone();
                    duotunnel_lib::ctld_proto::apply_config_operations(
                        &mut candidate,
                        &delta.operations,
                    )?;
                    candidate.resource_version = delta.target_revision.sequence;
                    let actual_hash = snapshot_content_hash(&candidate)?;
                    if actual_hash != delta.target_hash {
                        anyhow::bail!(
                            "generated delta target hash does not match the target snapshot"
                        );
                    }
                    state.last_acked_snapshot = target_snapshot.clone();
                    state.last_acked_revision = delta.target_revision.clone();
                    state.last_acked_hash = delta.target_hash.clone();
                    return Ok(AckOutcome::Applied);
                }
            }
            state.last_acked_snapshot = target_snapshot.clone();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::token::cache::SqliteTokenCacheProvider;
    use crate::storage::rules::{EgressUpstreamDef, RoutingData, RuleStore, UpstreamServer};
    use crate::storage::sqlite::{open_sqlite_pool, SqliteAuthStore};
    use crate::storage::sqlite_rules::SqliteRuleStore;
    use duotunnel_lib::ctld_proto::{
        recv_config_event, send_apply_response, send_watch_request, ApplyResponse, ConfigOperation,
    };
    use std::time::Duration;
    use tokio::io::{split, BufReader};

    fn base_routing() -> RoutingData {
        RoutingData {
            egress_upstreams: (0..32)
                .map(|index| EgressUpstreamDef {
                    name: format!("api-{index}"),
                    lb_policy: "round_robin".into(),
                    servers: vec![UpstreamServer {
                        address: format!("127.0.0.1:{}", 10_000 + index),
                        resolve: false,
                    }],
                })
                .collect(),
            ..RoutingData::default()
        }
    }

    async fn setup_service(initial_routing: &RoutingData) -> Arc<ControlService> {
        let pool = open_sqlite_pool("sqlite::memory:", 1).await.unwrap();
        let auth_store = Arc::new(SqliteAuthStore::from_pool(pool.clone()));
        auth_store.migrate().await.unwrap();
        let rule_store = Arc::new(SqliteRuleStore::new(pool.clone()));
        rule_store.migrate().await.unwrap();
        rule_store.save_routing(initial_routing).await.unwrap();
        ControlService::new(
            auth_store,
            rule_store,
            Arc::new(SqliteTokenCacheProvider::new(pool)),
        )
        .await
        .unwrap()
    }

    fn revision_for(snapshot: &ConfigSnapshot) -> ControlRevision {
        ControlRevision {
            epoch: "test-epoch".into(),
            sequence: snapshot.resource_version,
        }
    }

    fn state_for(snapshot: Arc<ConfigSnapshot>) -> WatchConnectionState {
        let revision = revision_for(snapshot.as_ref());
        let hash = snapshot_content_hash(snapshot.as_ref()).unwrap();
        WatchConnectionState {
            last_acked_snapshot: snapshot,
            last_acked_revision: revision,
            last_acked_hash: hash,
        }
    }

    async fn exchange(
        event: ConfigEvent,
        response: ApplyResponse,
        state: &mut WatchConnectionState,
        target_snapshot: &Arc<ConfigSnapshot>,
    ) -> anyhow::Result<AckOutcome> {
        let (server_io, client_io) = tokio::io::duplex(64 * 1024);
        let (mut server_reader, mut server_writer) = split(server_io);
        let (mut client_reader, mut client_writer) = split(client_io);
        let expected_event = event.clone();
        let client_task = tokio::spawn(async move {
            let received = recv_config_event(&mut client_reader).await?;
            assert_eq!(received, expected_event);
            send_apply_response(&mut client_writer, &response).await
        });
        let result = send_and_ack(
            &mut server_reader,
            &mut server_writer,
            &event,
            state,
            target_snapshot,
        )
        .await;
        client_task.await.unwrap().unwrap();
        result
    }

    #[tokio::test]
    async fn tcp_watch_sends_snapshot_then_delta_after_ack() {
        let initial_routing = base_routing();
        let svc = setup_service(&initial_routing).await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_task = tokio::spawn(
            WatchServer::new(Arc::clone(&svc), addr, None).run_with_listener(listener),
        );

        let stream = TcpStream::connect(addr).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        send_watch_request(
            &mut writer,
            &WatchRequest {
                token: None,
                last_applied_revision: None,
                last_applied_hash: None,
            },
        )
        .await
        .unwrap();

        let initial_event =
            tokio::time::timeout(Duration::from_secs(2), recv_config_event(&mut reader))
                .await
                .unwrap()
                .unwrap();
        let (initial_revision, initial_hash) = match &initial_event {
            ConfigEvent::Snapshot(snapshot) => {
                assert_eq!(snapshot.snapshot, *svc.snapshot());
                (snapshot.revision.clone(), snapshot.content_hash.clone())
            }
            ConfigEvent::Delta(_) => panic!("initial watch event must be a Snapshot"),
        };
        send_apply_response(
            &mut writer,
            &ApplyResponse {
                revision: initial_revision.clone(),
                content_hash: initial_hash.clone(),
                status: ApplyStatus::Applied,
                reason: None,
            },
        )
        .await
        .unwrap();

        let mut next_routing = initial_routing.clone();
        next_routing.egress_upstreams[0].servers[0].address = "127.0.0.1:20_000".into();
        svc.save_routing(&next_routing).await.unwrap();
        svc.do_publish().await.unwrap();

        let delta_event =
            tokio::time::timeout(Duration::from_secs(2), recv_config_event(&mut reader))
                .await
                .unwrap()
                .unwrap();
        let (target_revision, target_hash) = match &delta_event {
            ConfigEvent::Delta(delta) => {
                assert_eq!(delta.base_revision.sequence, initial_revision.sequence);
                assert_eq!(delta.operations.len(), 1);
                (delta.target_revision.clone(), delta.target_hash.clone())
            }
            ConfigEvent::Snapshot(_) => panic!("small routing change should use Delta"),
        };
        send_apply_response(
            &mut writer,
            &ApplyResponse {
                revision: target_revision,
                content_hash: target_hash,
                status: ApplyStatus::Applied,
                reason: None,
            },
        )
        .await
        .unwrap();

        drop(writer);
        drop(reader);
        server_task.abort();
        let _ = server_task.await;
    }

    #[tokio::test]
    async fn resync_recovers_on_same_connection_and_duplicate_is_idempotent() {
        let base = Arc::new(ConfigSnapshot {
            resource_version: 1,
            ingress_listeners: vec![],
            client_groups: vec![],
            egress_upstreams: vec![],
            egress_vhost_rules: vec![],
            token_cache: vec![],
        });
        let mut target_value = base.as_ref().clone();
        target_value.resource_version = 2;
        target_value.egress_upstreams.push(EgressUpstreamDef {
            name: "api".into(),
            lb_policy: "round_robin".into(),
            servers: vec![],
        });
        let target = Arc::new(target_value);
        let mut state = state_for(base.clone());
        let base_revision = state.last_acked_revision.clone();
        let base_hash = state.last_acked_hash.clone();
        let target_revision = revision_for(target.as_ref());
        let target_hash = snapshot_content_hash(target.as_ref()).unwrap();
        let delta = ConfigEvent::Delta(ConfigDelta {
            base_revision,
            base_hash,
            target_revision: target_revision.clone(),
            target_hash: target_hash.clone(),
            operations: vec![ConfigOperation::UpsertEgressUpstream(
                target.egress_upstreams[0].clone(),
            )],
        });

        let outcome = exchange(
            delta,
            ApplyResponse {
                revision: target_revision.clone(),
                content_hash: target_hash.clone(),
                status: ApplyStatus::ResyncRequired,
                reason: Some("base mismatch".into()),
            },
            &mut state,
            &target,
        )
        .await
        .unwrap();
        assert!(matches!(outcome, AckOutcome::Resync));
        assert_eq!(state.last_acked_snapshot, base);
        assert_eq!(state.last_acked_revision.sequence, 1);

        let snapshot_event = ConfigEvent::Snapshot(versioned_snapshot(
            target_revision.clone(),
            target_hash.clone(),
            target.as_ref().clone(),
        ));
        exchange(
            snapshot_event.clone(),
            ApplyResponse {
                revision: target_revision.clone(),
                content_hash: target_hash.clone(),
                status: ApplyStatus::Applied,
                reason: None,
            },
            &mut state,
            &target,
        )
        .await
        .unwrap();
        assert_eq!(state.last_acked_snapshot, target);

        let outcome = exchange(
            snapshot_event,
            ApplyResponse {
                revision: target_revision,
                content_hash: target_hash,
                status: ApplyStatus::Duplicate,
                reason: None,
            },
            &mut state,
            &target,
        )
        .await
        .unwrap();
        assert!(matches!(outcome, AckOutcome::Applied));
        assert_eq!(state.last_acked_snapshot, target);
    }

    #[tokio::test]
    async fn rejected_response_and_identity_mismatch_keep_last_ack_state() {
        let base = Arc::new(ConfigSnapshot {
            resource_version: 1,
            ingress_listeners: vec![],
            client_groups: vec![],
            egress_upstreams: vec![],
            egress_vhost_rules: vec![],
            token_cache: vec![],
        });
        let target = Arc::new(ConfigSnapshot {
            resource_version: 2,
            egress_upstreams: vec![EgressUpstreamDef {
                name: "api".into(),
                lb_policy: "round_robin".into(),
                servers: vec![],
            }],
            ..base.as_ref().clone()
        });
        let mut state = state_for(base.clone());
        let revision = revision_for(target.as_ref());
        let hash = snapshot_content_hash(target.as_ref()).unwrap();
        let event = ConfigEvent::Snapshot(versioned_snapshot(
            revision.clone(),
            hash.clone(),
            target.as_ref().clone(),
        ));

        let error = exchange(
            event.clone(),
            ApplyResponse {
                revision: revision.clone(),
                content_hash: hash.clone(),
                status: ApplyStatus::Rejected,
                reason: Some("runtime rejected config".into()),
            },
            &mut state,
            &target,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("runtime rejected config"));
        assert_eq!(state.last_acked_snapshot, base);

        let error = exchange(
            event,
            ApplyResponse {
                revision: ControlRevision {
                    epoch: "test-epoch".into(),
                    sequence: 99,
                },
                content_hash: "wrong-hash".into(),
                status: ApplyStatus::Applied,
                reason: None,
            },
            &mut state,
            &target,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("does not match the sent event"));
        assert_eq!(state.last_acked_snapshot, base);
    }
}
