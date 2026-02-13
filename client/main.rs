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

    // Exponential backoff for reconnection
    let mut retry_delay = std::time::Duration::from_secs(1);
    const MAX_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(60);
    const INITIAL_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(1);

    loop {
        match run_client(&config).await {
            Ok(_) => {
                info!("Connection closed gracefully, reconnecting immediately...");
                retry_delay = INITIAL_RETRY_DELAY; // Reset on graceful close
            }
            Err(e) => {
                error!(error = %e, retry_in_secs = %retry_delay.as_secs(), "Connection error, reconnecting...");
                tokio::time::sleep(retry_delay).await;
                // Exponential backoff with max cap
                retry_delay = std::cmp::min(retry_delay * 2, MAX_RETRY_DELAY);
            }
        }
    }
}

async fn run_client(config: &ClientConfigFile) -> Result<()> {
    let mut crypto = build_tls_config(config)?;
    // ALPN must match server's "tunnel-quic"
    crypto.alpn_protocols = vec![b"tunnel-quic".to_vec()];

    let mut client_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?
    ));

    // Configure transport settings
    let max_streams = config.max_concurrent_streams;
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(max_streams.into());
    transport_config.max_concurrent_uni_streams(max_streams.into());
    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(20)));
    transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into().unwrap()));
    client_config.transport_config(Arc::new(transport_config));
    debug!(max_streams = %max_streams, "QUIC transport configured");
    
    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

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

    let proxy_map = Arc::new(LocalProxyMap::from_config(&resp.config));

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
    let stream_semaphore = Arc::new(Semaphore::new(max_streams as usize));
    info!(max_concurrent_streams = %max_streams, "Stream backpressure configured");

    if let Some(entry_port) = config.http_entry_port {
        let conn = conn.clone();
        let token = cancel_token.clone();
        tokio::spawn(async move {
            if let Err(e) = entry::start_entry_listener(conn, entry_port, token).await {
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
                        tokio::spawn(async move {
                            let _permit = permit; // Hold permit until stream completes
                            if let Err(e) = handle_work_stream(send, recv, proxy_map).await {
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
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

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
