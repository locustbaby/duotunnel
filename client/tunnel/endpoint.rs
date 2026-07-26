use anyhow::{anyhow, Result};
use rustls::pki_types::pem::PemObject;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::bootstrap::config::ClientConfigFile;
use tunnel_lib::QuicTransportParams;

pub(crate) async fn build_quic_endpoint(config: &ClientConfigFile) -> Result<quinn::Endpoint> {
    let mut crypto = build_tls_config(config).await?;
    crypto.alpn_protocols = vec![tunnel_lib::TUNNEL_ALPN.to_vec()];
    let mut client_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?,
    ));
    let quic_params = QuicTransportParams::from(&config.quic);
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
