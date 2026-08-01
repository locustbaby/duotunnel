use crate::SniffPrefix;
use quinn::{RecvStream, SendStream};
use std::io::{IoSlice, Result};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct QuinnStream {
    pub send: SendStream,
    pub recv: RecvStream,
}
impl AsyncRead for QuinnStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.recv).poll_read(cx, buf)
    }
}
impl AsyncWrite for QuinnStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.send)
            .poll_write(cx, buf)
            .map_err(std::io::Error::other)
    }
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.send)
            .poll_write_vectored(cx, bufs)
            .map_err(std::io::Error::other)
    }
    fn is_write_vectored(&self) -> bool {
        self.send.is_write_vectored()
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.send)
            .poll_flush(cx)
            .map_err(std::io::Error::other)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.send)
            .poll_shutdown(cx)
            .map_err(std::io::Error::other)
    }
}
pub struct PrefixedReadWrite<S> {
    stream: S,
    prefix: SniffPrefix,
}
impl<S> PrefixedReadWrite<S> {
    pub fn new(stream: S, prefix: impl Into<SniffPrefix>) -> Self {
        Self {
            stream,
            prefix: prefix.into(),
        }
    }
}
impl<S: AsyncRead + Unpin> AsyncRead for PrefixedReadWrite<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        if !self.prefix.is_empty() {
            let len = std::cmp::min(self.prefix.len(), buf.remaining());
            buf.put_slice(&self.prefix.as_bytes()[..len]);
            self.prefix.advance(len);
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}
impl<S: AsyncWrite + Unpin> AsyncWrite for PrefixedReadWrite<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.stream).poll_write_vectored(cx, bufs)
    }
    fn is_write_vectored(&self) -> bool {
        self.stream.is_write_vectored()
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
pub struct PrefixedReadHalf<R> {
    reader: R,
    prefix: SniffPrefix,
}
impl<R: AsyncRead + Unpin> AsyncRead for PrefixedReadHalf<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        if !self.prefix.is_empty() {
            let len = std::cmp::min(self.prefix.len(), buf.remaining());
            buf.put_slice(&self.prefix.as_bytes()[..len]);
            self.prefix.advance(len);
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}
impl PrefixedReadWrite<tokio::net::TcpStream> {
    pub fn into_split(
        self,
    ) -> (
        PrefixedReadHalf<tokio::net::tcp::OwnedReadHalf>,
        tokio::net::tcp::OwnedWriteHalf,
    ) {
        let (r, w) = self.stream.into_split();
        (
            PrefixedReadHalf {
                reader: r,
                prefix: self.prefix,
            },
            w,
        )
    }
}
