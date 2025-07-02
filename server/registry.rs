use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse, HttpRequest, StreamType};
use tracing;
use function_name::named;
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

pub type LastHeartbeat = u64;
pub type StreamKey = (String, String); // (client_id, stream_id)

// group -> stream_type -> (client_id, stream_id) -> (tx, CancellationToken, LastHeartbeat)
pub type ConnectedStreams = DashMap<String, DashMap<StreamType, DashMap<StreamKey, (mpsc::Sender<TunnelMessage>, CancellationToken, LastHeartbeat)>>>;

pub struct ManagedClientRegistry {
    pub connected_streams: ConnectedStreams,
}

impl ManagedClientRegistry {
    pub fn new() -> Self {
        Self {
            connected_streams: ConnectedStreams::new(),
        }
    }

    /// 同步流状态：注册或心跳，更新 group/stream_type 下 (client_id, stream_id) 的 last_heartbeat
    pub fn sync_stream(&self, client_id: &str, group: &str, stream_id: &str, stream_type: StreamType, tx: mpsc::Sender<TunnelMessage>, token: CancellationToken) {
        let now = chrono::Utc::now().timestamp() as u64;
        let stream_map = self.connected_streams.entry(group.to_string()).or_insert_with(DashMap::new);
        let client_map = stream_map.entry(stream_type).or_insert_with(DashMap::new);
        client_map.insert((client_id.to_string(), stream_id.to_string()), (tx, token, now));
    }

    /// 获取 group 下所有健康的 (client_id, stream_type, stream_id)
    pub fn get_healthy_streams_in_group(&self, group: &str, stream_type: Option<StreamType>, timeout_secs: u64) -> Vec<(String, StreamType, String)> {
        let now = chrono::Utc::now().timestamp() as u64;
        let mut result = Vec::new();
        if let Some(stream_map) = self.connected_streams.get(group) {
            let stream_types: Vec<StreamType> = if let Some(st) = stream_type {
                vec![st]
            } else {
                stream_map.iter().map(|entry| *entry.key()).collect()
            };
            for st in stream_types {
                if let Some(client_map) = stream_map.get(&st) {
                    for entry in client_map.iter() {
                        let ((client_id, stream_id), (tx, token, last_heartbeat)) = entry.pair();
                        if now - *last_heartbeat < timeout_secs && !token.is_cancelled() {
                            result.push((client_id.clone(), st, stream_id.clone()));
                        }
                    }
                }
            }
        }
        result
    }

    /// 获取指定流的 tx, token 和 last_heartbeat
    pub fn get_stream_info(&self, group: &str, stream_type: StreamType, client_id: &str, stream_id: &str) -> Option<(mpsc::Sender<TunnelMessage>, CancellationToken, LastHeartbeat)> {
        let stream_map = self.connected_streams.get(group)?;
        let client_map = stream_map.get(&stream_type)?;
        let entry = client_map.get(&(client_id.to_string(), stream_id.to_string()))?;
        let (tx, token, last_heartbeat) = entry.value();
        Some((tx.clone(), token.clone(), *last_heartbeat))
    }

    /// 返回一个 Future，可由 main.rs 的 JoinSet 统一 spawn
    pub async fn cleanup_task(self: Arc<Self>, scan_interval_secs: u64, timeout_secs: u64, shutdown_token: CancellationToken) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(scan_interval_secs));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let now = chrono::Utc::now().timestamp() as u64;
                    for group_entry in self.connected_streams.iter() {
                        let group = group_entry.key();
                        for stream_type_entry in group_entry.value().iter() {
                            let stream_type = stream_type_entry.key();
                            let client_map = stream_type_entry.value();
                            client_map.retain(|(client_id, stream_id), (_tx, token, last_heartbeat)| {
                                let healthy = now - *last_heartbeat < timeout_secs && !token.is_cancelled();
                                if !healthy {
                                    tracing::info!(%group, ?stream_type, %client_id, %stream_id, "stream expired, removing");
                                }
                                healthy
                            });
                        }
                    }
                }
                _ = shutdown_token.cancelled() => {
                    tracing::info!("Cleanup task received shutdown signal, exiting");
                    break;
                }
            }
        }
    }
} 