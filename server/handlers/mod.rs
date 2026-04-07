pub mod http;
pub mod ingress;
pub mod metrics;
pub mod quic;
pub mod tcp;

/// How long to wait for a QUIC connection slot before rejecting.
/// HTTP/TCP connections use try_acquire (non-blocking) to avoid stalling the accept loop.
pub const SEMAPHORE_WAIT_MS: u64 = 500;
