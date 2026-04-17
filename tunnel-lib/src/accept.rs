use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

/// Run a single TCP accept worker.  Loops until `cancel` fires, calling
/// `on_conn` for each accepted connection.  On EMFILE (errno 24) backs
/// off for `emfile_backoff`; on other accept errors logs and continues.
///
/// `on_conn` is a synchronous callback — it is expected to spawn its
/// own task (whichever flavour the caller wants: `tokio::spawn`,
/// `crate::spawn_task`, a telemetry-aware wrapper, …) and return
/// quickly.  Blocking inside the callback will stall the accept loop.
///
/// Callers needing N parallel accept workers should spawn this function
/// N times, each cloning the shared `Arc<TcpListener>` (built with
/// `SO_REUSEPORT` for cross-worker load balancing).
pub async fn run_accept_worker<H>(
    listener: Arc<TcpListener>,
    cancel: CancellationToken,
    emfile_backoff: Duration,
    tag: &'static str,
    on_conn: H,
) where
    H: Fn(TcpStream, SocketAddr),
{
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                debug!(tag = tag, "accept worker cancelled");
                return;
            }
            result = listener.accept() => {
                match result {
                    Ok((stream, peer_addr)) => {
                        debug!(tag = tag, peer_addr = %peer_addr, "accepted connection");
                        on_conn(stream, peer_addr);
                    }
                    Err(e) => {
                        if e.raw_os_error() == Some(24) {
                            warn!(tag = tag, "accept: too many open files, backing off");
                            tokio::time::sleep(emfile_backoff).await;
                        } else {
                            warn!(tag = tag, error = %e, "accept error");
                        }
                    }
                }
            }
        }
    }
}
