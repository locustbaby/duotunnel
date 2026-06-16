use crate::bootstrap::config::UdpEntryConfig;
use crate::runtime::engine::ClientService;
use crate::tunnel::conn_pool::EntryConnPool;
use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{
    decode_udp_datagram_envelope, encode_udp_datagram_envelope, ProxyName, UdpDatagramEnvelope,
    UdpSessionKey, MAX_DATAGRAM_BYTES,
};

#[derive(Default)]
pub struct UdpListenerRegistry {
    sockets: RwLock<HashMap<ProxyName, Arc<UdpSocket>>>,
}

impl UdpListenerRegistry {
    pub fn register(&self, proxy_name: ProxyName, socket: Arc<UdpSocket>) {
        if let Ok(mut guard) = self.sockets.write() {
            guard.insert(proxy_name, socket);
        }
    }

    pub async fn forward_reply(&self, envelope: UdpDatagramEnvelope) -> Result<()> {
        let socket = if let Ok(guard) = self.sockets.read() {
            guard.get(&envelope.session.proxy_name).cloned()
        } else {
            None
        }
        .ok_or_else(|| anyhow::anyhow!("no udp listener for {}", envelope.session.proxy_name))?;
        let addr: std::net::IpAddr = envelope.session.client_addr.parse()?;
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
                        client_addr: client_addr.ip().to_string(),
                        client_port: client_addr.port(),
                    },
                    payload: buf[..n].to_vec(),
                };
                let encoded = match encode_udp_datagram_envelope(&envelope) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        warn!(proxy_name = %entry.proxy_name, client_addr = %client_addr, error = %e, "dropping oversized udp datagram");
                        continue;
                    }
                };
                let pool_size = pool.pool_size().await;
                let preferred_shard =
                    pool.shard_for_hash(&(entry.proxy_name.as_str(), client_addr.ip(), client_addr.port()));
                let mut tried_conn_ids = Vec::with_capacity(pool_size.min(8));
                let mut delivered = false;
                for _ in 0..pool_size.max(1) {
                    let Some(conn) = pool
                        .next_conn_for_shard_excluding(preferred_shard, tried_conn_ids.clone())
                        .await
                    else {
                        break;
                    };
                    tried_conn_ids.push(conn.handle.stable_id());
                    match conn
                        .handle
                        .send_datagram(bytes::Bytes::copy_from_slice(encoded.as_slice()))
                        .await
                    {
                        Ok(()) => {
                            delivered = true;
                            break;
                        }
                        Err(e) => {
                            warn!(conn_id = conn.handle.stable_id(), error = %e, "udp datagram send failed");
                            if conn.handle.close_reason().is_some() {
                                pool.remove_stable_id(conn.handle.stable_id()).await;
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

pub async fn forward_incoming_datagram(
    registry: &UdpListenerRegistry,
    payload: bytes::Bytes,
) -> Result<()> {
    let envelope = decode_udp_datagram_envelope(payload.as_ref())?;
    registry.forward_reply(envelope).await
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
