use hyper::header::{self, HeaderMap, HeaderName};
pub fn sanitize_request_headers(headers: &mut HeaderMap) {
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
    headers.remove(header::CONNECTION);
    headers.remove(HeaderName::from_static("keep-alive"));
    headers.remove(HeaderName::from_static("proxy-connection"));
    headers.remove(header::PROXY_AUTHENTICATE);
    headers.remove(header::PROXY_AUTHORIZATION);
    headers.remove(header::TRANSFER_ENCODING);
    headers.remove(header::UPGRADE);
    headers.remove(header::TRAILER);
    headers.remove(header::HOST);
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
pub fn sanitize_response_headers(headers: &mut HeaderMap) {
    headers.remove(header::TRANSFER_ENCODING);
    headers.remove(header::CONTENT_LENGTH);
}
