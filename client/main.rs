#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    not(target_env = "msvc")
))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::collections::HashSet;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "dial9-telemetry")]
use std::path::PathBuf;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use tunnel_lib::{recv_message, recv_message_type, send_message, Login, LoginResp, MessageType};
mod app;
mod config;
mod conn_pool;
mod connect;
mod entry;
mod pool;
mod proxy;
use app::LocalProxyMap;
use conn_pool::EntryConnPool;
use config::ClientConfigFile;
use connect::ConnectError;
use proxy::handle_work_stream;

#[cfg(feature = "dial9-telemetry")]
static DIAL9_HANDLE: std::sync::OnceLock<dial9_tokio_telemetry::telemetry::TelemetryHandle> =
    std::sync::OnceLock::new();

pub(crate) fn spawn_task<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[cfg(feature = "dial9-telemetry")]
    if std::env::var_os("DIAL9_TRACE_PATH").is_some() {
        if let Some(handle) = DIAL9_HANDLE.get() {
            return handle.spawn(future);
        }
    }

    tokio::task::spawn(future)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config/client.yaml")]
    config: String,
}
fn main() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    #[cfg(feature = "dial9-telemetry")]
    if let Some(trace_path) = std::env::var_os("DIAL9_TRACE_PATH").map(PathBuf::from) {
        return run_with_dial9(trace_path, async_main());
    }
    run_with_tokio(async_main())
}
async fn run_healthz_server(port: u16, ready: Arc<AtomicBool>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            warn!(addr = %addr, error = %e, "failed to bind healthz server");
            return;
        }
    };
    info!(addr = %addr, "healthz server started");
    loop {
        let Ok((mut stream, _)) = listener.accept().await else { continue };
        let ready = ready.clone();
        crate::spawn_task(async move {
            let mut buf = [0u8; 256];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
            let (status, body) = if req.starts_with("GET /healthz") {
                if ready.load(Ordering::Acquire) {
                    ("200 OK", "ok\n")
                } else {
                    ("503 Service Unavailable", "not ready\n")
                }
            } else {
                ("200 OK", "ok\n")
            };
            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}

async fn async_main() -> Result<()> {
    let args = Args::parse();
    let config = ClientConfigFile::load(&args.config)?;
    let log_level = config.log_level.as_deref().unwrap_or("info");
    tunnel_lib::infra::observability::init_tracing(log_level);
    info!("Starting DuoTunnel Client");
    info!(server = % config.server_address(), "Configuration loaded");
    let endpoint = build_quic_endpoint(&config)?;
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    crate::spawn_task(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Received Ctrl+C, shutting down...");
            cancel_clone.cancel();
        }
    });
    let ready = Arc::new(AtomicBool::new(false));
    if let Some(port) = config.metrics_port {
        crate::spawn_task(run_healthz_server(port, ready.clone()));
    }

    let entry_pool = EntryConnPool::new();

    if let Some(entry_port) = config.http_entry_port {
        let pool = entry_pool.clone();
        let token = cancel.clone();
        let max_entry_conns = config.quic.max_concurrent_streams;
        let entry_tcp_params = tunnel_lib::TcpParams::from(&config.tcp);
        let peek_buf_size = config.proxy_buffers.peek_buf_size;
        let open_stream_timeout = Duration::from_millis(config.reconnect.open_stream_timeout_ms);
        let accept_workers = config.entry_accept_workers.unwrap_or(tunnel_lib::DEFAULT_ACCEPT_WORKERS).max(1);
        crate::spawn_task(async move {
            if let Err(e) = entry::start_entry_listener(
                pool,
                entry_port,
                token,
                max_entry_conns,
                entry_tcp_params,
                peek_buf_size,
                open_stream_timeout,
                accept_workers,
            )
            .await
            {
                error!(port = entry_port, error = % e, "entry listener failed");
            }
        });
    }

    let run_cancel = cancel.clone();
    let run_ready = ready.clone();
    let run_fut = async move {
        if config.quic.connections > 1 {
            info!(
                connections = % config.quic.connections, "using multi-QUIC connection pool"
            );
            pool::run_pool(config, endpoint, run_cancel, run_ready, entry_pool).await
        } else {
            connect::run_supervisor(config, endpoint, run_cancel, run_ready, entry_pool).await
        }
    };
    run_fut.await
}
fn run_with_tokio(fut: impl Future<Output = Result<()>>) -> Result<()> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    tunnel_lib::apply_worker_threads(&mut builder);
    builder.enable_all();
    let runtime = builder.build()?;
    runtime.block_on(fut)
}
#[cfg(feature = "dial9-telemetry")]
fn run_with_dial9(trace_path: PathBuf, fut: impl Future<Output = Result<()>>) -> Result<()> {
    use dial9_tokio_telemetry::telemetry::{
        CpuProfilingConfig, RotatingWriter, SchedEventConfig, TracedRuntime,
    };
    let writer = RotatingWriter::builder()
        .base_path(&trace_path)
        .max_file_size(512 * 1024 * 1024)
        .max_total_size(512 * 1024 * 1024)
        .build()?;
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    tunnel_lib::apply_worker_threads(&mut builder);
    let trace_path_display = trace_path.display().to_string();
    let trace_file_display = {
        let stem = trace_path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("trace");
        trace_path
            .with_file_name(format!("{stem}.0.bin"))
            .display()
            .to_string()
    };
    let (runtime, guard) = TracedRuntime::builder()
        .with_task_tracking(true)
        .with_trace_path(trace_path)
        .with_cpu_profiling(CpuProfilingConfig::default())
        .with_sched_events(SchedEventConfig {
            include_kernel: true,
        })
        .build_and_start_with_writer(builder, writer)?;
    let _ = DIAL9_HANDLE.set(guard.handle());
    info!("dial9 trace started, base path: {trace_path_display}");
    let result = runtime.block_on(async {
        let mut sigterm = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )?;
        tokio::select! {
            r = fut => r,
            _ = sigterm.recv() => {
                info!("SIGTERM received, starting graceful shutdown");
                Ok(())
            }
        }
    });
    info!("runtime stopped, flushing dial9 trace (timeout 30s)");
    drop(runtime);
    match guard.graceful_shutdown(std::time::Duration::from_secs(30)) {
        Ok(()) => info!("dial9 trace flush complete, output: {trace_file_display}"),
        Err(e) => error!("dial9 trace flush error: {e}"),
    }
    result
}
fn build_quic_endpoint(config: &ClientConfigFile) -> Result<quinn::Endpoint> {
    let mut crypto = build_tls_config(config)?;
    crypto.alpn_protocols = vec![b"tunnel-quic".to_vec()];
    let mut client_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?,
    ));
    let quic_params = tunnel_lib::QuicTransportParams::from(&config.quic);
    let transport_config = tunnel_lib::build_transport_config(&quic_params);
    client_config.transport_config(transport_config);
    debug!(
        max_streams = % quic_params.max_concurrent_streams, keepalive_secs = %
        quic_params.keepalive_secs, congestion = ? quic_params.congestion,
        "QUIC transport configured"
    );
    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}
pub(crate) async fn run_client(
    config: &ClientConfigFile,
    endpoint: &quinn::Endpoint,
    shutdown: CancellationToken,
    ready: Arc<AtomicBool>,
    entry_pool: Arc<EntryConnPool>,
) -> std::result::Result<(), ConnectError> {
    let conn = connect_to_server(config, endpoint).await?;
    info!("Connected to server");
    let login_timeout = Duration::from_millis(config.reconnect.login_timeout_ms);
    let (mut send, mut recv) = timeout(login_timeout, conn.open_bi())
        .await
        .map_err(|_| ConnectError::transient(anyhow!("open_bi timed out")))?
        .map_err(|e| ConnectError::transient(anyhow!("failed to open login stream: {}", e)))?;
    let login = Login {
        token: config.auth_token.clone(),
    };
    timeout(
        login_timeout,
        send_message(&mut send, MessageType::Login, &login),
    )
    .await
    .map_err(|_| ConnectError::transient(anyhow!("sending login timed out")))?
    .map_err(|e| ConnectError::transient(anyhow!("failed to send login: {}", e)))?;
    debug!("Login message sent");
    let msg_type = timeout(login_timeout, recv_message_type(&mut recv))
        .await
        .map_err(|_| ConnectError::transient(anyhow!("waiting login response timed out")))?
        .map_err(|e| {
            ConnectError::transient(anyhow!("failed to read login response type: {}", e))
        })?;
    if msg_type != MessageType::LoginResp {
        return Err(ConnectError::fatal(anyhow!(
            "protocol mismatch: expected LoginResp, got {:?}",
            msg_type
        )));
    }
    let resp: LoginResp = timeout(login_timeout, recv_message(&mut recv))
        .await
        .map_err(|_| ConnectError::transient(anyhow!("reading LoginResp payload timed out")))?
        .map_err(|e| ConnectError::transient(anyhow!("failed to decode LoginResp: {}", e)))?;
    if !resp.success {
        return Err(connect::classify_login_failure(resp.error.as_deref()));
    }
    info!(
        client_group = %resp.client_group,
        upstreams = resp.config.upstreams.len(),
        "Login successful, config received"
    );
    let proxy_map = Arc::new(LocalProxyMap::from_config(
        &resp.config,
        &tunnel_lib::HttpClientParams::from(&config.http_pool),
    ));
    let session_cancel = CancellationToken::new();
    let max_streams = config.quic.max_concurrent_streams;
    let stream_semaphore = Arc::new(Semaphore::new(max_streams as usize));
    info!(max_concurrent_streams = % max_streams, "Stream backpressure configured");
    let tcp_params = tunnel_lib::TcpParams::from(&config.tcp);

    entry_pool.push(conn.clone());
    ready.store(true, Ordering::Release);
    let result = loop {
        tokio::select! {
            reason = conn.closed() => {
                warn!(reason = ?reason, "Connection closed by server");
                break Err(ConnectError::transient(anyhow!("connection closed by server: {:?}", reason)));
            }
            stream_result = conn.accept_bi() => {
                match stream_result {
                    Ok((mut send, recv)) => {
                        let permit = match stream_semaphore.clone().try_acquire_owned() {
                            Ok(permit) => permit,
                            Err(_) => {
                                warn!("Stream rejected: max concurrent streams reached");
                                // Signal the server that we cannot accept this stream.
                                let _ = send.reset(0u8.into());
                                continue;
                            }
                        };
                        debug!("Accepted work stream from server");
                        let proxy_map = proxy_map.clone();
                        let tcp_params = tcp_params.clone();
                        crate::spawn_task(async move {
                            let _permit = permit;
                            if let Err(e) = handle_work_stream(send, recv, proxy_map, tcp_params).await {
                                debug!(error = %e, "work stream error");
                            }
                        });
                    }
                    Err(e) => {
                        warn!(error = %e, "Connection error");
                        break Err(ConnectError::transient(anyhow!("accept_bi failed: {}", e)));
                    }
                }
            }
        }
    };
    session_cancel.cancel();
    entry_pool.remove(&conn);
    tokio::select! {
        _ = shutdown.cancelled() => {}
        _ = tokio::time::sleep(Duration::from_millis(config.reconnect.grace_ms)) => {}
    }
    result
}
async fn connect_to_server(
    config: &ClientConfigFile,
    endpoint: &quinn::Endpoint,
) -> std::result::Result<quinn::Connection, ConnectError> {
    let addrs = resolve_server_addresses(config).await?;
    let connect_timeout = Duration::from_millis(config.reconnect.connect_timeout_ms);
    let sni = config.tls_server_name().to_string();
    let mut errors = Vec::new();
    for addr in addrs {
        info!(server_addr = % addr, sni = % sni, "Connecting to server");
        let connecting = endpoint
            .connect(addr, &sni)
            .map_err(|e| ConnectError::transient(anyhow!("connect setup failed: {}", e)))?;
        match timeout(connect_timeout, connecting).await {
            Ok(Ok(conn)) => return Ok(conn),
            Ok(Err(e)) => {
                errors.push(format!("{}: {}", addr, e));
            }
            Err(_) => {
                errors.push(format!("{}: connect timeout", addr));
            }
        }
    }
    Err(ConnectError::transient(anyhow!(
        "all connection attempts failed ({})",
        errors.join("; ")
    )))
}
async fn resolve_server_addresses(
    config: &ClientConfigFile,
) -> std::result::Result<Vec<SocketAddr>, ConnectError> {
    if let Ok(ip) = config.server_addr.parse::<IpAddr>() {
        return Ok(vec![SocketAddr::new(ip, config.server_port)]);
    }
    let resolve_timeout = Duration::from_millis(config.reconnect.resolve_timeout_ms);
    let host = config.server_addr.clone();
    let lookup = tokio::net::lookup_host((host.as_str(), config.server_port));
    let resolved = timeout(resolve_timeout, lookup)
        .await
        .map_err(|_| {
            ConnectError::transient(anyhow!(
                "DNS resolve timed out for {}:{}",
                host,
                config.server_port
            ))
        })?
        .map_err(|e| {
            ConnectError::transient(anyhow!(
                "DNS resolve failed for {}:{}: {}",
                host,
                config.server_port,
                e
            ))
        })?;
    let mut seen = HashSet::new();
    let mut addrs = Vec::new();
    for addr in resolved {
        if seen.insert(addr) {
            addrs.push(addr);
        }
    }
    if addrs.is_empty() {
        return Err(ConnectError::transient(anyhow!(
            "no resolved addresses for {}:{}",
            host,
            config.server_port
        )));
    }
    Ok(addrs)
}
fn build_tls_config(config: &ClientConfigFile) -> Result<rustls::ClientConfig> {
    if config.tls_skip_verify {
        warn!("TLS certificate verification is DISABLED - this is insecure!");
        return Ok(rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(InsecureServerCertVerifier))
            .with_no_client_auth());
    }
    let mut root_store = rustls::RootCertStore::empty();
    if let Some(ca_path) = &config.tls_ca_cert {
        let ca_file = std::fs::File::open(ca_path)
            .map_err(|e| anyhow::anyhow!("Failed to open CA cert file {}: {}", ca_path, e))?;
        let mut reader = std::io::BufReader::new(ca_file);
        let certs = rustls_pemfile::certs(&mut reader)
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        if certs.is_empty() {
            return Err(anyhow::anyhow!(
                "No valid certificates found in {}",
                ca_path
            ));
        }
        for cert in certs {
            root_store
                .add(cert)
                .map_err(|e| anyhow::anyhow!("Failed to add CA cert: {}", e))?;
        }
        info!(ca_path = % ca_path, "Loaded custom CA certificate");
    } else {
        let native_certs = rustls_native_certs::load_native_certs();
        for cert in native_certs.certs {
            let _ = root_store.add(cert);
        }
        if root_store.is_empty() {
            if config.allow_insecure_fallback {
                warn!("No system root certificates found, falling back to insecure mode");
                return Ok(rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(InsecureServerCertVerifier))
                    .with_no_client_auth());
            }
            return Err(
                anyhow!(
                    "No system root certificates found; set tls_ca_cert or explicitly enable allow_insecure_fallback"
                ),
            );
        }
        debug!("Loaded {} system root certificates", root_store.len());
    }
    Ok(rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth())
}
#[derive(Debug)]
struct InsecureServerCertVerifier;
impl rustls::client::danger::ServerCertVerifier for InsecureServerCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
