use crossbeam_queue::ArrayQueue;
use quinn::{RecvStream, SendStream};
use std::cell::RefCell;
use std::sync::OnceLock;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

thread_local! {
    static LOCAL_POOL: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
}

fn global_pool() -> &'static ArrayQueue<Vec<u8>> {
    static GLOBAL_POOL: OnceLock<ArrayQueue<Vec<u8>>> = OnceLock::new();
    GLOBAL_POOL.get_or_init(|| ArrayQueue::new(1024))
}

fn take_buffer(buffer_size: usize) -> Vec<u8> {
    if let Some(mut buf) = LOCAL_POOL.with(|pool| pool.borrow_mut().pop()) {
        if buf.capacity() >= buffer_size {
            // SAFETY: The caller of `take_buffer` must immediately overwrite all bytes up to `buffer_size`
            // before reading them (e.g. by passing the slice to `reader.read(&mut buf)`).
            unsafe {
                buf.set_len(buffer_size);
            }
            return buf;
        }
    }
    let global = global_pool();
    while let Some(mut buf) = global.pop() {
        if buf.capacity() >= buffer_size {
            // SAFETY: The caller of `take_buffer` must immediately overwrite all bytes up to `buffer_size`
            // before reading them (e.g. by passing the slice to `reader.read(&mut buf)`).
            unsafe {
                buf.set_len(buffer_size);
            }
            return buf;
        }
    }
    let mut buf = Vec::with_capacity(buffer_size);
    unsafe {
        buf.set_len(buffer_size);
    }
    buf
}

fn return_buffer(buf: Vec<u8>, buffer_size: usize) {
    if buf.capacity() < buffer_size {
        return;
    }
    let mut maybe_buf = Some(buf);
    let stored_locally = LOCAL_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < 8 {
            if let Some(b) = maybe_buf.take() {
                pool.push(b);
            }
            true
        } else {
            false
        }
    });
    if stored_locally {
        return;
    }
    if let Some(b) = maybe_buf {
        let global = global_pool();
        let _ = global.push(b);
    }
}

pub struct PooledBufGuard {
    buf: Option<Vec<u8>>,
    expected_size: usize,
}

impl PooledBufGuard {
    pub fn new(buf: Vec<u8>, expected_size: usize) -> Self {
        Self {
            buf: Some(buf),
            expected_size,
        }
    }
}

impl std::ops::Deref for PooledBufGuard {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.buf.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledBufGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut().unwrap()
    }
}

impl Drop for PooledBufGuard {
    fn drop(&mut self) {
        if let Some(buf) = self.buf.take() {
            return_buffer(buf, self.expected_size);
        }
    }
}

async fn copy_buffered<R, W>(
    mut reader: R,
    mut writer: W,
    buffer_size: usize,
) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let buf = take_buffer(buffer_size);
    let mut guard = PooledBufGuard::new(buf, buffer_size);
    let mut copied = 0u64;
    loop {
        // SAFETY:
        // 1. The buffer's length was set to `buffer_size` using `set_len` in `take_buffer`.
        // 2. The memory in the buffer might be uninitialized.
        // 3. We immediately pass the mutable slice `&mut *guard` to `reader.read`,
        //    which will overwrite the bytes.
        // 4. We only read or write the slice of the buffer that has been successfully
        //    written to by the read operation: `&guard[..read]`.
        // 5. No uninitialized bytes are ever read.
        let read = reader.read(&mut *guard).await?;
        if read == 0 {
            break;
        }
        writer.write_all(&guard[..read]).await?;
        copied += read as u64;
    }
    Ok(copied)
}

pub async fn copy_buffered_then_shutdown<R, W>(
    reader: R,
    mut writer: W,
    buffer_size: usize,
) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let bytes = copy_buffered(reader, &mut writer, buffer_size).await?;
    let _ = writer.shutdown().await;
    Ok(bytes)
}

pub async fn copy_buffered_then_finish<R>(
    reader: R,
    mut writer: SendStream,
    buffer_size: usize,
) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin,
{
    let bytes = copy_buffered(reader, &mut writer, buffer_size).await?;
    if let Err(error) = writer.finish() {
        tracing::warn!(?error, "failed to finish quic send stream");
    }
    Ok(bytes)
}

pub async fn copy_quic_to_shutdown<W>(mut recv: RecvStream, mut writer: W) -> std::io::Result<u64>
where
    W: AsyncWrite + Unpin,
{
    let mut copied = 0u64;
    loop {
        match recv.read_chunk(usize::MAX, true).await {
            Ok(Some(chunk)) => {
                writer.write_all(&chunk.bytes).await?;
                copied += chunk.bytes.len() as u64;
            }
            Ok(None) => break,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
    let _ = writer.shutdown().await;
    Ok(copied)
}
