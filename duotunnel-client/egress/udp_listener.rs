use crate::bootstrap::config::UdpEntryConfig;
use crate::runtime::engine::ClientService;
use crate::tunnel::conn_pool::EntryConnPool;
use anyhow::Result;
use duotunnel_lib::{
    encode_udp_datagram_envelope, ProxyName, UdpDatagramEnvelope, UdpSessionKey, MAX_DATAGRAM_BYTES,
};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

const UDP_AFFINITY_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_UDP_AFFINITIES_PER_LISTENER: usize = 4096;
const UDP_AFFINITY_SWEEP_INTERVAL: u64 = 256;

struct UdpAffinity {
    stable_id: usize,
    last_seen: Instant,
}

#[derive(Default)]
pub struct UdpListenerRegistry {
    sockets: RwLock<HashMap<ProxyName, Arc<UdpSocket>>>,
}

impl UdpListenerRegistry {
    pub fn register(&self, proxy_name: ProxyName, socket: Arc<UdpSocket>) {
        self.sockets.write().insert(proxy_name, socket);
    }

    pub async fn forward_reply(&self, envelope: UdpDatagramEnvelope) -> Result<()> {
        let socket = self
            .sockets
            .read()
            .get(&envelope.session.proxy_name)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("no udp listener for {}", envelope.session.proxy_name)
            })?;
        let addr = envelope.session.client_addr;
        socket
            .send_to(
                &envelope.payload,
                SocketAddr::new(addr, envelope.session.client_port),
            )
            .await?;
        Ok(())
    }
}

pub struct UdpEgressListenerService {
    pub entry: UdpEntryConfig,
    pub pool: Arc<EntryConnPool>,
    pub registry: Arc<UdpListenerRegistry>,
}

async fn start_udp_listener(
    pool: Arc<EntryConnPool>,
    registry: Arc<UdpListenerRegistry>,
    shutdown: CancellationToken,
    entry: UdpEntryConfig,
) -> Result<()> {
    let addr: SocketAddr = format!("127.0.0.1:{}", entry.port).parse()?;
    let socket = Arc::new(UdpSocket::bind(addr).await?);
    registry.register(entry.proxy_name.clone().into(), socket.clone());
    info!(addr = %addr, proxy_name = %entry.proxy_name, "client udp listener started");

    let mut buf = [0u8; MAX_DATAGRAM_BYTES];
    let mut affinities = HashMap::<UdpSessionKey, UdpAffinity>::new();
    let mut received_datagrams = 0u64;
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                info!(addr = %addr, proxy_name = %entry.proxy_name, "client udp listener stopping");
                return Ok(());
            }
            recv = socket.recv_from(&mut buf) => {
                let (n, client_addr) = recv?;
                let envelope = UdpDatagramEnvelope {
                    session: UdpSessionKey {
                        proxy_name: entry.proxy_name.clone().into(),
                        client_addr: client_addr.ip(),
                        client_port: client_addr.port(),
                    },
                    payload: buf[..n].to_vec(),
                };
                let session_key = envelope.session.clone();
                let encoded = match encode_udp_datagram_envelope(&envelope) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        warn!(proxy_name = %entry.proxy_name, client_addr = %client_addr, error = %e, "dropping oversized udp datagram");
                        continue;
                    }
                };
                let encoded = bytes::Bytes::copy_from_slice(encoded.as_slice());
                received_datagrams = received_datagrams.wrapping_add(1);
                if received_datagrams.is_multiple_of(UDP_AFFINITY_SWEEP_INTERVAL) {
                    affinities.retain(|_, affinity| {
                        affinity.last_seen.elapsed() < UDP_AFFINITY_IDLE_TIMEOUT
                            && pool.connection_by_stable_id(affinity.stable_id).is_some()
                    });
                }
                let pool_size = pool.pool_size();
                let preferred_shard =
                    pool.shard_for_hash(&(entry.proxy_name.as_str(), client_addr.ip(), client_addr.port()));
                let mut tried_conn_ids = HashSet::with_capacity(pool_size.min(8));
                let mut delivered = false;
                if let Some(stable_id) = affinities.get(&session_key).map(|entry| entry.stable_id) {
                    tried_conn_ids.insert(stable_id);
                    if let Some(conn) = pool.connection_by_stable_id(stable_id) {
                        match conn.handle.send_datagram(encoded.clone()).await {
                            Ok(()) => {
                                conn.mark_business_completed();
                                if let Some(affinity) = affinities.get_mut(&session_key) {
                                    affinity.last_seen = Instant::now();
                                }
                                delivered = true;
                            }
                            Err(error) => {
                                warn!(conn_id = stable_id, error = %error, "sticky udp datagram send failed");
                                affinities.remove(&session_key);
                                if conn.handle.close_reason().is_some() {
                                    pool.remove_stable_id(stable_id).await.map_err(|actor_error| {
                                        anyhow::anyhow!(
                                            "failed to evict closed UDP connection {stable_id}: {actor_error}"
                                        )
                                    })?;
                                }
                            }
                        }
                    } else {
                        affinities.remove(&session_key);
                    }
                }
                if delivered {
                    continue;
                }
                for _ in 0..pool_size.max(1) {
                    let Some(conn) = pool
                        .next_conn_for_shard_excluding(preferred_shard, &tried_conn_ids)
                    else {
                        break;
                    };
                    tried_conn_ids.insert(conn.handle.stable_id());
                    match conn
                        .handle
                        .send_datagram(encoded.clone())
                        .await
                    {
                        Ok(()) => {
                            conn.mark_business_completed();
                            if affinities.len() >= MAX_UDP_AFFINITIES_PER_LISTENER {
                                if let Some(oldest) = affinities
                                    .iter()
                                    .min_by_key(|(_, affinity)| affinity.last_seen)
                                    .map(|(key, _)| key.clone())
                                {
                                    affinities.remove(&oldest);
                                }
                            }
                            affinities.insert(
                                session_key.clone(),
                                UdpAffinity {
                                    stable_id: conn.handle.stable_id(),
                                    last_seen: Instant::now(),
                                },
                            );
                            delivered = true;
                            break;
                        }
                        Err(e) => {
                            warn!(conn_id = conn.handle.stable_id(), error = %e, "udp datagram send failed");
                            if conn.handle.close_reason().is_some() {
                                pool.remove_stable_id(conn.handle.stable_id())
                                    .await
                                    .map_err(|actor_error| {
                                        anyhow::anyhow!(
                                            "failed to evict closed UDP connection {}: {actor_error}",
                                            conn.handle.stable_id()
                                        )
                                    })?;
                            }
                        }
                    }
                }
                if !delivered {
                    debug!(proxy_name = %entry.proxy_name, client_addr = %client_addr, "no active quic connection available for udp datagram");
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl ClientService for UdpEgressListenerService {
    fn name(&self) -> &'static str {
        "egress-udp-listener"
    }

    async fn start(&self, shutdown: CancellationToken) -> Result<()> {
        start_udp_listener(
            self.pool.clone(),
            self.registry.clone(),
            shutdown,
            self.entry.clone(),
        )
        .await
    }
}
