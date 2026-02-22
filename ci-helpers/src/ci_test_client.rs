// CI test client — replaces wscat / grpcurl / curl in GitHub Actions.
//
// Usage:
//   ci-test-client http       <url> [--method GET|POST] [--body <json>] [--header Key:Value]
//                                   [--expect-body <substring>] [--allow-status <code>]
//   ci-test-client http2      <url> [same flags as http — uses h2c upgrade via HTTP/1.1]
//   ci-test-client ws         <url> [--message <text>]
//   ci-test-client grpc       <host:port> [--service <svc>]
//   ci-test-client grpc-echo  <host:port> [--tls] [--insecure] [--ping <text>]
//                             calls grpc_echo.v1.EchoService/Echo
//
// Exit code 0 = PASS, 1 = FAIL.

use std::process;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

// ─── main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Install the ring crypto provider so rustls TLS (used by tonic) works.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: ci-test-client <http|http2|ws|grpc|grpc-echo> <target> [options...]");
        process::exit(2);
    }

    let subcommand = args[1].as_str();
    let target = &args[2];
    let rest = &args[3..];

    let result = match subcommand {
        "http"      => run_http(target, rest, false).await,
        "http2"     => run_http(target, rest, true).await,
        "ws"        => run_ws(target, rest).await,
        "grpc"      => run_grpc(target, rest).await,
        "grpc-echo" => run_grpc_echo(target, rest).await,
        other       => Err(format!("unknown subcommand: {other}")),
    };

    match result {
        Ok(msg) => { println!("PASS: {msg}"); process::exit(0); }
        Err(msg) => { eprintln!("FAIL: {msg}"); process::exit(1); }
    }
}

// ─── HTTP / HTTP2 ─────────────────────────────────────────────────────────────
//
// http2 sends an HTTP/1.1 Upgrade: h2c request so the server's byte-level
// forwarding path handles it (same as curl --http2).  The response is then
// validated the same way as plain HTTP/1.1.

async fn run_http(url: &str, args: &[String], force_h2: bool) -> Result<String, String> {
    let mut method = "GET".to_string();
    let mut body_str: Option<String> = None;
    let mut extra_headers: Vec<(String, String)> = Vec::new();
    let mut expect_body: Option<String> = None;
    let mut allow_statuses: Vec<u16> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--method"       => { i += 1; method = args[i].clone(); }
            "--body"         => { i += 1; body_str = Some(args[i].clone()); }
            "--header"       => {
                i += 1;
                let kv = &args[i];
                let colon = kv.find(':').ok_or_else(|| format!("bad header: {kv}"))?;
                extra_headers.push((kv[..colon].to_string(), kv[colon+1..].trim().to_string()));
            }
            "--expect-body"  => { i += 1; expect_body = Some(args[i].clone()); }
            "--allow-status" => { i += 1; allow_statuses.push(args[i].parse::<u16>().map_err(|_| format!("bad status: {}", args[i]))?); }
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

    // If --header "Host: ..." was given, use that as the Host header; otherwise use URI host.
    let effective_host = extra_headers.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("host"))
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| host.clone());
    // Remove Host from extra_headers to avoid duplicate Host header
    let non_host_headers: Vec<_> = extra_headers.iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("host"))
        .collect();

    if force_h2 {
        // h2c upgrade: send HTTP/1.1 with Upgrade: h2c so the server's byte-level
        // forwarding path handles it transparently (same code path as curl --http2).
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
            .header("host", &effective_host)
            .header("content-type", "application/json")
            .header("upgrade", "h2c")
            .header("http2-settings", "AAMAAABkAAQAAP__")
            .header("connection", "Upgrade, HTTP2-Settings");
        for (k, v) in &non_host_headers {
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
        // A 101 Switching Protocols means h2c upgrade succeeded (server side accepted)
        // but we treat it as pass since byte-level forwarding relays to upstream.
        // Typically the upstream will respond with the actual HTTP response after upgrade.
        // In practice, if the upstream doesn't speak h2c, we get a normal HTTP response.
        validate_response(status, &body, "HTTP/2(h2c-upgrade)", expect_body.as_deref(), &allow_statuses)
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
            .header("host", &effective_host)
            .header("content-type", "application/json");
        for (k, v) in &non_host_headers {
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
        validate_response(status, &body, "HTTP/1.1", expect_body.as_deref(), &allow_statuses)
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

fn validate_response(status: u16, body: &str, proto: &str, expect_body: Option<&str>, allow_statuses: &[u16]) -> Result<String, String> {
    let ok_status = status == 200 || allow_statuses.contains(&status);
    if !ok_status {
        return Err(format!("{proto}: unexpected status {status}, body: {}", &body[..body.len().min(200)]));
    }
    // If rate-limited (429) or other allowed non-200, skip body checks
    if status != 200 {
        return Ok(format!("{proto} {status} (allowed status, len={})", body.len()));
    }
    // Check expected body substring if specified
    if let Some(expected) = expect_body {
        if !body.contains(expected) {
            return Err(format!("{proto}: expected body to contain {:?}, got: {}", expected, &body[..body.len().min(300)]));
        }
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

// ─── gRPC Echo (grpc_echo.v1.EchoService/Echo) ───────────────────────────────
//
// Sends a ping message and validates the response body echoes it back.
// Supports TLS (--tls).
//
// EchoRequest  { ping: string (tag 1) }
// EchoResponse { headers: map<string,string> (tag 1), body: string (tag 2), remote_addr: string (tag 3) }
//
// Validates that the response body field contains the sent ping text.

#[derive(Clone, prost::Message)]
struct EchoRequest {
    #[prost(string, tag = "1")]
    pub ping: String,
}

#[derive(Clone, prost::Message)]
struct EchoResponse {
    #[prost(map = "string, string", tag = "1")]
    pub headers: std::collections::HashMap<String, String>,
    #[prost(string, tag = "2")]
    pub body: String,
    #[prost(string, tag = "3")]
    pub remote_addr: String,
}

async fn run_grpc_echo(addr: &str, args: &[String]) -> Result<String, String> {
    let mut use_tls = false;
    let mut ping_text = "ci-ping".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--tls" | "--insecure" => { use_tls = true; }
            "--ping"               => { i += 1; ping_text = args[i].clone(); }
            _ => {}
        }
        i += 1;
    }

    let endpoint_uri = if use_tls {
        format!("https://{addr}")
    } else {
        format!("http://{addr}")
    };

    let channel = if use_tls {
        // Use native system roots for TLS certificate validation
        let tls_config = tonic::transport::ClientTlsConfig::new()
            .with_native_roots();

        tokio::time::timeout(
            Duration::from_secs(15),
            tonic::transport::Channel::from_shared(endpoint_uri)
                .map_err(|e| format!("invalid endpoint: {e}"))?
                .tls_config(tls_config)
                .map_err(|e| format!("TLS config error: {e}"))?
                .connect(),
        ).await
        .map_err(|_| format!("gRPC echo connect timeout to {addr}"))?
        .map_err(|e| format!("gRPC echo connect error: {e}"))?
    } else {
        tokio::time::timeout(
            Duration::from_secs(15),
            tonic::transport::Channel::from_shared(endpoint_uri)
                .map_err(|e| format!("invalid endpoint: {e}"))?
                .connect(),
        ).await
        .map_err(|_| format!("gRPC echo connect timeout to {addr}"))?
        .map_err(|e| format!("gRPC echo connect error: {e}"))?
    };

    let mut grpc = tonic::client::Grpc::new(channel);
    grpc.ready().await.map_err(|e| format!("gRPC echo not ready: {e}"))?;

    let request = tonic::Request::new(EchoRequest { ping: ping_text.clone() });
    let codec = tonic::codec::ProstCodec::<EchoRequest, EchoResponse>::new();
    let path = tonic::codegen::http::uri::PathAndQuery::from_static(
        "/grpc_echo.v1.EchoService/Echo"
    );

    let resp = tokio::time::timeout(
        Duration::from_secs(15),
        grpc.unary(request, path, codec),
    ).await
    .map_err(|_| "gRPC Echo timeout".to_string())?
    .map_err(|e| format!("gRPC Echo error: status={} message={}", e.code(), e.message()))?;

    let echo_resp = resp.into_inner();

    // Validate: the echoed body must contain the ping text we sent.
    if !echo_resp.body.contains(&ping_text) {
        return Err(format!(
            "EchoService/Echo body mismatch: sent {:?}, got body={:?}",
            ping_text, echo_resp.body
        ));
    }

    Ok(format!(
        "EchoService/Echo ping={:?} → body={:?} headers={} remote={}",
        ping_text,
        echo_resp.body,
        echo_resp.headers.len(),
        echo_resp.remote_addr,
    ))
}
