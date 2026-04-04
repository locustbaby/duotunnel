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
///   [msg_type: u8 = 0x06 ConfigPush][len: u32 BE][bincode(WatchEvent)]
///
/// All types here are self-contained (no tunnel_store dependency) so this module
/// can live in tunnel-lib and be used by both tunnel-service and server.
use serde::{Deserialize, Serialize};

// ── Ingress routing ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoIngressVhostRule {
    pub match_host: String,
    pub group_id: String,
    pub proxy_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtoIngressListenerMode {
    Http { vhost: Vec<ProtoIngressVhostRule> },
    Tcp { group_id: String, proxy_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoIngressListener {
    pub port: u16,
    pub mode: ProtoIngressListenerMode,
}

// ── Client groups & upstreams ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoUpstreamServer {
    pub address: String,
    pub resolve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoClientUpstream {
    pub name: String,
    pub lb_policy: String,
    pub servers: Vec<ProtoUpstreamServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoClientGroup {
    pub group_id: String,
    pub config_version: String,
    pub upstreams: Vec<ProtoClientUpstream>,
}

// ── Egress ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoEgressVhostRule {
    pub match_host: String,
    pub action_upstream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoEgressUpstreamDef {
    pub name: String,
    pub lb_policy: String,
    pub servers: Vec<ProtoUpstreamServer>,
}

// ── Token cache ───────────────────────────────────────────────────────────────

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

// ── Watch protocol ────────────────────────────────────────────────────────────

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
    pub ingress_listeners: Vec<ProtoIngressListener>,
    pub client_groups: Vec<ProtoClientGroup>,
    pub egress_upstreams: Vec<ProtoEgressUpstreamDef>,
    pub egress_vhost_rules: Vec<ProtoEgressVhostRule>,
    /// Flattened token table for server-side in-memory auth cache.
    pub token_cache: Vec<TokenCacheEntry>,
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
