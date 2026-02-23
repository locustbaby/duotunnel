use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::config::ServerConfigFile;
use crate::{build_routing_snapshot, ServerState};
use tunnel_lib::HttpClientParams;

pub fn spawn_config_watcher(config_path: String, state: Arc<ServerState>) {
    tokio::spawn(async move {
        if let Err(e) = watch_loop(config_path, state).await {
            error!(error = %e, "hot-reload watcher exited unexpectedly");
        }
    });
}

async fn watch_loop(config_path: String, state: Arc<ServerState>) -> anyhow::Result<()> {
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
        warn!(path = %config_path, dir = %watch_dir.display(), "hot-reload disabled: config directory not found");
        return Ok(());
    }
    watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;
    info!(path = %config_path, "hot-reload watcher started");

    loop {
        if rx.recv().await.is_none() {
            break;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        while rx.try_recv().is_ok() {}

        info!(path = %config_path, "config change detected, reloading");

        match reload_routing(&config_path, &state) {
            Ok(()) => info!(path = %config_path, "hot reload successful"),
            Err(e) => {
                warn!(path = %config_path, error = %e, "hot reload failed, keeping previous config")
            }
        }
    }

    Ok(())
}

fn reload_routing(config_path: &str, state: &Arc<ServerState>) -> anyhow::Result<()> {
    let new_config = ServerConfigFile::load(config_path)?;

    let http_params = HttpClientParams::from(&new_config.server.http_pool);
    let snapshot = build_routing_snapshot(
        &new_config.tunnel_management,
        &new_config.server_egress_upstream,
        &http_params,
    );

    state.routing.store(Arc::new(snapshot));
    Ok(())
}
