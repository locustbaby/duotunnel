use crate::config::ServerConfigFile;
use crate::service::BackgroundService;
use crate::{build_routing_snapshot, ServerState};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use tunnel_lib::HttpClientParams;

pub struct HotReloadService {
    pub config_path: String,
}

impl BackgroundService for HotReloadService {
    fn name(&self) -> &'static str {
        "hot-reload"
    }

    fn run(
        self: Box<Self>,
        state: Arc<ServerState>,
        shutdown: CancellationToken,
        _proxy_handle: tokio::runtime::Handle,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            if let Err(e) = watch_loop(self.config_path, state, shutdown).await {
                error!(error = %e, "hot-reload watcher exited unexpectedly");
            }
            Ok(())
        })
    }
}


async fn watch_loop(
    config_path: String,
    state: Arc<ServerState>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel::<()>(1);
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                use notify::EventKind::*;
                match event.kind {
                    Create(_) | Modify(_) | Remove(_) => {
                        let _ = tx.try_send(());
                    }
                    _ => {}
                }
            }
        },
        notify::Config::default(),
    )?;
    let watch_dir = std::path::Path::new(&config_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    if !watch_dir.exists() {
        warn!(
            path = %config_path,
            dir = %watch_dir.display(),
            "hot-reload disabled: config directory not found"
        );
        return Ok(());
    }
    watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;
    info!(path = %config_path, "hot-reload watcher started");
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            result = rx.recv() => {
                if result.is_none() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
                while rx.try_recv().is_ok() {}
                info!(path = %config_path, "config change detected, reloading");
                match reload_routing(&config_path, &state).await {
                    Ok(()) => info!(path = %config_path, "hot reload successful"),
                    Err(e) => warn!(
                        path = %config_path,
                        error = %e,
                        "hot reload failed, keeping previous config"
                    ),
                }
            }
        }
    }
    Ok(())
}

async fn reload_routing(config_path: &str, state: &Arc<ServerState>) -> anyhow::Result<()> {
    let http_params = HttpClientParams::from(&state.config.server.http_pool);
    let (tm, egress) = match ServerConfigFile::load(config_path) {
        Ok(new_config) => {
            if let Err(e) =
                crate::config::sync_file_to_db(&new_config, state.rule_store.as_ref()).await
            {
                warn!(error = %e, "failed to sync updated YAML to DB");
            }
            (
                new_config.tunnel_management,
                new_config.server_egress_upstream,
            )
        }
        Err(e) => {
            warn!(error = %e, "could not re-read config file; reloading from DB");
            state.config_source.load().await?
        }
    };
    let snapshot = build_routing_snapshot(&tm, &egress, &http_params);
    state.routing.store(Arc::new(snapshot));
    let listeners: Vec<_> = tm.server_ingress_routing.listeners.to_vec();
    crate::sync_listeners(state, &listeners);
    Ok(())
}
