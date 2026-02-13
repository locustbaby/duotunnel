use anyhow::Result;
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::metrics;

pub async fn run_metrics_server(port: u16) -> Result<()> {
    metrics::init();

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!(addr = %addr, "Prometheus metrics server started");

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let body = metrics::encode();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}
