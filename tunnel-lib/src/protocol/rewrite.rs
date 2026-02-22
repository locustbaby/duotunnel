use hyper::HeaderMap;

#[derive(Default)]
pub struct Rewriter {
    pub host_rewrite: Option<String>,
    pub path_prefix: Option<String>,
    pub header_add: Vec<(String, String)>,
    pub header_remove: Vec<String>,
}

impl Rewriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_host(mut self, host: String) -> Self {
        self.host_rewrite = Some(host);
        self
    }

    pub fn with_path_prefix(mut self, prefix: String) -> Self {
        self.path_prefix = Some(prefix);
        self
    }

    pub fn apply(&self, headers: &mut HeaderMap, path: &mut String) {
        if let Some(ref prefix) = self.path_prefix {
            *path = format!("{}{}", prefix, path);
        }

        if let Some(ref host) = self.host_rewrite {
            if let Ok(value) = hyper::header::HeaderValue::from_str(host) {
                headers.insert(hyper::header::HOST, value);
            }
        }

        for name in &self.header_remove {
            if let Ok(header_name) = hyper::header::HeaderName::from_bytes(name.as_bytes()) {
                headers.remove(&header_name);
            }
        }

        for (name, value) in &self.header_add {
            if let Ok(header_name) = hyper::header::HeaderName::from_bytes(name.as_bytes()) {
                if let Ok(header_value) = hyper::header::HeaderValue::from_str(value) {
                    headers.insert(header_name, header_value);
                }
            }
        }
    }
}
