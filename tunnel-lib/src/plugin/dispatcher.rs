use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;

use super::ctx::{AdmissionReq, PhaseOutcome, PhaseTiming, PhaseResult, RouteCtx, ServerCtx};
use super::ingress::{ProtocolHint, ProtocolKind};
use super::registry::PluginRegistry;
use super::service::TunnelService;
use crate::protocol::detect::{detect_protocol_and_host, extract_tls_sni};
use crate::proxy::core::Protocol;

const SNIFF_LIMIT: usize = 256;

// ── Protocol sniff (Phase 1, CORE — not pluggable) ────────────────────────────

/// Peek up to `SNIFF_LIMIT` bytes from the TCP stream and produce a
/// `ProtocolHint`.  The stream is NOT advanced — peeked bytes are stored
/// in `raw_preface` for the handler to replay.
async fn sniff(stream: &TcpStream) -> Result<ProtocolHint> {
    let mut buf = [0u8; SNIFF_LIMIT];
    let n = stream.peek(&mut buf).await?;
    let data = &buf[..n];

    let (legacy_proto, host) = detect_protocol_and_host(data);

    let kind = match legacy_proto {
        Protocol::H2 => ProtocolKind::H2c,
        Protocol::H1 | Protocol::WebSocket => ProtocolKind::Http1,
        Protocol::Tcp | Protocol::Unknown => {
            // detect.rs maps TLS ClientHello to Protocol::Tcp + SNI as host.
            // Re-check: if first byte is 0x16 it's TLS.
            if n > 0 && data[0] == 0x16 {
                ProtocolKind::Tls
            } else {
                ProtocolKind::Tcp
            }
        }
    };

    let raw_preface = bytes::Bytes::copy_from_slice(data);
    let mut hint = ProtocolHint::new(kind, raw_preface);

    match kind {
        ProtocolKind::Tls => {
            if let Some(sni) = extract_tls_sni(data) {
                hint = hint.with_sni(sni);
            }
        }
        ProtocolKind::H2c | ProtocolKind::Http1 => {
            if let Some(h) = host {
                hint = hint.with_authority(h);
            }
        }
        ProtocolKind::Tcp => {}
    }

    Ok(hint)
}

// ── IngressDispatcher ─────────────────────────────────────────────────────────

/// Runs the 5-phase ingress pipeline for every accepted TCP connection.
///
/// Phase ordering:
///   1. `sniff`                          — peek bytes, produce `ProtocolHint` (CORE)
///   2. `ConnectionModule::pre_admission` — IP/rate-limit modules, in order
///   3. `TunnelService::admission`        — token / auth check
///   4. `RouteResolver::resolve`          — vhost / static route lookup (from registry)
///   5. `IngressProtocolHandler::handle`  — TLS / H2c / H1 / TCP handler
///   6. `TunnelService::logging`          — metrics & access logs
pub struct IngressDispatcher {
    registry: Arc<PluginRegistry>,
    listener_port: u16,
}

impl IngressDispatcher {
    pub fn new(registry: Arc<PluginRegistry>, listener_port: u16) -> Self {
        Self { registry, listener_port }
    }

    pub async fn dispatch(
        &self,
        stream: TcpStream,
        svc: &dyn TunnelService,
        ctx: &mut ServerCtx,
    ) -> Result<()> {
        let mut timing = PhaseTiming::new();

        // ── Phase 1: protocol sniff ───────────────────────────────────────────
        let hint = sniff(&stream).await?;
        timing.sniff_done_at = Some(Instant::now());
        ctx.hint = Some(hint.clone());

        let admission_req = AdmissionReq {
            peer_addr: ctx.peer_addr,
            hint: Some(hint.clone()),
            token: None, // callers may populate before dispatch if needed
        };

        // ── Phase 2: ConnectionModule::pre_admission (in order) ───────────────
        for module in &self.registry.modules {
            match module.pre_admission(&admission_req).await? {
                PhaseResult::Reject { status, message } => {
                    let outcome = PhaseOutcome {
                        timing,
                        bytes_sent: 0,
                        bytes_recv: 0,
                        error: Some(format!(
                            "pre_admission rejected: status={} msg={}",
                            status,
                            String::from_utf8_lossy(&message)
                        )),
                    };
                    svc.logging(&outcome);
                    return Err(anyhow!("connection rejected at pre_admission (status={})", status));
                }
                PhaseResult::Continue(()) => {}
            }
        }

        // ── Phase 3: TunnelService::admission ────────────────────────────────
        match svc.admission(&admission_req).await? {
            PhaseResult::Reject { status, message } => {
                timing.admission_done_at = Some(Instant::now());
                let outcome = PhaseOutcome {
                    timing,
                    bytes_sent: 0,
                    bytes_recv: 0,
                    error: Some(format!(
                        "admission rejected: status={} msg={}",
                        status,
                        String::from_utf8_lossy(&message)
                    )),
                };
                svc.logging(&outcome);
                return Err(anyhow!("connection rejected at admission (status={})", status));
            }
            PhaseResult::Continue(()) => {
                ctx.admitted = true;
                timing.admission_done_at = Some(Instant::now());
            }
        }

        // ── Phase 4: RouteResolver::resolve (from registry) ───────────────────
        let route_ctx = RouteCtx {
            listener_port: self.listener_port,
            peer_addr: ctx.peer_addr,
            hint: hint.clone(),
        };
        let route = match self.registry.route_resolver.resolve(&route_ctx).await? {
            PhaseResult::Continue(r) => {
                timing.route_done_at = Some(Instant::now());
                r
            }
            PhaseResult::Reject { status, message } => {
                let outcome = PhaseOutcome {
                    timing,
                    bytes_sent: 0,
                    bytes_recv: 0,
                    error: Some(format!(
                        "route rejected: status={} msg={}",
                        status,
                        String::from_utf8_lossy(&message)
                    )),
                };
                svc.logging(&outcome);
                return Err(anyhow!("no route found (status={})", status));
            }
        };
        ctx.route = Some(route.clone());

        // ── Phase 5: IngressProtocolHandler::handle ───────────────────────────
        let handler = self
            .registry
            .ingress_handlers
            .get(&hint.kind)
            .ok_or_else(|| anyhow!("no ingress handler registered for {:?}", hint.kind))?;

        timing.tunnel_open_at = Some(Instant::now());
        ctx.timing = timing.clone();

        let handle_result = handler.handle(stream, route, ctx).await;

        // ── Phase 6: logging ──────────────────────────────────────────────────
        timing.completed_at = Some(Instant::now());
        let outcome = PhaseOutcome {
            timing,
            bytes_sent: 0,
            bytes_recv: 0,
            error: handle_result.as_ref().err().map(|e| e.to_string()),
        };
        svc.logging(&outcome);

        handle_result
    }
}
