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
    let transport_config = tunnel_lib::build_transport_config(&quic_params)?;
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
    let root_store = if let Some(ca_path) = &config.tls_ca_cert {
        let mut root_store = rustls::RootCertStore::empty();
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
        root_store
    } else {
        match load_native_root_store() {
            Ok(root_store) => root_store,
            Err(error) if config.allow_insecure_fallback => {
                warn!(error = %error, "Failed to load system root certificates, falling back to insecure mode");
                return Ok(rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(InsecureServerCertVerifier))
                    .with_no_client_auth());
            }
            Err(error) => return Err(error),
        }
    };
    Ok(rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth())
}

fn load_native_root_store() -> Result<rustls::RootCertStore> {
    let native_certs = rustls_native_certs::load_native_certs();
    let errors = native_certs
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    build_native_root_store(native_certs.certs, errors)
}

fn build_native_root_store(
    certs: Vec<rustls::pki_types::CertificateDer<'static>>,
    errors: Vec<String>,
) -> Result<rustls::RootCertStore> {
    let mut root_store = rustls::RootCertStore::empty();
    let mut invalid_cert_errors = Vec::new();
    for cert in certs {
        if let Err(error) = root_store.add(cert) {
            invalid_cert_errors.push(error.to_string());
        }
    }
    if root_store.is_empty() {
        let details = errors
            .iter()
            .chain(invalid_cert_errors.iter())
            .take(3)
            .cloned()
            .collect::<Vec<_>>();
        let detail = if details.is_empty() {
            String::new()
        } else {
            format!(": {}", details.join("; "))
        };
        return Err(anyhow!(
            "No valid system root certificates found ({} load error(s), {} invalid certificate(s)){}; set tls_ca_cert or explicitly enable allow_insecure_fallback",
            errors.len(),
            invalid_cert_errors.len(),
            detail
        ));
    }
    if !errors.is_empty() || !invalid_cert_errors.is_empty() {
        warn!(
            load_errors = errors.len(),
            invalid_certificates = invalid_cert_errors.len(),
            valid_certificates = root_store.len(),
            "Some system root certificates could not be loaded; continuing with valid roots"
        );
    }
    debug!("Loaded {} system root certificates", root_store.len());
    Ok(root_store)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_root_loader_surfaces_platform_errors() {
        let error = build_native_root_store(
            Vec::new(),
            vec!["platform certificate store unavailable".to_string()],
        )
        .expect_err("native root errors must fail startup");

        assert!(error
            .to_string()
            .contains("platform certificate store unavailable"));
    }

    #[test]
    fn native_root_loader_rejects_empty_store() {
        let error = build_native_root_store(Vec::new(), Vec::new())
            .expect_err("empty native root store must fail startup");

        assert!(error
            .to_string()
            .contains("No valid system root certificates found"));
    }

    #[test]
    fn native_root_loader_keeps_valid_roots_when_some_loads_fail() {
        let (certs, _) =
            tunnel_lib::infra::pki::generate_self_signed_cert_for_host("example.com").unwrap();
        let roots = build_native_root_store(
            certs,
            vec!["one platform certificate was unreadable".to_string()],
        )
        .expect("valid roots must survive partial platform errors");

        assert!(!roots.is_empty());
    }
}
