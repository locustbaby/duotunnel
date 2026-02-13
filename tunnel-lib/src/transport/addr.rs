pub struct UpstreamAddr {
    pub is_https: bool,
    pub host: String,
    pub port: u16,
    pub connect_addr: String,
}

pub fn parse_upstream(addr: &str) -> UpstreamAddr {
    let is_https = addr.starts_with("https://") || addr.contains(":443");

    let clean_addr = addr
        .trim_start_matches("http://")
        .trim_start_matches("https://");

    let host = clean_addr.split(':').next().unwrap_or(clean_addr).to_string();

    let (connect_addr, port) = if clean_addr.contains(':') {
        let parts: Vec<&str> = clean_addr.split(':').collect();
        let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(if is_https { 443 } else { 80 });
        (clean_addr.to_string(), port)
    } else if is_https {
        (format!("{}:443", clean_addr), 443)
    } else {
        (format!("{}:80", clean_addr), 80)
    };

    UpstreamAddr { is_https, host, port, connect_addr }
}

pub fn normalize_host(host: &str) -> String {
    host.split(':').next().unwrap_or(host).to_lowercase()
}
