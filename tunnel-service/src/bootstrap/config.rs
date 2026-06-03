use anyhow::Result;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default = "default_database_url")]
    pub(crate) database_url: String,
    #[serde(default = "default_watch_addr")]
    pub(crate) watch_addr: String,
    #[serde(default)]
    pub(crate) watch_token: Option<String>,
    #[serde(default)]
    pub(crate) log_level: Option<String>,
    #[serde(default)]
    pub(crate) server_config: Option<String>,
    #[serde(default)]
    pub(crate) metrics_port: Option<u16>,
}

fn default_database_url() -> String {
    "sqlite://tunnel.db".to_string()
}

fn default_watch_addr() -> String {
    "127.0.0.1:7788".to_string()
}

impl Config {
    pub(crate) fn load(path: &str) -> Result<Self> {
        Ok(Figment::new()
            .merge(Yaml::file(path))
            .merge(Env::prefixed("CTLD__").split("__"))
            .extract()?)
    }

    pub(crate) fn log_level(&self) -> &str {
        self.log_level.as_deref().unwrap_or("info")
    }
}
