/// Minimal HTTP/1.1 + HTTP/2 echo server for CI integration tests.
///
/// Returns a JSON body echoing the request method, path, and all headers.
/// Listens on 127.0.0.1:<port> (default 9999).
///
/// Usage: http-echo-server [port]
use std::convert::Infallible;
use std::net::SocketAddr;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;

async fn handle(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();

    // Collect headers as a JSON object
    let headers: serde_json::Value = req
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_string(),
                serde_json::Value::String(v.to_str().unwrap_or("").to_string()),
            )
        })
        .collect::<serde_json::Map<String, serde_json::Value>>()
        .into();

    // Read body (up to 64 KB)
    let body_bytes = req
        .into_body()
        .collect()
        .await
        .map(|c| c.to_bytes())
        .unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes).into_owned();

    let json = serde_json::json!({
        "method": method,
        "path": path,
        "query": query,
        "headers": headers,
        "body": body_str,
        "server": "http-echo-server"
    });
    let body = json.to_string();

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Content-Length", body.len().to_string())
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9999);

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let listener = TcpListener::bind(&addr).await?;
    eprintln!("HTTP echo server listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::spawn(async move {
            if let Err(e) = Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(io, hyper::service::service_fn(handle))
                .await
            {
                // Connection closed by client is normal; suppress those errors.
                let msg = e.to_string();
                if !msg.contains("connection closed") && !msg.contains("broken pipe") {
                    eprintln!("HTTP echo connection error: {}", e);
                }
            }
        });
    }
}
