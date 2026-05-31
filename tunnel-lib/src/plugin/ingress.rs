use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::net::TcpStream;

use super::ctx::{Route, ServerCtx};
use crate::proxy::core::Protocol;

// ── ProtocolKind ─────────────────────────────────────────────────────────────

/// Discriminant output from the protocol-sniff phase (Phase 1).
///
/// Produced by `protocol::detect::detect_protocol_and_host` (CORE, not
/// pluggable) and used as the O(1) dispatch key into `PluginRegistry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolKind {
    Tls,   // TLS ClientHello (first byte 0x16)
    H2c,   // HTTP/2 cleartext preface
    Http1, // HTTP/1.x or WebSocket upgrade
    Tcp,   // Unrecognised — passthrough
}

impl ProtocolKind {
    pub fn as_label(self) -> &'static str {
        match self {
            ProtocolKind::Tls => "tls",
            ProtocolKind::H2c => "h2c",
            ProtocolKind::Http1 => "h1",
            ProtocolKind::Tcp => "tcp",
        }
    }
}

// ── ProtocolHint ─────────────────────────────────────────────────────────────

/// Full output of Phase 1: protocol kind plus any metadata extractable from
/// the peek buffer without consuming the stream.
///
/// `kind` and `protocol` coexist because they serve different purposes:
/// `kind` is the handler-dispatch key (four values, stable), while `protocol`
/// is the wire-level enum forwarded to the upstream via `RoutingInfo`. WS
/// shares `ProtocolKind::Http1` with H1 but needs `Protocol::WebSocket`
/// downstream, so the sniffer overrides `protocol` while keeping `kind`.
#[derive(Debug, Clone)]
pub struct ProtocolHint {
    pub kind: ProtocolKind,
    pub protocol: Protocol,
    /// SNI hostname from a TLS ClientHello, if present.
    pub sni: Option<String>,
    /// HTTP `Host` header / H2 `:authority`, if present.
    pub authority: Option<String>,
    /// The raw bytes that were peeked.  Passed to `IngressProtocolHandler::handle`
    /// so the handler can replay them into the upstream without re-reading.
    pub raw_preface: Bytes,
}

impl ProtocolHint {
    pub fn new(kind: ProtocolKind, raw_preface: impl Into<Bytes>) -> Self {
        let protocol = match kind {
            ProtocolKind::Tls | ProtocolKind::Tcp => Protocol::Tcp,
            ProtocolKind::H2c => Protocol::H2,
            ProtocolKind::Http1 => Protocol::H1,
        };
        Self {
            kind,
            protocol,
            sni: None,
            authority: None,
            raw_preface: raw_preface.into(),
        }
    }

    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    pub fn with_sni(mut self, sni: impl Into<String>) -> Self {
        self.sni = Some(sni.into());
        self
    }

    pub fn with_authority(mut self, authority: impl Into<String>) -> Self {
        self.authority = Some(authority.into());
        self
    }
}

#[async_trait]
pub trait IngressProtocolHandler: Send + Sync + 'static {
    fn protocol_kind(&self) -> ProtocolKind;
    async fn handle(&self, stream: crate::PrefixedReadWrite<tokio::net::TcpStream>, route: Option<Route>, ctx: &ServerCtx) -> Result<()>;
}
