use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tunnel_lib::frame::ProtocolType;
use crate::types::SessionState;
use lru::LruCache;
use std::num::NonZeroUsize;

const DEFAULT_MAX_SESSIONS: usize = 10000;
const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(300);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Clone)]
struct SessionEntry {
    state: Arc<Mutex<SessionState>>,
    created_at: Instant,
    last_accessed: Instant,
}

pub struct SessionManager {
    sessions: Arc<Mutex<LruCache<u64, SessionEntry>>>,
    max_size: usize,
    ttl: Duration,
}

impl SessionManager {
    pub fn new(max_size: Option<usize>, ttl: Option<Duration>) -> Self {
        let max_size = max_size.unwrap_or(DEFAULT_MAX_SESSIONS);
        let ttl = ttl.unwrap_or(DEFAULT_SESSION_TTL);
        
        Self {
            sessions: Arc::new(Mutex::new(
                LruCache::new(NonZeroUsize::new(max_size).unwrap_or(NonZeroUsize::new(1).unwrap()))
            )),
            max_size,
            ttl,
        }
    }

    pub async fn get_or_create(&self, id: u64, protocol_type: ProtocolType) -> Arc<Mutex<SessionState>> {
        let mut cache = self.sessions.lock().await;
        
        if let Some(entry) = cache.get(&id) {
            // Update last accessed time
            let mut entry = entry.clone();
            entry.last_accessed = Instant::now();
            cache.put(id, entry.clone());
            return entry.state;
        }
        
        // Create new session
        let state = Arc::new(Mutex::new(SessionState::new(protocol_type)));
        let entry = SessionEntry {
            state: state.clone(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
        };
        
        cache.put(id, entry);
        state
    }

    pub async fn get(&self, id: u64) -> Option<Arc<Mutex<SessionState>>> {
        let mut cache = self.sessions.lock().await;
        
        if let Some(entry) = cache.get(&id) {
            // Update last accessed time
            let mut entry = entry.clone();
            entry.last_accessed = Instant::now();
            cache.put(id, entry.clone());
            Some(entry.state)
        } else {
            None
        }
    }

    pub async fn remove(&self, id: u64) -> Option<Arc<Mutex<SessionState>>> {
        let mut cache = self.sessions.lock().await;
        cache.pop(&id).map(|entry| entry.state)
    }

    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.sessions.lock().await;
        let now = Instant::now();
        let mut removed = 0;
        
        let expired_ids: Vec<u64> = cache.iter()
            .filter(|(_, entry)| {
                now.duration_since(entry.created_at) > self.ttl ||
                now.duration_since(entry.last_accessed) > self.ttl
            })
            .map(|(id, _)| *id)
            .collect();
        
        for id in expired_ids {
            cache.pop(&id);
            removed += 1;
        }
        
        removed
    }

    pub async fn len(&self) -> usize {
        let cache = self.sessions.lock().await;
        cache.len()
    }

    pub async fn clear(&self) {
        let mut cache = self.sessions.lock().await;
        cache.clear();
    }

    pub fn start_cleanup_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                let removed = self.cleanup_expired().await;
                if removed > 0 {
                    tracing::debug!("Cleaned up {} expired sessions", removed);
                }
            }
        })
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(None, None)
    }
}
