use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
    #[arg(short, long, default_value = "config/server.yaml", global = true)]
    pub(crate) config: String,
    #[arg(long, global = true, default_value = "127.0.0.1:7788")]
    pub(crate) ctld_addr: String,
    #[arg(long, global = true)]
    pub(crate) ctld_token: Option<String>,
}

impl Cli {
    pub(crate) fn resolved_ctld_token(&self) -> Option<String> {
        self.ctld_token
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| {
                std::env::var("DUOTUNNEL_CTLD_TOKEN")
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            })
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run,
}
