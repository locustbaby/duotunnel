const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub fn detect_protocol_and_host(data: &[u8]) -> (&'static str, Option<String>) {
    if data.len() >= HTTP2_PREFACE.len() && &data[..HTTP2_PREFACE.len()] == HTTP2_PREFACE {
        return ("h2", None);
    }
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    if let Ok(status) = req.parse(data) {
        if status.is_complete() {
            // Extract Host and check Upgrade in a single pass over already-parsed headers.
            // Previously this called extract_host_from_http() which re-scanned the raw
            // bytes as a UTF-8 string — now we read directly from the httparse result.
            let mut host: Option<String> = None;
            let mut is_websocket = false;
            for h in req.headers.iter() {
                if h.name.eq_ignore_ascii_case("Host") {
                    host = std::str::from_utf8(h.value)
                        .ok()
                        .map(|s| s.trim().to_string());
                } else if h.name.eq_ignore_ascii_case("Upgrade") {
                    if std::str::from_utf8(h.value)
                        .unwrap_or("")
                        .eq_ignore_ascii_case("websocket")
                    {
                        is_websocket = true;
                    }
                }
            }
            if is_websocket {
                return ("websocket", host);
            }
            return ("h1", host);
        }
    }
    if data.len() > 5 && data[0] == 0x16 && data[1] == 0x03 {
        if let Some(sni) = extract_tls_sni(data) {
            return ("tcp", Some(sni));
        }
    }
    ("tcp", None)
}

pub fn extract_tls_sni(data: &[u8]) -> Option<String> {
    let mut pos = 5;
    if pos >= data.len() {
        return None;
    }
    if data[pos] != 0x01 {
        return None;
    }
    pos += 1;
    pos += 3 + 2 + 32;
    if pos >= data.len() {
        return None;
    }
    let session_id_len = data[pos] as usize;
    pos += 1 + session_id_len;
    if pos + 2 > data.len() {
        return None;
    }
    let cipher_suites_len = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
    pos += 2 + cipher_suites_len;
    if pos + 1 > data.len() {
        return None;
    }
    let comp_methods_len = data[pos] as usize;
    pos += 1 + comp_methods_len;
    if pos + 2 > data.len() {
        return None;
    }
    let ext_total_len = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
    pos += 2;
    let ext_end = std::cmp::min(pos + ext_total_len, data.len());
    while pos + 4 <= ext_end {
        let ext_type = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
        let ext_len = ((data[pos + 2] as usize) << 8) | (data[pos + 3] as usize);
        pos += 4;
        if ext_type == 0x00 {
            if pos + 2 <= ext_end {
                let _list_len = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                pos += 2;
                if pos + 3 <= ext_end {
                    let sni_type = data[pos];
                    let sni_len = ((data[pos + 1] as usize) << 8) | (data[pos + 2] as usize);
                    pos += 3;
                    if sni_type == 0x00 && pos + sni_len <= ext_end {
                        return String::from_utf8(data[pos..pos + sni_len].to_vec()).ok();
                    }
                }
            }
            return None;
        }
        pos += ext_len;
    }
    None
}
