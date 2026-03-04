use super::{ProtocolDriver, ProxyRequest};
use crate::protocol::http_utils::{sanitize_request_headers, sanitize_response_headers};
use anyhow::{Context, Result};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use http::{HeaderMap, Method, Response, Uri, Version};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use quinn::{RecvStream, SendStream};
use tokio::sync::oneshot;

/// Residual state returned by the body stream after it finishes consuming
/// the request body.  Contains the `RecvStream` and any over-read bytes
/// that belong to the next request.
struct Reclaim {
    recv: RecvStream,
    overflow: Bytes,
}

pub struct Http1Driver {
    send: SendStream,
    /// `None` while a body stream holds the recv.
    recv: Option<RecvStream>,
    scheme: String,
    authority: String,
    /// Persistent read buffer — survives across requests.  May contain
    /// leftover bytes from a previous read that belong to the next request.
    read_buf: BytesMut,
    /// Channel to reclaim `recv` + overflow from an in-flight body stream.
    inflight_reclaim: Option<oneshot::Receiver<Reclaim>>,
    /// Whether the connection should close after the current request.
    pub should_close: bool,
}

impl Http1Driver {
    pub fn new(
        send: SendStream,
        recv: RecvStream,
        scheme: String,
        authority: String,
        initial_bytes: Option<Bytes>,
    ) -> Self {
        let mut read_buf = BytesMut::with_capacity(8192);
        if let Some(data) = initial_bytes {
            read_buf.extend_from_slice(&data);
        }
        Self {
            send,
            recv: Some(recv),
            scheme,
            authority,
            read_buf,
            inflight_reclaim: None,
            should_close: false,
        }
    }

    pub async fn finish(&mut self) -> Result<()> {
        self.send.finish()?;
        Ok(())
    }

    /// Ensure `self.recv` is available.  If a body stream is still holding
    /// it, wait for the stream to finish and reclaim the recv + overflow.
    async fn reclaim_recv(&mut self) -> Result<()> {
        if self.recv.is_some() {
            return Ok(());
        }
        let rx = self
            .inflight_reclaim
            .take()
            .context("recv lost: no reclaim channel")?;

        let reclaim = rx
            .await
            .map_err(|_| anyhow::anyhow!("body stream dropped without returning recv"))?;

        self.recv = Some(reclaim.recv);
        if !reclaim.overflow.is_empty() {
            // Prepend overflow so the next read_request sees it first.
            let mut new_buf =
                BytesMut::with_capacity(reclaim.overflow.len() + self.read_buf.len());
            new_buf.extend_from_slice(&reclaim.overflow);
            new_buf.extend_from_slice(&self.read_buf);
            self.read_buf = new_buf;
        }
        Ok(())
    }
}

#[async_trait]
impl ProtocolDriver for Http1Driver {
    async fn read_request(&mut self) -> Result<Option<ProxyRequest>> {
        // ── 1. Reclaim recv from previous body stream if needed ──────
        self.reclaim_recv().await?;
        self.should_close = false;

        let mut recv = self
            .recv
            .take()
            .context("RecvStream missing after reclaim")?;

        // ── 2. Read until we have a complete header block ────────────
        loop {
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers);

            match req.parse(&self.read_buf) {
                Ok(httparse::Status::Complete(_)) => break,
                Ok(httparse::Status::Partial) => {
                    if self.read_buf.len() >= 8192 {
                        return Err(anyhow::anyhow!("Headers exceeding buffer size"));
                    }
                    let old_len = self.read_buf.len();
                    let spare = 8192 - old_len;
                    self.read_buf.resize(old_len + spare, 0);
                    match recv.read(&mut self.read_buf[old_len..]).await? {
                        Some(n) => self.read_buf.truncate(old_len + n),
                        None => {
                            self.read_buf.truncate(old_len);
                            if old_len == 0 {
                                // Clean EOF — connection closed.
                                self.recv = Some(recv);
                                return Ok(None);
                            }
                            return Err(anyhow::anyhow!("Unexpected EOF in headers"));
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        // ── 3. Parse the complete headers ────────────────────────────
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        let header_end = match req.parse(&self.read_buf)? {
            httparse::Status::Complete(n) => n,
            _ => unreachable!("Headers already validated as complete"),
        };

        let method_str = req.method.context("no method")?;
        let path = req.path.context("no path")?;

        let uri_str = format!("{}://{}{}", self.scheme, self.authority, path);
        let uri: Uri = uri_str.parse()?;

        let method = match method_str {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "PATCH" => Method::PATCH,
            _ => Method::from_bytes(method_str.as_bytes())
                .map_err(|_| anyhow::anyhow!("unsupported method: {}", method_str))?,
        };

        let mut header_map = HeaderMap::new();
        for h in req.headers.iter() {
            if !h.name.is_empty() {
                if let (Ok(name), Ok(value)) = (
                    http::header::HeaderName::from_bytes(h.name.as_bytes()),
                    http::header::HeaderValue::from_bytes(h.value),
                ) {
                    header_map.insert(name, value);
                }
            }
        }

        // ── 4. Determine keep-alive BEFORE sanitize removes Connection ──
        let http_minor = req.version.unwrap_or(1);

        let conn_header = header_map
            .get(http::header::CONNECTION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_ascii_lowercase());

        self.should_close = match http_minor {
            // HTTP/1.1: keep-alive by default, close only if explicit
            1 => conn_header.as_deref() == Some("close"),
            // HTTP/1.0: close by default, keep-alive only if explicit
            _ => conn_header.as_deref() != Some("keep-alive"),
        };

        let content_length: usize = header_map
            .get(http::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        sanitize_request_headers(&mut header_map);

        // ── 5. Split read_buf: body prefix vs next-request overflow ──
        // Consume header bytes, keep the rest.
        let _ = self.read_buf.split_to(header_end);

        let available = self.read_buf.len();
        let body_prefix_len = available.min(content_length);
        let body_prefix = Bytes::copy_from_slice(&self.read_buf[..body_prefix_len]);
        // Bytes beyond body belong to the next request — keep in read_buf.
        let _ = self.read_buf.split_to(body_prefix_len);

        let body_remaining = content_length.saturating_sub(body_prefix_len);

        // ── 6. Build the body stream ─────────────────────────────────
        let body = if content_length == 0 {
            // No body — recv stays with driver.
            self.recv = Some(recv);
            http_body_util::Empty::new()
                .map_err(|e| match e {})
                .boxed_unsync()
        } else if body_remaining == 0 {
            // The prefix already contains the entire body.  Reclaim recv
            // eagerly so it is returned even if hyper drops the body stream
            // without polling to completion.
            self.recv = Some(recv);

            let prefix = body_prefix;
            let stream = futures_util::stream::try_unfold(
                Some(prefix),
                |mut state| async move {
                    if let Some(data) = state.take() {
                        if !data.is_empty() {
                            return Ok(Some((hyper::body::Frame::data(data), None)));
                        }
                    }
                    Ok(None)
                },
            );

            http_body_util::StreamBody::new(stream).boxed_unsync()
        } else {
            // Body stream temporarily owns recv.
            let (reclaim_tx, reclaim_rx) = oneshot::channel::<Reclaim>();
            self.inflight_reclaim = Some(reclaim_rx);

            let stream = futures_util::stream::try_unfold(
                BodyState {
                    recv: Some(recv),
                    body_prefix: Some(body_prefix),
                    read: 0,
                    remaining: body_remaining,
                    reclaim_tx: Some(reclaim_tx),
                },
                |mut state| async move {
                    // Yield the prefix chunk first
                    if let Some(prefix) = state.body_prefix.take() {
                        if !prefix.is_empty() {
                            state.read += prefix.len();
                            return Ok(Some((hyper::body::Frame::data(prefix), state)));
                        }
                    }

                    // All body bytes consumed — reclaim recv.
                    if state.remaining == 0 {
                        if let (Some(tx), Some(recv)) =
                            (state.reclaim_tx.take(), state.recv.take())
                        {
                            let _ = tx.send(Reclaim {
                                recv,
                                overflow: Bytes::new(),
                            });
                        }
                        return Ok(None);
                    }

                    let recv = match state.recv.as_mut() {
                        Some(r) => r,
                        None => return Ok(None),
                    };

                    // Read more body from recv
                    let to_read = state.remaining.min(8192);
                    let mut buf = BytesMut::zeroed(to_read);
                    match recv.read(&mut buf[..]).await {
                        Ok(Some(n)) => {
                            let got = n.min(state.remaining);
                            state.read += got;
                            state.remaining -= got;

                            let data = Bytes::copy_from_slice(&buf[..got]);

                            // Over-read bytes belong to the next request.
                            let overflow = if n > got {
                                Bytes::copy_from_slice(&buf[got..n])
                            } else {
                                Bytes::new()
                            };

                            // If body done, reclaim immediately.
                            if state.remaining == 0 {
                                if let (Some(tx), Some(recv)) =
                                    (state.reclaim_tx.take(), state.recv.take())
                                {
                                    let _ = tx.send(Reclaim { recv, overflow });
                                }
                            }

                            Ok(Some((hyper::body::Frame::data(data), state)))
                        }
                        Ok(None) => {
                            // Unexpected EOF in body — connection broken.
                            Ok(None)
                        }
                        Err(e) => Err(std::io::Error::other(e)),
                    }
                },
            );

            http_body_util::StreamBody::new(stream).boxed_unsync()
        };

        Ok(Some(ProxyRequest {
            method,
            uri,
            headers: header_map,
            version: Version::HTTP_11,
            body,
        }))
    }

    async fn write_response(&mut self, mut response: Response<Incoming>) -> Result<()> {
        let status = response.status();

        // Check response Connection header for close signal.
        if let Some(conn) = response.headers().get(http::header::CONNECTION) {
            if let Ok(v) = conn.to_str() {
                if v.eq_ignore_ascii_case("close") {
                    self.should_close = true;
                }
            }
        }

        sanitize_response_headers(response.headers_mut());

        let headers = response.headers();
        let mut header_buf = BytesMut::with_capacity(32 + headers.len() * 48 + 32);

        use std::fmt::Write as FmtWrite;
        write!(
            header_buf,
            "HTTP/1.1 {} {}\r\n",
            status.as_u16(),
            status.canonical_reason().unwrap_or("OK")
        )
        .unwrap();

        for (name, value) in headers {
            header_buf.put_slice(name.as_str().as_bytes());
            header_buf.put_slice(b": ");
            header_buf.put_slice(value.as_bytes());
            header_buf.put_slice(b"\r\n");
        }
        header_buf.put_slice(b"transfer-encoding: chunked\r\n\r\n");

        self.send.write_all(&header_buf).await?;

        let mut body = response.into_body();
        let mut accumulated_trailers = HeaderMap::new();
        let mut chunk_buf = BytesMut::with_capacity(8192 + 24);

        loop {
            match body.frame().await {
                Some(Ok(frame)) => {
                    if let Some(chunk) = frame.data_ref() {
                        if !chunk.is_empty() {
                            chunk_buf.clear();
                            write!(chunk_buf, "{:x}\r\n", chunk.len()).unwrap();
                            chunk_buf.put_slice(chunk);
                            chunk_buf.put_slice(b"\r\n");
                            self.send.write_all(&chunk_buf).await?;
                        }
                    }
                    if let Ok(trailers) = frame.into_trailers() {
                        accumulated_trailers.extend(trailers);
                    }
                }
                Some(Err(e)) => {
                    return Err(anyhow::anyhow!("error reading response frame: {}", e))
                }
                None => break,
            }
        }

        if accumulated_trailers.is_empty() {
            self.send.write_all(b"0\r\n\r\n").await?;
        } else {
            let mut tail = BytesMut::with_capacity(8 + accumulated_trailers.len() * 48);
            tail.put_slice(b"0\r\n");
            for (name, value) in &accumulated_trailers {
                tail.put_slice(name.as_str().as_bytes());
                tail.put_slice(b": ");
                tail.put_slice(value.as_bytes());
                tail.put_slice(b"\r\n");
            }
            tail.put_slice(b"\r\n");
            self.send.write_all(&tail).await?;
        }

        Ok(())
    }
}

/// Internal state for the body-reading stream.
struct BodyState {
    recv: Option<RecvStream>,
    body_prefix: Option<Bytes>,
    read: usize,
    remaining: usize,
    reclaim_tx: Option<oneshot::Sender<Reclaim>>,
}
