use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse, HttpRequest, StreamType};
use tracing;
use function_name::named;
use dashmap::DashMap;

pub type LastHeartbeat = u64;
pub type StreamKey = (String, String); // (client_id, stream_id)

pub struct ManagedClientRegistry {
    // group_id -> stream_type -> (client_id, stream_id) -> last_heartbeat
    pub groups: Arc<DashMap<String, DashMap<StreamType, DashMap<StreamKey, LastHeartbeat>>>>,
}

impl ManagedClientRegistry {
    pub fn new() -> Self {
        Self {
            groups: Arc::new(DashMap::new()),
        }
    }

    /// 注册/心跳：更新 group/stream_type 下 (client_id, stream_id) 的 last_heartbeat
    pub fn update_heartbeat_multi(&self, client_id: &str, group: &str, stream_type: StreamType, stream_id: &str) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let stream_map = self.groups.entry(group.to_string()).or_insert_with(DashMap::new);
        let client_map = stream_map.entry(stream_type).or_insert_with(DashMap::new);
        client_map.insert((client_id.to_string(), stream_id.to_string()), now);
        tracing::info!(
            "[client stream heartbeat] group={}, stream_type={:?}, client_id={}, stream_id={} last_heartbeat={}",
            group, stream_type, client_id, stream_id, now
        );
    }

    /// 获取 group 下所有健康的 (client_id, stream_type, stream_id)
    pub fn get_healthy_streams_in_group(&self, group: &str, stream_type: Option<StreamType>, timeout_secs: u64) -> Vec<(String, StreamType, String)> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut result = Vec::new();
        if let Some(stream_map) = self.groups.get(group) {
            let stream_types: Vec<StreamType> = if let Some(st) = stream_type {
                vec![st]
            } else {
                stream_map.iter().map(|entry| *entry.key()).collect()
            };
            for st in stream_types {
                if let Some(client_map) = stream_map.get(&st) {
                    for entry in client_map.iter() {
                        let ((client_id, stream_id), last_heartbeat) = entry.pair();
                        if now - *last_heartbeat < timeout_secs {
                            result.push((client_id.clone(), st, stream_id.clone()));
                        }
                    }
                }
            }
        }
        result
    }

    /// 获取 group 下所有 client_id（去重）
    pub fn get_clients_in_group(&self, group: &str, timeout_secs: u64) -> Vec<String> {
        let mut set = std::collections::HashSet::new();
        for (client_id, _, _) in self.get_healthy_streams_in_group(group, None, timeout_secs) {
            set.insert(client_id);
        }
        set.into_iter().collect()
    }
} 