use anyhow::Result;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use rustls::pki_types::ServerName;
use tracing::debug;

const MAX_IDLE_PER_HOST: usize = 10;
const IDLE_TIMEOUT: Duration = Duration::from_secs(90);

struct PooledTcpConnection {
    stream: Option<TcpStream>,
    last_used: Instant,
}

impl PooledTcpConnection {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: Some(stream),
            last_used: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.last_used.elapsed() > IDLE_TIMEOUT
    }

    fn take(mut self) -> Option<TcpStream> {
        self.stream.take()
    }
}

struct PooledTlsConnection {
    stream: Option<TlsStream<TcpStream>>,
    last_used: Instant,
}

impl PooledTlsConnection {
    fn new(stream: TlsStream<TcpStream>) -> Self {
        Self {
            stream: Some(stream),
            last_used: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.last_used.elapsed() > IDLE_TIMEOUT
    }

    fn take(mut self) -> Option<TlsStream<TcpStream>> {
        self.stream.take()
    }
}

pub struct ConnectionPool {
    http_pools: DashMap<String, VecDeque<PooledTcpConnection>>,
    https_pools: DashMap<String, VecDeque<PooledTlsConnection>>,
    tls_connector: Arc<TlsConnector>,
}

impl ConnectionPool {
    pub fn new(tls_connector: Arc<TlsConnector>) -> Self {
        Self {
            http_pools: DashMap::new(),
            https_pools: DashMap::new(),
            tls_connector,
        }
    }

    pub async fn get_or_create_http(
        &self,
        addr: &str,
    ) -> Result<TcpStream> {
        let key = addr.to_string();

        if let Some(mut entry) = self.http_pools.get_mut(&key) {
            while let Some(conn) = entry.pop_front() {
                if !conn.is_expired() {
                    if let Some(stream) = conn.take() {
                        debug!(addr = %addr, "reusing pooled HTTP connection");
                        return Ok(stream);
                    }
                }
            }
        }

        debug!(addr = %addr, "creating new HTTP connection");
        let stream = TcpStream::connect(addr).await?;
        Ok(stream)
    }

    pub async fn get_or_create_https(
        &self,
        addr: &str,
        host: &str,
    ) -> Result<TlsStream<TcpStream>> {
        let key = format!("{}:{}", host, addr);

        if let Some(mut entry) = self.https_pools.get_mut(&key) {
            while let Some(conn) = entry.pop_front() {
                if !conn.is_expired() {
                    if let Some(stream) = conn.take() {
                        debug!(host = %host, addr = %addr, "reusing pooled HTTPS connection");
                        return Ok(stream);
                    }
                }
            }
        }

        debug!(host = %host, addr = %addr, "creating new HTTPS connection");
        let tcp_stream = TcpStream::connect(addr).await?;
        
        let server_name = ServerName::try_from(host.to_string())
            .map_err(|_| anyhow::anyhow!("invalid server name: {}", host))?;

        let tls_stream = self.tls_connector.connect(server_name, tcp_stream).await?;
        Ok(tls_stream)
    }

    pub fn return_http(&self, addr: &str, stream: TcpStream) {
        let key = addr.to_string();
        
        let mut entry = self.http_pools.entry(key.clone()).or_insert_with(VecDeque::new);
        
        if entry.len() < MAX_IDLE_PER_HOST {
            entry.push_back(PooledTcpConnection::new(stream));
            debug!(addr = %addr, pool_size = entry.len(), "HTTP connection returned to pool");
        } else {
            debug!(addr = %addr, "HTTP pool full, dropping connection");
        }
    }

    pub fn return_https(&self, addr: &str, host: &str, stream: TlsStream<TcpStream>) {
        let key = format!("{}:{}", host, addr);
        
        let mut entry = self.https_pools.entry(key.clone()).or_insert_with(VecDeque::new);
        
        if entry.len() < MAX_IDLE_PER_HOST {
            entry.push_back(PooledTlsConnection::new(stream));
            debug!(host = %host, addr = %addr, pool_size = entry.len(), "HTTPS connection returned to pool");
        } else {
            debug!(host = %host, addr = %addr, "HTTPS pool full, dropping connection");
        }
    }

    pub fn cleanup_expired(&self) {
        let mut http_cleaned = 0;
        let mut https_cleaned = 0;

        for mut entry in self.http_pools.iter_mut() {
            let before = entry.len();
            entry.retain(|conn| !conn.is_expired());
            http_cleaned += before - entry.len();
        }

        for mut entry in self.https_pools.iter_mut() {
            let before = entry.len();
            entry.retain(|conn| !conn.is_expired());
            https_cleaned += before - entry.len();
        }

        if http_cleaned > 0 || https_cleaned > 0 {
            debug!(
                http_cleaned = http_cleaned,
                https_cleaned = https_cleaned,
                "cleaned up expired connections"
            );
        }
    }

    pub async fn start_cleanup_task(pool: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                pool.cleanup_expired();
            }
        });
    }
}
