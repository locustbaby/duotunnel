/// List-Watch protocol between tunnel-ctld and server.
///
/// Modelled after the Kubernetes list-watch pattern:
///   1. Server connects to ctld and sends a WatchRequest with resource_version=0
///      (meaning "give me the full snapshot first").
///   2. ctld responds with a WatchEvent::Snapshot (full state, sets the baseline version).
///   3. ctld keeps the TCP connection open and streams WatchEvent::Patch for every
///      subsequent mutation (token create/revoke/rotate, routing save).
///   4. If the server reconnects, it sends resource_version=N (last known version).
///      ctld responds with a full Snapshot again (simplest correct behaviour; diffs are
///      a future optimisation).
///
/// Wire framing: reuses the existing tunnel-lib send_message/recv_message bincode framing.
///   [msg_type: u8 = 0x06 ConfigPush][len: u32 BE][bincode(CtldMessage)]
use serde::{Deserialize, Serialize};
use tunnel_store::rules::{
    ClientGroup, EgressUpstreamDef, EgressVhostRule, IngressListener,
};
/// Sent by the server to initiate (or resume) a watch session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchRequest {
    /// The last resource_version the server has seen.
    /// 0 means "I have nothing; send me the full snapshot".
    pub resource_version: u64,
}

/// The full config snapshot pushed by ctld as the list response,
/// and also after each mutation (no delta in v1 for simplicity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Monotonically increasing. Incremented on every write to ctld.
    pub resource_version: u64,
    /// Full ingress listener + routing configuration.
    pub ingress_listeners: Vec<IngressListener>,
    pub client_groups: Vec<ClientGroup>,
    pub egress_upstreams: Vec<EgressUpstreamDef>,
    pub egress_vhost_rules: Vec<EgressVhostRule>,
    /// Flattened token table for server-side in-memory auth cache.
    /// Server uses constant-time prefix scan on this instead of hitting SQLite.
    pub token_cache: Vec<TokenCacheEntry>,
}

/// A single token entry in the in-memory cache sent to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCacheEntry {
    /// Full 64-char hex SHA-256 of the raw token.
    pub hash_hex: String,
    /// The client group this token belongs to.
    pub client_group: String,
    /// "active" | "disabled"
    pub client_status: String,
    /// "active" | "revoked"
    pub token_status: String,
}

/// Message envelope pushed from ctld → server over the watch stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchEvent {
    /// Full state snapshot (response to WatchRequest or post-reconnect).
    Snapshot(ConfigSnapshot),
    /// Incremental patch — currently always a full Snapshot re-send.
    /// Reserved for future delta optimisation.
    Patch(ConfigSnapshot),
}
