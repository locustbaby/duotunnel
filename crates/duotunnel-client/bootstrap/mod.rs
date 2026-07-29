pub(crate) mod cli;
pub(crate) mod config;

use anyhow::Result;

use self::cli::Args;
use self::config::ClientConfigFile;

pub(crate) struct ClientBootstrap {
    config_path: String,
    config: ClientConfigFile,
}

impl ClientBootstrap {
    pub(crate) fn from_args(args: &Args) -> Result<Self> {
        let config = ClientConfigFile::load(&args.config)?;
        Ok(Self {
            config_path: args.config.clone(),
            config,
        })
    }

    pub(crate) fn config_path(&self) -> &str {
        &self.config_path
    }

    pub(crate) fn config(&self) -> &ClientConfigFile {
        &self.config
    }

    pub(crate) fn log_level(&self) -> &str {
        self.config.log_level.as_deref().unwrap_or("info")
    }
}
