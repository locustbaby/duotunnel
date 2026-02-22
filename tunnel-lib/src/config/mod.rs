use anyhow::Result;

pub mod tcp;
pub mod quic;
pub mod http_pool;
pub mod proxy_buffers;

pub use tcp::TcpConfig;
pub use quic::QuicConfig;
pub use http_pool::HttpPoolConfig;
pub use proxy_buffers::ProxyBufferConfig;

/// Resolve a config file path with a `../` fallback for running from a workspace subdirectory.
pub fn resolve_config_path(path: &str) -> Result<String> {
    if std::path::Path::new(path).exists() {
        return Ok(path.to_string());
    }
    let alt = format!("../{}", path);
    if std::path::Path::new(&alt).exists() {
        return Ok(alt);
    }
    Err(anyhow::anyhow!("Config file not found: {} (checked ./ and ../)", path))
}
