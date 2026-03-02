use super::{ProtocolDriver, ProxyRequest};
use crate::protocol::http_utils::{sanitize_request_headers, sanitize_response_headers};
use anyhow::{Context, Result};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use http::{HeaderMap, Method, Response, Uri, Version};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use quinn::{RecvStream, SendStream};

pub struct Http1Driver {
    send: SendStream,
    recv: Option<RecvStream>,
    scheme: String,
    authority: String,
    initial_bytes: Option<Bytes>,
}

impl Http1Driver {
    pub fn new(
        send: SendStream,
        recv: RecvStream,
        scheme: String,
        authority: String,
        initial_bytes: Option<Bytes>,
    ) -> Self {
        Self {
            send,
            recv: Some(recv),
            scheme,
            authority,
            initial_bytes,
        }
    }

    pub async fn finish(&mut self) -> Result<()> {
        self.send.finish()?;
        Ok(())
    }

    pub fn into_parts(mut self) -> (SendStream, Option<RecvStream>) {
        (self.send, self.recv.take())
    }
}

#[async_trait]
impl ProtocolDriver for Http1Driver {
    async fn read_request(&mut self) -> Result<Option<ProxyRequest>> {
        let mut recv = self.recv.take().context("RecvStream already consumed")?;

        let mut first_buf = vec![0u8; 8192];
        let mut pos = 0;

        if let Some(data) = self.initial_bytes.take() {
            let n = std::cmp::min(data.len(), first_buf.len());
            first_buf[..n].copy_from_slice(&data[..n]);
            pos = n;
        }

        loop {
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers);

            match req.parse(&first_buf[..pos]) {
                Ok(httparse::Status::Complete(_n)) => {
                    break;
                }
                Ok(httparse::Status::Partial) => {
                    if pos >= first_buf.len() {
                        return Err(anyhow::anyhow!("Headers exceeding buffer size"));
                    }

                    match recv.read(&mut first_buf[pos..]).await? {
                        Some(n) => pos += n,
                        None => {
                            if pos == 0 {
                                return Ok(None);
                            }
                            return Err(anyhow::anyhow!("Unexpected EOF in headers"));
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        let body_start = match req.parse(&first_buf[..pos])? {
            httparse::Status::Complete(n) => n,
            _ => unreachable!("Headers already validated as complete"),
        };

        let n = pos;

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
            _ => Method::GET,
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

        sanitize_request_headers(&mut header_map);

        let remaining_first = first_buf[body_start..n].to_vec();

        let content_length: usize = header_map
            .get(http::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let stream = futures_util::stream::try_unfold(
            (
                recv,
                Some(Bytes::from(remaining_first)),
                0usize,
                content_length,
            ),
            |(mut recv, first_chunk, mut read, total)| async move {
                if let Some(chunk) = first_chunk {
                    let len = chunk.len();
                    if len > 0 {
                        read += len;
                        return Ok(Some((
                            hyper::body::Frame::data(chunk),
                            (recv, None, read, total),
                        )));
                    }
                }

                if total > 0 && read >= total {
                    return Ok(None);
                }

                let mut buf = BytesMut::with_capacity(8192);
                // SAFETY: quinn fills the slice before returning n; we freeze
                // only [0..n] via copy_from_slice below.
                unsafe { buf.set_len(8192) };
                match recv.read(&mut buf[..]).await {
                    Ok(Some(n)) => {
                        read += n;
                        let data = Bytes::copy_from_slice(&buf[..n]);
                        Ok(Some((
                            hyper::body::Frame::data(data),
                            (recv, None, read, total),
                        )))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(std::io::Error::other(e)),
                }
            },
        );

        let body = if content_length > 0 {
            http_body_util::StreamBody::new(stream).boxed_unsync()
        } else {
            http_body_util::Empty::new()
                .map_err(|e| match e {})
                .boxed_unsync()
        };

        Ok(Some(ProxyRequest {
            method,
            uri,
            headers: header_map,
            version: Version::HTTP_11,
            body,
        }))
    }

    async fn write_response(&mut self, response: Response<Incoming>) -> Result<()> {
        // ── Response headers ─────────────────────────────────────────────
        // Batch all header lines into a single BytesMut and flush once,
        // instead of one write_all() per header (which can be 15-20 writes).
        let status = response.status();
        let sanitized_headers = sanitize_response_headers(response.headers());

        // Estimate capacity: status line (~30B) + avg 30B/header + fixed suffix
        let mut header_buf = BytesMut::with_capacity(
            32 + sanitized_headers.len() * 48 + 32,
        );

        // Status line
        use std::fmt::Write as FmtWrite;
        write!(
            header_buf,
            "HTTP/1.1 {} {}\r\n",
            status.as_u16(),
            status.canonical_reason().unwrap_or("OK")
        )
        .unwrap();

        for (name, value) in &sanitized_headers {
            header_buf.put_slice(name.as_str().as_bytes());
            header_buf.put_slice(b": ");
            header_buf.put_slice(value.as_bytes());
            header_buf.put_slice(b"\r\n");
        }
        header_buf.put_slice(b"transfer-encoding: chunked\r\n\r\n");

        self.send.write_all(&header_buf).await?;

        // ── Response body (chunked) ───────────────────────────────────────
        // Each chunk: "<hex-len>\r\n<data>\r\n"
        // We gather the three parts into a single buffer per chunk, cutting
        // two redundant write_all() calls per body frame.
        let mut body = response.into_body();
        let mut accumulated_trailers = HeaderMap::new();
        // Reusable chunk-framing buffer (avoids repeated allocation)
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
                Some(Err(e)) => return Err(anyhow::anyhow!("error reading response frame: {}", e)),
                None => break,
            }
        }

        // ── Chunked terminator + optional trailers ────────────────────────
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
