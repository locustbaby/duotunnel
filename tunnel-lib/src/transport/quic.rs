use anyhow::Result;
use quinn::crypto::rustls::QuicServerConfig;
use quinn::ServerConfig;
use std::convert::TryInto;
use std::sync::Arc;
#[derive(Debug, Clone)]
pub struct QuicTransportParams {
    pub max_concurrent_streams: u32,
    pub stream_receive_window_bytes: u64,
    pub connection_receive_window_bytes: u64,
    pub send_window_bytes: u64,
    pub keepalive_secs: u64,
    pub idle_timeout_secs: u64,
    pub congestion: Option<String>,
}
impl Default for QuicTransportParams {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 1000,
            stream_receive_window_bytes: 4 * 1024 * 1024,
            connection_receive_window_bytes: 32 * 1024 * 1024,
            send_window_bytes: 8 * 1024 * 1024,
            keepalive_secs: 20,
            idle_timeout_secs: 60,
            congestion: Some("bbr".to_string()),
        }
    }
}
fn apply_transport_params(tc: &mut quinn::TransportConfig, params: &QuicTransportParams) {
    tc.max_concurrent_bidi_streams(params.max_concurrent_streams.into());
    tc.max_concurrent_uni_streams(params.max_concurrent_streams.into());
    tc.stream_receive_window(params.stream_receive_window_bytes.try_into().unwrap());
    tc.receive_window(params.connection_receive_window_bytes.try_into().unwrap());
    tc.send_window(params.send_window_bytes);
    tc.keep_alive_interval(Some(std::time::Duration::from_secs(params.keepalive_secs)));
    tc.max_idle_timeout(Some(
        std::time::Duration::from_secs(params.idle_timeout_secs)
            .try_into()
            .unwrap(),
    ));
    match params.congestion.as_deref().unwrap_or("bbr") {
        m if m.eq_ignore_ascii_case("bbr") => {
            tc.congestion_controller_factory(Arc::new(quinn::congestion::BbrConfig::default()));
        }
        m if m.eq_ignore_ascii_case("cubic") => {
            tc.congestion_controller_factory(Arc::new(quinn::congestion::CubicConfig::default()));
        }
        m if m.eq_ignore_ascii_case("new_reno") || m.eq_ignore_ascii_case("newreno") => {
            tc.congestion_controller_factory(Arc::new(quinn::congestion::NewRenoConfig::default()));
        }
        _ => {
            // unknown value, fall back to quinn default (NewReno)
        }
    }
}
pub fn create_server_config_with(params: &QuicTransportParams) -> Result<ServerConfig> {
    let (certs, key) = crate::infra::pki::generate_self_signed_cert()?;
    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_crypto.alpn_protocols = vec![b"tunnel-quic".to_vec()];
    let mut server_config =
        ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    let mut transport_config = quinn::TransportConfig::default();
    apply_transport_params(&mut transport_config, params);
    server_config.transport_config(Arc::new(transport_config));
    Ok(server_config)
}
pub fn build_transport_config(params: &QuicTransportParams) -> Arc<quinn::TransportConfig> {
    let mut tc = quinn::TransportConfig::default();
    apply_transport_params(&mut tc, params);
    Arc::new(tc)
}
