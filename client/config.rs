use serde::Deserialize;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub client_group_id: String,
    pub auth_token: Option<String>,
    pub http_entry_port: Option<u16>,
    pub grpc_entry_port: Option<u16>,
    pub upstreams: Option<HashMap<String, Upstream>>,
    pub log_level: String,
    pub trace_enabled: Option<bool>,
    #[serde(default)]
    pub performance: ClientPerformanceConfig,
}

/// 客户端性能配置
#[derive(Debug, Deserialize, Clone)]
pub struct ClientPerformanceConfig {
    /// Channel 容量
    #[serde(default = "default_channel_capacity")]
    pub channel_capacity: usize,
    
    /// 请求超时时间 (秒)
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
    
    /// 心跳间隔 (秒)
    #[serde(default = "default_heartbeat_interval_secs")]
    pub heartbeat_interval_secs: u64,
    
    /// 配置同步间隔 (秒)
    #[serde(default = "default_config_sync_interval_secs")]
    pub config_sync_interval_secs: u64,
    
    /// 重连间隔 (秒)
    #[serde(default = "default_reconnect_interval_secs")]
    pub reconnect_interval_secs: u64,
}

// 默认值函数
fn default_channel_capacity() -> usize { 10000 }
fn default_request_timeout_secs() -> u64 { 30 }
fn default_heartbeat_interval_secs() -> u64 { 15 }
fn default_config_sync_interval_secs() -> u64 { 30 }
fn default_reconnect_interval_secs() -> u64 { 2 }

impl Default for ClientPerformanceConfig {
    fn default() -> Self {
        Self {
            channel_capacity: default_channel_capacity(),
            request_timeout_secs: default_request_timeout_secs(),
            heartbeat_interval_secs: default_heartbeat_interval_secs(),
            config_sync_interval_secs: default_config_sync_interval_secs(),
            reconnect_interval_secs: default_reconnect_interval_secs(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Upstream {
    pub servers: Vec<ServerAddr>,
    pub lb_policy: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ServerAddr {
    pub address: String,
    pub resolve: bool,
}

impl ClientConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ClientConfig = toml::from_str(&content)?;
        Ok(config)
    }
} 