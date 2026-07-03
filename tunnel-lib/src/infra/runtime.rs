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

/// CPUs allowed by the process cgroup (systemd `CPUQuota` etc.), if capped.
pub fn cgroup_cpu_limit() -> Option<usize> {
    cgroup_cpu_limit_inner()
}

fn cgroup_cpu_limit_inner() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        if let Some(cpus) = read_cgroup_v2_cpu_max() {
            return Some(cpus);
        }
        read_cgroup_v1_cpu_quota()
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn read_cgroup_v2_cpu_max() -> Option<usize> {
    let cgroup = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    for line in cgroup.lines() {
        let (_, path) = line.split_once("::")?;
        let path = path.trim_start_matches('/');
        let cpu_max = if path.is_empty() {
            std::path::PathBuf::from("/sys/fs/cgroup/cpu.max")
        } else {
            std::path::PathBuf::from("/sys/fs/cgroup").join(path).join("cpu.max")
        };
        if let Ok(contents) = std::fs::read_to_string(&cpu_max) {
            if let Some(cpus) = parse_cpu_max_contents(&contents) {
                return Some(cpus);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn read_cgroup_v1_cpu_quota() -> Option<usize> {
    let cgroup = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    for line in cgroup.lines() {
        if line.contains("::") {
            continue;
        }
        let path = line.rsplit_once(':').map(|(_, p)| p.trim_start_matches('/'))?;
        let base = std::path::PathBuf::from("/sys/fs/cgroup/cpu").join(path);
        let quota = std::fs::read_to_string(base.join("cpu.cfs_quota_us")).ok()?;
        let period = std::fs::read_to_string(base.join("cpu.cfs_period_us")).ok()?;
        if let Some(cpus) = parse_cfs_quota_contents(quota.trim(), period.trim()) {
            return Some(cpus);
        }
    }
    None
}

#[cfg(any(test, target_os = "linux"))]
fn parse_cpu_max_contents(contents: &str) -> Option<usize> {
    let mut parts = contents.split_whitespace();
    let quota = parts.next()?;
    let period: u64 = parts.next()?.parse().ok()?;
    if quota == "max" || period == 0 {
        return None;
    }
    let quota: u64 = quota.parse().ok()?;
    Some(cpu_count_from_quota_period(quota, period))
}

#[cfg(any(test, target_os = "linux"))]
fn parse_cfs_quota_contents(quota: &str, period: &str) -> Option<usize> {
    let quota: i64 = quota.parse().ok()?;
    let period: u64 = period.parse().ok()?;
    if quota < 0 || period == 0 {
        return None;
    }
    Some(cpu_count_from_quota_period(quota as u64, period))
}

#[cfg(any(test, target_os = "linux"))]
fn cpu_count_from_quota_period(quota: u64, period: u64) -> usize {
    quota.div_ceil(period).max(1) as usize
}

pub fn effective_runtime_parallelism() -> usize {
    let requested = configured_worker_threads().unwrap_or_else(available_parallelism);
    match cgroup_cpu_limit() {
        Some(cgroup_cpus) => requested.min(cgroup_cpus).max(1),
        None => requested.max(1),
    }
}

pub fn resolve_shard_count(configured: Option<usize>, max_shards: Option<usize>) -> usize {
    let parallelism = effective_runtime_parallelism().max(1);
    let mut resolved = configured.unwrap_or(parallelism).max(1);
    if let Some(limit) = max_shards.map(|value| value.max(1)) {
        resolved = resolved.min(limit);
    }
    resolved.max(1)
}

pub fn resolve_accept_workers(configured: Option<usize>) -> usize {
    configured
        .filter(|n| *n > 0)
        .unwrap_or_else(effective_runtime_parallelism)
        .max(1)
}

pub fn resolve_connection_count(configured: u32) -> u32 {
    if configured == 0 {
        effective_runtime_parallelism().max(1) as u32
    } else {
        configured.max(1)
    }
}

pub fn apply_worker_threads(builder: &mut tokio::runtime::Builder) {
    builder.worker_threads(effective_runtime_parallelism());
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
    use super::{
        cpu_count_from_quota_period, parse_cpu_max_contents, parse_cfs_quota_contents,
        parse_worker_threads, resolve_accept_workers, resolve_connection_count,
        effective_runtime_parallelism,
    };

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

    #[test]
    fn resolve_accept_workers_uses_explicit_value() {
        assert_eq!(resolve_accept_workers(Some(3)), 3);
    }

    #[test]
    fn resolve_accept_workers_ignores_zero() {
        assert_eq!(
            resolve_accept_workers(Some(0)),
            effective_runtime_parallelism().max(1)
        );
    }

    #[test]
    fn resolve_accept_workers_defaults_to_parallelism() {
        assert_eq!(
            resolve_accept_workers(None),
            effective_runtime_parallelism().max(1)
        );
    }

    #[test]
    fn resolve_connection_count_auto_uses_parallelism() {
        assert_eq!(
            resolve_connection_count(0),
            effective_runtime_parallelism().max(1) as u32
        );
    }

    #[test]
    fn resolve_connection_count_explicit() {
        assert_eq!(resolve_connection_count(5), 5);
    }

    #[test]
    fn parse_cpu_max_one_cpu() {
        assert_eq!(parse_cpu_max_contents("100000 100000"), Some(1));
    }

    #[test]
    fn parse_cpu_max_four_cpus() {
        assert_eq!(parse_cpu_max_contents("400000 100000"), Some(4));
    }

    #[test]
    fn parse_cpu_max_unlimited() {
        assert_eq!(parse_cpu_max_contents("max 100000"), None);
    }

    #[test]
    fn parse_cfs_quota_one_cpu() {
        assert_eq!(parse_cfs_quota_contents("100000", "100000"), Some(1));
    }

    #[test]
    fn cpu_count_rounds_up_partial_cpu() {
        assert_eq!(cpu_count_from_quota_period(150_000, 100_000), 2);
    }
}
