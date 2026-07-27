use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, DistinguishedName, DnType, IsCa, Issuer,
    KeyPair, KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, OnceLock,
};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

const MAX_CERT_CACHE_ENTRIES: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PkiParams {
    pub cert_cache_ttl_secs: u64,
    pub max_parallel_generations: usize,
    #[serde(default = "default_ca_cert_path")]
    pub ca_cert_path: Option<String>,
    #[serde(default = "default_ca_key_path")]
    pub ca_key_path: Option<String>,
}

fn default_ca_cert_path() -> Option<String> {
    Some("ca.crt".to_string())
}

fn default_ca_key_path() -> Option<String> {
    Some("ca.key".to_string())
}

impl Default for PkiParams {
    fn default() -> Self {
        Self {
            cert_cache_ttl_secs: 3600,
            max_parallel_generations: 4,
            ca_cert_path: Some("ca.crt".to_string()),
            ca_key_path: Some("ca.key".to_string()),
        }
    }
}

pub fn init_cert_cache(params: &PkiParams) {
    let mut guard = CERT_STATE.write();
    *guard = Some(CertState::new(
        Duration::from_secs(params.cert_cache_ttl_secs),
        params.max_parallel_generations,
        params.ca_cert_path.as_deref(),
        params.ca_key_path.as_deref(),
    ));
}

struct CachedEntry {
    server_config: Arc<rustls::ServerConfig>,
    created_at: Instant,
}

struct InflightEntry {
    lock: Arc<Mutex<()>>,
    users: Arc<AtomicUsize>,
}

struct CertState {
    entries: HashMap<String, CachedEntry>,
    inflight: HashMap<String, InflightEntry>,
    ttl: Duration,
    permits: Arc<Semaphore>,
    root_ca: Arc<RootCa>,
}

impl CertState {
    fn new(
        ttl: Duration,
        max_parallel_generations: usize,
        ca_cert_path: Option<&str>,
        ca_key_path: Option<&str>,
    ) -> Self {
        Self {
            entries: HashMap::new(),
            inflight: HashMap::new(),
            ttl,
            permits: Arc::new(Semaphore::new(max_parallel_generations.max(1))),
            root_ca: Arc::new(
                RootCa::load_or_generate(ca_cert_path, ca_key_path)
                    .expect("failed to initialize root CA"),
            ),
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
        if self.entries.len() >= MAX_CERT_CACHE_ENTRIES && !self.entries.contains_key(&host) {
            if let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.created_at)
                .map(|(host, _)| host.clone())
            {
                self.entries.remove(&oldest);
            }
        }
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

struct InflightCleanup {
    host: String,
    lock: Arc<Mutex<()>>,
    users: Arc<AtomicUsize>,
}

impl Drop for InflightCleanup {
    fn drop(&mut self) {
        if self.users.fetch_sub(1, Ordering::AcqRel) != 1 {
            return;
        }
        let mut guard = CERT_STATE.write();
        if let Some(state) = guard.as_mut() {
            let should_remove = state.inflight.get(&self.host).is_some_and(|entry| {
                Arc::ptr_eq(&entry.lock, &self.lock)
                    && Arc::ptr_eq(&entry.users, &self.users)
                    && self.users.load(Ordering::Acquire) == 0
            });
            if should_remove {
                state.inflight.remove(&self.host);
            }
        }
    }
}

enum RootCaIssuer {
    Generated(CertifiedIssuer<'static, KeyPair>),
    Loaded(Issuer<'static, KeyPair>),
}

impl Deref for RootCaIssuer {
    type Target = Issuer<'static, KeyPair>;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Generated(i) => i,
            Self::Loaded(i) => i,
        }
    }
}

struct RootCa {
    issuer: RootCaIssuer,
    cert_der: CertificateDer<'static>,
}

impl RootCa {
    fn generate() -> Result<Self> {
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "DuoTunnel Local Root CA");
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::CrlSign,
        ];
        let key = KeyPair::generate()?;
        let issuer = CertifiedIssuer::self_signed(params, key)?;
        let cert_der = issuer.der().clone();
        Ok(Self {
            issuer: RootCaIssuer::Generated(issuer),
            cert_der,
        })
    }

    fn load_or_generate(cert_path: Option<&str>, key_path: Option<&str>) -> Result<Self> {
        if let (Some(c_path), Some(k_path)) = (cert_path, key_path) {
            let c_path = std::path::Path::new(c_path);
            let k_path = std::path::Path::new(k_path);
            if c_path.exists() && k_path.exists() {
                harden_existing_private_key(k_path);
                if let (Ok(cert_pem), Ok(key_pem)) =
                    (read_existing_file(c_path), read_existing_file(k_path))
                {
                    match KeyPair::from_pem(&key_pem) {
                        Ok(key_pair) => match Issuer::from_ca_cert_pem(&cert_pem, key_pair) {
                            Ok(issuer) => match pem_to_der(&cert_pem) {
                                Ok(der_bytes) => {
                                    tracing::info!(cert_path = %c_path.display(), key_path = %k_path.display(), "loaded persistent Root CA");
                                    return Ok(Self {
                                        issuer: RootCaIssuer::Loaded(issuer),
                                        cert_der: CertificateDer::from(der_bytes).into_owned(),
                                    });
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "failed to decode loaded CA PEM to DER; regenerating CA");
                                }
                            },
                            Err(e) => {
                                tracing::warn!(error = %e, "failed to parse CA certificate from PEM; regenerating CA");
                            }
                        },
                        Err(e) => {
                            tracing::warn!(error = %e, "failed to parse CA private key; regenerating CA");
                        }
                    }
                }
            }
        }

        // Generate new CA
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "DuoTunnel Local Root CA");
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::CrlSign,
        ];
        let key = KeyPair::generate()?;

        // Save new CA to disk if paths are provided
        if let (Some(c_path), Some(k_path)) = (cert_path, key_path) {
            let c_path = std::path::Path::new(c_path);
            let k_path = std::path::Path::new(k_path);
            if let Some(parent) = c_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Some(parent) = k_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let key_pem = key.serialize_pem();
            let issuer = CertifiedIssuer::self_signed(params, key)?;
            let cert_pem = issuer.pem();

            if let Err(e) = write_certificate(c_path, cert_pem.as_bytes()) {
                tracing::error!(error = %e, path = %c_path.display(), "failed to save CA cert to disk");
            }
            if let Err(e) = write_private_key(k_path, key_pem.as_bytes()) {
                tracing::error!(error = %e, path = %k_path.display(), "failed to save CA private key to disk");
            }

            let cert_der = issuer.der().clone();
            Ok(Self {
                issuer: RootCaIssuer::Generated(issuer),
                cert_der,
            })
        } else {
            let issuer = CertifiedIssuer::self_signed(params, key)?;
            let cert_der = issuer.der().clone();
            Ok(Self {
                issuer: RootCaIssuer::Generated(issuer),
                cert_der,
            })
        }
    }
}

/// Write key material owner-read/write only.
///
/// The default key path is relative, so it resolves against the process CWD,
/// which may be writable by other local users. A pre-planted `ca.key` must not
/// be able to capture the root CA key: `O_NOFOLLOW` refuses a symlink,
/// `O_NONBLOCK` keeps a FIFO from blocking the open until the attacker attaches
/// as reader, the file-type and uid checks (done through the returned fd, so
/// they cannot be raced) reject anything but a regular file we own, and the
/// fchmod covers the case where the file already existed, since `mode` only
/// applies to files this call creates. Truncation happens only after those
/// checks pass, so a rejected path is left untouched.
#[cfg(unix)]
fn read_existing_file(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::Read;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let meta = file.metadata()?;
    if !meta.file_type().is_file() || meta.uid() != unsafe { libc::getuid() } {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "refusing to read CA material from unsafe path {}",
                path.display()
            ),
        ));
    }
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(not(unix))]
fn read_existing_file(path: &std::path::Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

#[cfg(unix)]
fn write_certificate(path: &std::path::Path, pem: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o644)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let meta = file.metadata()?;
    if !meta.file_type().is_file() || meta.uid() != unsafe { libc::getuid() } {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "refusing to write CA certificate to unsafe path {}",
                path.display()
            ),
        ));
    }
    file.set_permissions(std::fs::Permissions::from_mode(0o644))?;
    file.set_len(0)?;
    file.write_all(pem)
}

#[cfg(not(unix))]
fn write_certificate(path: &std::path::Path, pem: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(pem)
}

#[cfg(unix)]
fn write_private_key(path: &std::path::Path, pem: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;

    let meta = file.metadata()?;
    if !meta.file_type().is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "refusing to write CA private key to non-regular file {}",
                path.display()
            ),
        ));
    }
    let uid = unsafe { libc::getuid() };
    if meta.uid() != uid {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "refusing to write CA private key to {}: owned by uid {}, expected {}",
                path.display(),
                meta.uid(),
                uid
            ),
        ));
    }

    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    file.set_len(0)?;
    file.write_all(pem)
}

#[cfg(not(unix))]
fn write_private_key(path: &std::path::Path, pem: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(pem)
}

/// Tighten an already-persisted CA private key to 0600.
///
/// Deployments created before `write_private_key` enforced the mode still carry
/// world-readable keys, and nothing else in the load path narrows them.
#[cfg(unix)]
fn harden_existing_private_key(path: &std::path::Path) {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let file = match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
    {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(error = %e, path = %path.display(), "could not open CA private key to check permissions");
            return;
        }
    };
    let mode = match file.metadata() {
        Ok(meta) => meta.permissions().mode() & 0o777,
        Err(e) => {
            tracing::warn!(error = %e, path = %path.display(), "could not stat CA private key to check permissions");
            return;
        }
    };
    if mode & 0o177 == 0 {
        return;
    }
    match file.set_permissions(std::fs::Permissions::from_mode(0o600)) {
        Ok(()) => {
            tracing::warn!(
                path = %path.display(),
                found_mode = format!("{mode:04o}"),
                "CA private key had permissions wider than 0600; tightened to 0600"
            );
        }
        Err(e) => {
            tracing::warn!(error = %e, path = %path.display(), found_mode = format!("{mode:04o}"), "CA private key has permissions wider than 0600 and could not be tightened");
        }
    }
}

#[cfg(not(unix))]
fn harden_existing_private_key(_path: &std::path::Path) {}

fn pem_to_der(pem: &str) -> Result<Vec<u8>> {
    use base64::prelude::*;
    let mut base64_str = String::new();
    let mut in_cert = false;
    for line in pem.lines() {
        let line = line.trim();
        if line == "-----BEGIN CERTIFICATE-----" {
            in_cert = true;
        } else if line == "-----END CERTIFICATE-----" {
            break;
        } else if in_cert {
            base64_str.push_str(line);
        }
    }
    if base64_str.is_empty() {
        return Err(anyhow!("invalid PEM certificate"));
    }
    let bytes = BASE64_STANDARD
        .decode(base64_str.as_bytes())
        .map_err(|e| anyhow!("failed to decode base64: {e}"))?;
    Ok(bytes)
}

fn ensure_state() {
    let mut guard = CERT_STATE.write();
    if guard.is_none() {
        let params = PkiParams::default();
        *guard = Some(CertState::new(
            Duration::from_secs(params.cert_cache_ttl_secs),
            params.max_parallel_generations,
            params.ca_cert_path.as_deref(),
            params.ca_key_path.as_deref(),
        ));
    }
}

pub async fn get_or_create_server_config(host: &str) -> Result<Arc<rustls::ServerConfig>> {
    let _ = PKI_RUNTIME.set(tokio::runtime::Handle::current());
    ensure_state();
    {
        let guard = CERT_STATE.read();
        if let Some(state) = guard.as_ref() {
            if let Some(config) = state.get(host) {
                return Ok(config);
            }
        }
    }

    let (entry_lock, entry_users, permits, root_ca) = {
        let mut guard = CERT_STATE.write();
        let state = guard.as_mut().unwrap();
        if let Some(config) = state.get(host) {
            return Ok(config);
        }
        let entry = state
            .inflight
            .entry(host.to_string())
            .or_insert_with(|| InflightEntry {
                lock: Arc::new(Mutex::new(())),
                users: Arc::new(AtomicUsize::new(0)),
            });
        entry.users.fetch_add(1, Ordering::AcqRel);
        (
            entry.lock.clone(),
            entry.users.clone(),
            state.permits.clone(),
            Arc::clone(&state.root_ca),
        )
    };

    let _inflight_cleanup = InflightCleanup {
        host: host.to_string(),
        lock: entry_lock.clone(),
        users: entry_users.clone(),
    };
    let _entry_guard = entry_lock.clone().lock_owned().await;
    drop(entry_lock);
    {
        let guard = CERT_STATE.read();
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
    let config = tokio::task::spawn_blocking(move || {
        build_server_config_with_permit(host_owned, root_ca, permit)
    })
    .await
    .map_err(|e| anyhow!("pki generation join error: {e}"))??;
    let config = Arc::new(config);

    let mut guard = CERT_STATE.write();
    let state = guard.as_mut().unwrap();
    state.insert(host.to_string(), Arc::clone(&config));
    Ok(config)
}

fn build_server_config_with_permit(
    host: String,
    root_ca: Arc<RootCa>,
    _permit: OwnedSemaphorePermit,
) -> Result<rustls::ServerConfig> {
    build_server_config(&host, root_ca.as_ref())
}

fn build_server_config(host: &str, root_ca: &RootCa) -> Result<rustls::ServerConfig> {
    let (certs, key) = generate_signed_cert_for_host(host, root_ca)?;
    let mut cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(cfg)
}

pub fn generate_self_signed_cert_for_host(
    host: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let root_ca = RootCa::generate()?;
    generate_signed_cert_for_host(host, &root_ca)
}

fn generate_signed_cert_for_host(
    host: &str,
    root_ca: &RootCa,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let params = CertificateParams::new(vec![host.to_string()])?;
    let leaf_key = KeyPair::generate()?;
    let leaf_cert = params.signed_by(&leaf_key, &root_ca.issuer)?;
    let key_der = leaf_key.serialize_der();
    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;
    Ok((vec![leaf_cert.der().clone(), root_ca.cert_der.clone()], key))
}

pub fn generate_self_signed_cert() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>
{
    generate_self_signed_cert_for_host("localhost")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_host_cert_includes_ca_chain() {
        let (certs, _key) = generate_self_signed_cert_for_host("example.com").unwrap();
        assert_eq!(certs.len(), 2);
    }
}
