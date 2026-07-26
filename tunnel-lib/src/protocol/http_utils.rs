use hyper::header::{self, HeaderMap, HeaderName};

/// Extract the value of the `Host` header from a raw HTTP/1.x request buffer.
/// Returns `None` if the request cannot be parsed or has no valid Host header.
pub fn extract_host_from_http(data: &[u8]) -> Option<String> {
    let mut lines = data.split(|byte| *byte == b'\n');
    lines.next()?;

    for line in lines {
        let line = trim_line_end(line);
        if line.is_empty() {
            return None;
        }
        if let Some((name, value)) = split_once_byte(line, b':') {
            if name.eq_ignore_ascii_case(b"host") {
                let value = trim_ascii(value);
                return std::str::from_utf8(value).ok().map(ToOwned::to_owned);
            }
        } else if !starts_with_ascii_whitespace(line) {
            return None;
        }
    }

    if let Some(line) = data.rsplit(|byte| *byte == b'\n').next() {
        let line = trim_line_end(line);
        if let Some((name, value)) = split_once_byte(line, b':') {
            if name.eq_ignore_ascii_case(b"host") {
                let value = trim_ascii(value);
                return std::str::from_utf8(value).ok().map(ToOwned::to_owned);
            }
        }
    }

    None
}

/// Extract the HTTP method and path from the first line of a raw HTTP/1.x request buffer.
pub fn extract_method_path_from_http(data: &[u8]) -> Option<(String, String)> {
    let first_line = trim_line_end(data.split(|byte| *byte == b'\n').next()?);
    let mut parts = first_line.split(|byte| byte.is_ascii_whitespace());
    let method = parts.find(|part| !part.is_empty())?;
    let path = parts.find(|part| !part.is_empty())?;

    Some((
        std::str::from_utf8(method).ok()?.to_string(),
        std::str::from_utf8(path).ok()?.to_string(),
    ))
}

fn split_once_byte(value: &[u8], delimiter: u8) -> Option<(&[u8], &[u8])> {
    let idx = value.iter().position(|byte| *byte == delimiter)?;
    Some((&value[..idx], &value[idx + 1..]))
}

fn starts_with_ascii_whitespace(value: &[u8]) -> bool {
    value.first().is_some_and(|byte| byte.is_ascii_whitespace())
}

fn trim_line_end(value: &[u8]) -> &[u8] {
    value.strip_suffix(b"\r").unwrap_or(value)
}

fn trim_ascii(value: &[u8]) -> &[u8] {
    let start = value
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|idx| idx + 1)
        .unwrap_or(start);
    &value[start..end]
}
/// Drop the connection-specific fields listed in RFC 9110 §7.6.1 plus the
/// fields `Connection` itself names. Applies to both directions: a hop-by-hop
/// field forwarded onward leaks this hop's connection state to the peer.
fn remove_hop_by_hop_headers(headers: &mut HeaderMap) {
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
}
pub fn sanitize_request_headers(headers: &mut HeaderMap) {
    remove_hop_by_hop_headers(headers);
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
    remove_hop_by_hop_headers(headers);
    headers.remove(header::TE);
    // The caller re-derives framing from the length it captured before this
    // point, so the upstream's own framing fields must not survive.
    headers.remove(header::CONTENT_LENGTH);
}

/// Whether a trailer field may be relayed to the downstream client.
///
/// RFC 9112 §7.1.2 forbids framing, routing and control fields in a trailer
/// section; relaying them unfiltered lets an upstream inject `Content-Length`
/// or `Transfer-Encoding` after the chunked terminator, which a shared
/// downstream reads as the start of a second response. Mirrors the disallow
/// list hyper applies when it encodes trailers, plus the hop-by-hop fields.
pub fn is_forwardable_trailer(name: &HeaderName) -> bool {
    if name.as_str().starts_with("proxy-") || name.as_str() == "keep-alive" {
        return false;
    }
    !matches!(
        *name,
        header::AUTHORIZATION
            | header::CACHE_CONTROL
            | header::CONNECTION
            | header::CONTENT_ENCODING
            | header::CONTENT_LENGTH
            | header::CONTENT_RANGE
            | header::CONTENT_TYPE
            | header::HOST
            | header::MAX_FORWARDS
            | header::SET_COOKIE
            | header::TE
            | header::TRAILER
            | header::TRANSFER_ENCODING
            | header::UPGRADE
    )
}

/// Parse one `Content-Length` field value.
///
/// RFC 9112 §6.2 defines the value as `1*DIGIT`, so every byte is validated
/// here: `usize::from_str` also accepts a `+` prefix, and a length we accept
/// but the upstream parser rejects is forwarded verbatim next to the chunked
/// framing hyper then adds — a CL.TE smuggling request of our own making.
/// RFC 9110 §8.6 permits accepting a comma-separated list (legacy proxies
/// merge duplicate fields) as long as every element carries the same value.
pub fn parse_content_length(value: &[u8]) -> Option<u64> {
    let mut result: Option<u64> = None;
    for element in value.split(|byte| *byte == b',') {
        let parsed = from_digits(element.trim_ascii())?;
        match result {
            None => result = Some(parsed),
            Some(previous) if previous == parsed => {}
            Some(_) => return None,
        }
    }
    result
}

/// Resolve a message's effective `Content-Length`, rejecting malformed or
/// conflicting field values exactly as [`parse_content_length`] does.
pub fn content_length_from_headers(headers: &HeaderMap) -> Option<u64> {
    let mut result: Option<u64> = None;
    for value in headers.get_all(header::CONTENT_LENGTH) {
        let parsed = parse_content_length(value.as_bytes())?;
        match result {
            None => result = Some(parsed),
            Some(previous) if previous == parsed => {}
            Some(_) => return None,
        }
    }
    result
}

fn from_digits(digits: &[u8]) -> Option<u64> {
    if digits.is_empty() {
        return None;
    }
    let mut result: u64 = 0;
    for byte in digits {
        let digit = match byte {
            b'0'..=b'9' => u64::from(byte - b'0'),
            _ => return None,
        };
        result = result.checked_mul(10)?.checked_add(digit)?;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_host_case_insensitively_with_port() {
        let req = b"GET / HTTP/1.1\r\nHOST: example.com:8443\r\n\r\n";

        assert_eq!(
            extract_host_from_http(req),
            Some("example.com:8443".to_string())
        );
    }

    #[test]
    fn extracts_host_with_surrounding_whitespace() {
        let req = b"GET / HTTP/1.1\r\nHost: \t example.com  \r\n\r\n";

        assert_eq!(extract_host_from_http(req), Some("example.com".to_string()));
    }

    #[test]
    fn extracts_host_from_partial_header_buffer() {
        let req = b"GET / HTTP/1.1\r\nHost: example.com";

        assert_eq!(extract_host_from_http(req), Some("example.com".to_string()));
    }

    #[test]
    fn missing_host_returns_none() {
        let req = b"GET / HTTP/1.1\r\nUser-Agent: test\r\n\r\n";

        assert_eq!(extract_host_from_http(req), None);
    }

    #[test]
    fn invalid_utf8_host_returns_none() {
        let req = b"GET / HTTP/1.1\r\nHost: \xff\r\n\r\n";

        assert_eq!(extract_host_from_http(req), None);
    }

    #[test]
    fn invalid_utf8_non_host_header_does_not_block_host() {
        let req = b"GET / HTTP/1.1\r\nX-Raw: \xff\r\nHost: example.com\r\n\r\n";

        assert_eq!(extract_host_from_http(req), Some("example.com".to_string()));
    }

    #[test]
    fn extracts_method_and_path() {
        let req = b"POST /submit?x=1 HTTP/1.1\r\nHost: example.com\r\n\r\n";

        assert_eq!(
            extract_method_path_from_http(req),
            Some(("POST".to_string(), "/submit?x=1".to_string()))
        );
    }

    #[test]
    fn extracts_method_and_path_without_headers() {
        let req = b"GET /ready";

        assert_eq!(
            extract_method_path_from_http(req),
            Some(("GET".to_string(), "/ready".to_string()))
        );
    }

    #[test]
    fn missing_path_returns_none() {
        let req = b"GET\r\nHost: example.com\r\n\r\n";

        assert_eq!(extract_method_path_from_http(req), None);
    }

    #[test]
    fn content_length_rejects_signed_prefix() {
        assert_eq!(parse_content_length(b"+5"), None);
        assert_eq!(parse_content_length(b"-5"), None);
        assert_eq!(parse_content_length(b""), None);
        assert_eq!(parse_content_length(b"5x"), None);
        assert_eq!(parse_content_length(b"0x10"), None);
    }

    #[test]
    fn content_length_accepts_repeated_identical_values() {
        assert_eq!(parse_content_length(b" 5 "), Some(5));
        assert_eq!(parse_content_length(b"5, 5"), Some(5));
        assert_eq!(parse_content_length(b"5, 6"), None);
        assert_eq!(parse_content_length(b"5,"), None);
    }

    #[test]
    fn content_length_from_headers_rejects_conflicts() {
        let mut headers = HeaderMap::new();
        headers.append(header::CONTENT_LENGTH, "7".parse().unwrap());
        assert_eq!(content_length_from_headers(&headers), Some(7));

        headers.append(header::CONTENT_LENGTH, "7".parse().unwrap());
        assert_eq!(content_length_from_headers(&headers), Some(7));

        headers.append(header::CONTENT_LENGTH, "8".parse().unwrap());
        assert_eq!(content_length_from_headers(&headers), None);
    }

    #[test]
    fn response_sanitizer_strips_hop_by_hop_fields() {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONNECTION, "keep-alive, X-Foo".parse().unwrap());
        headers.insert(
            HeaderName::from_static("keep-alive"),
            "timeout=5".parse().unwrap(),
        );
        headers.insert(HeaderName::from_static("x-foo"), "leak".parse().unwrap());
        headers.insert(header::UPGRADE, "websocket".parse().unwrap());
        headers.insert(header::PROXY_AUTHENTICATE, "Basic".parse().unwrap());
        headers.insert(header::TRANSFER_ENCODING, "chunked".parse().unwrap());
        headers.insert(header::CONTENT_LENGTH, "3".parse().unwrap());
        headers.insert(header::SERVER, "upstream".parse().unwrap());

        sanitize_response_headers(&mut headers);

        assert_eq!(headers.len(), 1);
        assert!(headers.contains_key(header::SERVER));
    }

    #[test]
    fn trailer_filter_rejects_framing_fields() {
        for name in [
            "content-length",
            "transfer-encoding",
            "te",
            "trailer",
            "host",
            "connection",
            "keep-alive",
            "upgrade",
            "proxy-authenticate",
            "set-cookie",
        ] {
            assert!(
                !is_forwardable_trailer(&HeaderName::from_bytes(name.as_bytes()).unwrap()),
                "{name} must not be relayed as a trailer"
            );
        }
        assert!(is_forwardable_trailer(&HeaderName::from_static(
            "grpc-status"
        )));
    }
}
