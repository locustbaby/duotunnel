pub fn apply_worker_threads(builder: &mut tokio::runtime::Builder) {
    if let Ok(s) = std::env::var("TOKIO_WORKER_THREADS") {
        match s.parse::<usize>() {
            Ok(n) if n > 0 => { builder.worker_threads(n); }
            Ok(_) => tracing::warn!("TOKIO_WORKER_THREADS=0, using default"),
            Err(_) => tracing::warn!("TOKIO_WORKER_THREADS={:?} is not a valid integer, using default", s),
        }
    }
}
