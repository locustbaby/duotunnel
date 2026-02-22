/// CI test client — replaces wscat / grpcurl / curl in GitHub Actions.
///
/// Usage:
///   ci-test-client http  <url> [--method GET|POST] [--body <json>] [--header Key:Value]
///   ci-test-client http2 <url>
///   ci-test-client ws    <url> [--message <text>]
///   ci-test-client grpc  <host:port> [--service <svc>]
///
/// Exit code 0 = PASS, 1 = FAIL.

use std::process;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

// ─── main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: ci-test-client <http|http2|ws|grpc> <target> [options...]");
        process::exit(2);
    }

    let subcommand = args[1].as_str();
    let target = &args[2];
    let rest = &args[3..];

    let result = match subcommand {
        "http"  => run_http(target, rest, false).await,
        "http2" => run_http(target, rest, true).await,
        "ws"    => run_ws(target, rest).await,
        "grpc"  => run_grpc(target, rest).await,
        other   => Err(format!("unknown subcommand: {other}")),
    };

    match result {
        Ok(msg) => { println!("PASS: {msg}"); process::exit(0); }
        Err(msg) => { eprintln!("FAIL: {msg}"); process::exit(1); }
    }
}

// ─── HTTP / HTTP2 ─────────────────────────────────────────────────────────────

async fn run_http(url: &str, args: &[String], force_h2: bool) -> Result<String, String> {
    let mut method = "GET".to_string();
    let mut body_str: Option<String> = None;
    let mut extra_headers: Vec<(String, String)> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--method" => { i += 1; method = args[i].clone(); }
            "--body"   => { i += 1; body_str = Some(args[i].clone()); }
            "--header" => {
                i += 1;
                let kv = &args[i];
                let colon = kv.find(':').ok_or_else(|| format!("bad header: {kv}"))?;
                extra_headers.push((kv[..colon].to_string(), kv[colon+1..].trim().to_string()));
            }
            _ => {}
        }
        i += 1;
    }

    // Build request
    let uri: hyper::Uri = url.parse().map_err(|e| format!("bad url: {e}"))?;
    let host = uri.host().unwrap_or("localhost").to_string();
    let port = uri.port_u16().unwrap_or(80);
    let addr = format!("{host}:{port}");

    let stream = tokio::time::timeout(
        Duration::from_secs(15),
        tokio::net::TcpStream::connect(&addr),
    ).await
    .map_err(|_| format!("connect timeout to {addr}"))?
    .map_err(|e| format!("connect error: {e}"))?;

    let _ = stream.set_nodelay(true);

    let body_bytes = body_str.map(|s| s.into_bytes()).unwrap_or_default();

    if force_h2 {
        use hyper_util::rt::TokioIo;
        use hyper::client::conn::http2;
        use http_body_util::Full;
        use bytes::Bytes;

        let io = TokioIo::new(stream);
        let exec = hyper_util::rt::TokioExecutor::new();
        let (mut sender, conn) = http2::handshake(exec, io).await
            .map_err(|e| format!("h2 handshake: {e}"))?;
        tokio::spawn(conn);

        let mut req = hyper::Request::builder()
            .method(method.as_str())
            .uri(url)
            .header("host", &host)
            .header("content-type", "application/json");
        for (k, v) in &extra_headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let req = req
            .body(Full::new(Bytes::from(body_bytes)))
            .map_err(|e| format!("build request: {e}"))?;

        let resp = tokio::time::timeout(Duration::from_secs(15), sender.send_request(req))
            .await
            .map_err(|_| "request timeout".to_string())?
            .map_err(|e| format!("request error: {e}"))?;

        let status = resp.status().as_u16();
        let body = read_body(resp.into_body()).await?;
        validate_response(status, &body, "HTTP/2")
    } else {
        use hyper_util::rt::TokioIo;
        use hyper::client::conn::http1;
        use http_body_util::Full;
        use bytes::Bytes;

        let io = TokioIo::new(stream);
        let (mut sender, conn) = http1::handshake(io).await
            .map_err(|e| format!("h1 handshake: {e}"))?;
        tokio::spawn(conn);

        let path = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");
        let mut req = hyper::Request::builder()
            .method(method.as_str())
            .uri(path)
            .header("host", &host)
            .header("content-type", "application/json");
        for (k, v) in &extra_headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let req = req
            .body(Full::new(Bytes::from(body_bytes)))
            .map_err(|e| format!("build request: {e}"))?;

        let resp = tokio::time::timeout(Duration::from_secs(15), sender.send_request(req))
            .await
            .map_err(|_| "request timeout".to_string())?
            .map_err(|e| format!("request error: {e}"))?;

        let status = resp.status().as_u16();
        let body = read_body(resp.into_body()).await?;
        validate_response(status, &body, "HTTP/1.1")
    }
}

async fn read_body<B>(body: B) -> Result<String, String>
where
    B: hyper::body::Body<Data = bytes::Bytes>,
    B::Error: std::fmt::Display,
{
    use http_body_util::BodyExt;
    let collected = tokio::time::timeout(
        Duration::from_secs(10),
        body.collect(),
    ).await
    .map_err(|_| "body read timeout".to_string())?
    .map_err(|e| format!("body read error: {e}"))?;
    Ok(String::from_utf8_lossy(&collected.to_bytes()).into_owned())
}

fn validate_response(status: u16, body: &str, proto: &str) -> Result<String, String> {
    if status != 200 {
        return Err(format!("{proto}: unexpected status {status}, body: {}", &body[..body.len().min(200)]));
    }
    // The echo server returns JSON with a "method" field
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(v) if v.get("method").is_some() => {
            let method = v["method"].as_str().unwrap_or("?");
            Ok(format!("{proto} {status} method={method}"))
        }
        Ok(_) => Ok(format!("{proto} {status} (non-echo response, len={})", body.len())),
        Err(_) => Ok(format!("{proto} {status} (non-JSON, len={})", body.len())),
    }
}

// ─── WebSocket ────────────────────────────────────────────────────────────────

async fn run_ws(url: &str, args: &[String]) -> Result<String, String> {
    let mut message = "hello-ci-ws".to_string();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--message" { i += 1; message = args[i].clone(); }
        i += 1;
    }

    let (mut ws, _) = tokio::time::timeout(
        Duration::from_secs(15),
        tokio_tungstenite::connect_async(url),
    ).await
    .map_err(|_| format!("WebSocket connect timeout to {url}"))?
    .map_err(|e| format!("WebSocket connect error: {e}"))?;

    ws.send(Message::Text(message.clone())).await
        .map_err(|e| format!("ws send error: {e}"))?;

    // Read until we see the echo or hit timeout
    let echo = tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(Message::Text(t)) => {
                    if t.contains(&message) { return Ok(t.to_string()); }
                }
                Ok(Message::Binary(b)) => {
                    let s = String::from_utf8_lossy(&b).into_owned();
                    if s.contains(&message) { return Ok(s); }
                }
                Ok(Message::Close(_)) => return Err("server closed connection before echo".to_string()),
                Ok(_) => {}
                Err(e) => return Err(format!("ws recv error: {e}")),
            }
        }
        Err("connection closed without echo".to_string())
    }).await
    .map_err(|_| "WebSocket echo timeout".to_string())??;

    let _ = ws.close(None).await;
    Ok(format!("echo received: {}", &echo[..echo.len().min(80)]))
}

// ─── gRPC Health ──────────────────────────────────────────────────────────────
//
// Speaks gRPC-over-HTTP/2 natively via the tonic client.
// We use tonic::transport::Channel with a plaintext endpoint.

async fn run_grpc(addr: &str, args: &[String]) -> Result<String, String> {
    let mut service = String::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--service" { i += 1; service = args[i].clone(); }
        i += 1;
    }

    let endpoint = format!("http://{addr}");
    let channel = tokio::time::timeout(
        Duration::from_secs(15),
        tonic::transport::Channel::from_shared(endpoint)
            .map_err(|e| format!("invalid gRPC endpoint: {e}"))?
            .connect(),
    ).await
    .map_err(|_| format!("gRPC connect timeout to {addr}"))?
    .map_err(|e| format!("gRPC connect error: {e}"))?;

    let mut client = tonic_health::pb::health_client::HealthClient::new(channel);
    let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
        service: service.clone(),
    });

    let resp = tokio::time::timeout(
        Duration::from_secs(15),
        client.check(request),
    ).await
    .map_err(|_| "gRPC Health/Check timeout".to_string())?
    .map_err(|e| format!("gRPC Health/Check error: status={} message={}", e.code(), e.message()))?;

    let status = resp.into_inner().status;
    // ServingStatus::Serving == 1
    if status == 1 {
        Ok(format!("Health/Check service={:?} → SERVING", service))
    } else {
        Err(format!("Health/Check returned non-SERVING status: {status}"))
    }
}
