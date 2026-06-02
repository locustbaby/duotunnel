use anyhow::{anyhow, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PkiParams {
    pub cert_cache_ttl_secs: u64,
    pub max_parallel_generations: usize,
}

impl Default for PkiParams {
    fn default() -> Self {
        Self {
            cert_cache_ttl_secs: 3600,
            max_parallel_generations: 4,
        }
    }
}

pub fn init_cert_cache(params: &PkiParams) {
    let mut guard = CERT_STATE.write().unwrap();
    *guard = Some(CertState::new(
        Duration::from_secs(params.cert_cache_ttl_secs),
        params.max_parallel_generations,
    ));
}

struct CachedEntry {
    server_config: Arc<rustls::ServerConfig>,
    created_at: Instant,
}

struct InflightEntry {
    lock: Arc<Mutex<()>>,
}

struct CertState {
    entries: HashMap<String, CachedEntry>,
    inflight: HashMap<String, InflightEntry>,
    ttl: Duration,
    permits: Arc<Semaphore>,
}

impl CertState {
    fn new(ttl: Duration, max_parallel_generations: usize) -> Self {
        Self {
            entries: HashMap::new(),
            inflight: HashMap::new(),
            ttl,
            permits: Arc::new(Semaphore::new(max_parallel_generations.max(1))),
        }
    }

    fn get(&self, host: &str) -> Option<Arc<rustls::ServerConfig>> {
        self.entries.get(host).and_then(|entry| {
            if entry.created_at.elapsed() < self.ttl {
                Some(Arc::clone(&entry.server_config))
            } else {
                None
            }
        })
    }

    fn insert(&mut self, host: String, server_config: Arc<rustls::ServerConfig>) {
        let ttl = self.ttl;
        self.entries
            .retain(|_, entry| entry.created_at.elapsed() < ttl);
        self.entries.insert(
            host,
            CachedEntry {
                server_config,
                created_at: Instant::now(),
            },
        );
    }
}

static CERT_STATE: RwLock<Option<CertState>> = RwLock::new(None);
static PKI_RUNTIME: OnceLock<tokio::runtime::Handle> = OnceLock::new();

fn ensure_state() {
    let mut guard = CERT_STATE.write().unwrap();
    if guard.is_none() {
        let params = PkiParams::default();
        *guard = Some(CertState::new(
            Duration::from_secs(params.cert_cache_ttl_secs),
            params.max_parallel_generations,
        ));
    }
}

pub async fn get_or_create_server_config(host: &str) -> Result<Arc<rustls::ServerConfig>> {
    let _ = PKI_RUNTIME.set(tokio::runtime::Handle::current());
    ensure_state();
    {
        let guard = CERT_STATE.read().unwrap();
        if let Some(state) = guard.as_ref() {
            if let Some(config) = state.get(host) {
                return Ok(config);
            }
        }
    }

    let (entry_lock, permits) = {
        let mut guard = CERT_STATE.write().unwrap();
        let state = guard.as_mut().unwrap();
        if let Some(config) = state.get(host) {
            return Ok(config);
        }
        let entry = state
            .inflight
            .entry(host.to_string())
            .or_insert_with(|| InflightEntry {
                lock: Arc::new(Mutex::new(())),
            });
        (entry.lock.clone(), state.permits.clone())
    };

    let _entry_guard = entry_lock.lock().await;
    {
        let guard = CERT_STATE.read().unwrap();
        if let Some(state) = guard.as_ref() {
            if let Some(config) = state.get(host) {
                return Ok(config);
            }
        }
    }

    let permit = permits
        .acquire_owned()
        .await
        .map_err(|_| anyhow!("pki generation limiter closed"))?;
    let host_owned = host.to_string();
    let config =
        tokio::task::spawn_blocking(move || build_server_config_with_permit(host_owned, permit))
            .await
            .map_err(|e| anyhow!("pki generation join error: {e}"))??;
    let config = Arc::new(config);

    let mut guard = CERT_STATE.write().unwrap();
    let state = guard.as_mut().unwrap();
    state.insert(host.to_string(), Arc::clone(&config));
    state.inflight.remove(host);
    Ok(config)
}

fn build_server_config_with_permit(
    host: String,
    _permit: OwnedSemaphorePermit,
) -> Result<rustls::ServerConfig> {
    build_server_config(&host)
}

fn build_server_config(host: &str) -> Result<rustls::ServerConfig> {
    let cert = rcgen::generate_simple_self_signed(vec![host.to_string()])?;
    let key_der = cert.signing_key.serialize_der();
    let cert_der = cert.cert.der().to_vec();
    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;
    let certs = vec![CertificateDer::from(cert_der)];
    let mut cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(cfg)
}

pub fn generate_self_signed_cert_for_host(
    host: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert = rcgen::generate_simple_self_signed(vec![host.to_string()])?;
    let key_der = cert.signing_key.serialize_der();
    let cert_der = cert.cert.der().to_vec();
    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;
    Ok((vec![CertificateDer::from(cert_der)], key))
}

pub fn generate_self_signed_cert() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>
{
    generate_self_signed_cert_for_host("localhost")
}
