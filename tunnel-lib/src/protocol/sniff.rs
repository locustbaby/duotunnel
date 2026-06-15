use bytes::{Buf, Bytes};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::plugin::{ProtocolHint, ProtocolKind};
use crate::protocol::detect::extract_tls_sni;
use crate::proxy::core::Protocol;
use crate::PeekBufPool;

const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub struct PooledBufInner {
    buf: Vec<u8>,
    pool: PeekBufPool,
}

impl Drop for PooledBufInner {
    fn drop(&mut self) {
        let buf = std::mem::take(&mut self.buf);
        if !buf.is_empty() {
            self.pool.put(buf);
        }
    }
}

impl std::fmt::Debug for PooledBufInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledBufInner")
            .field("buf_len", &self.buf.len())
            .field("buf_cap", &self.buf.capacity())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum SniffPrefix {
    Empty,
    Bytes(Bytes),
    Pooled {
        inner: Arc<PooledBufInner>,
        offset: usize,
        len: usize,
    },
}

impl SniffPrefix {
    pub fn new(bytes: Bytes) -> Self {
        if bytes.is_empty() {
            Self::Empty
        } else {
            Self::Bytes(bytes)
        }
    }

    pub fn new_pooled(buf: Vec<u8>, len: usize, pool: PeekBufPool) -> Self {
        if len == 0 {
            if !buf.is_empty() {
                pool.put(buf);
            }
            Self::Empty
        } else {
            Self::Pooled {
                inner: Arc::new(PooledBufInner { buf, pool }),
                offset: 0,
                len,
            }
        }
    }

    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Bytes(b) => b.len(),
            Self::Pooled { len, .. } => *len,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Empty => &[],
            Self::Bytes(b) => b,
            Self::Pooled { inner, offset, len } => &inner.buf[*offset..*offset + *len],
        }
    }

    pub fn advance(&mut self, amount: usize) {
        match self {
            Self::Empty => {
                assert_eq!(amount, 0);
            }
            Self::Bytes(b) => {
                b.advance(amount);
                if b.is_empty() {
                    *self = Self::Empty;
                }
            }
            Self::Pooled { offset, len, .. } => {
                assert!(amount <= *len);
                *offset += amount;
                *len -= amount;
                if *len == 0 {
                    *self = Self::Empty;
                }
            }
        }
    }

    pub fn into_bytes(self) -> Bytes {
        match self {
            Self::Empty => Bytes::new(),
            Self::Bytes(b) => b,
            Self::Pooled { inner, offset, len } => {
                Bytes::copy_from_slice(&inner.buf[offset..offset + len])
            }
        }
    }
}

impl AsRef<[u8]> for SniffPrefix {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Bytes> for SniffPrefix {
    fn from(value: Bytes) -> Self {
        Self::new(value)
    }
}

impl From<SniffPrefix> for Bytes {
    fn from(value: SniffPrefix) -> Self {
        value.into_bytes()
    }
}

impl From<Vec<u8>> for SniffPrefix {
    fn from(value: Vec<u8>) -> Self {
        Self::new(Bytes::from(value))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SniffPolicy {
    pub initial_read_bytes: usize,
    pub max_sniff_bytes: usize,
    pub max_read_rounds: usize,
}

impl Default for SniffPolicy {
    fn default() -> Self {
        Self {
            initial_read_bytes: 512,
            max_sniff_bytes: 4096,
            max_read_rounds: 4,
        }
    }
}

pub type SniffStream<S> = crate::PrefixedReadWrite<S>;

#[derive(Clone, Debug)]
pub struct SniffResult {
    pub hint: ProtocolHint,
    pub prefix: SniffPrefix,
    pub bytes_read: usize,
    pub complete: bool,
}

impl SniffResult {
    pub fn into_stream<S>(self, stream: S) -> SniffStream<S> {
        crate::PrefixedReadWrite::new(stream, self.prefix)
    }
}

#[derive(Clone, Debug)]
pub enum SniffOutcome {
    Matched(ProtocolHint),
    NeedMore,
    NoMatch,
}

pub trait ProtocolDetector: Send + Sync {
    fn detect(&self, buf: &[u8]) -> SniffOutcome;
}

pub struct H2cDetector;
pub struct Http1Detector;
pub struct TlsClientHelloDetector;

impl ProtocolDetector for H2cDetector {
    fn detect(&self, buf: &[u8]) -> SniffOutcome {
        if buf.is_empty() {
            return SniffOutcome::NeedMore;
        }
        if buf.len() < HTTP2_PREFACE.len() {
            if HTTP2_PREFACE.starts_with(buf) {
                return SniffOutcome::NeedMore;
            }
            return SniffOutcome::NoMatch;
        }
        if &buf[..HTTP2_PREFACE.len()] == HTTP2_PREFACE {
            return SniffOutcome::Matched(ProtocolHint::new(ProtocolKind::H2c, Bytes::new()));
        }
        SniffOutcome::NoMatch
    }
}

impl ProtocolDetector for Http1Detector {
    fn detect(&self, buf: &[u8]) -> SniffOutcome {
        if buf.is_empty() {
            return SniffOutcome::NeedMore;
        }
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(buf) {
            Ok(status) if status.is_complete() => {
                let mut host = None;
                let mut is_websocket = false;
                for h in req.headers.iter() {
                    if h.name.eq_ignore_ascii_case("Host") {
                        host = std::str::from_utf8(h.value)
                            .ok()
                            .map(|s| s.trim().to_string());
                    } else if h.name.eq_ignore_ascii_case("Upgrade")
                        && std::str::from_utf8(h.value)
                            .unwrap_or("")
                            .eq_ignore_ascii_case("websocket")
                    {
                        is_websocket = true;
                    }
                }
                let mut hint = ProtocolHint::new(ProtocolKind::Http1, Bytes::new());
                if let Some(host) = host {
                    hint = hint.with_authority(host);
                }
                if is_websocket {
                    hint = hint.with_protocol(Protocol::WebSocket);
                }
                SniffOutcome::Matched(hint)
            }
            Ok(_) => SniffOutcome::NeedMore,
            Err(_) => SniffOutcome::NoMatch,
        }
    }
}

impl ProtocolDetector for TlsClientHelloDetector {
    fn detect(&self, buf: &[u8]) -> SniffOutcome {
        match tls_record_state(buf) {
            TlsRecordState::NeedMore => SniffOutcome::NeedMore,
            TlsRecordState::NoMatch => SniffOutcome::NoMatch,
            TlsRecordState::Matched(record) => {
                let mut hint = ProtocolHint::new(ProtocolKind::Tls, Bytes::new());
                if let Some(sni) = extract_tls_sni(record) {
                    hint = hint.with_sni(sni);
                }
                SniffOutcome::Matched(hint)
            }
        }
    }
}

enum TlsRecordState<'a> {
    Matched(&'a [u8]),
    NeedMore,
    NoMatch,
}

fn tls_record_state(buf: &[u8]) -> TlsRecordState<'_> {
    if buf.is_empty() {
        return TlsRecordState::NeedMore;
    }
    if buf[0] != 0x16 {
        return TlsRecordState::NoMatch;
    }
    if buf.len() < 5 {
        return TlsRecordState::NeedMore;
    }
    if buf[1] != 0x03 {
        return TlsRecordState::NoMatch;
    }
    let record_len = u16::from_be_bytes([buf[3], buf[4]]) as usize;
    let total_len = 5 + record_len;
    if buf.len() < total_len {
        return TlsRecordState::NeedMore;
    }
    if buf[5] != 0x01 {
        return TlsRecordState::NoMatch;
    }
    TlsRecordState::Matched(&buf[..total_len])
}

static H2C_DETECTOR: H2cDetector = H2cDetector;
static HTTP1_DETECTOR: Http1Detector = Http1Detector;
static TLS_CLIENT_HELLO_DETECTOR: TlsClientHelloDetector = TlsClientHelloDetector;
static DEFAULT_INGRESS_DETECTORS: [&'static dyn ProtocolDetector; 3] =
    [&H2C_DETECTOR, &HTTP1_DETECTOR, &TLS_CLIENT_HELLO_DETECTOR];
static DEFAULT_CLIENT_DETECTORS: [&'static dyn ProtocolDetector; 3] =
    [&H2C_DETECTOR, &HTTP1_DETECTOR, &TLS_CLIENT_HELLO_DETECTOR];
static DEFAULT_PROXYENGINE_DETECTORS: [&'static dyn ProtocolDetector; 3] =
    [&H2C_DETECTOR, &HTTP1_DETECTOR, &TLS_CLIENT_HELLO_DETECTOR];

pub fn default_ingress_detectors() -> &'static [&'static dyn ProtocolDetector] {
    &DEFAULT_INGRESS_DETECTORS
}

pub fn default_client_detectors() -> &'static [&'static dyn ProtocolDetector] {
    &DEFAULT_CLIENT_DETECTORS
}

pub fn default_proxyengine_detectors() -> &'static [&'static dyn ProtocolDetector] {
    &DEFAULT_PROXYENGINE_DETECTORS
}

pub struct SniffRuntime<'a> {
    policy: SniffPolicy,
    detectors: &'a [&'a dyn ProtocolDetector],
}

impl<'a> SniffRuntime<'a> {
    pub fn new(policy: SniffPolicy, detectors: &'a [&'a dyn ProtocolDetector]) -> Self {
        Self { policy, detectors }
    }

    pub async fn sniff<R: AsyncRead + Unpin>(
        &self,
        stream: &mut R,
        pool: &PeekBufPool,
    ) -> std::io::Result<SniffResult> {
        let mut buf = pool.take();
        if buf.len() < self.policy.max_sniff_bytes {
            buf.resize(self.policy.max_sniff_bytes, 0);
        }

        let mut total = 0usize;
        let mut rounds = 0usize;
        let mut target = self
            .policy
            .initial_read_bytes
            .min(self.policy.max_sniff_bytes);

        while rounds < self.policy.max_read_rounds && total < self.policy.max_sniff_bytes {
            let read_end = target.max(total + 1).min(self.policy.max_sniff_bytes);
            let n = stream.read(&mut buf[total..read_end]).await?;
            rounds += 1;
            if n == 0 {
                break;
            }
            total += n;
            let data = &buf[..total];
            let mut need_more = false;

            for detector in self.detectors {
                match detector.detect(data) {
                    SniffOutcome::Matched(mut hint) => {
                        let prefix = SniffPrefix::new_pooled(buf, total, *pool);
                        hint.raw_preface = prefix.clone();
                        return Ok(SniffResult {
                            hint,
                            prefix,
                            bytes_read: total,
                            complete: true,
                        });
                    }
                    SniffOutcome::NeedMore => need_more = true,
                    SniffOutcome::NoMatch => {}
                }
            }

            if !need_more || total >= self.policy.max_sniff_bytes {
                break;
            }
            target = self.policy.max_sniff_bytes;
        }

        let prefix = SniffPrefix::new_pooled(buf, total, *pool);
        Ok(SniffResult {
            hint: ProtocolHint::new(ProtocolKind::Tcp, prefix.clone()),
            prefix,
            bytes_read: total,
            complete: false,
        })
    }
}
