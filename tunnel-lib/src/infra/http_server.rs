use anyhow::Result;
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn run_http1_server<F, Fut>(
    port: u16,
    shutdown: CancellationToken,
    name: &str,
    handler: F,
) -> Result<()>
where
    F: Fn(Request<Incoming>) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Result<Response<Full<Bytes>>, Infallible>> + Send + 'static,
{
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!(addr = %addr, "{} started", name);
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            result = listener.accept() => {
                let (stream, _) = result?;
                let handler = handler.clone();
                tokio::task::spawn(async move {
                    let io = TokioIo::new(stream);
                    let _ = http1::Builder::new()
                        .serve_connection(io, service_fn(handler))
                        .await;
                });
            }
        }
    }
    Ok(())
}

pub fn healthz_handler(
    ready: Arc<AtomicBool>,
) -> impl Fn(Request<Incoming>) -> std::future::Ready<Result<Response<Full<Bytes>>, Infallible>>
       + Clone
       + Send
       + 'static {
    move |req: Request<Incoming>| {
        let resp = if req.uri().path() == "/healthz" {
            let is_ready = ready.load(Ordering::Acquire);
            let status = if is_ready { 200 } else { 503 };
            let body = if is_ready { "ok\n" } else { "not ready\n" };
            Response::builder()
                .status(status)
                .body(Full::new(Bytes::from(body)))
                .unwrap()
        } else {
            Response::builder()
                .status(404)
                .body(Full::new(Bytes::new()))
                .unwrap()
        };
        std::future::ready(Ok(resp))
    }
}
