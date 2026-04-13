use crate::metrics;
use anyhow::Result;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn run_metrics_server(
    port: u16,
    ready: Arc<AtomicBool>,
    shutdown: CancellationToken,
) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!(port, "metrics server started");
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            result = listener.accept() => {
                let (stream, _) = result?;
                let ready = ready.clone();
                tokio::task::spawn(async move {
                    let io = TokioIo::new(stream);
                    let _ = http1::Builder::new()
                        .keep_alive(true)
                        .serve_connection(
                            io,
                            service_fn(move |req| {
                                let ready = ready.clone();
                                async move { handle_request(req, &ready).await }
                            }),
                        )
                        .await;
                });
            }
        }
    }
    Ok(())
}

async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    ready: &Arc<AtomicBool>,
) -> Result<hyper::Response<Full<bytes::Bytes>>, std::convert::Infallible> {
    if req.uri().path() == "/healthz" {
        let (status, body) = if ready.load(Ordering::Acquire) {
            (200u16, "ok\n")
        } else {
            (503u16, "not ready\n")
        };
        Ok(hyper::Response::builder()
            .status(status)
            .body(Full::new(bytes::Bytes::from(body)))
            .unwrap())
    } else {
        let body = metrics::encode();
        Ok(hyper::Response::builder()
            .status(200)
            .header("content-type", "text/plain; charset=utf-8")
            .body(Full::new(bytes::Bytes::from(body)))
            .unwrap())
    }
}
