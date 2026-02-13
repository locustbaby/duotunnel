pub mod models;
pub mod infra;
pub mod transport;
pub mod protocol;
pub mod engine;
pub mod proxy;
pub mod egress;

// Re-exports
pub use models::msg::{
    MessageType, Login, LoginResp, ClientConfig, ProxyConfig, 
    UpstreamConfig, UpstreamServer, RuleConfig, RoutingInfo,
    send_message, recv_message, recv_message_type,
    send_routing_info, recv_routing_info,
};

pub use engine::bridge::{relay as bridge_relay, relay_quic_to_tcp, relay_with_first_data, QuicBiStream};

pub use transport::listener::{
    start_tcp_listener, peek_bytes,
    VhostRouter, PortRouter, SharedVhostRouter, new_shared_vhost_router,
    extract_host_from_http, extract_method_path_from_http,
};

pub use transport::addr::{parse_upstream, normalize_host, UpstreamAddr};
pub use protocol::rewrite::Rewriter;
pub use protocol::detect::detect_protocol_and_host;
pub use engine::relay::{relay_bidirectional, relay_with_initial};
pub use egress::http::{forward_http, create_https_client};
pub use transport::quinn_io::{QuinnStream, PrefixedReadWrite};