pub mod config;
pub mod egress;
pub mod engine;
pub mod error;
pub mod infra;
pub mod lb;
pub mod models;
pub mod plugin;
pub mod protocol;
pub mod proxy;
pub mod transport;

pub use config::{resolve_config_path, HttpPoolConfig, ProxyBufferConfig, QuicConfig, TcpConfig};
pub use egress::http::{
    create_h2c_client, create_h2c_client_with, create_https_client, create_https_client_with,
    forward_http, H2cClient, HttpClientParams,
};
pub use engine::bridge::relay_quic_to_tcp;
pub use error::{ErrorKind, ErrorSource, ProxyError, RetryType};
pub use infra::dns_cache::EgressDnsCache;
pub use infra::metrics::{wait_for_resource_drain, METRICS};
pub use infra::peek_buf::PeekBufPool;
pub use infra::pki::{get_or_create_server_config, init_cert_cache, PkiParams};
pub use infra::runtime::{apply_worker_threads, build_proxy_runtime, build_single_thread_runtime};
pub use infra::runtime::{
    available_parallelism, cgroup_cpu_limit, configured_worker_threads,
    effective_runtime_parallelism, resolve_accept_workers, resolve_connection_count,
    resolve_shard_count,
};
pub use infra::timeout::{sleep, timeout, tokio_timeout, Elapsed as TimeoutElapsed};
pub use lb::inflight::{
    begin_inflight, inflight_load, inflight_notify, new_inflight_table, pick_least_inflight,
    pick_p2c_inflight, ConnectionState, InflightGuard, InflightTable,
};
pub use lb::overload::{
    maybe_slow_path, BackoffStrategy, OverloadLimits, OverloadMode as SharedOverloadMode,
};
pub use lb::shard::{
    pick_from_preferred_shards, pick_p2c_inflight_owned, stable_shard_index,
    DEFAULT_P2C_MAX_RETRIES, DEFAULT_P2C_THRESHOLD,
};
pub use models::defs::{
    ClientGroupDef, ClientStatus, ClientUpstreamDef, EgressUpstreamDef, EgressVhostRuleDef,
    IngressListenerDef, IngressListenerModeDef, IngressVhostRuleDef, TokenCacheEntryDef,
    TokenStatus, UpstreamServerDef,
};
pub use models::id::{ClientId, GroupId, ProxyName, ReuseHash};
pub use models::msg::{
    decode_udp_datagram_envelope, encode_udp_datagram_envelope, negotiate_protocol, recv_message,
    recv_message_bounded, recv_message_type, recv_routing_info, recv_typed_message, send_message,
    send_routing_info, ClientConfig, Login, LoginResp, MessageType, NegotiatedProtocol,
    RoutingInfo, UdpDatagramEnvelope, UdpSessionKey, UpstreamConfig, UpstreamServer, CAP_NONE,
    MAX_DATAGRAM_BYTES, MAX_LOGIN_BYTES, MIN_SUPPORTED_VERSION, PROTOCOL_VERSION,
    SUPPORTED_CAPABILITIES,
};
pub use protocol::detect::detect_protocol_and_host;
pub use protocol::sniff::{
    default_client_detectors, default_ingress_detectors, default_proxyengine_detectors,
    H2cDetector, Http1Detector, ProtocolDetector, SniffOutcome, SniffPolicy, SniffPrefix,
    SniffResult, SniffRuntime, SniffStream, TlsClientHelloDetector,
};
pub use proxy::h2_proxy::{forward_h2_request, new_h2_sender, EmptyBodyRetryTemplate, H2Sender};
pub use proxy::ProxyBufferParams;
pub use proxy::UpstreamGroup;
pub use transport::accept::{run_accept_worker, AcceptedConn};
pub use transport::connection_handle::{ConnectionHandle, OpenStreamRequest, OpenWaitObserver};
pub use transport::listener::{
    build_reuseport_listener, canonicalize_egress_host, extract_host_from_http, RouteTarget,
    VhostRouter, DEFAULT_ACCEPT_WORKERS,
};
pub use transport::open_bi::{open_bi_guarded, OpenBiOutcome, OpenedStream};
pub use transport::quic::{
    build_transport_config, build_udp_socket, QuicTransportParams, TUNNEL_ALPN,
};
pub use transport::quinn_io::{PrefixedReadWrite, QuinnStream};
pub use transport::tcp_params::TcpParams;

pub mod ctld_proto {
    pub use crate::protocol::ctld::*;
}

pub mod shared {
    pub use crate::models::defs::*;
}
