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
                        tokio::spawn(on_conn(AcceptedConn {
                            stream,
                            peer_addr,
                            accepted_at: Instant::now(),
                            listener_tag: tag,
                        }));
                    }
                    Err(e) => {
                        let os_err = e.raw_os_error();
                        if os_err == Some(24) || os_err == Some(23) {
                            warn!(tag = tag, "accept: too many open files, backing off");
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
