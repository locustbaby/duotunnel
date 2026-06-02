use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::plugin::{ProtocolHint, ProtocolKind};
use crate::protocol::detect::extract_tls_sni;
use crate::proxy::core::Protocol;
use crate::PeekBufPool;

const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

#[derive(Clone, Debug)]
pub struct SniffPrefix(Bytes);

impl SniffPrefix {
    pub fn new(prefix: Bytes) -> Self {
        Self(prefix)
    }

    pub fn empty() -> Self {
        Self(Bytes::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

impl AsRef<[u8]> for SniffPrefix {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Bytes> for SniffPrefix {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl From<SniffPrefix> for Bytes {
    fn from(value: SniffPrefix) -> Self {
        value.0
    }
}

impl From<Vec<u8>> for SniffPrefix {
    fn from(value: Vec<u8>) -> Self {
        Self(Bytes::from(value))
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
                        let prefix = Bytes::copy_from_slice(data);
                        hint.raw_preface = prefix.clone();
                        pool.put(buf);
                        return Ok(SniffResult {
                            hint,
                            prefix: SniffPrefix::new(prefix),
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

        let prefix = if total == 0 {
            Bytes::new()
        } else {
            Bytes::copy_from_slice(&buf[..total])
        };
        pool.put(buf);
        Ok(SniffResult {
            hint: ProtocolHint::new(ProtocolKind::Tcp, prefix.clone()),
            prefix: SniffPrefix::new(prefix),
            bytes_read: total,
            complete: false,
        })
    }
}
