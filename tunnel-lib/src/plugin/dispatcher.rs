use anyhow::{anyhow, Result};
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tracing::error;

use super::ctx::{AdmissionReq, PhaseOutcome, PhaseResult, PhaseTiming, RouteCtx, ServerCtx};
use super::ingress::{ProtocolHint, ProtocolKind};
use super::metrics::observe_proxy_error;
use super::registry::PluginRegistry;
use super::service::TunnelService;
use crate::protocol::sniff::{default_ingress_detectors, SniffPolicy, SniffRuntime};
use crate::ProxyError;

/// Call `svc.logging` inside `catch_unwind` so a broken implementation can't
/// tear down the accept worker. Emission errors are dropped — this is the
/// logging path, nothing above it can recover from a panic here.
fn safe_logging(svc: &dyn TunnelService, ctx: &ServerCtx, outcome: &PhaseOutcome) {
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| svc.logging(ctx, outcome)));
    if result.is_err() {
        error!("TunnelService::logging panicked — metrics/logs dropped for this connection");
    }
}

/// Upper bound on bytes that Phase 1 peeks before producing a
/// `ProtocolHint`. Exposed so ingress handlers can size stack buffers
/// (e.g. for discarding peeked bytes before handing the stream to a
/// relay) without hard-coding the number.
pub const SNIFF_LIMIT: usize = 4096;

async fn sniff(
    mut stream: TcpStream,
    sniff_timeout: std::time::Duration,
) -> Result<(ProtocolHint, crate::PrefixedReadWrite<TcpStream>)> {
    let runtime = SniffRuntime::new(SniffPolicy::default(), default_ingress_detectors());
    let pool = crate::PeekBufPool::new(SNIFF_LIMIT);
    let sniffed = match tokio::time::timeout(sniff_timeout, runtime.sniff(&mut stream, &pool)).await
    {
        Ok(res) => res?,
        Err(_) => {
            return Err(anyhow!(
                "protocol sniffing timed out (Slowloris protection)"
            ));
        }
    };
    let mut hint = sniffed.hint.clone();
    if hint.kind == ProtocolKind::Tcp
        && sniffed.bytes_read > 0
        && sniffed.prefix.as_bytes()[0] == 0x16
    {
        hint.kind = ProtocolKind::Tls;
    }
    let prefixed = sniffed.into_stream(stream);
    Ok((hint, prefixed))
}

pub struct IngressDispatcher {
    registry: Arc<PluginRegistry>,
    listener_port: u16,
}

impl IngressDispatcher {
    pub fn new(registry: Arc<PluginRegistry>, listener_port: u16) -> Self {
        Self {
            registry,
            listener_port,
        }
    }

    pub async fn dispatch(
        &self,
        stream: TcpStream,
        svc: &dyn TunnelService,
        ctx: &mut ServerCtx,
    ) -> Result<()> {
        let mut timing = PhaseTiming::new();

        let (hint, stream) = sniff(
            stream,
            std::time::Duration::from_millis(ctx.timeouts.sniff_ms),
        )
        .await?;
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
                    safe_logging(svc, ctx, &outcome);
                    return Err(anyhow!(
                        "connection rejected at pre_admission (status={})",
                        status
                    ));
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
                safe_logging(svc, ctx, &outcome);
                return Err(anyhow!(
                    "connection rejected at admission (status={})",
                    status
                ));
            }
            PhaseResult::Continue(()) => {
                ctx.admitted = true;
                timing.admission_done_at = Some(Instant::now());
            }
        }

        // ── Phase 4: RouteResolver::resolve (from registry) ───────────────────
        let handler = self
            .registry
            .ingress_handlers
            .get(&hint.kind)
            .ok_or_else(|| anyhow!("no ingress handler registered for {:?}", hint.kind))?;

        let route = if hint.kind == ProtocolKind::H2c {
            timing.route_done_at = Some(Instant::now());
            None
        } else {
            let route_ctx = RouteCtx {
                listener_port: self.listener_port,
                client_addr: ctx.peer_addr,
                hint: hint.clone(),
            };
            match self.registry.route_resolver.resolve(&route_ctx).await? {
                PhaseResult::Continue(r) => {
                    timing.route_done_at = Some(Instant::now());
                    Some(r)
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
                    safe_logging(svc, ctx, &outcome);
                    return Err(anyhow!("no route found (status={})", status));
                }
            }
        };
        ctx.route = route.clone();

        // ── Phase 5: IngressProtocolHandler::handle ───────────────────────────
        timing.tunnel_open_at = Some(Instant::now());
        ctx.timing = timing.clone();

        let handle_result = handler.handle(stream, route, ctx).await;
        if let Err(err) = &handle_result {
            if let Some(proxy_error) = err.downcast_ref::<ProxyError>() {
                observe_proxy_error(ctx.metrics.as_ref(), hint.kind.as_label(), proxy_error);
            }
        }

        // ── Phase 6: logging ──────────────────────────────────────────────────
        timing.completed_at = Some(Instant::now());
        let outcome = PhaseOutcome {
            timing,
            bytes_sent: 0,
            bytes_recv: 0,
            error: handle_result.as_ref().err().map(|e| e.to_string()),
        };
        safe_logging(svc, ctx, &outcome);

        handle_result
    }
}
