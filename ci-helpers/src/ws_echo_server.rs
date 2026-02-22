/// Minimal WebSocket echo server for CI integration tests.
///
/// Listens on 127.0.0.1:<port> (default 8765).
/// Every text or binary frame received is echoed back verbatim.
///
/// Usage: ws-echo-server [port]
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(8765);

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let listener = TcpListener::bind(&addr).await?;
    eprintln!("WebSocket echo server listening on ws://{}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(mut ws) => {
                    eprintln!("WS client connected: {}", peer);
                    while let Some(msg) = ws.next().await {
                        match msg {
                            Ok(m) if m.is_text() || m.is_binary() => {
                                if ws.send(m).await.is_err() {
                                    break;
                                }
                            }
                            Ok(m) if m.is_close() => break,
                            _ => {}
                        }
                    }
                }
                Err(e) => eprintln!("WS handshake error from {}: {}", peer, e),
            }
        });
    }
}
