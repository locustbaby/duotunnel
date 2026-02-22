use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{info, error, debug, warn};
use tunnel_lib::{
    MessageType, Login, LoginResp,
    send_message, recv_message, recv_message_type,
};

mod config;
mod proxy;
mod app;
mod pool;

mod entry;
mod utils;

use config::ClientConfigFile;
use proxy::handle_work_stream;
use app::LocalProxyMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config/client.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install the ring crypto provider for rustls
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let args = Args::parse();

    let config = ClientConfigFile::load(&args.config)?;

    let log_level = config.log_level.as_deref().unwrap_or("info");
    tunnel_lib::infra::observability::init_tracing(log_level);

    info!("Starting DuoTunnel Client");
    info!(
        server = %config.server_address(),
        client_id = %config.client_id,
        group_id = ?config.client_group_id,
        "Configuration loaded"
    );

    // Build the QUIC endpoint once; reuse it across reconnections to avoid
    // binding a new UDP socket (and potentially leaking fds) on every reconnect.
    let endpoint = build_quic_endpoint(&config)?;

    // Shared cancel token — fired by Ctrl+C to shut down all connection slots cleanly.
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Received Ctrl+C, shutting down...");
            cancel_clone.cancel();
        }
    });

    if config.quic.connections > 1 {
        // Multi-connection pool: each slot runs its own reconnect loop independently.
        info!(connections = %config.quic.connections, "using multi-QUIC connection pool");
        pool::run_pool(&config, &endpoint, cancel).await
    } else {
        // Single-connection mode (default): classic exponential backoff reconnect loop.
        let initial_delay = std::time::Duration::from_millis(config.reconnect.initial_delay_ms);
        let max_delay = std::time::Duration::from_millis(config.reconnect.max_delay_ms);
        let mut retry_delay = initial_delay;

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    info!("Shutdown signal received, exiting.");
                    return Ok(());
                }
                result = run_client(&config, &endpoint) => {
                    match result {
                        Ok(_) => {
                            info!("Connection closed gracefully, reconnecting immediately...");
                            retry_delay = initial_delay;
                        }
                        Err(e) => {
                            error!(error = %e, retry_in_ms = %retry_delay.as_millis(), "Connection error, reconnecting...");
                            tokio::select! {
                                _ = cancel.cancelled() => {
                                    info!("Shutdown signal received, exiting.");
                                    return Ok(());
                                }
                                _ = tokio::time::sleep(retry_delay) => {}
                            }
                            retry_delay = std::cmp::min(retry_delay * 2, max_delay);
                        }
                    }
                }
            }
        }
    }
}

fn build_quic_endpoint(config: &ClientConfigFile) -> Result<quinn::Endpoint> {
    let mut crypto = build_tls_config(config)?;
    // ALPN must match server's "tunnel-quic"
    crypto.alpn_protocols = vec![b"tunnel-quic".to_vec()];

    let mut client_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?
    ));

    // Build transport config from YAML parameters (falls back to sensible defaults).
    let quic_params = tunnel_lib::QuicTransportParams::from(&config.quic);
    let transport_config = tunnel_lib::build_transport_config(&quic_params);
    client_config.transport_config(transport_config);
    debug!(
        max_streams = %quic_params.max_concurrent_streams,
        keepalive_secs = %quic_params.keepalive_secs,
        congestion = ?quic_params.congestion,
        "QUIC transport configured"
    );

    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

pub(crate) async fn run_client(config: &ClientConfigFile, endpoint: &quinn::Endpoint) -> Result<()> {
    let server_addr = config.server_address().parse()?;
    info!(server_addr = %server_addr, "Connecting to server...");
    
    let conn = endpoint.connect(server_addr, "localhost")?.await?;

    info!("Connected to server");

    let (mut send, mut recv) = conn.open_bi().await?;
    
    let login = Login {
        client_id: config.client_id.clone(),
        group_id: config.client_group_id.clone(),
        token: config.auth_token.clone().unwrap_or_default(),
    };
    send_message(&mut send, MessageType::Login, &login).await?;
    debug!("Login message sent");

    let msg_type = recv_message_type(&mut recv).await?;
    if msg_type != MessageType::LoginResp {
        return Err(anyhow::anyhow!("Expected LoginResp, got {:?}", msg_type));
    }
    
    let resp: LoginResp = recv_message(&mut recv).await?;
    
    if !resp.success {
        return Err(anyhow::anyhow!("Login failed: {:?}", resp.error));
    }
    
    info!(
        proxies = resp.config.proxies.len(),
        upstreams = resp.config.upstreams.len(),
        rules = resp.config.rules.len(),
        "Login successful, config received"
    );

    let proxy_map = Arc::new(LocalProxyMap::from_config(&resp.config, &tunnel_lib::HttpClientParams::from(&config.http_pool)));

    for proxy in &resp.config.proxies {
        debug!(
            name = %proxy.name,
            proxy_type = %proxy.proxy_type,
            domains = ?proxy.domains,
            "registered proxy"
        );
    }

    let cancel_token = CancellationToken::new();

    // P1: Semaphore for client-side backpressure on incoming streams
    let max_streams = config.quic.max_concurrent_streams;
    let stream_semaphore = Arc::new(Semaphore::new(max_streams as usize));
    info!(max_concurrent_streams = %max_streams, "Stream backpressure configured");

    let tcp_params = tunnel_lib::TcpParams::from(&config.tcp);

    if let Some(entry_port) = config.http_entry_port {
        let conn = conn.clone();
        let token = cancel_token.clone();
        let max_entry_conns = config.quic.max_concurrent_streams;
        let entry_tcp_params = tcp_params.clone();
        let peek_buf_size = config.proxy_buffers.peek_buf_size;
        tokio::spawn(async move {
            if let Err(e) = entry::start_entry_listener(conn, entry_port, token, max_entry_conns, entry_tcp_params, peek_buf_size).await {
                error!(port = entry_port, error = %e, "entry listener failed");
            }
        });
    }

    let result = loop {
        tokio::select! {
            // Monitor connection close - this will fire immediately when server disconnects
            reason = conn.closed() => {
                warn!(reason = ?reason, "Connection closed by server");
                break Err(anyhow::anyhow!("Connection closed: {:?}", reason));
            }
            // Accept incoming work streams from server
            stream_result = conn.accept_bi() => {
                match stream_result {
                    Ok((send, recv)) => {
                        let permit = match stream_semaphore.clone().try_acquire_owned() {
                            Ok(permit) => permit,
                            Err(_) => {
                                warn!("Stream rejected: max concurrent streams reached");
                                continue;
                            }
                        };
                        debug!("Accepted work stream from server");
                        let proxy_map = proxy_map.clone();
                        let tcp_params = tcp_params.clone();
                        tokio::spawn(async move {
                            let _permit = permit; // Hold permit until stream completes
                            if let Err(e) = handle_work_stream(send, recv, proxy_map, tcp_params).await {
                                debug!(error = %e, "work stream error");
                            }
                        });
                    }
                    Err(e) => {
                        warn!(error = %e, "Connection error");
                        break Err(e.into());
                    }
                }
            }
        }
    };

    // Cancel all reverse proxy listeners before reconnecting
    cancel_token.cancel();
    // Give listeners time to shutdown
    tokio::time::sleep(std::time::Duration::from_millis(config.reconnect.grace_ms)).await;

    result
}

fn build_tls_config(config: &ClientConfigFile) -> Result<rustls::ClientConfig> {
    if config.tls_skip_verify {
        warn!("TLS certificate verification is DISABLED - this is insecure!");
        return Ok(rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(InsecureServerCertVerifier))
            .with_no_client_auth());
    }

    // Build with proper certificate validation
    let mut root_store = rustls::RootCertStore::empty();

    if let Some(ca_path) = &config.tls_ca_cert {
        // Load custom CA certificate
        let ca_file = std::fs::File::open(ca_path)
            .map_err(|e| anyhow::anyhow!("Failed to open CA cert file {}: {}", ca_path, e))?;
        let mut reader = std::io::BufReader::new(ca_file);
        let certs = rustls_pemfile::certs(&mut reader)
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();

        if certs.is_empty() {
            return Err(anyhow::anyhow!("No valid certificates found in {}", ca_path));
        }

        for cert in certs {
            root_store.add(cert)
                .map_err(|e| anyhow::anyhow!("Failed to add CA cert: {}", e))?;
        }
        info!(ca_path = %ca_path, "Loaded custom CA certificate");
    } else {
        // Use system root certificates
        let native_certs = rustls_native_certs::load_native_certs();
        for cert in native_certs.certs {
            let _ = root_store.add(cert);
        }
        if root_store.is_empty() {
            warn!("No system root certificates found, falling back to insecure mode");
            return Ok(rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(InsecureServerCertVerifier))
                .with_no_client_auth());
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
