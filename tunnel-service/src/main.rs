use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Result;
use clap::{Parser, Subcommand};
use figment::{providers::{Env, Format, Yaml}, Figment};
use serde::Deserialize;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tunnel_store::sqlite::{open_sqlite_pool, SqliteAuthStore};
use tunnel_store::sqlite_rules::SqliteRuleStore;

mod cli;
mod proto;
mod service;
mod token;
mod watch;

use cli::{CliCommand, run_cli};
use service::ControlService;
use watch::WatchServer;

// ── Config ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default = "default_database_url")]
    database_url: String,
    #[serde(default = "default_watch_addr")]
    watch_addr: String,
    #[serde(default)]
    log_level: Option<String>,
}

fn default_database_url() -> String {
    "sqlite://tunnel.db".to_string()
}

fn default_watch_addr() -> String {
    "0.0.0.0:7788".to_string()
}

impl Config {
    fn load(path: &str) -> Result<Self> {
        let cfg: Config = Figment::new()
            .merge(Yaml::file(path))
            .merge(Env::prefixed("CTLD__").split("__"))
            .extract()?;
        Ok(cfg)
    }
}

// ── CLI args ─────────────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
#[command(name = "tunnel-ctld", about = "Tunnel control daemon")]
struct Args {
    /// Path to the YAML config file.
    #[arg(short, long, default_value = "ctld.yaml")]
    config: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the control daemon (default when no subcommand given).
    Serve,
    /// Admin token/client management commands.
    #[command(subcommand)]
    Client(CliCommand),
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let cfg = Config::load(&args.config)?;

    let level = cfg.log_level.as_deref().unwrap_or("info");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(level))
        .init();

    info!(database_url = %cfg.database_url, watch_addr = %cfg.watch_addr, "starting tunnel-ctld");

    let pool = open_sqlite_pool(&cfg.database_url).await?;

    let auth_store_inner = SqliteAuthStore::from_pool(pool.clone());
    auth_store_inner.migrate().await?;

    let rule_store_inner = SqliteRuleStore::new(pool.clone());
    rule_store_inner.migrate().await?;

    let auth_store: Arc<dyn tunnel_store::AuthStore> = Arc::new(auth_store_inner);
    let rule_store: Arc<dyn tunnel_store::RuleStore> = Arc::new(rule_store_inner);

    let svc = ControlService::new(auth_store, rule_store, pool).await?;

    match args.command.unwrap_or(Command::Serve) {
        Command::Serve => {
            let addr: SocketAddr = cfg.watch_addr.parse()?;
            WatchServer::new(Arc::clone(&svc), addr).run().await?;
        }
        Command::Client(cmd) => {
            run_cli(cmd, &svc).await?;
        }
    }

    Ok(())
}
