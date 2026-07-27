use bytes::BytesMut;
use crossbeam_queue::ArrayQueue;
use quinn::{RecvStream, SendStream};
use std::cell::RefCell;
use std::sync::OnceLock;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

thread_local! {
    static LOCAL_POOL: RefCell<Vec<BytesMut>> = const { RefCell::new(Vec::new()) };
}

fn global_pool() -> &'static ArrayQueue<BytesMut> {
    static GLOBAL_POOL: OnceLock<ArrayQueue<BytesMut>> = OnceLock::new();
    GLOBAL_POOL.get_or_init(|| ArrayQueue::new(1024))
}

// Buffers are handed out empty (len 0) with at least `buffer_size` capacity;
// `read_buf` fills the uninitialized capacity directly, so no zeroing and no
// `set_len` over uninitialized memory is ever needed.
fn take_buffer(buffer_size: usize) -> BytesMut {
    if let Some(buf) = LOCAL_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        pool.iter()
            .position(|buf| buf.capacity() >= buffer_size)
            .map(|index| pool.swap_remove(index))
    }) {
        return buf;
    }
    let global = global_pool();
    for _ in 0..3 {
        if let Some(buf) = global.pop() {
            if buf.capacity() >= buffer_size {
                return buf;
            }
            // Discard too-small buffers to avoid queue pollution and busy CAS loop
        } else {
            break;
        }
    }
    BytesMut::with_capacity(buffer_size)
}

fn return_buffer(mut buf: BytesMut, buffer_size: usize) {
    buf.clear();
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
            if let Some(ref returned_buf) = maybe_buf {
                let returned_cap = returned_buf.capacity();
                if let Some((index, _)) = pool.iter()
                    .enumerate()
                    .min_by_key(|(_, b)| b.capacity())
                {
                    if pool[index].capacity() < returned_cap {
                        let _old_small_buf = std::mem::replace(&mut pool[index], maybe_buf.take().unwrap());
                        return true;
                    }
                }
            }
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
    buf: Option<BytesMut>,
    expected_size: usize,
}

impl PooledBufGuard {
    pub fn new(buf: BytesMut, expected_size: usize) -> Self {
        Self {
            buf: Some(buf),
            expected_size,
        }
    }
}

impl std::ops::Deref for PooledBufGuard {
    type Target = BytesMut;
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
        guard.clear();
        let read = reader.read_buf(&mut *guard).await?;
        if read == 0 {
            break;
        }
        writer.write_all(&guard[..]).await?;
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
            Err(e) => return Err(std::io::Error::other(e)),
        }
    }
    let _ = writer.shutdown().await;
    Ok(copied)
}
