use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
}

impl HttpVersion {
    /// Detect HTTP version from request bytes
    /// Checks for HTTP/2 connection preface first, then parses request line
    pub fn detect_from_request(request_bytes: &[u8]) -> Result<Self> {
        // Check for HTTP/2 connection preface: "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
        // or check if it starts with HTTP/2 binary frame (first 3 bytes: 0x00, 0x00, length)
        if request_bytes.len() >= 24 {
            let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
            if request_bytes.starts_with(preface) {
                return Ok(HttpVersion::Http2);
            }
        }
        
        // Check for HTTP/2 binary frame format (starts with frame length)
        // HTTP/2 frames start with 3-byte length field
        if request_bytes.len() >= 3 {
            // If it's not ASCII text, might be HTTP/2 binary
            if !request_bytes[..3].iter().all(|&b| b.is_ascii() && (b.is_ascii_alphanumeric() || b == b' ' || b == b'/' || b == b'\r' || b == b'\n')) {
                // Could be HTTP/2, but we'll parse request line to be sure
            }
        }
        
        // Parse request line to extract HTTP version
        let request_str = match std::str::from_utf8(request_bytes) {
            Ok(s) => s,
            Err(_) => {
                // If not valid UTF-8, might be HTTP/2 binary
                // Default to HTTP/1.1 for now
                return Ok(HttpVersion::Http11);
            }
        };
        
        // Find first line (request line)
        if let Some(first_line_end) = request_str.find('\n') {
            let first_line = &request_str[..first_line_end].trim();
            
            // Parse: METHOD PATH HTTP/VERSION
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 3 {
                let version_str = parts[2].to_uppercase();
                if version_str.starts_with("HTTP/2") {
                    return Ok(HttpVersion::Http2);
                } else if version_str.starts_with("HTTP/1.1") {
                    return Ok(HttpVersion::Http11);
                } else if version_str.starts_with("HTTP/1.0") {
                    return Ok(HttpVersion::Http10);
                }
            }
        }
        
        // Default to HTTP/1.1
        Ok(HttpVersion::Http11)
    }
    
    /// Get HTTP version string for response status line
    pub fn to_status_line_string(self) -> &'static str {
        match self {
            HttpVersion::Http10 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
            HttpVersion::Http2 => "HTTP/2",
        }
    }
    
    /// Check if version supports HTTP/2 features
    pub fn supports_http2_features(self) -> bool {
        matches!(self, HttpVersion::Http2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_http11() {
        let request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert_eq!(HttpVersion::detect_from_request(request).unwrap(), HttpVersion::Http11);
    }
    
    #[test]
    fn test_detect_http10() {
        let request = b"GET / HTTP/1.0\r\nHost: example.com\r\n\r\n";
        assert_eq!(HttpVersion::detect_from_request(request).unwrap(), HttpVersion::Http10);
    }
    
    #[test]
    fn test_detect_http2_preface() {
        let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
        assert_eq!(HttpVersion::detect_from_request(preface).unwrap(), HttpVersion::Http2);
    }
}

