use crate::extract_host_from_http;

const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// Detect the application protocol and optional Host/SNI from the initial bytes.
///
/// Returns `(&'static str, Option<String>)` so the protocol label is a zero-cost
/// static string slice — no heap allocation on the hot path.  Callers that need
/// to store it in a `String` field call `.to_string()` once at the storage site.
pub fn detect_protocol_and_host(data: &[u8]) -> (&'static str, Option<String>) {
    if data.len() >= HTTP2_PREFACE.len() && &data[..HTTP2_PREFACE.len()] == HTTP2_PREFACE {
        return ("h2", None);
    }

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    if let Ok(status) = req.parse(data) {
        if status.is_complete() || status.is_partial() {
            let host = extract_host_from_http(data);

            for h in req.headers.iter() {
                if h.name.eq_ignore_ascii_case("Upgrade") {
                    if let Ok(val) = std::str::from_utf8(h.value) {
                        if val.eq_ignore_ascii_case("websocket") {
                            return ("websocket", host);
                        }
                    }
                }
            }

            return ("h1", host);
        }
    }

    // Check for TLS ClientHello (SNI)
    // Content Type: 0x16 (Handshake), Version: 0x030X
    if data.len() > 5 && data[0] == 0x16 && data[1] == 0x03 {
        if let Some(sni) = extract_tls_sni(data) {
             // Return "tcp" for transparent TLS forwarding; SNI drives routing.
             return ("tcp", Some(sni));
        }
    }

    ("tcp", None)
}

pub fn extract_tls_sni(data: &[u8]) -> Option<String> {
    // Basic parser for TLS ClientHello to extract SNI extension
    let mut pos = 5; // Skip Record Header (5 bytes)
    
    if pos >= data.len() { return None; }
    
    // Handshake Type must be 0x01 (ClientHello)
    if data[pos] != 0x01 { return None; }
    pos += 1;
    
    // Skip Handshake Length (3 bytes) + Version (2 bytes) + Random (32 bytes)
    pos += 3 + 2 + 32;
    
    if pos >= data.len() { return None; }
    
    // Session ID Length
    let session_id_len = data[pos] as usize;
    pos += 1 + session_id_len;
    
    if pos + 2 > data.len() { return None; }
    // Cipher Suites Length
    let cipher_suites_len = ((data[pos] as usize) << 8) | (data[pos+1] as usize);
    pos += 2 + cipher_suites_len;
    
    if pos + 1 > data.len() { return None; }
    // Compression Methods Length
    let comp_methods_len = data[pos] as usize;
    pos += 1 + comp_methods_len;
    
    if pos + 2 > data.len() { return None; }
    // Extensions Length
    let ext_total_len = ((data[pos] as usize) << 8) | (data[pos+1] as usize);
    pos += 2;
    
    let ext_end = std::cmp::min(pos + ext_total_len, data.len());
    
    while pos + 4 <= ext_end {
        let ext_type = ((data[pos] as usize) << 8) | (data[pos+1] as usize);
        let ext_len = ((data[pos+2] as usize) << 8) | (data[pos+3] as usize);
        pos += 4;
        
        if ext_type == 0x00 { // Server Name Indication
            if pos + 2 <= ext_end {
                 // SNI List Length
                 let _list_len = ((data[pos] as usize) << 8) | (data[pos+1] as usize);
                 pos += 2;
                 
                 if pos + 3 <= ext_end {
                     // SNI Type (0x00 = HostName)
                     let sni_type = data[pos];
                     // SNI Length
                     let sni_len = ((data[pos+1] as usize) << 8) | (data[pos+2] as usize);
                     pos += 3;
                     
                     if sni_type == 0x00 && pos + sni_len <= ext_end {
                         return String::from_utf8(data[pos..pos+sni_len].to_vec()).ok();
                     }
                 }
            }
            return None;
        }
        
        pos += ext_len;
    }
    
    None
}

