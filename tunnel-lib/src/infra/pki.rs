use anyhow::{Result, anyhow};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Cached certificate with expiry tracking
struct CachedCert {
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
    created_at: Instant,
}

/// Global certificate cache with TTL
static CERT_CACHE: RwLock<Option<CertCache>> = RwLock::new(None);

struct CertCache {
    certs: HashMap<String, CachedCert>,
    ttl: Duration,
}

impl CertCache {
    fn new(ttl: Duration) -> Self {
        Self {
            certs: HashMap::new(),
            ttl,
        }
    }

    fn get(&self, host: &str) -> Option<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        self.certs.get(host).and_then(|cached| {
            if cached.created_at.elapsed() < self.ttl {
                Some((cached.certs.clone(), cached.key.clone_key()))
            } else {
                None
            }
        })
    }

    fn insert(&mut self, host: String, certs: Vec<CertificateDer<'static>>, key: PrivateKeyDer<'static>) {
        self.certs.insert(host, CachedCert {
            certs,
            key,
            created_at: Instant::now(),
        });
    }
}

/// Generate a self-signed certificate for the given hostname with caching.
/// Certificates are cached for 1 hour by default to avoid expensive crypto operations.
pub fn generate_self_signed_cert_for_host(host: &str) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    // Try to get from cache first
    {
        let cache_guard = CERT_CACHE.read().unwrap();
        if let Some(cache) = cache_guard.as_ref() {
            if let Some(result) = cache.get(host) {
                return Ok(result);
            }
        }
    }

    // Generate new certificate
    let cert = rcgen::generate_simple_self_signed(vec![host.to_string()])?;
    let key_der = cert.signing_key.serialize_der();
    let cert_der = cert.cert.der().to_vec();

    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;
    let cert = CertificateDer::from(cert_der);
    let certs = vec![cert];

    // Store in cache
    {
        let mut cache_guard = CERT_CACHE.write().unwrap();
        let cache = cache_guard.get_or_insert_with(|| CertCache::new(Duration::from_secs(3600)));
        cache.insert(host.to_string(), certs.clone(), key.clone_key());
    }

    Ok((certs, key))
}

/// Generate a self-signed certificate for localhost (backward compatible).
pub fn generate_self_signed_cert() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    generate_self_signed_cert_for_host("localhost")
}
