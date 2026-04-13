pub fn apply_worker_threads(builder: &mut tokio::runtime::Builder) {
    if let Ok(s) = std::env::var("TOKIO_WORKER_THREADS") {
        match s.parse::<usize>() {
            Ok(n) if n > 0 => { builder.worker_threads(n); }
            Ok(_) => tracing::warn!("TOKIO_WORKER_THREADS=0, using default"),
            Err(_) => tracing::warn!("TOKIO_WORKER_THREADS={:?} is not a valid integer, using default", s),
        }
    }
}

pub fn build_proxy_runtime() -> tokio::runtime::Runtime {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    apply_worker_threads(&mut builder);
    builder.build().expect("failed to build proxy runtime")
}

pub fn build_single_thread_runtime(name: &str) -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name(name)
        .build()
        .expect("failed to build single-thread runtime")
}
