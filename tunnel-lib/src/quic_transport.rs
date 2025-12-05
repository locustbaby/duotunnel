use anyhow::{Result, anyhow};
use quinn::{Endpoint, ServerConfig, ClientConfig, Connection, SendStream, RecvStream};
use quinn::crypto::rustls::{QuicServerConfig, QuicClientConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use std::convert::TryInto;
use tracing::{info, debug, error};

pub fn generate_self_signed_cert() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let key_der = cert.key_pair.serialize_der();
    let cert_der = cert.cert.der().to_vec();
    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;
    let cert = CertificateDer::from(cert_der);
    Ok((vec![cert], key))
}

pub fn create_server_config() -> Result<ServerConfig> {
    let (certs, key) = generate_self_signed_cert()?;
    
    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    
    server_crypto.alpn_protocols = vec![b"tunnel-quic".to_vec()];
    
    let mut server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    

    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100_u32.into());
    transport_config.max_concurrent_uni_streams(100_u32.into());

    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(20)));


    transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into().unwrap()));
    
    server_config.transport_config(Arc::new(transport_config));
    
    Ok(server_config)
}

pub fn create_client_config() -> Result<ClientConfig> {
    let mut client_crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();
    
    client_crypto.alpn_protocols = vec![b"tunnel-quic".to_vec()];
    
    let mut client_config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto)?));
    

    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100_u32.into());
    transport_config.max_concurrent_uni_streams(100_u32.into());

    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(20)));


    transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into().unwrap()));
    
    client_config.transport_config(Arc::new(transport_config));
    
    Ok(client_config)
}

#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

pub struct QuicServer {
    endpoint: Endpoint,
}

impl QuicServer {

    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let server_config = create_server_config()?;
        let endpoint = Endpoint::server(server_config, addr)?;
        info!("QUIC server listening on {}", addr);
        Ok(Self { endpoint })
    }


    pub async fn accept(&self) -> Option<Connection> {
        match self.endpoint.accept().await {
            Some(connecting) => {
                match connecting.await {
                    Ok(connection) => {
                        debug!("Accepted QUIC connection from {}", connection.remote_address());
                        Some(connection)
                    }
                    Err(e) => {
                        error!("Failed to complete QUIC connection: {}", e);
                        None
                    }
                }
            }
            None => None,
        }
    }
}

pub struct QuicClient {
    endpoint: Endpoint,
}

impl QuicClient {

    pub fn new() -> Result<Self> {
        let client_config = create_client_config()?;
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);
        Ok(Self { endpoint })
    }


    pub async fn connect(&self, addr: SocketAddr, server_name: &str) -> Result<Connection> {
        debug!("Connecting to QUIC server at {}", addr);
        let connection = self.endpoint.connect(addr, server_name)?.await?;
        info!("Connected to QUIC server at {}", addr);
        Ok(connection)
    }
}

pub mod stream {
    use super::*;
    use bytes::{BytesMut, BufMut};


    pub async fn open_bi(conn: &Connection) -> Result<(SendStream, RecvStream)> {
        Ok(conn.open_bi().await?)
    }


    pub async fn accept_bi(conn: &Connection) -> Result<(SendStream, RecvStream)> {
        match conn.accept_bi().await {
            Ok(streams) => Ok(streams),
            Err(e) => Err(anyhow!("Failed to accept bidirectional stream: {}", e)),
        }
    }


    pub async fn send_data(send: &mut SendStream, data: &[u8]) -> Result<()> {
        let len = data.len() as u32;
        let mut buf = BytesMut::with_capacity(4 + data.len());
        buf.put_u32(len);
        buf.put_slice(data);
        send.write_all(&buf).await?;
        Ok(())
    }


    pub async fn recv_data(recv: &mut RecvStream) -> Result<Vec<u8>> {

        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;


        let mut data = vec![0u8; len];
        recv.read_exact(&mut data).await?;
        Ok(data)
    }


    pub fn finish(send: &mut SendStream) -> Result<()> {
        send.finish()?;
        Ok(())
    }
}
