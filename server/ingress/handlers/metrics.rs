use crate::runtime::metrics;
use anyhow::Result;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::info;

const METRICS_MAX_CONNECTIONS: usize = 64;
const METRICS_IO_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn run_metrics_server(
    port: u16,
    health: Arc<crate::runtime::health::ServerHealthFacts>,
    shutdown: CancellationToken,
) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let capacity = Arc::new(Semaphore::new(METRICS_MAX_CONNECTIONS));
    info!(port, "metrics server started");
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            result = listener.accept() => {
                let (stream, _) = result?;
                let Ok(permit) = capacity.clone().try_acquire_owned() else {
                    continue;
                };
                let health = health.clone();
                tokio::task::spawn(async move {
                    let _permit = permit;
                    let io = TokioIo::new(stream);
                    let connection = http1::Builder::new()
                        .keep_alive(false)
                        .serve_connection(io, service_fn(move |req| {
                            let health = health.clone();
                            async move { handle_request(req, &health).await }
                        }));
                    let _ = tokio::time::timeout(METRICS_IO_TIMEOUT, connection).await;
                });
            }
        }
    }
    Ok(())
}

async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    health: &Arc<crate::runtime::health::ServerHealthFacts>,
) -> Result<hyper::Response<Full<bytes::Bytes>>, std::convert::Infallible> {
    if req.uri().path() == "/healthz" {
        let (status, body) = if health.is_ready() {
            match health.control_freshness() {
                crate::runtime::health::ControlFreshness::Degraded => (200u16, "degraded\n"),
                _ => (200u16, "ok\n"),
            }
        } else {
            (503u16, "not ready\n")
        };
        Ok(hyper::Response::builder()
            .status(status)
            .body(Full::new(bytes::Bytes::from(body)))
            .unwrap())
    } else if req.uri().path() == "/metrics" {
        let body = metrics::encode();
        Ok(hyper::Response::builder()
            .status(200)
            .header("content-type", "text/plain; charset=utf-8")
            .body(Full::new(bytes::Bytes::from(body)))
            .unwrap())
    } else {
        Ok(hyper::Response::builder()
            .status(404)
            .body(Full::new(bytes::Bytes::from_static(b"not found\n")))
            .unwrap())
    }
}
