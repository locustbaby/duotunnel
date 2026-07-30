use anyhow::Result;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;
use std::collections::HashSet;

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
    pub(crate) config: ConfigSources,
    #[serde(default = "default_admin_socket")]
    pub(crate) admin_socket: String,
    #[serde(default)]
    pub(crate) metrics_port: Option<u16>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ConfigSources {
    #[serde(default)]
    pub(crate) sources: Vec<ConfigSourceSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConfigSourceSpec {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) path: Option<String>,
    #[serde(default)]
    pub(crate) database_url: Option<String>,
    #[serde(default)]
    pub(crate) priority: i32,
}

fn default_database_url() -> String {
    "sqlite://tunnel.db".to_string()
}

fn default_watch_addr() -> String {
    "127.0.0.1:7788".to_string()
}

fn default_admin_socket() -> String {
    "./data/duotunnel-ctld.admin.sock".to_string()
}

impl Config {
    pub(crate) fn load(path: &str) -> Result<Self> {
        let config: Self = Figment::new()
            .merge(Yaml::file(path))
            .merge(Env::prefixed("DUOTUNNEL_CTLD__").split("__"))
            .extract()?;
        config.validate_sources()?;
        Ok(config)
    }

    fn validate_sources(&self) -> Result<()> {
        let mut kinds = HashSet::new();
        let mut priorities = HashSet::new();
        for source in &self.config.sources {
            let kind = source.kind.trim().to_ascii_lowercase();
            if !matches!(kind.as_str(), "yaml" | "sqlite") {
                anyhow::bail!("unsupported config source type: {}", source.kind);
            }
            if !kinds.insert(kind.clone()) {
                anyhow::bail!("duplicate config source type: {}", source.kind);
            }
            if !priorities.insert(source.priority) {
                anyhow::bail!(
                    "duplicate config source priority {} creates an ambiguous merge order",
                    source.priority
                );
            }
            match kind.as_str() {
                "yaml" if source.path.as_deref().is_none_or(str::is_empty) => {
                    anyhow::bail!("YAML config source requires path")
                }
                "sqlite" if source.database_url.as_deref().is_some_and(str::is_empty) => {
                    anyhow::bail!("SQLite config source database_url must not be empty")
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn log_level(&self) -> &str {
        self.log_level.as_deref().unwrap_or("info")
    }
}
