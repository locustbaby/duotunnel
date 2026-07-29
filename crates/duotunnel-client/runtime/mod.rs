use anyhow::Result;
use std::future::Future;
#[cfg(feature = "dial9-telemetry")]
use std::path::PathBuf;
#[cfg(feature = "dial9-telemetry")]
use tracing::{error, info};

mod app;
pub(crate) mod engine;

#[cfg(feature = "dial9-telemetry")]
static DIAL9_HANDLE: std::sync::OnceLock<dial9_tokio_telemetry::telemetry::TelemetryHandle> =
    std::sync::OnceLock::new();

pub(crate) use app::ClientApp;

pub(crate) fn run(fut: impl Future<Output = Result<()>>) -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    #[cfg(feature = "dial9-telemetry")]
    if let Some(trace_path) = std::env::var_os("DIAL9_TRACE_PATH").map(PathBuf::from) {
        return run_with_dial9(trace_path, fut);
    }
    run_with_tokio(fut)
}

pub(crate) fn init_observability(log_level: &str) {
    duotunnel_core::infra::observability::init_tracing(log_level);
}

pub(crate) fn spawn_task<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[cfg(feature = "dial9-telemetry")]
    if std::env::var_os("DIAL9_TRACE_PATH").is_some() {
        if let Some(handle) = DIAL9_HANDLE.get() {
            return handle.spawn(future);
        }
    }

    tokio::task::spawn(future)
}

fn run_with_tokio(fut: impl Future<Output = Result<()>>) -> Result<()> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    duotunnel_core::apply_worker_threads(&mut builder);
    builder.enable_all();
    let runtime = builder.build()?;
    runtime.block_on(fut)
}

#[cfg(feature = "dial9-telemetry")]
fn run_with_dial9(trace_path: PathBuf, fut: impl Future<Output = Result<()>>) -> Result<()> {
    use dial9_tokio_telemetry::telemetry::cpu_profile::{CpuProfilingConfig, SchedEventConfig};
    use dial9_tokio_telemetry::telemetry::{RotatingWriter, TracedRuntime};

    let writer = RotatingWriter::builder()
        .base_path(&trace_path)
        .max_file_size(512 * 1024 * 1024)
        .max_total_size(512 * 1024 * 1024)
        .build()?;
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    duotunnel_core::apply_worker_threads(&mut builder);
    let trace_path_display = trace_path.display().to_string();
    let trace_file_display = {
        let stem = trace_path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("trace");
        trace_path
            .with_file_name(format!("{stem}.0.bin"))
            .display()
            .to_string()
    };
    let (runtime, guard) = TracedRuntime::builder()
        .with_task_tracking(true)
        .with_trace_path(trace_path)
        .with_cpu_profiling(CpuProfilingConfig::default())
        .with_sched_events(SchedEventConfig::default().include_kernel(true))
        .build_and_start_with_writer(builder, writer)?;
    let _ = DIAL9_HANDLE.set(guard.handle());
    info!("dial9 trace started, base path: {trace_path_display}");
    let result = runtime.block_on(async {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            r = fut => r,
            _ = sigterm.recv() => {
                info!("SIGTERM received, starting graceful shutdown");
                Ok(())
            }
        }
    });
    info!("runtime stopped, flushing dial9 trace (timeout 30s)");
    drop(runtime);
    match guard.graceful_shutdown(std::time::Duration::from_secs(30)) {
        Ok(()) => info!("dial9 trace flush complete, output: {trace_file_display}"),
        Err(e) => error!("dial9 trace flush error: {e}"),
    }
    result
}
