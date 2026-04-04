use anyhow::{anyhow, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PkiParams {
    pub cert_cache_ttl_secs: u64,
}
impl Default for PkiParams {
    fn default() -> Self {
        Self { cert_cache_ttl_secs: 3600 }
    }
}

pub fn init_cert_cache(params: &PkiParams) {
    let mut guard = CERT_CACHE.write().unwrap();
    *guard = Some(CertCache::new(Duration::from_secs(params.cert_cache_ttl_secs)));
}

struct CachedEntry {
    /// Pre-built ServerConfig with ALPN [h2, http/1.1] set, ready for TlsAcceptor.
    server_config: Arc<rustls::ServerConfig>,
    created_at: Instant,
}

static CERT_CACHE: RwLock<Option<CertCache>> = RwLock::new(None);

struct CertCache {
    entries: HashMap<String, CachedEntry>,
    ttl: Duration,
}

impl CertCache {
    fn new(ttl: Duration) -> Self {
        Self { entries: HashMap::new(), ttl }
    }

    fn get(&self, host: &str) -> Option<Arc<rustls::ServerConfig>> {
        self.entries.get(host).and_then(|e| {
            if e.created_at.elapsed() < self.ttl {
                Some(Arc::clone(&e.server_config))
            } else {
                None
            }
        })
    }

    fn insert(&mut self, host: String, server_config: Arc<rustls::ServerConfig>) {
        let ttl = self.ttl;
        self.entries.retain(|_, e| e.created_at.elapsed() < ttl);
        self.entries.insert(host, CachedEntry { server_config, created_at: Instant::now() });
    }
}

/// Returns a cached `Arc<rustls::ServerConfig>` for `host`, creating one if needed.
///
/// The config is built with a fresh self-signed certificate and ALPN `[h2, http/1.1]`.
/// Cache TTL defaults to 3600s (configurable via `init_cert_cache`).
///
/// Uses a single write-lock check+insert to avoid the double-checked-locking race
/// where concurrent first-time requests for the same host each generate a certificate.
pub fn get_or_create_server_config(host: &str) -> Result<Arc<rustls::ServerConfig>> {
    // Fast path: read lock.
    {
        let guard = CERT_CACHE.read().unwrap();
        if let Some(cache) = guard.as_ref() {
            if let Some(cfg) = cache.get(host) {
                return Ok(cfg);
            }
        }
    }
    // Slow path: write lock — check again, then generate.
    let mut guard = CERT_CACHE.write().unwrap();
    let cache = guard.get_or_insert_with(|| {
        CertCache::new(Duration::from_secs(PkiParams::default().cert_cache_ttl_secs))
    });
    // Second check inside write lock eliminates the race.
    if let Some(cfg) = cache.get(host) {
        return Ok(cfg);
    }
    let cfg = build_server_config(host)?;
    let arc = Arc::new(cfg);
    cache.insert(host.to_string(), Arc::clone(&arc));
    Ok(arc)
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

/// Legacy: generate or retrieve raw cert+key (still used by client/app.rs).
pub fn generate_self_signed_cert_for_host(
    host: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    // Reuse the cached ServerConfig's internal cert by rebuilding from scratch.
    // This path is only used by client/app.rs which needs raw DER bytes.
    let cert = rcgen::generate_simple_self_signed(vec![host.to_string()])?;
    let key_der = cert.signing_key.serialize_der();
    let cert_der = cert.cert.der().to_vec();
    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;
    Ok((vec![CertificateDer::from(cert_der)], key))
}

pub fn generate_self_signed_cert() -> Result<
    (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>),
> {
    generate_self_signed_cert_for_host("localhost")
}
