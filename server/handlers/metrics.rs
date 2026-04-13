use crate::metrics;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Request, Response};
use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub async fn run_metrics_server(
    port: u16,
    ready: Arc<AtomicBool>,
    shutdown: CancellationToken,
) -> Result<()> {
    tunnel_lib::run_http1_server(port, shutdown, "metrics server", move |req: Request<Incoming>| {
        let ready = ready.clone();
        async move { handle_request(req, &ready).await }
    })
    .await
}

async fn handle_request(
    req: Request<Incoming>,
    ready: &Arc<AtomicBool>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if req.uri().path() == "/healthz" {
        let is_ready = ready.load(Ordering::Acquire);
        let status = if is_ready { 200 } else { 503 };
        let body = if is_ready { "ok\n" } else { "not ready\n" };
        Ok(Response::builder()
            .status(status)
            .body(Full::new(Bytes::from(body)))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from(metrics::encode())))
            .unwrap())
    }
}
