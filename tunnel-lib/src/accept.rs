use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

#[derive(Debug)]
pub struct AcceptedConn {
    pub stream: TcpStream,
    pub peer_addr: SocketAddr,
    pub accepted_at: Instant,
    pub listener_tag: &'static str,
}

pub async fn run_accept_worker<H, Fut>(
    listener: Arc<TcpListener>,
    cancel: CancellationToken,
    emfile_backoff: Duration,
    tag: &'static str,
    on_conn: H,
) where
    H: Fn(AcceptedConn) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = ()> + Send + 'static,
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
                        crate::infra::metrics::METRICS.accepted_connections_active.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let on_conn = on_conn.clone();
                        tokio::spawn(async move {
                            let _guard = crate::infra::metrics::ConnActiveGuard;
                            on_conn(AcceptedConn {
                                stream,
                                peer_addr,
                                accepted_at: Instant::now(),
                                listener_tag: tag,
                            }).await;
                        });
                    }
                    Err(e) => {
                        let os_err = e.raw_os_error();
                        if os_err == Some(24) || os_err == Some(23) {
                            warn!(
                                tag = tag,
                                error = %e,
                                "Accept worker stalled due to FD exhaustion (EMFILE / errno 24). \
                                Pausing accept loop for {}ms to prevent CPU thrashing. \
                                Raise file descriptor limits immediately to recover (e.g. run 'ulimit -n 65536').",
                                emfile_backoff.as_millis()
                            );
                            tokio::select! {
                                _ = cancel.cancelled() => {
                                    debug!(tag = tag, "accept worker cancelled during backoff");
                                    return;
                                }
                                _ = tokio::time::sleep(emfile_backoff) => {}
                            }
                        } else {
                            warn!(tag = tag, error = %e, "accept error");
                        }
                    }
                }
            }
        }
    }
}
