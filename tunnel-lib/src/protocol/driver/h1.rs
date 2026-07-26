use super::{ProtocolDriver, ProxyRequest};
use crate::protocol::http_utils::{sanitize_request_headers, sanitize_response_headers};
use anyhow::{Context, Result};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use http::{HeaderMap, Method, Response, Uri, Version};
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use quinn::{RecvStream, SendStream};
use tokio::sync::oneshot;
struct Reclaim {
    recv: RecvStream,
    overflow: Bytes,
}
pub struct Http1Driver {
    send: SendStream,
    recv: Option<RecvStream>,
    scheme: String,
    authority: String,
    read_buf: BytesMut,
    inflight_reclaim: Option<oneshot::Receiver<Reclaim>>,
    pub should_close: bool,
    last_method: Option<Method>,
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
            last_method: None,
        }
    }
    pub async fn finish(&mut self) -> Result<()> {
        self.send.finish()?;
        Ok(())
    }
    pub async fn write_502(&mut self) -> Result<()> {
        const RESP: &[u8] =
            b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        self.send.write_all(RESP).await?;
        Ok(())
    }
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
            if self.read_buf.is_empty() {
                self.read_buf = BytesMut::from(reclaim.overflow.as_ref());
            } else {
                let mut prepend = BytesMut::from(reclaim.overflow.as_ref());
                prepend.extend_from_slice(&self.read_buf);
                self.read_buf = prepend;
            }
        }
        Ok(())
    }
}

async fn read_into_bytes_mut(
    recv: &mut RecvStream,
    dst: &mut BytesMut,
    max_read: usize,
) -> std::result::Result<Option<usize>, quinn::ReadError> {
    match recv.read_chunk(max_read, true).await? {
        Some(chunk) => {
            dst.extend_from_slice(&chunk.bytes);
            Ok(Some(chunk.bytes.len()))
        }
        None => Ok(None),
    }
}
struct ParsedHead {
    header_end: usize,
    method: Method,
    uri: Uri,
    version: u8,
    header_map: HeaderMap,
    content_length: usize,
}

const RESP_400: &[u8] =
    b"HTTP/1.1 400 Bad Request\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
const RESP_411: &[u8] =
    b"HTTP/1.1 411 Length Required\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";

enum FramingDecision {
    Accept { content_length: usize },
    Reject {
        resp: &'static [u8],
        reason: &'static str,
    },
}

/// RFC 9112 §6.3 request-smuggling defenses. This driver frames request bodies
/// by content-length only, so a transfer-encoded body would otherwise be parsed
/// as the next request on the stream — reject it outright (411 tells the client
/// to resend with a content-length).
fn validate_framing(headers: &[httparse::Header<'_>]) -> FramingDecision {
    let mut has_te = false;
    let mut cl: Option<&[u8]> = None;
    for h in headers {
        if h.name.eq_ignore_ascii_case("transfer-encoding") {
            has_te = true;
        } else if h.name.eq_ignore_ascii_case("content-length") {
            match cl {
                Some(prev) if prev != h.value => {
                    return FramingDecision::Reject {
                        resp: RESP_400,
                        reason: "conflicting content-length headers",
                    };
                }
                _ => cl = Some(h.value),
            }
        }
    }
    if has_te {
        return if cl.is_some() {
            FramingDecision::Reject {
                resp: RESP_400,
                reason: "content-length with transfer-encoding",
            }
        } else {
            FramingDecision::Reject {
                resp: RESP_411,
                reason: "transfer-encoding request body unsupported",
            }
        };
    }
    let content_length = match cl {
        None => 0,
        Some(v) => match std::str::from_utf8(v)
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
        {
            Some(n) => n,
            None => {
                return FramingDecision::Reject {
                    resp: RESP_400,
                    reason: "invalid content-length",
                }
            }
        },
    };
    FramingDecision::Accept { content_length }
}
#[async_trait]
impl ProtocolDriver for Http1Driver {
    async fn read_request(&mut self) -> Result<Option<ProxyRequest>> {
        self.reclaim_recv().await?;
        self.should_close = false;
        let mut recv = self
            .recv
            .take()
            .context("RecvStream missing after reclaim")?;
        let parsed = loop {
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers);
            match req.parse(&self.read_buf) {
                Ok(httparse::Status::Complete(n)) => {
                    let content_length = match validate_framing(req.headers) {
                        FramingDecision::Reject { resp, reason } => break Err((resp, reason)),
                        FramingDecision::Accept { content_length } => content_length,
                    };
                    let method_str = req.method.context("no method")?;
                    let path = req.path.context("no path")?;
                    let uri_str = format!("{}://{}{}", self.scheme, self.authority, path);
                    let uri: Uri = uri_str.parse()?;
                    let method = Method::from_bytes(method_str.as_bytes())
                        .map_err(|_| anyhow::anyhow!("unsupported method: {}", method_str))?;
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
                    break Ok(ParsedHead {
                        header_end: n,
                        method,
                        uri,
                        version: req.version.unwrap_or(1),
                        header_map,
                        content_length,
                    });
                }
                Ok(httparse::Status::Partial) => {
                    if self.read_buf.len() >= 8192 {
                        return Err(anyhow::anyhow!("Headers exceeding buffer size"));
                    }
                    let old_len = self.read_buf.len();
                    let spare = 8192 - old_len;
                    match read_into_bytes_mut(&mut recv, &mut self.read_buf, spare).await? {
                        Some(_) => {}
                        None => {
                            if old_len == 0 {
                                self.recv = Some(recv);
                                return Ok(None);
                            }
                            return Err(anyhow::anyhow!("Unexpected EOF in headers"));
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        };
        let parsed = match parsed {
            Ok(parsed) => parsed,
            Err((resp, reason)) => {
                self.should_close = true;
                let _ = self.send.write_all(resp).await;
                let _ = self.send.finish();
                return Err(anyhow::anyhow!("rejected request: {}", reason));
            }
        };
        let header_end = parsed.header_end;
        let method = parsed.method;
        let uri = parsed.uri;
        let http_minor = parsed.version;
        let mut header_map = parsed.header_map;
        let conn_header = header_map
            .get(http::header::CONNECTION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_ascii_lowercase());
        self.should_close = match http_minor {
            1 => conn_header.as_deref() == Some("close"),
            _ => conn_header.as_deref() != Some("keep-alive"),
        };
        let content_length = parsed.content_length;
        self.last_method = Some(method.clone());
        if let Some(expect) = header_map.remove(http::header::EXPECT) {
            // Answer the interim response locally: the body is streamed to the
            // upstream unconditionally, so waiting for the upstream's own 100
            // would only stall the client until its timeout.
            if http_minor == 1 && expect.as_bytes().eq_ignore_ascii_case(b"100-continue") {
                self.send.write_all(b"HTTP/1.1 100 Continue\r\n\r\n").await?;
            }
        }
        sanitize_request_headers(&mut header_map);
        let _ = self.read_buf.split_to(header_end);
        let available = self.read_buf.len();
        let body_prefix_len = available.min(content_length);
        let body_prefix = self.read_buf.split_to(body_prefix_len).freeze();
        let body_remaining = content_length.saturating_sub(body_prefix_len);
        let body = if content_length == 0 {
            self.recv = Some(recv);
            http_body_util::Empty::new().map_err(|e| match e {}).boxed()
        } else if body_remaining == 0 {
            self.recv = Some(recv);
            let prefix = body_prefix;
            let stream = futures_util::stream::try_unfold(Some(prefix), |mut state| async move {
                if let Some(data) = state.take() {
                    if !data.is_empty() {
                        return Ok(Some((hyper::body::Frame::data(data), None)));
                    }
                }
                Ok(None)
            });
            http_body_util::StreamBody::new(stream).boxed()
        } else {
            let (reclaim_tx, reclaim_rx) = oneshot::channel::<Reclaim>();
            self.inflight_reclaim = Some(reclaim_rx);
            let stream = futures_util::stream::try_unfold(
                BodyState {
                    recv: Some(recv),
                    body_prefix: Some(body_prefix),
                    remaining: body_remaining,
                    reclaim_tx: Some(reclaim_tx),
                    scratch: vec![0u8; 8192],
                },
                |mut state| async move {
                    if let Some(prefix) = state.body_prefix.take() {
                        if !prefix.is_empty() {
                            return Ok(Some((hyper::body::Frame::data(prefix), state)));
                        }
                    }
                    if state.remaining == 0 {
                        if let (Some(tx), Some(recv)) = (state.reclaim_tx.take(), state.recv.take())
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
                    let to_read = state.remaining.min(state.scratch.len());
                    match recv.read(&mut state.scratch[..to_read]).await {
                        Ok(Some(n)) => {
                            state.remaining -= n;
                            let data = Bytes::copy_from_slice(&state.scratch[..n]);
                            if state.remaining == 0 {
                                if let (Some(tx), Some(recv)) =
                                    (state.reclaim_tx.take(), state.recv.take())
                                {
                                    let _ = tx.send(Reclaim {
                                        recv,
                                        overflow: Bytes::new(),
                                    });
                                }
                            }
                            Ok(Some((hyper::body::Frame::data(data), state)))
                        }
                        Ok(None) => Ok(None),
                        Err(e) => Err(std::io::Error::other(e)),
                    }
                },
            );
            http_body_util::StreamBody::new(stream).boxed()
        };
        Ok(Some(ProxyRequest {
            method,
            uri,
            headers: header_map,
            version: Version::HTTP_11,
            body,
        }))
    }
    async fn write_response(
        &mut self,
        mut response: Response<BoxBody<Bytes, std::io::Error>>,
    ) -> Result<()> {
        let status = response.status();
        if let Some(conn) = response.headers().get(http::header::CONNECTION) {
            if let Ok(v) = conn.to_str() {
                if v.eq_ignore_ascii_case("close") {
                    self.should_close = true;
                }
            }
        }
        sanitize_response_headers(response.headers_mut());
        // RFC 9112 §6.1 framing: 1xx/204/304 and responses to HEAD carry no
        // body and no framing headers; a body with a known exact length is
        // framed by content-length; chunked is reserved for genuinely
        // unknown-length streaming bodies (the only ones that may carry
        // trailers).
        let head_only = self.last_method.as_ref() == Some(&Method::HEAD);
        let no_body = head_only
            || status.is_informational()
            || status == http::StatusCode::NO_CONTENT
            || status == http::StatusCode::NOT_MODIFIED;
        let exact_len = if no_body {
            None
        } else {
            use hyper::body::Body as _;
            response.body().size_hint().exact()
        };
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
        if no_body {
            header_buf.put_slice(b"\r\n");
            self.send.write_all(&header_buf).await?;
            return Ok(());
        }
        if let Some(len) = exact_len {
            write!(header_buf, "content-length: {}\r\n\r\n", len).unwrap();
            self.send.write_all(&header_buf).await?;
            let mut body = response.into_body();
            let mut written: u64 = 0;
            while let Some(frame) = body.frame().await {
                let frame =
                    frame.map_err(|e| anyhow::anyhow!("error reading response frame: {}", e))?;
                match frame.into_data() {
                    Ok(data) => {
                        written += data.len() as u64;
                        if written > len {
                            self.should_close = true;
                            return Err(anyhow::anyhow!(
                                "response body exceeded declared length"
                            ));
                        }
                        self.send.write_chunk(data).await?;
                    }
                    Err(frame) => {
                        if frame.into_trailers().is_ok() {
                            tracing::debug!("dropping trailers on content-length response");
                        }
                    }
                }
            }
            if written != len {
                // Closing (instead of padding) keeps the peer from misreading
                // the next response as this one's remainder.
                self.should_close = true;
                return Err(anyhow::anyhow!("response body shorter than declared length"));
            }
            return Ok(());
        }
        header_buf.put_slice(b"transfer-encoding: chunked\r\n\r\n");
        self.send.write_all(&header_buf).await?;
        let mut body = response.into_body();
        let mut accumulated_trailers = HeaderMap::new();
        loop {
            match body.frame().await {
                Some(Ok(frame)) => match frame.into_data() {
                    Ok(chunk) => {
                        if chunk.is_empty() {
                            continue;
                        }
                        let mut prefix = [0u8; 32];
                        let prefix_len = {
                            use std::io::Write as IoWrite;
                            let mut cursor = std::io::Cursor::new(&mut prefix[..]);
                            write!(&mut cursor, "{:x}\r\n", chunk.len()).unwrap();
                            cursor.position() as usize
                        };
                        let mut parts = [
                            Bytes::copy_from_slice(&prefix[..prefix_len]),
                            chunk,
                            Bytes::from_static(b"\r\n"),
                        ];
                        self.send.write_all_chunks(&mut parts).await?;
                    }
                    Err(frame) => {
                        if let Ok(trailers) = frame.into_trailers() {
                            accumulated_trailers.extend(trailers);
                        }
                    }
                },
                Some(Err(e)) => {
                    return Err(anyhow::anyhow!("error reading response frame: {}", e));
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
struct BodyState {
    recv: Option<RecvStream>,
    body_prefix: Option<Bytes>,
    remaining: usize,
    reclaim_tx: Option<oneshot::Sender<Reclaim>>,
    scratch: Vec<u8>,
}
