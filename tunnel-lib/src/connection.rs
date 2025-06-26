use crate::tunnel::*;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct ConnectionManager {
    client_id: String,
    last_heartbeat: Arc<Mutex<Instant>>,
    tx: mpsc::Sender<TunnelMessage>,
    heartbeat_interval: Duration,
}

impl ConnectionManager {
    pub fn new(client_id: String, tx: mpsc::Sender<TunnelMessage>) -> Self {
        Self {
            client_id,
            last_heartbeat: Arc::new(Mutex::new(Instant::now())),
            tx,
            heartbeat_interval: Duration::from_secs(30),
        }
    }

    pub async fn start_heartbeat(&self) {
        let mut interval = interval(self.heartbeat_interval);
        let tx = self.tx.clone();
        let client_id = self.client_id.clone();
        
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                let heartbeat = TunnelMessage {
                    client_id: client_id.clone(),
                    request_id: uuid::Uuid::new_v4().to_string(),
                    direction: Direction::ClientToServer as i32,
                    payload: Some(tunnel_message::Payload::Heartbeat(Heartbeat {
                        timestamp: chrono::Utc::now().timestamp(),
                    })),
                    trace_id: String::new(),
                };
                
                if tx.send(heartbeat).await.is_err() {
                    eprintln!("Failed to send heartbeat");
                    break;
                }
            }
        });
    }

    pub fn update_heartbeat(&self) {
        *self.last_heartbeat.lock().unwrap() = Instant::now();
    }

    pub fn is_alive(&self) -> bool {
        let last = *self.last_heartbeat.lock().unwrap();
        last.elapsed() < self.heartbeat_interval * 2
    }
}

#[derive(Clone)]
pub struct ClientRegistry {
    clients: Arc<tokio::sync::RwLock<HashMap<String, mpsc::Sender<TunnelMessage>>>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_client(&self, client_id: String, tx: mpsc::Sender<TunnelMessage>) {
        self.clients.write().await.insert(client_id, tx);
    }

    pub async fn unregister_client(&self, client_id: &str) {
        self.clients.write().await.remove(client_id);
    }

    pub async fn send_to_client(&self, client_id: &str, msg: TunnelMessage) -> Result<(), String> {
        let tx = {
            let clients = self.clients.read().await;
            clients.get(client_id).cloned()
        };
        
        if let Some(tx) = tx {
            tx.send(msg).await.map_err(|e| e.to_string())
        } else {
            Err(format!("Client {} not found", client_id))
        }
    }

    pub async fn get_client_count(&self) -> usize {
        self.clients.read().await.len()
    }
} 