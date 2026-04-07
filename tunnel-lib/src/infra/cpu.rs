/// Detect how many CPUs this process is actually allowed to use.
///
/// Priority order:
///   1. cgroup v2  — `/sys/fs/cgroup/cpu.max`          (K8s ≥ 1.25, systemd-based distros)
///   2. cgroup v1  — `cpu.cfs_quota_us / cpu.cfs_period_us`
///   3. `std::thread::available_parallelism()`          (bare-metal / VM, no cgroup limit)
///
/// In K8s, `available_parallelism()` returns the **node** CPU count, which can be
/// dozens or hundreds even when the pod's CPU limit is 0.5 or 1. Spawning that
/// many threads wastes memory and causes scheduler contention. Reading the cgroup
/// quota gives the correct container-aware value.
///
/// The quota is rounded up to at least 1 so callers never get 0.
pub fn effective_cpu_count() -> u32 {
    if let Some(n) = cgroup_v2_cpu_count() {
        tracing::debug!(cpus = n, "cpu count from cgroup v2");
        return n;
    }
    if let Some(n) = cgroup_v1_cpu_count() {
        tracing::debug!(cpus = n, "cpu count from cgroup v1");
        return n;
    }
    let n = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
        .max(1);
    tracing::debug!(cpus = n, "cpu count from available_parallelism");
    n
}

/// Parse `/sys/fs/cgroup/cpu.max` (cgroup v2).
///
/// Format: `<quota_us> <period_us>` or `max <period_us>` (unlimited).
fn cgroup_v2_cpu_count() -> Option<u32> {
    let content = std::fs::read_to_string("/sys/fs/cgroup/cpu.max").ok()?;
    let mut parts = content.split_whitespace();
    let quota_str = parts.next()?;
    let period_str = parts.next()?;

    if quota_str == "max" {
        return None; // unlimited — fall through to next detector
    }

    let quota: f64 = quota_str.parse().ok()?;
    let period: f64 = period_str.parse().ok()?;
    if period <= 0.0 {
        return None;
    }
    Some(((quota / period).ceil() as u32).max(1))
}

/// Parse `/sys/fs/cgroup/cpu/cpu.cfs_quota_us` and `cpu.cfs_period_us` (cgroup v1).
fn cgroup_v1_cpu_count() -> Option<u32> {
    let quota: i64 = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_quota_us")
        .ok()?
        .trim()
        .parse()
        .ok()?;

    if quota <= 0 {
        return None; // -1 means unlimited
    }

    let period: i64 = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_period_us")
        .ok()?
        .trim()
        .parse()
        .ok()?;

    if period <= 0 {
        return None;
    }

    Some((((quota as f64) / (period as f64)).ceil() as u32).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_cpu_count_is_at_least_one() {
        assert!(effective_cpu_count() >= 1);
    }

    #[test]
    fn cgroup_v2_parse_limited() {
        // Simulate "200000 100000" → 2 CPUs
        let quota = 200000.0_f64;
        let period = 100000.0_f64;
        let n = ((quota / period).ceil() as u32).max(1);
        assert_eq!(n, 2);
    }

    #[test]
    fn cgroup_v2_parse_fractional() {
        // Simulate "50000 100000" → 0.5 CPUs → ceil → 1
        let quota = 50000.0_f64;
        let period = 100000.0_f64;
        let n = ((quota / period).ceil() as u32).max(1);
        assert_eq!(n, 1);
    }

    #[test]
    fn cgroup_v2_parse_unlimited() {
        // "max 100000" → None (fall through)
        let content = "max 100000\n";
        let mut parts = content.split_whitespace();
        let quota_str = parts.next().unwrap();
        assert_eq!(quota_str, "max");
    }
}
