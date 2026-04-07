use std::sync::atomic::{AtomicUsize, Ordering};

/// Trait for pluggable load-balancing algorithms.
///
/// Implementations select an index into a caller-supplied slice of `n` items.
/// Returning `None` means "no item available" (e.g. empty pool). The caller is
/// responsible for any health filtering before calling `pick_index`.
///
/// Implementations must be `Send + Sync` so they can live inside an `Arc`.
pub trait Balancer: Send + Sync {
    /// Pick an index in `0..n`, or `None` if `n == 0`.
    fn pick_index(&self, n: usize) -> Option<usize>;
}

/// Atomic round-robin balancer.
///
/// Uses a single lock-free counter. The counter wraps at `usize::MAX + 1`; the
/// modulo keeps the distribution uniform across restarts.
pub struct RoundRobin {
    cursor: AtomicUsize,
}

impl RoundRobin {
    pub fn new() -> Self {
        Self {
            cursor: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobin {
    fn default() -> Self {
        Self::new()
    }
}

impl Balancer for RoundRobin {
    fn pick_index(&self, n: usize) -> Option<usize> {
        if n == 0 {
            return None;
        }
        let raw = self.cursor.fetch_add(1, Ordering::Relaxed);
        let idx = if n.is_power_of_two() {
            raw & (n - 1)
        } else {
            raw % n
        };
        Some(idx)
    }
}

/// Convenience extension: pick directly from a slice.
pub trait BalancerExt: Balancer {
    fn pick<'a, T>(&self, items: &'a [T]) -> Option<&'a T> {
        self.pick_index(items.len()).and_then(|i| items.get(i))
    }
}

impl<B: Balancer + ?Sized> BalancerExt for B {}
