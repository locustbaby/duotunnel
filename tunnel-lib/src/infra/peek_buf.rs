#[cfg(feature = "buf-pool")]
use std::cell::RefCell;

#[cfg(feature = "buf-pool")]
thread_local! {
    static PEEK_BUF_POOL: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
}

/// Thread-local free-list for peek / read-ahead buffers.
///
/// Each tokio worker thread keeps its own pool — no lock, no cross-thread contention.
/// Callers `take()` a buffer sized to `buf_size`, use it for a single peek/read,
/// then `put()` it back so the allocation is reused on the next call.
#[derive(Clone, Copy)]
pub struct PeekBufPool {
    buf_size: usize,
}

impl PeekBufPool {
    /// Max buffers kept per thread. With 8–16 worker threads this caps total idle
    /// RAM at 16 × 32 × 16 KiB ≈ 8 MiB — same order as the old global cap.
    #[cfg(feature = "buf-pool")]
    const MAX_IDLE_PER_THREAD: usize = 32;

    pub fn new(buf_size: usize) -> Self {
        Self { buf_size }
    }

    /// Take a buffer of exactly `buf_size` bytes from the pool (or allocate fresh).
    pub fn take(&self) -> Vec<u8> {
        #[cfg(feature = "buf-pool")]
        {
            PEEK_BUF_POOL.with(|cell| {
                let buf = cell.borrow_mut().pop();
                match buf {
                    Some(mut b) if b.capacity() >= self.buf_size => {
                        b.truncate(self.buf_size);
                        if b.len() < self.buf_size {
                            b.resize(self.buf_size, 0);
                        }
                        b
                    }
                    _ => vec![0u8; self.buf_size],
                }
            })
        }
        #[cfg(not(feature = "buf-pool"))]
        {
            vec![0u8; self.buf_size]
        }
    }

    /// Return a buffer to the pool. Drops it if undersized or the pool is full.
    pub fn put(&self, mut _buf: Vec<u8>) {
        #[cfg(feature = "buf-pool")]
        {
            if _buf.capacity() < self.buf_size {
                return; // undersized — drop
            }
            // Maintain the length up to buf_size so we don't have to resize it later.
            _buf.truncate(self.buf_size);
            if _buf.len() < self.buf_size {
                _buf.resize(self.buf_size, 0);
            }
            PEEK_BUF_POOL.with(|cell| {
                let mut pool = cell.borrow_mut();
                if pool.len() < Self::MAX_IDLE_PER_THREAD {
                    pool.push(_buf);
                }
            });
        }
    }
}
