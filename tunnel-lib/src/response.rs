use serde_json;
use crate::tunnel::HttpResponse;

#[derive(Debug)]
pub enum ProxyErrorKind {
    NoMatchRules,
    NoUpstream,
    Internal,
    BadRequest,
    // 可扩展更多类型
}

impl ProxyErrorKind {
    pub fn code(&self) -> u16 {
        match self {
            ProxyErrorKind::NoMatchRules => 404,
            ProxyErrorKind::NoUpstream => 502,
            ProxyErrorKind::Internal => 500,
            ProxyErrorKind::BadRequest => 400,
        }
    }
    pub fn message(&self) -> &'static str {
        match self {
            ProxyErrorKind::NoMatchRules => "No matching rules",
            ProxyErrorKind::NoUpstream => "No upstream available",
            ProxyErrorKind::Internal => "Internal server error",
            ProxyErrorKind::BadRequest => "Bad request",
        }
    }
    pub fn error_type(&self) -> &'static str {
        match self {
            ProxyErrorKind::NoMatchRules => "no_match_rules",
            ProxyErrorKind::NoUpstream => "no_upstream",
            ProxyErrorKind::Internal => "internal_error",
            ProxyErrorKind::BadRequest => "bad_request",
        }
    }
}

/// 统一错误响应构造
pub fn error_response(
    kind: ProxyErrorKind,
    context: Option<&str>,
    trace_id: Option<&str>,
    request_id: Option<&str>,
    identity: Option<&str>, // Some("server") or Some(client_id)
) -> HttpResponse {
    let (client_id, service) = match identity {
        Some("server") => (None, Some("server")),
        Some(id) => (Some(id), Some(id)),
        None => (None, None),
    };
    let body = serde_json::json!({
        "code": kind.code(),
        "type": kind.error_type(),
        "message": kind.message(),
        "context": context.unwrap_or(""),
        "trace_id": trace_id.unwrap_or(""),
        "request_id": request_id.unwrap_or(""),
        "client_id": client_id.unwrap_or(""),
        "service": service.unwrap_or("")
    });
    let mut headers = std::collections::HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    HttpResponse {
        status_code: kind.code() as i32,
        headers,
        body: body.to_string().into_bytes(),
    }
}

// 常用快捷函数
pub fn resp_404(trace_id: Option<&str>, request_id: Option<&str>, identity: Option<&str>) -> HttpResponse {
    error_response(ProxyErrorKind::NoMatchRules, None, trace_id, request_id, identity)
}
pub fn resp_502(trace_id: Option<&str>, request_id: Option<&str>, identity: Option<&str>) -> HttpResponse {
    error_response(ProxyErrorKind::NoUpstream, None, trace_id, request_id, identity)
}
pub fn resp_500(context: Option<&str>, trace_id: Option<&str>, request_id: Option<&str>, identity: Option<&str>) -> HttpResponse {
    error_response(ProxyErrorKind::Internal, context, trace_id, request_id, identity)
} 