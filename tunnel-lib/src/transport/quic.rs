use anyhow::{anyhow, Result};
use quinn::crypto::rustls::QuicServerConfig;
use quinn::ServerConfig;
use socket2::{Domain, Protocol, Socket, Type};
use std::convert::TryInto;
use std::net::SocketAddr;
use std::sync::Arc;

/// ALPN string carrying the wire-protocol generation; client and server must
/// reference this single constant. Breaking layout changes bump the generation
/// suffix so incompatible peers fail at the QUIC/TLS handshake — early and
/// with a clear error — instead of on rkyv validation after connect.
/// v1 replaced the unversioned "tunnel-quic" in one cut: the login structs
/// gained version/capability fields (a layout break old clients could not
/// parse anyway), so both ends upgrade together rather than carrying a
/// v0-compat path.
pub const TUNNEL_ALPN: &[u8] = b"tunnel-quic/v1";

#[derive(Debug, Clone)]
pub struct QuicTransportParams {
    pub max_concurrent_streams: u32,
    pub stream_receive_window_bytes: u64,
    pub connection_receive_window_bytes: u64,
    pub send_window_bytes: u64,
    pub keepalive_secs: u64,
    pub idle_timeout_secs: u64,
    pub congestion: Option<String>,
    pub udp_recv_buf_bytes: usize,
    pub udp_send_buf_bytes: usize,
}
impl Default for QuicTransportParams {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 1000,
            stream_receive_window_bytes: 4 * 1024 * 1024,
            connection_receive_window_bytes: 32 * 1024 * 1024,
            send_window_bytes: 8 * 1024 * 1024,
            keepalive_secs: 20,
            idle_timeout_secs: 180,
            congestion: Some("bbr".to_string()),
            udp_recv_buf_bytes: 8 * 1024 * 1024,
            udp_send_buf_bytes: 8 * 1024 * 1024,
        }
    }
}
fn apply_transport_params(
    tc: &mut quinn::TransportConfig,
    params: &QuicTransportParams,
) -> Result<()> {
    let stream_receive_window = quinn::VarInt::from_u64(params.stream_receive_window_bytes)
        .map_err(|e| {
            anyhow!(
                "invalid QUIC stream receive window {} bytes: {}",
                params.stream_receive_window_bytes,
                e
            )
        })?;
    let connection_receive_window = quinn::VarInt::from_u64(params.connection_receive_window_bytes)
        .map_err(|e| {
            anyhow!(
                "invalid QUIC connection receive window {} bytes: {}",
                params.connection_receive_window_bytes,
                e
            )
        })?;
    let idle_timeout = std::time::Duration::from_secs(params.idle_timeout_secs)
        .try_into()
        .map_err(|e| {
            anyhow!(
                "invalid QUIC idle timeout {} seconds: {}",
                params.idle_timeout_secs,
                e
            )
        })?;

    tc.max_concurrent_bidi_streams(params.max_concurrent_streams.into());
    tc.max_concurrent_uni_streams(params.max_concurrent_streams.into());
    tc.stream_receive_window(stream_receive_window);
    tc.receive_window(connection_receive_window);
    tc.send_window(params.send_window_bytes);
    tc.keep_alive_interval(Some(std::time::Duration::from_secs(params.keepalive_secs)));
    tc.max_idle_timeout(Some(idle_timeout));
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
    Ok(())
}
pub fn build_udp_socket(
    addr: SocketAddr,
    params: &QuicTransportParams,
) -> Result<std::net::UdpSocket> {
    let domain = Domain::for_address(addr);
    let sock = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
    sock.set_reuse_address(true)?;
    #[cfg(unix)]
    sock.set_reuse_port(true)?;
    sock.set_recv_buffer_size(params.udp_recv_buf_bytes)?;
    sock.set_send_buffer_size(params.udp_send_buf_bytes)?;
    sock.set_nonblocking(true)?;
    sock.bind(&addr.into())?;
    Ok(sock.into())
}

pub fn create_server_config_with(params: &QuicTransportParams) -> Result<ServerConfig> {
    let (certs, key) = crate::infra::pki::generate_self_signed_cert()?;
    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_crypto.alpn_protocols = vec![TUNNEL_ALPN.to_vec()];
    let mut server_config =
        ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    let mut transport_config = quinn::TransportConfig::default();
    apply_transport_params(&mut transport_config, params)?;
    server_config.transport_config(Arc::new(transport_config));
    Ok(server_config)
}

pub fn build_transport_config(params: &QuicTransportParams) -> Result<Arc<quinn::TransportConfig>> {
    let mut tc = quinn::TransportConfig::default();
    apply_transport_params(&mut tc, params)?;
    Ok(Arc::new(tc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_transport_config_rejects_out_of_range_stream_window() {
        let params = QuicTransportParams {
            stream_receive_window_bytes: 1 << 62,
            ..QuicTransportParams::default()
        };

        let error = build_transport_config(&params)
            .err()
            .expect("out-of-range stream window must fail");
        assert!(error.to_string().contains("stream receive window"));
    }

    #[test]
    fn build_transport_config_rejects_out_of_range_connection_window() {
        let params = QuicTransportParams {
            connection_receive_window_bytes: 1 << 62,
            ..QuicTransportParams::default()
        };

        let error = build_transport_config(&params)
            .err()
            .expect("out-of-range connection window must fail");
        assert!(error.to_string().contains("connection receive window"));
    }

    #[test]
    fn build_transport_config_rejects_out_of_range_idle_timeout() {
        let params = QuicTransportParams {
            idle_timeout_secs: u64::MAX,
            ..QuicTransportParams::default()
        };

        let error = build_transport_config(&params)
            .err()
            .expect("out-of-range idle timeout must fail");
        assert!(error.to_string().contains("idle timeout"));
    }
}
