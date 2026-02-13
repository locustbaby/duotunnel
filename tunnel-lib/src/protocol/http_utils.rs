use hyper::header::{self, HeaderMap, HeaderName, HeaderValue};

const HOP_BY_HOP_REQUEST: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-connection",
    "proxy-authenticate",
    "proxy-authorization",
    "transfer-encoding",
    "upgrade",
    // "te", // Handled manually to allow "trailers"
    "trailer",
    "host",
];

const HOP_BY_HOP_RESPONSE: &[&str] = &[
    "transfer-encoding",
    "content-length",
];

pub fn sanitize_request_headers(headers: &mut HeaderMap) {
    // 1. Remove headers listed in Connection header (RFC 7230 Section 6.1)
    let mut headers_to_remove = Vec::new();
    
    if let Some(connection) = headers.get(header::CONNECTION) {
        if let Ok(conn_str) = connection.to_str() {
            for header_name in conn_str.split(',') {
                let header_name = header_name.trim();
                if !header_name.is_empty() {
                    if let Ok(name) = HeaderName::from_bytes(header_name.as_bytes()) {
                        headers_to_remove.push(name);
                    }
                }
            }
        }
    }
    
    for name in headers_to_remove {
        headers.remove(name);
    }

    // 2. Remove standard hop-by-hop headers
    for name in HOP_BY_HOP_REQUEST {
        if let Ok(header_name) = HeaderName::from_bytes(name.as_bytes()) {
            headers.remove(&header_name);
        }
    }

    // 3. Special handling for TE header: only "trailers" is allowed in HTTP/2 (RFC 7540 Section 8.1.2.2)
    if let Some(te) = headers.get(header::TE) {
        let is_trailers = if let Ok(te_str) = te.to_str() {
            te_str.eq_ignore_ascii_case("trailers")
        } else {
            false
        };
        
        if !is_trailers {
            headers.remove(header::TE);
        }
    }
}

pub fn sanitize_response_headers(headers: &HeaderMap) -> HeaderMap {
    let mut result = HeaderMap::new();
    for (name, value) in headers {
        let name_str = name.as_str();
        if !HOP_BY_HOP_RESPONSE.contains(&name_str) {
            result.insert(name.clone(), value.clone());
        }
    }
    result
}
