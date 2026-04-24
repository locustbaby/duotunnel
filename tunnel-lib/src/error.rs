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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    QuicOpenTimeout,
    QuicOpenConnection,
    HttpUpstreamRequest,
    H2cMissingAuthority,
    H2cMisdirected,
    H2cRouteResolve,
    H2cNoRoute,
    H2cNoClient,
    H2cForward,
}

#[derive(Debug, Clone)]
pub struct ProxyError {
    pub kind: ErrorKind,
    pub source: ErrorSource,
    pub retry: RetryType,
    pub detail: Option<String>,
}

impl ProxyError {
    pub fn quic_open_timeout(timeout: Duration) -> Self {
        Self {
            kind: ErrorKind::QuicOpenTimeout,
            source: ErrorSource::Internal,
            retry: RetryType::Safe,
            detail: Some(format!("open_bi timed out after {:?}", timeout)),
        }
    }

    pub fn quic_open_connection(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::QuicOpenConnection,
            source: ErrorSource::Internal,
            retry: RetryType::Safe,
            detail: Some(detail.into()),
        }
    }

    pub fn http_upstream_request(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::HttpUpstreamRequest,
            source: ErrorSource::Upstream,
            retry: RetryType::ReusedOnly,
            detail: Some(detail.into()),
        }
    }

    pub fn h2c_missing_authority() -> Self {
        Self {
            kind: ErrorKind::H2cMissingAuthority,
            source: ErrorSource::Downstream,
            retry: RetryType::Never,
            detail: None,
        }
    }

    pub fn h2c_misdirected(route_host: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cMisdirected,
            source: ErrorSource::Downstream,
            retry: RetryType::Never,
            detail: Some(route_host.into()),
        }
    }

    pub fn h2c_route_resolve(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cRouteResolve,
            source: ErrorSource::Internal,
            retry: RetryType::Never,
            detail: Some(detail.into()),
        }
    }

    pub fn h2c_no_route(route_host: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cNoRoute,
            source: ErrorSource::Downstream,
            retry: RetryType::Never,
            detail: Some(route_host.into()),
        }
    }

    pub fn h2c_no_client(group_id: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cNoClient,
            source: ErrorSource::Upstream,
            retry: RetryType::Safe,
            detail: Some(group_id.into()),
        }
    }

    pub fn h2c_forward(detail: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::H2cForward,
            source: ErrorSource::Upstream,
            retry: RetryType::Safe,
            detail: Some(detail.into()),
        }
    }

    pub fn http_status(&self) -> Option<StatusCode> {
        Some(match self.kind {
            ErrorKind::HttpUpstreamRequest
            | ErrorKind::H2cRouteResolve
            | ErrorKind::H2cForward => StatusCode::BAD_GATEWAY,
            ErrorKind::H2cMissingAuthority => StatusCode::BAD_REQUEST,
            ErrorKind::H2cMisdirected => StatusCode::MISDIRECTED_REQUEST,
            ErrorKind::H2cNoRoute => StatusCode::NOT_FOUND,
            ErrorKind::H2cNoClient => StatusCode::SERVICE_UNAVAILABLE,
            ErrorKind::QuicOpenTimeout | ErrorKind::QuicOpenConnection => return None,
        })
    }
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.kind {
            ErrorKind::QuicOpenTimeout => "quic open timeout",
            ErrorKind::QuicOpenConnection => "quic open connection error",
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
