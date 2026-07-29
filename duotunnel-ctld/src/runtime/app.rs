use crate::bootstrap::{cli, CtldBootstrap};
use crate::control::layer::{ConfigSource, YamlConfigSource};
use crate::control::revision::SqliteControlRevisionStore;
use crate::control::service::ControlService;
use crate::control::token::cache::{SqliteTokenCacheProvider, TokenCacheProvider};
use crate::control::watch::WatchServer;
use crate::storage::sqlite::{open_sqlite_pool, SqliteAuthStore};
use crate::storage::sqlite_rules::SqliteRuleStore;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

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
            "starting duotunnel-ctld"
        );

        let ready = Arc::new(AtomicBool::new(false));
        match bootstrap.command().cloned().unwrap_or(cli::Command::Serve) {
            cli::Command::Serve => {
                let yaml_source = configured_yaml_source(cfg).await?;
                let svc = build_control_service(cfg, yaml_source.as_ref()).await?;
                if let Some(source) = yaml_source {
                    let svc_ref = Arc::clone(&svc);
                    let mut changes = source.subscribe();
                    tokio::spawn(async move {
                        while changes.changed().await.is_ok() {
                            let layer = changes.borrow().clone();
                            if let Err(error) = svc_ref.apply_yaml_layer(&layer).await {
                                tracing::warn!(error = %error, "failed to apply updated YAML layer");
                            }
                        }
                    });
                }
                let ready_state = Arc::clone(&ready);
                let svc_ref = Arc::clone(&svc);
                tokio::spawn(run_admin_server(cfg.admin_socket.clone(), Arc::clone(&svc)));
                if let Some(port) = cfg.metrics_port {
                    tokio::spawn(run_healthz_server(port, ready_state, svc_ref));
                }
                ready.store(true, Ordering::Release);
                let addr: SocketAddr = cfg.watch_addr.parse()?;
                WatchServer::new(Arc::clone(&svc), addr, cfg.watch_token.clone())
                    .run()
                    .await?;
            }
            cli::Command::Client(cmd) => {
                cli::run_client_cli(&cfg.admin_socket, cmd).await?;
            }
            cli::Command::Token(cmd) => {
                cli::run_token_cli(&cfg.admin_socket, cmd).await?;
            }
        }

        Ok(())
    }
}

async fn build_control_service(
    cfg: &crate::bootstrap::config::Config,
    yaml_source: Option<&Arc<YamlConfigSource>>,
) -> Result<Arc<ControlService>> {
    let database_url = configured_database_url(cfg);
    let pool = open_sqlite_pool(database_url, 8).await?;

    let auth_store_inner = SqliteAuthStore::from_pool(pool.clone());
    auth_store_inner.migrate().await?;

    let rule_store_inner = SqliteRuleStore::new(pool.clone());
    rule_store_inner.migrate().await?;
    crate::control::layer::initialize_sqlite_layer(&pool, &rule_store_inner).await?;

    let auth_store: Arc<dyn crate::storage::AuthStore> = Arc::new(auth_store_inner);
    let rule_store: Arc<dyn crate::storage::RuleStore> = Arc::new(rule_store_inner);
    let token_cache: Arc<dyn TokenCacheProvider> =
        Arc::new(SqliteTokenCacheProvider::new(pool.clone()));

    let revision_store = Arc::new(SqliteControlRevisionStore::initialize(pool.clone()).await?);
    let svc = ControlService::new_with_sqlite_revision_store(
        auth_store,
        rule_store,
        token_cache,
        revision_store,
        pool,
    )
    .await?;
    if let Some(source) = yaml_source {
        let layer = source.load().await?;
        svc.apply_yaml_layer(&layer).await?;
    }
    Ok(svc)
}

async fn configured_yaml_source(
    cfg: &crate::bootstrap::config::Config,
) -> Result<Option<Arc<YamlConfigSource>>> {
    let yaml = cfg
        .config
        .sources
        .iter()
        .filter(|source| source.kind.eq_ignore_ascii_case("yaml"))
        .max_by_key(|source| source.priority);
    let Some(spec) = yaml else { return Ok(None) };
    let path = spec
        .path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("YAML config source requires path"))?;
    Ok(Some(YamlConfigSource::new(path).await?))
}

fn configured_database_url(cfg: &crate::bootstrap::config::Config) -> &str {
    cfg.config
        .sources
        .iter()
        .filter(|source| source.kind.eq_ignore_ascii_case("sqlite"))
        .max_by_key(|source| source.priority)
        .and_then(|source| source.database_url.as_deref())
        .unwrap_or(&cfg.database_url)
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

async fn run_admin_server(socket_path: String, svc: Arc<ControlService>) {
    use tokio::io::AsyncWriteExt;
    use tokio::net::UnixListener;

    if let Some(parent) = std::path::Path::new(&socket_path).parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if std::path::Path::new(&socket_path).exists() {
        let _ = tokio::fs::remove_file(&socket_path).await;
    }
    let listener = match UnixListener::bind(&socket_path) {
        Ok(listener) => listener,
        Err(error) => {
            tracing::warn!(path = %socket_path, error = %error, "failed to bind ctld admin socket");
            return;
        }
    };
    let _ = tokio::fs::set_permissions(
        &socket_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )
    .await;
    info!(path = %socket_path, "ctld admin socket started");
    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let svc_ref = Arc::clone(&svc);
        tokio::spawn(async move {
            let (status, body) = match read_admin_request(&mut stream).await {
                Ok(request) => cli::handle_admin_request(&request, svc_ref.as_ref()).await,
                Err(error) => (400, error.to_string()),
            };
            let response = format!(
                "HTTP/1.1 {status} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                if status == 200 { "OK" } else { "Bad Request" },
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}

async fn read_admin_request(stream: &mut tokio::net::UnixStream) -> Result<String> {
    const MAX_REQUEST_BYTES: usize = 256 * 1024;
    let read = async {
        let mut request = Vec::with_capacity(8192);
        let mut chunk = [0u8; 8192];
        let mut expected_len = None;
        loop {
            let size = tokio::io::AsyncReadExt::read(stream, &mut chunk).await?;
            if size == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..size]);
            if request.len() > MAX_REQUEST_BYTES {
                anyhow::bail!("admin request exceeds {} bytes", MAX_REQUEST_BYTES);
            }
            if let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                let header_end = header_end + 4;
                if expected_len.is_none() {
                    let headers = std::str::from_utf8(&request[..header_end])?;
                    expected_len = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                        })
                        .flatten();
                    if expected_len.is_none()
                        && headers
                            .lines()
                            .next()
                            .is_some_and(|line| line.starts_with("GET "))
                    {
                        return Ok(request);
                    }
                }
                if request.len() >= header_end + expected_len.unwrap_or(0) {
                    return Ok(request);
                }
            }
        }
        anyhow::bail!("incomplete admin request")
    };
    let request = tokio::time::timeout(std::time::Duration::from_secs(5), read).await??;
    Ok(std::str::from_utf8(&request)?.to_string())
}
