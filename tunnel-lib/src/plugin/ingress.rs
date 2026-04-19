use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::net::TcpStream;

use super::ctx::{Route, ServerCtx};

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

// ── ProtocolHint ─────────────────────────────────────────────────────────────

/// Full output of Phase 1: protocol kind plus any metadata extractable from
/// the peek buffer without consuming the stream.
#[derive(Debug, Clone)]
pub struct ProtocolHint {
    pub kind: ProtocolKind,
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
        Self {
            kind,
            sni: None,
            authority: None,
            raw_preface: raw_preface.into(),
        }
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

// ── IngressProtocolHandler ────────────────────────────────────────────────────

/// Handler for a specific protocol on the ingress (server) side.
///
/// Unlike v1's `probe()` competition, the framework dispatches via
/// `ProtocolKind` after a single sniff pass.  The handler never needs to
/// identify the protocol — it only needs to *process* it.
#[async_trait]
pub trait IngressProtocolHandler: Send + Sync + 'static {
    /// The protocol this handler serves.  Used as the registry dispatch key.
    fn protocol_kind(&self) -> ProtocolKind;

    /// Handle an accepted TCP connection.
    ///
    /// `route` has already been resolved by Phase 3.
    /// `ctx.hint.raw_preface` contains the bytes that were peeked; the handler
    /// is responsible for prepending them when forwarding to the upstream.
    async fn handle(
        &self,
        stream: TcpStream,
        route: Route,
        ctx: &ServerCtx,
    ) -> Result<()>;
}
