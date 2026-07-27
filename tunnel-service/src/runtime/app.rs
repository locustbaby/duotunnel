use crate::bootstrap::{cli, CtldBootstrap};
use crate::control::revision::SqliteControlRevisionStore;
use crate::control::service::ControlService;
use crate::control::token::cache::{SqliteTokenCacheProvider, TokenCacheProvider};
use crate::control::watch::WatchServer;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tunnel_store::sqlite::{open_sqlite_pool, SqliteAuthStore};
use tunnel_store::sqlite_rules::SqliteRuleStore;

pub(crate) struct CtldApp {
    args: cli::Args,
}

impl CtldApp {
    pub(crate) fn new(args: cli::Args) -> Self {
        Self { args }
    }

    pub(crate) async fn run(self) -> Result<()> {
        let bootstrap = CtldBootstrap::from_args(self.args)?;
        let cfg = bootstrap.config();

        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(cfg.log_level()))
            .init();

        info!(
            database_url = %cfg.database_url,
            watch_addr = %cfg.watch_addr,
            watch_auth = cfg.watch_token.as_ref().is_some_and(|t| !t.trim().is_empty()),
            "starting tunnel-ctld"
        );

        let ready = Arc::new(AtomicBool::new(false));
        let svc = build_control_service(cfg).await?;

        if let Some(port) = cfg.metrics_port {
            let ready_state = Arc::clone(&ready);
            let svc_ref = Arc::clone(&svc);
            tokio::spawn(run_healthz_server(port, ready_state, svc_ref));
        }

        ready.store(true, Ordering::Release);

        match bootstrap.command().cloned().unwrap_or(cli::Command::Serve) {
            cli::Command::Serve => {
                let addr: SocketAddr = cfg.watch_addr.parse()?;
                WatchServer::new(Arc::clone(&svc), addr, cfg.watch_token.clone())
                    .run()
                    .await?;
            }
            cli::Command::Client(cmd) => {
                cli::run_cli(cmd, svc.as_ref()).await?;
            }
        }

        Ok(())
    }
}

async fn build_control_service(
    cfg: &crate::bootstrap::config::Config,
) -> Result<Arc<ControlService>> {
    let pool = open_sqlite_pool(&cfg.database_url, 8).await?;

    let auth_store_inner = SqliteAuthStore::from_pool(pool.clone());
    auth_store_inner.migrate().await?;

    let rule_store_inner = SqliteRuleStore::new(pool.clone());
    rule_store_inner.migrate().await?;

    let auth_store: Arc<dyn tunnel_store::AuthStore> = Arc::new(auth_store_inner);
    let rule_store: Arc<dyn tunnel_store::RuleStore> = Arc::new(rule_store_inner);
    let token_cache: Arc<dyn TokenCacheProvider> =
        Arc::new(SqliteTokenCacheProvider::new(pool.clone()));

    if let Some(server_cfg_path) = cfg.server_config.as_ref() {
        seed_routing_if_needed(rule_store.as_ref(), server_cfg_path).await;
    }

    let revision_store = Arc::new(SqliteControlRevisionStore::initialize(pool).await?);
    ControlService::new_with_revision_store(auth_store, rule_store, token_cache, revision_store)
        .await
}

async fn seed_routing_if_needed(rule_store: &dyn tunnel_store::RuleStore, server_cfg_path: &str) {
    match rule_store.is_routing_empty().await {
        Ok(true) => {
            info!(path = %server_cfg_path, "routing DB empty — seeding from server config");
            match tunnel_store::server_config::ServerConfigFile::load(server_cfg_path) {
                Ok(server_cfg) => {
                    let data =
                        tunnel_store::server_config::routing_data_from_server_config(&server_cfg);
                    if let Err(error) = rule_store.save_routing(&data).await {
                        tracing::warn!(error = %error, "failed to seed routing from server config (non-fatal)");
                    } else {
                        info!(
                            listeners = data.ingress_listeners.len(),
                            groups = data.client_groups.len(),
                            egress_upstreams = data.egress_upstreams.len(),
                            "routing seeded from server config"
                        );
                    }
                }
                Err(error) => tracing::warn!(
                    error = %error,
                    path = %server_cfg_path,
                    "failed to parse server config for routing seed (non-fatal)"
                ),
            }
        }
        Ok(false) => info!("routing DB already populated, skipping server config seed"),
        Err(error) => {
            tracing::warn!(error = %error, "could not check routing DB state, skipping seed")
        }
    }
}

async fn run_healthz_server(port: u16, ready: Arc<AtomicBool>, svc: Arc<ControlService>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let addr = format!("0.0.0.0:{port}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::warn!(addr = %addr, error = %error, "failed to bind healthz server");
            return;
        }
    };

    info!(addr = %addr, "healthz server started");
    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let ready_state = Arc::clone(&ready);
        let svc_ref = Arc::clone(&svc);
        tokio::spawn(async move {
            let mut buf = [0u8; 256];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
            let (status, body) = if req.starts_with("GET /healthz") {
                if ready_state.load(Ordering::Acquire) {
                    ("200 OK", "ok\n")
                } else {
                    ("503 Service Unavailable", "not ready\n")
                }
            } else if req.starts_with("POST /api/reload") {
                tracing::info!("manual reload triggered via api");
                svc_ref.publish();
                ("200 OK", "reloaded\n")
            } else {
                ("404 Not Found", "not found\n")
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
