use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;

use super::ctx::{DialCtx, PickCtx, Target};
use crate::proxy::tcp::UpstreamScheme;

// ── LoadBalancer ──────────────────────────────────────────────────────────────

/// Select one target from a candidate list, returning its index.
///
/// Returning an index (rather than a reference) lets callers keep any
/// parallel data structures in sync without a fragile `ptr::eq` lookup.
/// Sync because selection algorithms are CPU-bound and called on every stream.
pub trait LoadBalancer: Send + Sync + 'static {
    fn pick(&self, targets: &[Target], ctx: &PickCtx) -> Option<usize>;
}

// ── Connected ─────────────────────────────────────────────────────────────────

/// An open upstream connection returned by `UpstreamDialer::dial`.
///
/// Transitional type kept for the currently unused dialer seam. The live
/// path has moved to `UpstreamResolver -> PeerSpec -> connect_peer`.
pub struct Connected {
    pub peer: crate::proxy::peers::PeerKind,
    pub remote_addr: SocketAddr,
}

// ── UpstreamDialer ────────────────────────────────────────────────────────────

/// Open a connection to a single upstream target.
///
/// Currently unused — the live path is `UpstreamResolver -> PeerSpec ->
/// connect_peer` in `proxy/core.rs`. Kept here for symmetry with
/// `LoadBalancer` / `Resolver` and to reserve the name, but not part of any
/// live dispatch path.
#[doc(hidden)]
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
