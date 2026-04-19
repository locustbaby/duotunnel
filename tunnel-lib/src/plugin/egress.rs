use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;

use super::ctx::{DialCtx, PickCtx, Target};
use crate::proxy::tcp::UpstreamScheme;

// ── LoadBalancer ──────────────────────────────────────────────────────────────

/// Select one target from a candidate list.
///
/// Sync (no async) because selection algorithms are CPU-bound and must be
/// fast enough to call on every stream.
pub trait LoadBalancer: Send + Sync + 'static {
    fn pick<'a>(&self, targets: &'a [Target], ctx: &PickCtx) -> Option<&'a Target>;
}

// ── Connected ─────────────────────────────────────────────────────────────────

/// An open upstream connection returned by `UpstreamDialer::dial`.
///
/// Wraps a boxed `UpstreamPeer` so dialers can return any peer variant
/// without the framework needing to know the concrete type.
pub struct Connected {
    pub peer: crate::proxy::peers::PeerKind,
    pub remote_addr: SocketAddr,
}

// ── UpstreamDialer ────────────────────────────────────────────────────────────

/// Open a connection to a single upstream target.
///
/// Dialers declare which schemes they handle via `matches_scheme`; the
/// framework picks the first matching dialer in registration order.
#[async_trait]
pub trait UpstreamDialer: Send + Sync + 'static {
    /// Return true if this dialer can handle the given scheme.
    fn matches_scheme(&self, scheme: &UpstreamScheme) -> bool;

    /// Open a connection to `target`.
    async fn dial(&self, target: &Target, ctx: &DialCtx) -> Result<Connected>;
}

// ── Resolver ─────────────────────────────────────────────────────────────────

/// DNS resolver abstraction.
///
/// Allows swapping in a cached resolver, a stub resolver, or a DoH client
/// without changing the egress pipeline.
#[async_trait]
pub trait Resolver: Send + Sync + 'static {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>>;
}

// ── SystemResolver ────────────────────────────────────────────────────────────

/// Fallback resolver using `tokio::net::lookup_host`.  Always compiled in as
/// the CORE resolver; `resolver-cached` and friends wrap it.
pub struct SystemResolver;

#[async_trait]
impl Resolver for SystemResolver {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>> {
        let addrs: Vec<SocketAddr> = tokio::net::lookup_host(format!("{}:{}", host, port))
            .await?
            .collect();
        if addrs.is_empty() {
            anyhow::bail!("DNS resolution returned no addresses for {}:{}", host, port);
        }
        Ok(addrs)
    }
}
