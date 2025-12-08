use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::Mutex;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_rustls::HttpsConnector;
use http_body_util::Full;
use bytes::Bytes;
use tracing::{debug, warn};

/// Connection pool trait for extensibility
#[async_trait::async_trait]
pub trait ConnectionPool: Send + Sync {
    async fn get_connection(&self, target: &str) -> Result<Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>, PoolError>;
    async fn return_connection(&self, target: &str, conn: Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>);
    async fn health_check(&self, target: &str) -> bool;
}

#[derive(Debug)]
pub enum PoolError {
    PoolError(String),
    CircuitBreakerOpen,
    RateLimitExceeded,
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::PoolError(msg) => write!(f, "Connection pool error: {}", msg),
            PoolError::CircuitBreakerOpen => write!(f, "Circuit breaker is open"),
            PoolError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
        }
    }
}

impl std::error::Error for PoolError {}

/// Base connection pool implementation
pub struct BaseConnectionPool {
    client: Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>,
}

impl BaseConnectionPool {
    pub fn new(client: Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl ConnectionPool for BaseConnectionPool {
    async fn get_connection(&self, _target: &str) -> Result<Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>, PoolError> {
        Ok(self.client.clone())
    }

    async fn return_connection(&self, _target: &str, _conn: Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>) {
        // Base pool doesn't need to track connections
    }

    async fn health_check(&self, target: &str) -> bool {
        // Simple health check: try to connect
        use hyper::{Method, Request};
        use std::str::FromStr;
        
        if let Ok(uri) = hyper::Uri::from_str(target) {
            if let Ok(request) = Request::builder()
                .method(Method::HEAD)
                .uri(uri)
                .body(Full::new(Bytes::new()))
            {
                if let Ok(response) = self.client.request(request).await {
                    return response.status().is_success();
                }
            }
        }
        false
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreakerData {
    state: CircuitBreakerState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    success_count: u32,
}

impl Default for CircuitBreakerData {
    fn default() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure_time: None,
            success_count: 0,
        }
    }
}

/// Circuit breaker decorator for connection pool
pub struct CircuitBreakerDecorator {
    inner: Arc<dyn ConnectionPool>,
    state: Arc<Mutex<HashMap<String, CircuitBreakerData>>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

impl CircuitBreakerDecorator {
    pub fn new(
        inner: Arc<dyn ConnectionPool>,
        failure_threshold: u32,
        success_threshold: u32,
        timeout: Duration,
    ) -> Self {
        Self {
            inner,
            state: Arc::new(Mutex::new(HashMap::new())),
            failure_threshold,
            success_threshold,
            timeout,
        }
    }

    async fn get_circuit_state(&self, target: &str) -> CircuitBreakerState {
        let mut state_map = self.state.lock().await;
        let data = state_map.entry(target.to_string()).or_insert_with(CircuitBreakerData::default);
        
        // Check if timeout has passed for open state
        if matches!(data.state, CircuitBreakerState::Open) {
            if let Some(last_failure) = data.last_failure_time {
                if last_failure.elapsed() > self.timeout {
                    data.state = CircuitBreakerState::HalfOpen;
                    data.success_count = 0;
                    debug!("Circuit breaker for {} moved to HalfOpen", target);
                }
            }
        }
        
        data.state
    }

    async fn record_success(&self, target: &str) {
        let mut state_map = self.state.lock().await;
        let data = state_map.entry(target.to_string()).or_insert_with(CircuitBreakerData::default);
        
        match data.state {
            CircuitBreakerState::HalfOpen => {
                data.success_count += 1;
                if data.success_count >= self.success_threshold {
                    data.state = CircuitBreakerState::Closed;
                    data.failure_count = 0;
                    debug!("Circuit breaker for {} closed after {} successes", target, data.success_count);
                }
            }
            CircuitBreakerState::Closed => {
                data.failure_count = 0;
            }
            _ => {}
        }
    }

    async fn record_failure(&self, target: &str) {
        let mut state_map = self.state.lock().await;
        let data = state_map.entry(target.to_string()).or_insert_with(CircuitBreakerData::default);
        
        data.failure_count += 1;
        data.last_failure_time = Some(Instant::now());
        
        if data.failure_count >= self.failure_threshold {
            data.state = CircuitBreakerState::Open;
            warn!("Circuit breaker for {} opened after {} failures", target, data.failure_count);
        } else if matches!(data.state, CircuitBreakerState::HalfOpen) {
            data.state = CircuitBreakerState::Open;
            warn!("Circuit breaker for {} reopened after failure in HalfOpen", target);
        }
    }
}

#[async_trait::async_trait]
impl ConnectionPool for CircuitBreakerDecorator {
    async fn get_connection(&self, target: &str) -> Result<Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>, PoolError> {
        let state = self.get_circuit_state(target).await;
        
        match state {
            CircuitBreakerState::Open => {
                return Err(PoolError::CircuitBreakerOpen);
            }
            CircuitBreakerState::HalfOpen | CircuitBreakerState::Closed => {
                match self.inner.get_connection(target).await {
                    Ok(conn) => {
                        self.record_success(target).await;
                        Ok(conn)
                    }
                    Err(e) => {
                        self.record_failure(target).await;
                        Err(e)
                    }
                }
            }
        }
    }

    async fn return_connection(&self, target: &str, conn: Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>) {
        self.inner.return_connection(target, conn).await;
    }

    async fn health_check(&self, target: &str) -> bool {
        let state = self.get_circuit_state(target).await;
        if matches!(state, CircuitBreakerState::Open) {
            return false;
        }
        self.inner.health_check(target).await
    }
}

/// Rate limiter decorator for connection pool
pub struct RateLimitDecorator {
    inner: Arc<dyn ConnectionPool>,
    limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
    max_requests: u32,
    window: Duration,
}

struct RateLimiter {
    requests: Vec<Instant>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            requests: Vec::new(),
            max_requests,
            window,
        }
    }

    fn check(&mut self) -> bool {
        let now = Instant::now();
        
        // Remove old requests outside the window
        self.requests.retain(|&time| now.duration_since(time) < self.window);
        
        if self.requests.len() >= self.max_requests as usize {
            false
        } else {
            self.requests.push(now);
            true
        }
    }
}

impl RateLimitDecorator {
    pub fn new(
        inner: Arc<dyn ConnectionPool>,
        max_requests: u32,
        window: Duration,
    ) -> Self {
        Self {
            inner,
            limiters: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }
}

#[async_trait::async_trait]
impl ConnectionPool for RateLimitDecorator {
    async fn get_connection(&self, target: &str) -> Result<Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>, PoolError> {
        let mut limiters = self.limiters.lock().await;
        let limiter = limiters.entry(target.to_string())
            .or_insert_with(|| RateLimiter::new(self.max_requests, self.window));
        
        if !limiter.check() {
            return Err(PoolError::RateLimitExceeded);
        }
        
        drop(limiters);
        self.inner.get_connection(target).await
    }

    async fn return_connection(&self, target: &str, conn: Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>) {
        self.inner.return_connection(target, conn).await;
    }

    async fn health_check(&self, target: &str) -> bool {
        self.inner.health_check(target).await
    }
}

/// Health check decorator for connection pool
pub struct HealthCheckDecorator {
    inner: Arc<dyn ConnectionPool>,
    health_status: Arc<Mutex<HashMap<String, (bool, Instant)>>>,
    check_interval: Duration,
    unhealthy_threshold: Duration,
}

impl HealthCheckDecorator {
    pub fn new(
        inner: Arc<dyn ConnectionPool>,
        check_interval: Duration,
        unhealthy_threshold: Duration,
    ) -> Self {
        Self {
            inner,
            health_status: Arc::new(Mutex::new(HashMap::new())),
            check_interval,
            unhealthy_threshold,
        }
    }

    pub fn start_health_check_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.check_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                
                // Get all targets that need health check
                let targets: Vec<String> = {
                    let status = self.health_status.lock().await;
                    status.keys().cloned().collect()
                };
                
                for target in targets {
                    let is_healthy = self.inner.health_check(&target).await;
                    let mut status = self.health_status.lock().await;
                    status.insert(target.clone(), (is_healthy, Instant::now()));
                    
                    if !is_healthy {
                        warn!("Health check failed for {}", target);
                    }
                }
            }
        })
    }

    async fn is_healthy(&self, target: &str) -> bool {
        let status = self.health_status.lock().await;
        if let Some((healthy, last_check)) = status.get(target) {
            // If last check was too long ago, consider it unknown (allow connection)
            if last_check.elapsed() < self.unhealthy_threshold {
                return *healthy;
            }
        }
        // Unknown status - allow connection
        true
    }
}

#[async_trait::async_trait]
impl ConnectionPool for HealthCheckDecorator {
    async fn get_connection(&self, target: &str) -> Result<Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>, PoolError> {
        if !self.is_healthy(target).await {
            return Err(PoolError::PoolError(format!("Target {} is unhealthy", target)));
        }
        
        self.inner.get_connection(target).await
    }

    async fn return_connection(&self, target: &str, conn: Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>>) {
        self.inner.return_connection(target, conn).await;
    }

    async fn health_check(&self, target: &str) -> bool {
        self.inner.health_check(target).await
    }
}

