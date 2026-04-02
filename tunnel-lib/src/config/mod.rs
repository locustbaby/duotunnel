use anyhow::Result;
pub mod http_pool;
pub mod proxy_buffers;
pub mod quic;
pub mod tcp;
pub use http_pool::HttpPoolConfig;
pub use proxy_buffers::ProxyBufferConfig;
pub use quic::QuicConfig;
pub use tcp::TcpConfig;
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
