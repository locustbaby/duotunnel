fn parse_worker_threads(value: Option<&str>) -> Option<usize> {
    match value {
        Some(raw) => match raw.parse::<usize>() {
            Ok(n) if n > 0 => Some(n),
            Ok(_) => {
                tracing::warn!("TOKIO_WORKER_THREADS=0, using default");
                None
            }
            Err(_) => {
                tracing::warn!(
                    "TOKIO_WORKER_THREADS={:?} is not a valid integer, using default",
                    raw
                );
                None
            }
        },
        None => None,
    }
}

pub fn available_parallelism() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}

pub fn configured_worker_threads() -> Option<usize> {
    parse_worker_threads(std::env::var("TOKIO_WORKER_THREADS").ok().as_deref())
}

pub fn effective_runtime_parallelism() -> usize {
    configured_worker_threads().unwrap_or_else(available_parallelism)
}

pub fn resolve_shard_count(configured: Option<usize>, max_shards: Option<usize>) -> usize {
    let parallelism = effective_runtime_parallelism().max(1);
    let mut resolved = configured.unwrap_or(parallelism).max(1);
    if let Some(limit) = max_shards.map(|value| value.max(1)) {
        resolved = resolved.min(limit);
    }
    resolved.max(1)
}

pub fn apply_worker_threads(builder: &mut tokio::runtime::Builder) {
    if let Some(n) = configured_worker_threads() {
        builder.worker_threads(n);
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

#[cfg(test)]
mod tests {
    use super::parse_worker_threads;

    #[test]
    fn parse_worker_threads_accepts_positive_values() {
        assert_eq!(parse_worker_threads(Some("4")), Some(4));
    }

    #[test]
    fn parse_worker_threads_rejects_zero() {
        assert_eq!(parse_worker_threads(Some("0")), None);
    }

    #[test]
    fn parse_worker_threads_rejects_invalid_values() {
        assert_eq!(parse_worker_threads(Some("abc")), None);
    }

    #[test]
    fn parse_worker_threads_handles_missing_value() {
        assert_eq!(parse_worker_threads(None), None);
    }
}
