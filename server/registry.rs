use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use tunnel_lib::tunnel::{TunnelMessage, HttpResponse, HttpRequest};
use tracing;
use function_name::named;

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
    #[named]
    pub fn register_client(&self, client_id: &str, group: &str) -> bool {
        let mut clients = self.clients.lock().unwrap();
        let before: Vec<_> = clients.iter().map(|(id, info)| (id.clone(), info.group.clone())).collect();
        let existed = clients.contains_key(client_id);
        clients.insert(client_id.to_string(), ClientInfo {
            group: group.to_string(),
            last_heartbeat: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        });
        let after: Vec<_> = clients.iter().map(|(id, info)| (id.clone(), info.group.clone())).collect();
        tracing::info!(target: module_path!(), "[{}] {}: client_id={}, group={}, existed={}, before={:?}, after={:?}", function_name!(), "register_client", client_id, group, existed, before, after);
        true
    }

    /// 更新 client 心跳时间
    pub fn update_heartbeat(&self, client_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        let before = clients.iter().map(|(id, info)| (id.clone(), info.group.clone())).collect::<Vec<_>>();
        if let Some(client_info) = clients.get_mut(client_id) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let old_group = client_info.group.clone();
            client_info.last_heartbeat = now;
            let after_group = client_info.group.clone();
            if old_group != after_group {
                tracing::info!("[update_heartbeat] client {} group changed: from {} to {}", client_id, old_group, after_group);
            }
        }
        // 不再打印BEFORE/AFTER，只有变化时才打印
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
    #[named]
    pub fn cleanup_unhealthy_clients(&self) {
        let mut clients = self.clients.lock().unwrap();
        let before: Vec<_> = clients.iter().map(|(id, info)| (id.clone(), info.group.clone())).collect();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut removed = Vec::new();
        let mut retain_vec = Vec::new();
        for (client_id, info) in clients.iter() {
            let healthy = self.is_client_healthy_internal(info.last_heartbeat, now);
            if healthy {
                retain_vec.push((client_id.clone(), info.clone()));
            } else {
                removed.push(client_id.clone());
            }
        }
        clients.clear();
        for (id, info) in retain_vec {
            clients.insert(id, info);
        }
        let after: Vec<_> = clients.iter().map(|(id, info)| (id.clone(), info.group.clone())).collect();
        tracing::info!(target: module_path!(), "[{}] {}: removed_clients={:?}, before={:?}, after={:?}", function_name!(), "cleanup_unhealthy_clients", removed, before, after);
        tracing::info!(target: module_path!(), "[{}] All clients after cleanup: {:?}", function_name!(), *clients);
    }

    /// 获取所有 client (id, group) 列表
    pub fn get_all_clients(&self) -> Vec<(String, String)> {
        let clients = self.clients.lock().unwrap();
        clients.iter().map(|(id, info)| (id.clone(), info.group.clone())).collect()
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

    #[named]
    pub fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        let before: Vec<_> = clients.iter().map(|(id, info)| (id.clone(), info.group.clone())).collect();
        let existed = clients.contains_key(client_id);
        clients.remove(client_id);
        let after: Vec<_> = clients.iter().map(|(id, info)| (id.clone(), info.group.clone())).collect();
        tracing::info!(target: module_path!(), "[{}] {}: client_id={}, existed={}, before={:?}, after={:?}", function_name!(), "remove_client", client_id, existed, before, after);
        tracing::info!(target: module_path!(), "[{}] Removed client {}", function_name!(), client_id);
        tracing::info!(target: module_path!(), "[{}] All clients after remove: {:?}", function_name!(), *clients);
    }
} 