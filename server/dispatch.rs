//! Cross-thread ingress dispatch.
//!
//! The server runs N OS threads, each with its own `current_thread` tokio runtime
//! and its own `quinn::Endpoint`. A tunnel client connection lives entirely on the
//! thread it lands on (kernel SO_REUSEPORT 4-tuple hash).
//!
//! External requests arrive on ingress TCP listeners that run on the *outer*
//! `multi_thread` runtime (so listener_mgr / hot_reload remain unchanged). After
//! accept + peek + routing lookup, the listener determines which OS thread owns
//! the target tunnel client (via [`crate::ServerState::placement`]) and forwards
//! the raw socket through this dispatch channel.
//!
//! The receiving OS thread re-registers the socket on its local reactor and runs
//! the existing protocol handler — but now `open_bi()` and every subsequent poll
//! happen entirely within that thread's `current_thread` runtime, so quinn's
//! per-`Connection` state Mutex is never contended → no `futex_wait`.

use crate::handlers;
use crate::ServerState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

/// A unit of work dispatched from an ingress acceptor to an endpoint thread.
///
/// The `stream` is a raw `std::net::TcpStream` (already detached from the
/// dispatcher's reactor) so it can be moved between runtimes safely.
pub enum DispatchJob {
    Http {
        stream: std::net::TcpStream,
        peer_addr: SocketAddr,
        port: u16,
    },
    Tcp {
        stream: std::net::TcpStream,
        peer_addr: SocketAddr,
        port: u16,
        proxy_name: String,
        group_id: String,
    },
}

pub type DispatchSender = mpsc::UnboundedSender<DispatchJob>;
pub type DispatchReceiver = mpsc::UnboundedReceiver<DispatchJob>;

/// Per-thread dispatcher loop. Runs on the endpoint thread's `current_thread`
/// runtime, awaits cross-thread `DispatchJob`s and spawns the appropriate
/// handler locally.
pub async fn run_dispatcher(
    state: Arc<ServerState>,
    mut rx: DispatchReceiver,
    cancel: CancellationToken,
) {
    loop {
        let job = tokio::select! {
            _ = cancel.cancelled() => {
                debug!("dispatcher cancelled");
                return;
            }
            j = rx.recv() => match j {
                Some(j) => j,
                None => {
                    debug!("dispatch channel closed");
                    return;
                }
            },
        };
        match job {
            DispatchJob::Http { stream, peer_addr, port } => {
                let stream = match stream.set_nonblocking(true) {
                    Ok(()) => stream,
                    Err(e) => {
                        warn!(error = %e, "set_nonblocking on dispatched http stream failed");
                        continue;
                    }
                };
                let stream = match tokio::net::TcpStream::from_std(stream) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(error = %e, "TcpStream::from_std for http dispatch failed");
                        continue;
                    }
                };
                let state = state.clone();
                tokio::spawn(async move {
                    handlers::http::handle_dispatched_http(state, stream, peer_addr, port).await;
                });
            }
            DispatchJob::Tcp { stream, peer_addr, port, proxy_name, group_id } => {
                let stream = match stream.set_nonblocking(true) {
                    Ok(()) => stream,
                    Err(e) => {
                        warn!(error = %e, "set_nonblocking on dispatched tcp stream failed");
                        continue;
                    }
                };
                let stream = match tokio::net::TcpStream::from_std(stream) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(error = %e, "TcpStream::from_std for tcp dispatch failed");
                        continue;
                    }
                };
                let state = state.clone();
                tokio::spawn(async move {
                    handlers::tcp::handle_dispatched_tcp(
                        state,
                        stream,
                        peer_addr,
                        port,
                        proxy_name,
                        group_id,
                    )
                    .await;
                });
            }
        }
    }
}

/// Look up which OS thread owns the tunnel client that should serve `host` on
/// `port`. Returns `None` if no route exists or no client is currently
/// registered for the matched group.
pub fn locate_owner(state: &ServerState, port: u16, host: &str) -> Option<u32> {
    let snapshot = state.routing.load();
    let router = snapshot.http_routers.get(&port)?;
    let (group_id, _) = router.get(host)?;
    state.placement.get(group_id.as_ref()).map(|r| *r.value())
}

/// Look up the owner thread for a TCP-mode listener (route is fixed by listener
/// config, no host peek needed).
pub fn locate_owner_for_group(state: &ServerState, group_id: &str) -> Option<u32> {
    state.placement.get(group_id).map(|r| *r.value())
}
