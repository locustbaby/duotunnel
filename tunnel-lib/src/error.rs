use http::StatusCode;
use std::error::Error as StdError;
use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSource {
    Upstream,
    Downstream,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryType {
    Never,
    Safe,
    ReusedOnly,
}

impl RetryType {
    pub fn as_label(self) -> &'static str {
        match self {
            RetryType::Never => "never",
            RetryType::Safe => "safe",
            RetryType::ReusedOnly => "reused_only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    QuicStreamLimit,
    QuicConnectionLost,
    QuicConnectionFatal,
    RoutingMissingInfo,
    RoutingMissingHost,
    RouteNotFound,
    NoClientAvailable,
    ResolveUpstream,
    UnsupportedProtocol,
    DownstreamConnection,
    UpstreamConnect,
    UpstreamForward,
    TlsHandshake,
    HttpUpstreamRequest,
    H2cMissingAuthority,
    H2cMisdirected,
    H2cRouteResolve,
    H2cNoRoute,
    H2cNoClient,
    H2cForward,
}

impl ErrorKind {
    pub fn as_label(self) -> &'static str {
        match self {
            ErrorKind::QuicStreamLimit => "quic_stream_limit",
            ErrorKind::QuicConnectionLost => "quic_connection_lost",
            ErrorKind::QuicConnectionFatal => "quic_connection_fatal",
            ErrorKind::RoutingMissingInfo => "routing_missing_info",
            ErrorKind::RoutingMissingHost => "routing_missing_host",
            ErrorKind::RouteNotFound => "route_not_found",
            ErrorKind::NoClientAvailable => "no_client_available",
            ErrorKind::ResolveUpstream => "resolve_upstream",
            ErrorKind::UnsupportedProtocol => "unsupported_protocol",
            ErrorKind::DownstreamConnection => "downstream_connection",
            ErrorKind::UpstreamConnect => "upstream_connect",
            ErrorKind::UpstreamForward => "upstream_forward",
            ErrorKind::TlsHandshake => "tls_handshake",
            ErrorKind::HttpUpstreamRequest => "http_upstream_request",
            ErrorKind::H2cMissingAuthority => "h2c_missing_authority",
            ErrorKind::H2cMisdirected => "h2c_misdirected",
            ErrorKind::H2cRouteResolve => "h2c_route_resolve",
            ErrorKind::H2cNoRoute => "h2c_no_route",
            ErrorKind::H2cNoClient => "h2c_no_client",
            ErrorKind::H2cForward => "h2c_forward",
        }
    }

    pub fn source(self) -> ErrorSource {
        match self {
            ErrorKind::QuicStreamLimit
            | ErrorKind::RoutingMissingInfo
            | ErrorKind::QuicConnectionFatal
            | ErrorKind::H2cRouteResolve => ErrorSource::Internal,
            ErrorKind::RoutingMissingHost
            | ErrorKind::RouteNotFound
            | ErrorKind::UnsupportedProtocol
            | ErrorKind::DownstreamConnection
            | ErrorKind::H2cMissingAuthority
            | ErrorKind::H2cMisdirected
            | ErrorKind::H2cNoRoute => ErrorSource::Downstream,
            ErrorKind::QuicConnectionLost
            | ErrorKind::NoClientAvailable
            | ErrorKind::ResolveUpstream
            | ErrorKind::UpstreamConnect
            | ErrorKind::UpstreamForward
            | ErrorKind::TlsHandshake
            | ErrorKind::HttpUpstreamRequest
            | ErrorKind::H2cNoClient
            | ErrorKind::H2cForward => ErrorSource::Upstream,
        }
    }

    pub fn retry(self) -> RetryType {
        match self {
            ErrorKind::QuicStreamLimit
            | ErrorKind::QuicConnectionLost
            | ErrorKind::NoClientAvailable
            | ErrorKind::ResolveUpstream
            | ErrorKind::UpstreamConnect
            | ErrorKind::UpstreamForward
            | ErrorKind::H2cNoClient
            | ErrorKind::H2cForward => RetryType::Safe,
            ErrorKind::HttpUpstreamRequest => RetryType::ReusedOnly,
            ErrorKind::QuicConnectionFatal
            | ErrorKind::RoutingMissingInfo
            | ErrorKind::RoutingMissingHost
            | ErrorKind::RouteNotFound
            | ErrorKind::UnsupportedProtocol
            | ErrorKind::DownstreamConnection
            | ErrorKind::TlsHandshake
            | ErrorKind::H2cMissingAuthority
            | ErrorKind::H2cMisdirected
            | ErrorKind::H2cRouteResolve
            | ErrorKind::H2cNoRoute => RetryType::Never,
        }
    }
}

impl ErrorSource {
    pub fn as_label(self) -> &'static str {
        match self {
            ErrorSource::Upstream => "upstream",
            ErrorSource::Downstream => "downstream",
            ErrorSource::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyError {
    pub kind: ErrorKind,
    pub detail: Option<String>,
}

impl ProxyError {
    pub fn quic_stream_limit(timeout: Duration) -> Self {
        Self {
            kind: ErrorKind::QuicStreamLimit,
            detail: Some(format!(
                "open_bi waited {:?} for bidirectional stream capacity",
                timeout
            )),
        }
    }

    pub fn quic_connection_lost(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::QuicConnectionLost,
            detail: Some(detail.into()),
        }
    }

    pub fn quic_connection_fatal(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::QuicConnectionFatal,
            detail: Some(detail.into()),
        }
    }

    pub fn http_upstream_request(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::HttpUpstreamRequest,
            detail: Some(detail.into()),
        }
    }

    pub fn routing_missing_info() -> Self {
        Self {
            kind: ErrorKind::RoutingMissingInfo,
            detail: None,
        }
    }

    pub fn routing_missing_host() -> Self {
        Self {
            kind: ErrorKind::RoutingMissingHost,
            detail: None,
        }
    }

    pub fn route_not_found(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::RouteNotFound,
            detail: Some(detail.into()),
        }
    }

    pub fn no_client_available(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::NoClientAvailable,
            detail: Some(detail.into()),
        }
    }

    pub fn resolve_upstream(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::ResolveUpstream,
            detail: Some(detail.into()),
        }
    }

    pub fn unsupported_protocol(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::UnsupportedProtocol,
            detail: Some(detail.into()),
        }
    }

    pub fn downstream_connection(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::DownstreamConnection,
            detail: Some(detail.into()),
        }
    }

    pub fn upstream_connect(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::UpstreamConnect,
            detail: Some(detail.into()),
        }
    }

    pub fn upstream_forward(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::UpstreamForward,
            detail: Some(detail.into()),
        }
    }

    pub fn tls_handshake(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::TlsHandshake,
            detail: Some(detail.into()),
        }
    }

    pub fn source(&self) -> ErrorSource {
        self.kind.source()
    }

    pub fn retry(&self) -> RetryType {
        self.kind.retry()
    }

    pub fn should_retry(&self, was_reused: bool) -> bool {
        match self.retry() {
            RetryType::Never => false,
            RetryType::Safe => true,
            RetryType::ReusedOnly => was_reused,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn h2c_missing_authority() -> Self {
        Self {
            kind: ErrorKind::H2cMissingAuthority,
            detail: None,
        }
    }

    pub fn h2c_misdirected(route_host: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cMisdirected,
            detail: Some(route_host.into()),
        }
    }

    pub fn h2c_route_resolve(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cRouteResolve,
            detail: Some(detail.into()),
        }
    }

    pub fn h2c_no_route(route_host: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cNoRoute,
            detail: Some(route_host.into()),
        }
    }

    pub fn h2c_no_client(group_id: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cNoClient,
            detail: Some(group_id.into()),
        }
    }

    pub fn h2c_forward(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cForward,
            detail: Some(detail.into()),
        }
    }

    pub fn http_status(&self) -> Option<StatusCode> {
        Some(match self.kind {
            ErrorKind::HttpUpstreamRequest
            | ErrorKind::ResolveUpstream
            | ErrorKind::UpstreamConnect
            | ErrorKind::UpstreamForward
            | ErrorKind::TlsHandshake
            | ErrorKind::H2cRouteResolve
            | ErrorKind::H2cForward => StatusCode::BAD_GATEWAY,
            ErrorKind::RoutingMissingInfo
            | ErrorKind::RoutingMissingHost
            | ErrorKind::UnsupportedProtocol
            | ErrorKind::H2cMissingAuthority => StatusCode::BAD_REQUEST,
            ErrorKind::H2cMisdirected => StatusCode::MISDIRECTED_REQUEST,
            ErrorKind::RouteNotFound | ErrorKind::H2cNoRoute => StatusCode::NOT_FOUND,
            ErrorKind::NoClientAvailable | ErrorKind::H2cNoClient => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            ErrorKind::DownstreamConnection => return None,
            ErrorKind::QuicStreamLimit
            | ErrorKind::QuicConnectionLost
            | ErrorKind::QuicConnectionFatal => return None,
        })
    }
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.kind {
            ErrorKind::QuicStreamLimit => "quic stream limit",
            ErrorKind::QuicConnectionLost => "quic connection lost",
            ErrorKind::QuicConnectionFatal => "quic connection fatal",
            ErrorKind::RoutingMissingInfo => "routing info missing",
            ErrorKind::RoutingMissingHost => "routing host missing",
            ErrorKind::RouteNotFound => "route not found",
            ErrorKind::NoClientAvailable => "no client available",
            ErrorKind::ResolveUpstream => "upstream resolve failed",
            ErrorKind::UnsupportedProtocol => "unsupported protocol",
            ErrorKind::DownstreamConnection => "downstream connection failed",
            ErrorKind::UpstreamConnect => "upstream connect failed",
            ErrorKind::UpstreamForward => "upstream forward failed",
            ErrorKind::TlsHandshake => "tls handshake failed",
            ErrorKind::HttpUpstreamRequest => "http upstream request failed",
            ErrorKind::H2cMissingAuthority => "h2c missing authority",
            ErrorKind::H2cMisdirected => "h2c misdirected request",
            ErrorKind::H2cRouteResolve => "h2c route resolve failed",
            ErrorKind::H2cNoRoute => "h2c no route",
            ErrorKind::H2cNoClient => "h2c no client available",
            ErrorKind::H2cForward => "h2c upstream forward failed",
        };
        match &self.detail {
            Some(detail) => write!(f, "{}: {}", label, detail),
            None => write!(f, "{}", label),
        }
    }
}

impl StdError for ProxyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_error_labels_are_stable_for_metrics() {
        let err = ProxyError::quic_stream_limit(Duration::from_millis(10));

        assert_eq!(err.kind.as_label(), "quic_stream_limit");
        assert_eq!(err.source().as_label(), "internal");
        assert_eq!(err.retry().as_label(), "safe");
    }

    #[test]
    fn retry_policy_honors_reused_only() {
        let err = ProxyError::http_upstream_request("idle close");

        assert!(!err.should_retry(false));
        assert!(err.should_retry(true));
    }

    #[test]
    fn http_status_maps_client_availability_to_503() {
        let err = ProxyError::no_client_available("group-a");

        assert_eq!(err.http_status(), Some(StatusCode::SERVICE_UNAVAILABLE));
    }
}
