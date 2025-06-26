use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse, HttpRequest};
use tracing;

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub group: String,
    pub last_heartbeat: u64, // timestamp
}

pub struct ClientRegistry {
    // client_id -> ClientInfo
    clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 注册 client 到指定 group
    pub fn register_client(&self, client_id: &str, group: &str) -> bool {
        let mut clients = self.clients.lock().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        tracing::info!("[register_client] BEFORE: {:?}", clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        clients.insert(client_id.to_string(), ClientInfo {
            group: group.to_string(),
            last_heartbeat: now,
        });
        tracing::info!("[register_client] AFTER: {:?}", clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        tracing::info!("Registered client {} to group {}", client_id, group);
        true
    }

    /// 更新 client 心跳时间
    pub fn update_heartbeat(&self, client_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        tracing::info!("[update_heartbeat] BEFORE: {:?}", clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        if let Some(client_info) = clients.get_mut(client_id) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            client_info.last_heartbeat = now;
            tracing::debug!("Updated heartbeat for client {}", client_id);
        }
        tracing::info!("[update_heartbeat] AFTER: {:?}", clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        tracing::debug!("All clients after heartbeat: {:?}", *clients);
    }

    /// 获取指定 group 下的所有健康 client
    pub fn get_clients_in_group(&self, group: &str) -> Vec<String> {
        let clients = self.clients.lock().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        clients.iter()
            .filter(|(_, info)| {
                info.group == group && self.is_client_healthy_internal(info.last_heartbeat, now)
            })
            .map(|(client_id, _)| client_id.clone())
            .collect()
    }

    /// 检查 client 是否健康（60秒内有心跳）
    pub fn is_client_healthy(&self, client_id: &str) -> bool {
        let clients = self.clients.lock().unwrap();
        if let Some(client_info) = clients.get(client_id) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            self.is_client_healthy_internal(client_info.last_heartbeat, now)
        } else {
            false
        }
    }

    fn is_client_healthy_internal(&self, last_heartbeat: u64, now: u64) -> bool {
        now - last_heartbeat < 60 // 60秒超时
    }

    /// 选择 group 下的一个健康 client（简单轮询）
    pub fn select_client_in_group(&self, group: &str) -> Option<String> {
        let healthy_clients = self.get_clients_in_group(group);
        let clients = self.clients.lock().unwrap();
        tracing::info!("[select_client_in_group] group={}, healthy_clients={:?}, all_clients={:?}", group, healthy_clients, clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        if healthy_clients.is_empty() {
            None
        } else {
            // 简单选择第一个，后续可实现轮询
            Some(healthy_clients[0].clone())
        }
    }

    /// 清理不健康的 client
    pub fn cleanup_unhealthy_clients(&self) {
        let mut clients = self.clients.lock().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        tracing::info!("[cleanup_unhealthy_clients] BEFORE: {:?}", clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        clients.retain(|client_id, info| {
            let healthy = self.is_client_healthy_internal(info.last_heartbeat, now);
            if !healthy {
                tracing::info!("Removing unhealthy client: {}", client_id);
            }
            healthy
        });
        tracing::info!("[cleanup_unhealthy_clients] AFTER: {:?}", clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        tracing::info!("All clients after cleanup: {:?}", *clients);
    }

    /// 获取所有 client 状态（用于调试）
    pub fn get_all_clients(&self) -> HashMap<String, ClientInfo> {
        self.clients.lock().unwrap().clone()
    }

    /// 发送消息到指定 client
    pub async fn send_to_client(&self, client_id: &str, _msg: TunnelMessage) -> Result<(mpsc::Sender<TunnelMessage>, oneshot::Receiver<HttpResponse>), String> {
        // 这是一个简化的实现，实际应该维护到客户端的连接映射
        // 这里返回一个模拟的通道对，实际实现需要更复杂的连接管理
        let (_tx, _rx) = mpsc::channel::<TunnelMessage>(128);
        let (_resp_tx, _resp_rx) = oneshot::channel::<HttpResponse>();
        
        // 在实际实现中，这里应该查找活跃的客户端连接并发送消息
        // 目前返回错误，因为我们还没有完整的连接管理
        Err(format!("Client {} not connected", client_id))
    }

    /// 发送消息到指定 group
    pub async fn send_to_group(&self, _group_id: &str, _msg: HttpRequest) -> Result<(mpsc::Sender<TunnelMessage>, oneshot::Receiver<HttpResponse>), Box<dyn std::error::Error + Send + Sync>> {
        // 这是一个简化的实现，实际应该维护到客户端的连接映射
        let (_tx, _rx) = mpsc::channel::<TunnelMessage>(128);
        let (_resp_tx, resp_rx) = oneshot::channel::<HttpResponse>();
        
        // TODO: 实现实际的 group 消息发送逻辑
        Ok((_tx, resp_rx))
    }

    /// 根据请求查找对应的 group
    pub fn find_group_for_request(&self, host: &str, _path: &str) -> Option<String> {
        let clients = self.clients.lock().unwrap();
        clients.iter()
            .filter(|(_, info)| {
                info.group == host && self.is_client_healthy_internal(info.last_heartbeat, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs())
            })
            .map(|(client_id, _)| client_id.clone())
            .next()
    }

    pub fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        tracing::info!("[remove_client] BEFORE: {:?}", clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        clients.remove(client_id);
        tracing::info!("[remove_client] AFTER: {:?}", clients.iter().map(|(id, info)| (id, &info.group)).collect::<Vec<_>>());
        tracing::info!("Removed client {}", client_id);
        tracing::info!("All clients after remove: {:?}", *clients);
    }
} 