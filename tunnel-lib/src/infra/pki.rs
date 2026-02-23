use anyhow::{anyhow, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PkiParams {
    pub cert_cache_ttl_secs: u64,
}

impl Default for PkiParams {
    fn default() -> Self {
        Self {
            cert_cache_ttl_secs: 3600,
        }
    }
}

pub fn init_cert_cache(params: &PkiParams) {
    let mut guard = CERT_CACHE.write().unwrap();
    *guard = Some(CertCache::new(Duration::from_secs(
        params.cert_cache_ttl_secs,
    )));
}

struct CachedCert {
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
    created_at: Instant,
}

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

    fn insert(
        &mut self,
        host: String,
        certs: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
    ) {
        let ttl = self.ttl;
        self.certs.retain(|_, c| c.created_at.elapsed() < ttl);
        self.certs.insert(
            host,
            CachedCert {
                certs,
                key,
                created_at: Instant::now(),
            },
        );
    }
}

pub fn generate_self_signed_cert_for_host(
    host: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    {
        let cache_guard = CERT_CACHE.read().unwrap();
        if let Some(cache) = cache_guard.as_ref() {
            if let Some(result) = cache.get(host) {
                return Ok(result);
            }
        }
    }

    let cert = rcgen::generate_simple_self_signed(vec![host.to_string()])?;
    let key_der = cert.signing_key.serialize_der();
    let cert_der = cert.cert.der().to_vec();

    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;
    let cert = CertificateDer::from(cert_der);
    let certs = vec![cert];

    {
        let mut cache_guard = CERT_CACHE.write().unwrap();
        let cache = cache_guard.get_or_insert_with(|| {
            CertCache::new(Duration::from_secs(
                PkiParams::default().cert_cache_ttl_secs,
            ))
        });
        cache.insert(host.to_string(), certs.clone(), key.clone_key());
    }

    Ok((certs, key))
}

pub fn generate_self_signed_cert() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>
{
    generate_self_signed_cert_for_host("localhost")
}
