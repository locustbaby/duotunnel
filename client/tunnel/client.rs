use anyhow::{anyhow, Result};
use rustls::pki_types::pem::PemObject;
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use tunnel_lib::{recv_message, recv_message_type, send_message, Login, LoginResp, MessageType};

use crate::bootstrap::config::ClientConfigFile;
use crate::ingress::app::LocalProxyMap;
use crate::ingress::handler::handle_work_stream;
use crate::plugins;
use crate::tunnel::conn_pool::EntryConnPool;
use crate::tunnel::supervisor::ConnectError;

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
    let (mut send, mut recv) = tunnel_lib::timeout(login_timeout, conn.open_bi())
        .await
        .map_err(|_| ConnectError::transient(anyhow!("open_bi timed out")))?
        .map_err(|e| ConnectError::transient(anyhow!("failed to open login stream: {}", e)))?;
    let login = Login {
        token: config.auth_token.clone(),
    };
    tunnel_lib::timeout(
        login_timeout,
        send_message(&mut send, MessageType::Login, &login),
    )
    .await
    .map_err(|_| ConnectError::transient(anyhow!("sending login timed out")))?
    .map_err(|e| ConnectError::transient(anyhow!("failed to send login: {}", e)))?;
    debug!("Login message sent");
    let msg_type = tunnel_lib::timeout(login_timeout, recv_message_type(&mut recv))
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
    let resp: LoginResp = tunnel_lib::timeout(login_timeout, recv_message(&mut recv))
        .await
        .map_err(|_| ConnectError::transient(anyhow!("reading LoginResp payload timed out")))?
        .map_err(|e| ConnectError::transient(anyhow!("failed to decode LoginResp: {}", e)))?;
    if !resp.success {
        return Err(crate::tunnel::supervisor::classify_login_failure(
            resp.error.as_deref(),
        ));
    }
    info!(
        client_group = %resp.client_group,
        upstreams = resp.config.upstreams.len(),
        "Login successful, config received"
    );
    let lb = Arc::new(plugins::lb_round_robin::RoundRobinLb::new());
    let resolver = Arc::new(plugins::resolver_cached::CachedResolver::new());
    let proxy_map = Arc::new(LocalProxyMap::from_config(
        &resp.config,
        &tunnel_lib::HttpClientParams::from(&config.http_pool),
        lb,
        resolver,
    ));
    let session_cancel = CancellationToken::new();
    let tcp_params = tunnel_lib::TcpParams::from(&config.tcp);

    entry_pool.push(conn.clone()).await;
    ready.store(true, Ordering::Release);
    let result = loop {
        tokio::select! {
            reason = conn.closed() => {
                warn!(reason = ?reason, "Connection closed by server");
                break Err(ConnectError::transient(anyhow!("connection closed by server: {:?}", reason)));
            }
            stream_result = conn.accept_bi() => {
                match stream_result {
                    Ok((send, recv)) => {
                        debug!("Accepted work stream from server");
                        let proxy_map = proxy_map.clone();
                        let tcp_params = tcp_params.clone();
                        crate::runtime::spawn_task(async move {
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
    entry_pool.remove(&conn).await;
    ready.store(false, Ordering::Release);
    tokio::select! {
        _ = shutdown.cancelled() => {}
        _ = tokio::time::sleep(Duration::from_millis(config.reconnect.grace_ms)) => {}
    }
    result
}

pub(crate) async fn build_quic_endpoint(config: &ClientConfigFile) -> Result<quinn::Endpoint> {
    let mut crypto = build_tls_config(config).await?;
    crypto.alpn_protocols = vec![b"tunnel-quic".to_vec()];
    let mut client_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?,
    ));
    let quic_params = tunnel_lib::QuicTransportParams::from(&config.quic);
    let transport_config = tunnel_lib::build_transport_config(&quic_params);
    client_config.transport_config(transport_config);
    debug!(
        max_streams = % quic_params.max_concurrent_streams,
        keepalive_secs = % quic_params.keepalive_secs,
        congestion = ? quic_params.congestion,
        "QUIC transport configured"
    );
    let udp_socket = tunnel_lib::build_udp_socket("0.0.0.0:0".parse()?, &quic_params)?;
    let mut endpoint = quinn::Endpoint::new(
        quinn::EndpointConfig::default(),
        None,
        udp_socket,
        Arc::new(quinn::TokioRuntime),
    )?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
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
        info!(server_addr = %addr, sni = %sni, "Connecting to server");
        let connecting = endpoint
            .connect(addr, &sni)
            .map_err(|e| ConnectError::transient(anyhow!("connect setup failed: {}", e)))?;
        match tunnel_lib::timeout(connect_timeout, connecting).await {
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
    let resolved = tunnel_lib::timeout(resolve_timeout, lookup)
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

async fn build_tls_config(config: &ClientConfigFile) -> Result<rustls::ClientConfig> {
    if config.tls_skip_verify {
        warn!("TLS certificate verification is DISABLED - this is insecure!");
        return Ok(rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(InsecureServerCertVerifier))
            .with_no_client_auth());
    }
    let mut root_store = rustls::RootCertStore::empty();
    if let Some(ca_path) = &config.tls_ca_cert {
        let ca_file_bytes = tokio::fs::read(ca_path)
            .await
            .map_err(|e| anyhow!("Failed to read CA cert file {}: {}", ca_path, e))?;
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(ca_file_bytes));
        let certs = rustls::pki_types::CertificateDer::pem_reader_iter(&mut reader)
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        if certs.is_empty() {
            return Err(anyhow!("No valid certificates found in {}", ca_path));
        }
        for cert in certs {
            root_store
                .add(cert)
                .map_err(|e| anyhow!("Failed to add CA cert: {}", e))?;
        }
        info!(ca_path = %ca_path, "Loaded custom CA certificate");
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
            return Err(anyhow!(
                "No system root certificates found; set tls_ca_cert or explicitly enable allow_insecure_fallback"
            ));
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
