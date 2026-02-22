use hpack::Decoder;

const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub fn extract_h2_authority(data: &[u8]) -> Option<String> {
    let start = if data.starts_with(HTTP2_PREFACE) {
        HTTP2_PREFACE.len()
    } else {
        0
    };
    
    let remaining = &data[start..];
    
    let mut pos = 0;
    while pos + 9 <= remaining.len() {
        let length = ((remaining[pos] as usize) << 16) 
            | ((remaining[pos + 1] as usize) << 8) 
            | (remaining[pos + 2] as usize);
        let frame_type = remaining[pos + 3];
        
        if frame_type == 0x01 {
            let flags = remaining[pos + 4];
            let mut payload_start = pos + 9;
            let mut payload_end = std::cmp::min(pos + 9 + length, remaining.len());
            
            // Handle PADDED flag (0x8)
            let mut pad_length = 0;
            if flags & 0x08 != 0 && payload_start < remaining.len() {
                pad_length = remaining[payload_start] as usize;
                payload_start += 1;
            }
            
            // Handle PRIORITY flag (0x20)
            if flags & 0x20 != 0 {
                payload_start += 5; // Stream Dependency (4) + Weight (1)
            }
            
            if payload_start <= payload_end {
                // Remove padding from end
                if pad_length > 0 && payload_end >= pad_length {
                    payload_end -= pad_length;
                }
                
                if payload_start < payload_end {
                    let header_block = &remaining[payload_start..payload_end];
                    if let Some(authority) = decode_hpack_authority(header_block) {
                        return Some(authority);
                    }
                }
            }
        }
        
        pos += 9 + length;
        if pos > remaining.len() {
            break;
        }
    }
    
    None
}

fn decode_hpack_authority(data: &[u8]) -> Option<String> {
    let mut decoder = Decoder::new();
    
    if let Ok(headers) = decoder.decode(data) {
        for (name, value) in headers {
            let name_str = String::from_utf8_lossy(&name);
            if name_str == ":authority" {
                return String::from_utf8(value).ok();
            }
        }
    }
    
    None
}
