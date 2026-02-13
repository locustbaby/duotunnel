use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::Result;
use quinn::{SendStream, RecvStream};
use bytes::Bytes;

/// Adapts a pair of Quinn streams into a single AsyncRead + AsyncWrite object
pub struct QuinnStream {
    pub send: SendStream,
    pub recv: RecvStream,
}

impl AsyncRead for QuinnStream {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.recv).poll_read(cx, buf)
    }
}

impl AsyncWrite for QuinnStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.send).poll_write(cx, buf).map_err(std::io::Error::other)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.send).poll_flush(cx).map_err(std::io::Error::other)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.send).poll_shutdown(cx).map_err(std::io::Error::other)
    }
}

/// A wrapper that reads from a prefix buffer first, then the underlying stream.
/// Writes go directly to the underlying stream.
pub struct PrefixedReadWrite<S> {
    stream: S,
    prefix: Option<Bytes>,
}

impl<S> PrefixedReadWrite<S> {
    pub fn new(stream: S, prefix: Bytes) -> Self {
        Self {
            stream,
            prefix: if prefix.is_empty() { None } else { Some(prefix) },
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for PrefixedReadWrite<S> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        if let Some(prefix) = &mut self.prefix {
             let len = std::cmp::min(prefix.len(), buf.remaining());
             buf.put_slice(&prefix[..len]);
             
             // Advance the prefix slice
             if len == prefix.len() {
                 self.prefix = None;
             } else {
                 let remaining = prefix.split_off(len);
                 *prefix = remaining;
             }
             
             return Poll::Ready(Ok(()));
        }
        
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for PrefixedReadWrite<S> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
