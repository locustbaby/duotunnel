pub fn available_parallelism() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}

pub fn resolve_shard_count(configured: Option<usize>, max_shards: Option<usize>) -> usize {
    let cpu = available_parallelism().max(1);
    let mut resolved = configured.unwrap_or(cpu).max(1);
    if let Some(limit) = max_shards.map(|value| value.max(1)) {
        resolved = resolved.min(limit);
    }
    resolved.max(1)
}

pub fn apply_worker_threads(builder: &mut tokio::runtime::Builder) {
    if let Ok(s) = std::env::var("TOKIO_WORKER_THREADS") {
        match s.parse::<usize>() {
            Ok(n) if n > 0 => {
                builder.worker_threads(n);
            }
            Ok(_) => tracing::warn!("TOKIO_WORKER_THREADS=0, using default"),
            Err(_) => tracing::warn!(
                "TOKIO_WORKER_THREADS={:?} is not a valid integer, using default",
                s
            ),
        }
    }
}

pub fn build_proxy_runtime() -> tokio::runtime::Runtime {
    let mut b = tokio::runtime::Builder::new_multi_thread();
    apply_worker_threads(&mut b);
    b.enable_all()
        .thread_name("proxy-worker")
        .build()
        .expect("proxy runtime")
}

pub fn build_single_thread_runtime(name: &str) -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name(name)
        .build()
        .expect("single thread runtime")
}
