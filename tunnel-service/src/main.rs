use anyhow::Result;
use clap::{Parser, Subcommand};
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tunnel_store::sqlite::{open_sqlite_pool, SqliteAuthStore};
use tunnel_store::sqlite_rules::SqliteRuleStore;

mod cli;
mod proto;
mod service;
mod token;
mod watch;

use cli::{run_cli, CliCommand};
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
    watch_token: Option<String>,
    #[serde(default)]
    log_level: Option<String>,
    /// Path to a server.yaml whose routing sections (`tunnel_management` +
    /// `server_egress_upstream`) are seeded into the DB on first boot (when
    /// the routing table is empty).  No-op on subsequent boots.
    #[serde(default)]
    server_config: Option<String>,
    /// Port for the HTTP metrics + healthz endpoint.
    #[serde(default)]
    metrics_port: Option<u16>,
}

fn default_database_url() -> String {
    "sqlite://tunnel.db".to_string()
}

fn default_watch_addr() -> String {
    "127.0.0.1:7788".to_string()
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

// ── Healthz HTTP server ───────────────────────────────────────────────────────

async fn run_healthz_server(port: u16, ready: Arc<AtomicBool>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(addr = %addr, error = %e, "failed to bind healthz server");
            return;
        }
    };
    info!(addr = %addr, "healthz server started");
    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let ready = ready.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 256];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
            let (status, body) = if req.starts_with("GET /healthz") {
                if ready.load(Ordering::Acquire) {
                    ("200 OK", "ok\n")
                } else {
                    ("503 Service Unavailable", "not ready\n")
                }
            } else {
                ("200 OK", "ok\n")
            };
            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
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

    info!(
        database_url = %cfg.database_url,
        watch_addr = %cfg.watch_addr,
        watch_auth = cfg.watch_token.as_ref().is_some_and(|t| !t.trim().is_empty()),
        "starting tunnel-ctld"
    );

    let ready = Arc::new(AtomicBool::new(false));

    if let Some(port) = cfg.metrics_port {
        let ready2 = ready.clone();
        tokio::spawn(run_healthz_server(port, ready2));
    }

    let pool = open_sqlite_pool(&cfg.database_url).await?;

    let auth_store_inner = SqliteAuthStore::from_pool(pool.clone());
    auth_store_inner.migrate().await?;

    let rule_store_inner = SqliteRuleStore::new(pool.clone());
    rule_store_inner.migrate().await?;

    let auth_store: Arc<dyn tunnel_store::AuthStore> = Arc::new(auth_store_inner);
    let rule_store: Arc<dyn tunnel_store::RuleStore> = Arc::new(rule_store_inner);

    // Seed routing from server.yaml on first boot (when the routing table is empty).
    if let Some(ref server_cfg_path) = cfg.server_config {
        match rule_store.is_routing_empty().await {
            Ok(true) => {
                info!(path = %server_cfg_path, "routing DB empty — seeding from server config");
                match tunnel_store::server_config::ServerConfigFile::load(server_cfg_path) {
                    Ok(server_cfg) => {
                        let data = tunnel_store::server_config::routing_data_from_server_config(
                            &server_cfg,
                        );
                        if let Err(e) = rule_store.save_routing(&data).await {
                            tracing::warn!(error = %e, "failed to seed routing from server config (non-fatal)");
                        } else {
                            info!(
                                listeners = data.ingress_listeners.len(),
                                groups = data.client_groups.len(),
                                egress_upstreams = data.egress_upstreams.len(),
                                "routing seeded from server config"
                            );
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, path = %server_cfg_path,
                        "failed to parse server config for routing seed (non-fatal)"),
                }
            }
            Ok(false) => info!("routing DB already populated, skipping server config seed"),
            Err(e) => tracing::warn!(error = %e, "could not check routing DB state, skipping seed"),
        }
    }

    let svc = ControlService::new(auth_store, rule_store, pool).await?;

    // All init done — mark ready.
    ready.store(true, Ordering::Release);

    match args.command.unwrap_or(Command::Serve) {
        Command::Serve => {
            let addr: SocketAddr = cfg.watch_addr.parse()?;
            WatchServer::new(Arc::clone(&svc), addr, cfg.watch_token.clone())
                .run()
                .await?;
        }
        Command::Client(cmd) => {
            run_cli(cmd, &svc).await?;
        }
    }

    Ok(())
}
