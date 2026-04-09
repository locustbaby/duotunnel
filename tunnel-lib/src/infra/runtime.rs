pub fn apply_worker_threads(builder: &mut tokio::runtime::Builder) {
    match std::env::var("TOKIO_WORKER_THREADS") {
        Ok(s) => match s.parse::<usize>() {
            Ok(n) if n > 0 => { builder.worker_threads(n); }
            Ok(_) => tracing::warn!("TOKIO_WORKER_THREADS=0, using default"),
            Err(_) => tracing::warn!("TOKIO_WORKER_THREADS={:?} is not a valid integer, using default", s),
        },
        Err(_) => {}
    }
}
