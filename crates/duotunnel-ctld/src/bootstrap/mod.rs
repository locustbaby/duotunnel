pub(crate) mod cli;
pub(crate) mod config;

use anyhow::Result;

pub(crate) struct CtldBootstrap {
    args: cli::Args,
    config: config::Config,
}

impl CtldBootstrap {
    pub(crate) fn from_args(args: cli::Args) -> Result<Self> {
        let config = config::Config::load(&args.config)?;
        Ok(Self { args, config })
    }

    pub(crate) fn command(&self) -> Option<&cli::Command> {
        self.args.command.as_ref()
    }

    pub(crate) fn config(&self) -> &config::Config {
        &self.config
    }
}
