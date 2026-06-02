use crate::proxy::core::Protocol;
use crate::sniff::{default_ingress_detectors, SniffOutcome};

pub fn detect_protocol_and_host(data: &[u8]) -> (Protocol, Option<String>) {
    let mut need_more = false;

    for detector in default_ingress_detectors() {
        match detector.detect(data) {
            SniffOutcome::Matched(hint) => {
                return (hint.protocol, hint.sni.or(hint.authority));
            }
            SniffOutcome::NeedMore => need_more = true,
            SniffOutcome::NoMatch => {}
        }
    }

    if need_more && data.starts_with(b"PRI") {
        return (Protocol::Tcp, None);
    }

    (Protocol::Tcp, None)
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
