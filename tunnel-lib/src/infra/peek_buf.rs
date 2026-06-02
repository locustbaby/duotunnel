use std::cell::RefCell;

thread_local! {
    static PEEK_BUF_POOL: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
}

/// Thread-local free-list for peek / read-ahead buffers.
///
/// Each tokio worker thread keeps its own pool — no lock, no cross-thread contention.
/// Callers `take()` a buffer sized to `buf_size`, use it for a single peek/read,
/// then `put()` it back so the allocation is reused on the next call.
///
/// `take()` guarantees returning a zeroed buffer, ensuring no stale memory is leaked.
pub struct PeekBufPool {
    buf_size: usize,
}

impl PeekBufPool {
    /// Max buffers kept per thread. With 8–16 worker threads this caps total idle
    /// RAM at 16 × 32 × 16 KiB ≈ 8 MiB — same order as the old global cap.
    const MAX_IDLE_PER_THREAD: usize = 32;

    pub fn new(buf_size: usize) -> Self {
        Self { buf_size }
    }

    /// Take a buffer of exactly `buf_size` bytes from the pool (or allocate fresh).
    ///
    /// The returned buffer is zero-initialized to prevent exposing stale data.
    pub fn take(&self) -> Vec<u8> {
        PEEK_BUF_POOL.with(|cell| {
            let buf = cell.borrow_mut().pop();
            match buf {
                Some(mut b) if b.capacity() >= self.buf_size => {
                    b.resize(self.buf_size, 0);
                    b
                }
                _ => vec![0u8; self.buf_size],
            }
        })
    }

    /// Return a buffer to the pool. Drops it if undersized or the pool is full.
    pub fn put(&self, mut buf: Vec<u8>) {
        if buf.capacity() < self.buf_size {
            return; // undersized — drop
        }
        // Reset length so the next take() can safely set_len without stale data.
        buf.truncate(0);
        PEEK_BUF_POOL.with(|cell| {
            let mut pool = cell.borrow_mut();
            if pool.len() < Self::MAX_IDLE_PER_THREAD {
                pool.push(buf);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_returns_zeroed_buffer() {
        let pool = PeekBufPool::new(10);

        // Take a buffer, it should be zeroed (or freshly allocated and zeroed)
        let mut buf = pool.take();
        assert_eq!(buf.len(), 10);
        assert!(buf.iter().all(|&b| b == 0));

        // Fill it with stale data
        buf.fill(0xFF);

        // Put it back to the pool
        pool.put(buf);

        // Take it again, we expect it to be zeroed out
        let buf = pool.take();
        assert_eq!(buf.len(), 10);
        // If it wasn't correctly zeroed (e.g. by resize), it would contain 0xFF.
        assert!(buf.iter().all(|&b| b == 0));
    }
}
