use parking_lot::Mutex;
use quinn::SendStream;
use std::cell::RefCell;
use std::sync::OnceLock;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

thread_local! {
    static LOCAL_POOL: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
}

fn global_pool() -> &'static Mutex<Vec<Vec<u8>>> {
    static GLOBAL_POOL: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
    GLOBAL_POOL.get_or_init(|| Mutex::new(Vec::new()))
}

fn take_buffer(buffer_size: usize) -> Vec<u8> {
    if let Some(mut buf) = LOCAL_POOL.with(|pool| pool.borrow_mut().pop()) {
        if buf.capacity() == buffer_size {
            if buf.len() != buffer_size {
                buf.resize(buffer_size, 0);
            }
            return buf;
        }
    }
    let mut global = global_pool().lock();
    if let Some(index) = global.iter().position(|buf| buf.capacity() == buffer_size) {
        let mut buf = global.swap_remove(index);
        if buf.len() != buffer_size {
            buf.resize(buffer_size, 0);
        }
        return buf;
    }
    vec![0; buffer_size]
}

fn return_buffer(buf: Vec<u8>, buffer_size: usize) {
    if buf.capacity() != buffer_size {
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
        let mut global = global_pool().lock();
        if global.len() < 256 {
            global.push(b);
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
    let mut buf = take_buffer(buffer_size);
    let mut copied = 0u64;
    let result = async {
        loop {
            let read = reader.read(&mut buf).await?;
            if read == 0 {
                break;
            }
            writer.write_all(&buf[..read]).await?;
            copied += read as u64;
        }
        Ok::<u64, std::io::Error>(copied)
    }
    .await;
    return_buffer(buf, buffer_size);
    result
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
