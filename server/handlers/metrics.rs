use crate::metrics;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::info;

pub async fn run_metrics_server(port: u16, ready: Arc<AtomicBool>) -> Result<()> {
    metrics::init();
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!(addr = %addr, "metrics server started");
    loop {
        let (mut stream, _) = listener.accept().await?;
        let ready = ready.clone();
        crate::spawn_task(async move {
            let mut buf = [0u8; 512];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
            let is_health = req.starts_with("GET /healthz");
            let (status, body) = if is_health {
                if ready.load(Ordering::Acquire) {
                    ("200 OK", "ok\n".to_string())
                } else {
                    ("503 Service Unavailable", "not ready\n".to_string())
                }
            } else {
                ("200 OK", metrics::encode())
            };
            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}
