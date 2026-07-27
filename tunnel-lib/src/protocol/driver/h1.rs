use super::{ProtocolDriver, ProxyRequest};
use crate::protocol::http_utils::{
    content_length_from_headers, is_forwardable_trailer, parse_content_length,
    sanitize_request_headers, sanitize_response_headers,
};
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
    last_http_minor: u8,
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
            last_http_minor: 1,
        }
    }
    pub async fn finish(&mut self) -> Result<()> {
        self.send.finish()?;
        Ok(())
    }
    pub async fn write_502(&mut self, error_msg: &str) -> Result<()> {
        let resp = format!(
            "HTTP/1.1 502 Bad Gateway\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
            error_msg.len(),
            error_msg
        );
        self.send.write_all(resp.as_bytes()).await?;
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
    /// Answer a request we refuse to relay, then close: a bare connection reset
    /// leaves the client unable to tell a protocol error from a network fault.
    async fn reject(&mut self, resp: &'static [u8]) {
        self.should_close = true;
        let _ = self.send.write_all(resp).await;
        let _ = self.send.finish();
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
const RESP_417: &[u8] =
    b"HTTP/1.1 417 Expectation Failed\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
const RESP_431: &[u8] =
    b"HTTP/1.1 431 Request Header Fields Too Large\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";

enum FramingDecision {
    Accept {
        content_length: usize,
    },
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
    let mut cl_seen = false;
    let mut cl_value: Option<u64> = None;
    let mut cl_invalid = false;
    for h in headers {
        if h.name.eq_ignore_ascii_case("transfer-encoding") {
            has_te = true;
        } else if h.name.eq_ignore_ascii_case("content-length") {
            cl_seen = true;
            match parse_content_length(h.value) {
                Some(n) if cl_value.is_none_or(|prev| prev == n) => cl_value = Some(n),
                _ => cl_invalid = true,
            }
        }
    }
    if has_te {
        return if cl_seen {
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
    if cl_invalid {
        return FramingDecision::Reject {
            resp: RESP_400,
            reason: "invalid or conflicting content-length",
        };
    }
    let content_length = match cl_value.map(usize::try_from) {
        None => 0,
        Some(Ok(n)) => n,
        Some(Err(_)) => {
            return FramingDecision::Reject {
                resp: RESP_400,
                reason: "content-length out of range",
            }
        }
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
                        FramingDecision::Reject { resp, reason } => {
                            break Err((resp, reason.to_string()))
                        }
                        FramingDecision::Accept { content_length } => content_length,
                    };
                    let (Some(method_str), Some(path)) = (req.method, req.path) else {
                        break Err((
                            RESP_400,
                            "request line without method or target".to_string(),
                        ));
                    };
                    let uri_str = format!("{}://{}{}", self.scheme, self.authority, path);
                    let Ok(uri) = uri_str.parse::<Uri>() else {
                        break Err((RESP_400, format!("unparsable request target: {path}")));
                    };
                    let Ok(method) = Method::from_bytes(method_str.as_bytes()) else {
                        break Err((RESP_400, format!("unsupported method: {method_str}")));
                    };
                    let mut header_map = HeaderMap::new();
                    let mut host_count = 0;
                    let mut duplicate_host = false;
                    for h in req.headers.iter() {
                        if !h.name.is_empty() {
                            if h.name.eq_ignore_ascii_case("host") {
                                host_count += 1;
                                if host_count > 1 {
                                    duplicate_host = true;
                                    break;
                                }
                            }
                            if let (Ok(name), Ok(value)) = (
                                http::header::HeaderName::from_bytes(h.name.as_bytes()),
                                http::header::HeaderValue::from_bytes(h.value),
                            ) {
                                header_map.append(name, value);
                            }
                        }
                    }
                    if duplicate_host {
                        break Err((RESP_400, "multiple Host headers".to_string()));
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
                        break Err((RESP_431, "request header fields too large".to_string()));
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
                            break Err((RESP_400, "unexpected EOF in headers".to_string()));
                        }
                    }
                }
                Err(e) => break Err((RESP_400, format!("malformed request head: {e}"))),
            }
        };
        let parsed = match parsed {
            Ok(parsed) => parsed,
            Err((resp, reason)) => {
                self.reject(resp).await;
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
        self.last_http_minor = http_minor;
        if let Some(expect) = header_map.remove(http::header::EXPECT) {
            if expect.as_bytes().eq_ignore_ascii_case(b"100-continue") {
                // Answer the interim response locally: the body is streamed to
                // the upstream unconditionally, so waiting for the upstream's
                // own 100 would only stall the client until its timeout.
                // An HTTP/1.0 sender's expectation is ignored (RFC 9110
                // §10.1.1) — it cannot be relied on to await a 1xx.
                if http_minor == 1 {
                    self.send
                        .write_all(b"HTTP/1.1 100 Continue\r\n\r\n")
                        .await?;
                }
            } else {
                // RFC 9110 §10.1.1: an expectation this hop cannot meet must be
                // refused. Silently dropping it deadlocks the client, which
                // keeps waiting for an interim response before sending a body.
                self.reject(RESP_417).await;
                return Err(anyhow::anyhow!(
                    "rejected request: unsupported expectation: {:?}",
                    expect
                ));
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
                    let to_read = state.remaining.min(8192);
                    match recv.read_chunk(to_read, true).await {
                        Ok(Some(chunk)) => {
                            let n = chunk.bytes.len();
                            state.remaining -= n;
                            let data = chunk.bytes;
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
        // Read the upstream length before sanitizing removes the field: it is
        // the only dependable source here, because the proxied body is wrapped
        // in http-body-util's `MapFrame`, which does not forward `size_hint` and
        // so reports an unknown length for every response. `size_hint` still
        // covers bodies this crate builds itself (e.g. `Empty` -> 0).
        // Transfer-encoding takes precedence over any content-length (the
        // precedence hyper's own decoder applies), so a length sent alongside it
        // describes nothing about the body we are about to relay.
        let upstream_len = if response
            .headers()
            .contains_key(http::header::TRANSFER_ENCODING)
        {
            None
        } else {
            content_length_from_headers(response.headers())
        };
        sanitize_response_headers(response.headers_mut());
        // RFC 9112 §6.1 framing: 1xx/204/304 and responses to HEAD carry no
        // body; a body with a known exact length is framed by content-length;
        // chunked is reserved for genuinely unknown-length streaming bodies
        // (the only ones that may carry trailers).
        let head_only = self.last_method.as_ref() == Some(&Method::HEAD);
        let no_body = head_only
            || status.is_informational()
            || status == http::StatusCode::NO_CONTENT
            || status == http::StatusCode::NOT_MODIFIED;
        let exact_len = if no_body {
            None
        } else {
            use hyper::body::Body as _;
            upstream_len.or_else(|| response.body().size_hint().exact())
        };
        // RFC 9112 §6.1 forbids chunked framing toward an HTTP/1.0 recipient:
        // such a client reads the chunk metadata as body bytes and then waits
        // for a close that never comes. Without an exact length the only legal
        // framing left is close-delimited.
        let close_delimited = !no_body && exact_len.is_none() && self.last_http_minor == 0;
        if close_delimited {
            self.should_close = true;
        }
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
            // RFC 9110 §9.3.2: a HEAD response carries the same header fields
            // the equivalent GET would, content-length included, and a 304 may
            // repeat the length of the cached representation. RFC 9112 §6.3
            // rule 1 terminates all of these at the blank line regardless, so
            // echoing the length cannot desynchronise keep-alive. A 204 must
            // never claim a length.
            let echoed_len = if status.is_informational() || status == http::StatusCode::NO_CONTENT
            {
                None
            } else if head_only || status == http::StatusCode::NOT_MODIFIED {
                upstream_len
            } else {
                None
            };
            if let Some(len) = echoed_len {
                write!(header_buf, "content-length: {}\r\n", len).unwrap();
            }
            header_buf.put_slice(b"\r\n");
            self.send.write_all(&header_buf).await?;
            return Ok(());
        }
        if close_delimited {
            // The client cannot infer the end of an unframed body unless it
            // knows the connection is closing (it may have asked for
            // keep-alive), so say so explicitly.
            header_buf.put_slice(b"connection: close\r\n\r\n");
            self.send.write_all(&header_buf).await?;
            let mut body = response.into_body();
            while let Some(frame) = body.frame().await {
                let frame =
                    frame.map_err(|e| anyhow::anyhow!("error reading response frame: {}", e))?;
                if let Ok(data) = frame.into_data() {
                    if !data.is_empty() {
                        self.send.write_chunk(data).await?;
                    }
                }
            }
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
                            return Err(anyhow::anyhow!("response body exceeded declared length"));
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
                return Err(anyhow::anyhow!(
                    "response body shorter than declared length"
                ));
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
                            for (name, value) in trailers.iter() {
                                if is_forwardable_trailer(name) {
                                    accumulated_trailers.append(name.clone(), value.clone());
                                } else {
                                    tracing::debug!(
                                        field = %name,
                                        "dropping disallowed upstream trailer field"
                                    );
                                }
                            }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decide(headers: &[(&str, &[u8])]) -> FramingDecision {
        let parsed: Vec<httparse::Header<'_>> = headers
            .iter()
            .map(|(name, value)| httparse::Header { name, value })
            .collect();
        validate_framing(&parsed)
    }

    fn accepted(headers: &[(&str, &[u8])]) -> Option<usize> {
        match decide(headers) {
            FramingDecision::Accept { content_length } => Some(content_length),
            FramingDecision::Reject { .. } => None,
        }
    }

    fn rejected_with(headers: &[(&str, &[u8])]) -> Option<&'static [u8]> {
        match decide(headers) {
            FramingDecision::Accept { .. } => None,
            FramingDecision::Reject { resp, .. } => Some(resp),
        }
    }

    #[test]
    fn framing_rejects_non_digit_content_length() {
        // A `+5` forwarded verbatim is re-framed as chunked by the upstream
        // client while the illegal length survives: a CL.TE smuggling request.
        assert_eq!(rejected_with(&[("Content-Length", b"+5")]), Some(RESP_400));
        assert_eq!(rejected_with(&[("content-length", b"")]), Some(RESP_400));
        assert_eq!(rejected_with(&[("content-length", b"5 5")]), Some(RESP_400));
    }

    #[test]
    fn framing_accepts_repeated_identical_content_length() {
        assert_eq!(accepted(&[("Content-Length", b"5, 5")]), Some(5));
        assert_eq!(
            accepted(&[("content-length", b"5"), ("Content-Length", b" 5 ")]),
            Some(5)
        );
        assert_eq!(
            rejected_with(&[("content-length", b"5"), ("content-length", b"6")]),
            Some(RESP_400)
        );
    }

    #[test]
    fn framing_rejects_transfer_encoding() {
        assert_eq!(
            rejected_with(&[("Transfer-Encoding", b"chunked")]),
            Some(RESP_411)
        );
        assert_eq!(
            rejected_with(&[("transfer-encoding", b"chunked"), ("content-length", b"5")]),
            Some(RESP_400)
        );
        // Still 400 even when the length is the malformed kind: the conflict is
        // what matters, and 411 would invite a resend of the same attack.
        assert_eq!(
            rejected_with(&[("transfer-encoding", b"chunked"), ("content-length", b"+5")]),
            Some(RESP_400)
        );
    }

    #[test]
    fn framing_defaults_to_empty_body() {
        assert_eq!(accepted(&[("host", b"example.com")]), Some(0));
    }
}
