use crate::bootstrap::{cli, CtldBootstrap};
use crate::control::layer::{ConfigSource, SqliteConfigSource, YamlConfigSource};
use crate::control::revision::SqliteControlRevisionStore;
use crate::control::service::{ControlService, DegradedSource};
use crate::control::token::cache::{SqliteTokenCacheProvider, TokenCacheProvider};
use crate::control::watch::WatchServer;
use crate::storage::sqlite::{open_sqlite_pool, SqliteAuthStore};
use crate::storage::sqlite_rules::SqliteRuleStore;
use anyhow::Result;
use figment::{
    providers::{Format, Yaml},
    Figment,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::{TcpListener, UnixListener, UnixStream};
use tokio::sync::oneshot;
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
        let config_path = self.args.config.clone();
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
                let legacy_server_config = configured_legacy_server_config_path(&config_path)?;
                let yaml_source =
                    configured_yaml_source(cfg, legacy_server_config.as_deref()).await?;
                let (svc, sqlite_source) = build_control_service(
                    cfg,
                    yaml_source.as_ref(),
                    legacy_server_config.is_some(),
                )
                .await?;
                {
                    let svc_ref = Arc::clone(&svc);
                    let mut changes = sqlite_source.subscribe();
                    let svc_degraded = Arc::clone(&svc);
                    let mut degraded = sqlite_source.subscribe_degraded();
                    tokio::spawn(async move {
                        while degraded.changed().await.is_ok() {
                            if *degraded.borrow() {
                                if let Err(error) = svc_degraded
                                    .set_source_degraded(DegradedSource::Sqlite, true)
                                    .await
                                {
                                    tracing::warn!(
                                        error = %error,
                                        "failed to persist degraded SQLite source state"
                                    );
                                }
                            } else if let Err(error) = svc_degraded
                                .set_source_degraded(DegradedSource::Sqlite, false)
                                .await
                            {
                                tracing::warn!(
                                    error = %error,
                                    "failed to clear degraded SQLite source state"
                                );
                            }
                        }
                    });
                    tokio::spawn(async move {
                        while changes.changed().await.is_ok() {
                            let mut layer = changes.borrow().clone();
                            let mut backoff = std::time::Duration::from_secs(1);
                            loop {
                                match svc_ref.apply_sqlite_layer(&layer).await {
                                    Ok(_) => break,
                                    Err(error) => {
                                        tracing::warn!(
                                            error = %error,
                                            backoff_secs = backoff.as_secs(),
                                            "failed to apply updated SQLite source layer; retrying"
                                        );
                                        let sleep = tokio::time::sleep(backoff);
                                        tokio::pin!(sleep);
                                        tokio::select! {
                                            changed = changes.changed() => {
                                                if changed.is_err() {
                                                    return;
                                                }
                                                let next = changes.borrow().clone();
                                                backoff = std::time::Duration::from_secs(1);
                                                layer = next;
                                            }
                                            _ = &mut sleep => {
                                                backoff =
                                                    (backoff * 2).min(std::time::Duration::from_secs(30));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
                if let Some(source) = yaml_source {
                    let svc_ref = Arc::clone(&svc);
                    let mut changes = source.subscribe();
                    let svc_degraded = Arc::clone(&svc);
                    let mut degraded = source.subscribe_degraded();
                    tokio::spawn(async move {
                        while degraded.changed().await.is_ok() {
                            if *degraded.borrow() {
                                if let Err(error) = svc_degraded
                                    .set_source_degraded(DegradedSource::Yaml, true)
                                    .await
                                {
                                    tracing::warn!(
                                        error = %error,
                                        "failed to persist degraded YAML state"
                                    );
                                }
                            } else if let Err(error) = svc_degraded
                                .set_source_degraded(DegradedSource::Yaml, false)
                                .await
                            {
                                tracing::warn!(
                                    error = %error,
                                    "failed to clear degraded YAML source state"
                                );
                            }
                        }
                    });
                    tokio::spawn(async move {
                        while changes.changed().await.is_ok() {
                            let mut layer = changes.borrow().clone();
                            let mut backoff = std::time::Duration::from_secs(1);
                            loop {
                                match svc_ref.apply_yaml_layer(&layer).await {
                                    Ok(_) => {
                                        if let Err(error) = svc_ref
                                            .set_source_degraded(DegradedSource::Yaml, false)
                                            .await
                                        {
                                            tracing::warn!(
                                                error = %error,
                                                "failed to clear degraded YAML state"
                                            );
                                        }
                                        break;
                                    }
                                    Err(error) => {
                                        tracing::warn!(
                                            error = %error,
                                            backoff_secs = backoff.as_secs(),
                                            "failed to apply updated YAML layer; retrying"
                                        );
                                        if let Err(mark_error) = svc_ref
                                            .set_source_degraded(DegradedSource::Yaml, true)
                                            .await
                                        {
                                            tracing::warn!(
                                                error = %mark_error,
                                                "failed to persist degraded YAML state"
                                            );
                                        }
                                        let sleep = tokio::time::sleep(backoff);
                                        tokio::pin!(sleep);
                                        tokio::select! {
                                            changed = changes.changed() => {
                                                if changed.is_err() {
                                                    return;
                                                }
                                                layer = changes.borrow().clone();
                                                backoff = std::time::Duration::from_secs(1);
                                            }
                                            _ = &mut sleep => {
                                                backoff =
                                                    (backoff * 2).min(std::time::Duration::from_secs(30));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
                let addr: SocketAddr = cfg.watch_addr.parse()?;
                let watch_listener = TcpListener::bind(addr).await?;

                let (admin_ready_tx, admin_ready_rx) = oneshot::channel();
                tokio::spawn(run_admin_server(
                    cfg.admin_socket.clone(),
                    Arc::clone(&svc),
                    admin_ready_tx,
                ));
                if let Some(port) = cfg.metrics_port {
                    let (health_ready_tx, health_ready_rx) = oneshot::channel();
                    tokio::spawn(run_healthz_server(
                        port,
                        Arc::clone(&ready),
                        health_ready_tx,
                    ));
                    await_ready(admin_ready_rx, "admin socket").await?;
                    await_ready(health_ready_rx, "healthz").await?;
                } else {
                    await_ready(admin_ready_rx, "admin socket").await?;
                }
                ready.store(true, Ordering::Release);
                WatchServer::new(Arc::clone(&svc), addr, cfg.watch_token.clone())
                    .run_with_listener(watch_listener)
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
    legacy_server_config_source: bool,
) -> Result<(Arc<ControlService>, Arc<SqliteConfigSource>)> {
    let database_url = configured_database_url(cfg);
    let pool = open_sqlite_pool(database_url, 8).await?;

    let auth_store_inner = SqliteAuthStore::from_pool(pool.clone());
    auth_store_inner.migrate().await?;

    let rule_store_inner = SqliteRuleStore::new(pool.clone());
    rule_store_inner.migrate().await?;
    crate::control::layer::initialize_sqlite_layer(
        &pool,
        &rule_store_inner,
        legacy_server_config_source,
    )
    .await?;
    let sqlite_source = SqliteConfigSource::new(pool.clone()).await?;

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
    let sqlite_layer = sqlite_source.load().await?;
    svc.apply_sqlite_layer(&sqlite_layer).await?;
    if yaml_source.is_some() {
        svc.do_publish().await?;
    }
    Ok((svc, sqlite_source))
}

async fn configured_yaml_source(
    cfg: &crate::bootstrap::config::Config,
    legacy_server_config: Option<&str>,
) -> Result<Option<Arc<YamlConfigSource>>> {
    let yaml = cfg
        .config
        .sources
        .iter()
        .filter(|source| source.kind.eq_ignore_ascii_case("yaml"))
        .max_by_key(|source| source.priority);
    let Some(spec) = yaml else {
        return match legacy_server_config {
            Some(path) => Ok(Some(YamlConfigSource::new(path).await?)),
            None => Ok(None),
        };
    };
    let path = spec
        .path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("YAML config source requires path"))?;
    Ok(Some(YamlConfigSource::new(path).await?))
}

#[derive(Debug, Deserialize, Default)]
struct LegacyCtldConfig {
    #[serde(default)]
    server_config: Option<String>,
}

fn configured_legacy_server_config_path(config_path: &str) -> Result<Option<String>> {
    let legacy: LegacyCtldConfig = Figment::new().merge(Yaml::file(config_path)).extract()?;
    Ok(legacy
        .server_config
        .map(|path| path.trim().to_owned())
        .filter(|path| !path.is_empty()))
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

async fn await_ready(
    receiver: oneshot::Receiver<std::result::Result<(), String>>,
    component: &str,
) -> Result<()> {
    receiver
        .await
        .map_err(|_| anyhow::anyhow!("{component} startup task exited before binding"))?
        .map_err(|error| anyhow::anyhow!("{component} failed to bind: {error}"))
}

async fn run_healthz_server(
    port: u16,
    ready: Arc<AtomicBool>,
    ready_sender: oneshot::Sender<std::result::Result<(), String>>,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let addr = format!("0.0.0.0:{port}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(error) => {
            let _ = ready_sender.send(Err(error.to_string()));
            tracing::warn!(addr = %addr, error = %error, "failed to bind healthz server");
            return;
        }
    };
    let _ = ready_sender.send(Ok(()));

    info!(addr = %addr, "healthz server started");
    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let ready_state = Arc::clone(&ready);
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

async fn run_admin_server(
    socket_path: String,
    svc: Arc<ControlService>,
    ready_sender: oneshot::Sender<std::result::Result<(), String>>,
) {
    use tokio::io::AsyncWriteExt;

    if let Some(parent) = std::path::Path::new(&socket_path).parent() {
        if let Err(error) = tokio::fs::create_dir_all(parent).await {
            let _ = ready_sender.send(Err(error.to_string()));
            return;
        }
    }
    if std::path::Path::new(&socket_path).exists() {
        match UnixStream::connect(&socket_path).await {
            Ok(_) => {
                let _ =
                    ready_sender.send(Err("another ctld admin socket is already active".into()));
                tracing::warn!(path = %socket_path, "another ctld admin socket is already active");
                return;
            }
            Err(connect_error) => {
                if let Err(remove_error) = tokio::fs::remove_file(&socket_path).await {
                    tracing::warn!(
                        path = %socket_path,
                        connect_error = %connect_error,
                        remove_error = %remove_error,
                        "stale ctld admin socket could not be removed"
                    );
                    let _ = ready_sender.send(Err(remove_error.to_string()));
                    return;
                }
            }
        }
    }
    let listener = match UnixListener::bind(&socket_path) {
        Ok(listener) => listener,
        Err(error) => {
            let _ = ready_sender.send(Err(error.to_string()));
            tracing::warn!(path = %socket_path, error = %error, "failed to bind ctld admin socket");
            return;
        }
    };
    if let Err(error) = tokio::fs::set_permissions(
        &socket_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )
    .await
    {
        let _ = ready_sender.send(Err(error.to_string()));
        let _ = tokio::fs::remove_file(&socket_path).await;
        return;
    }
    let _ = ready_sender.send(Ok(()));
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
                admin_reason_phrase(status),
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}

fn admin_reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        410 => "Gone",
        428 => "Precondition Required",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Unknown Status",
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
