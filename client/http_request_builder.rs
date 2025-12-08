use anyhow::{Result, Context};
use httparse::{Request, Status};
use hyper::http::Method;
use hyper::Uri;
use std::str::FromStr;
use bytes::BytesMut;

pub struct HttpRequestBuilder {
    method: Option<String>,
    uri: Option<String>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpRequestBuilder {
    pub fn new() -> Self {
        Self {
            method: None,
            uri: None,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn from_raw_bytes(bytes: &[u8]) -> Result<Self> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = Request::new(&mut headers);
        
        let parse_result = req.parse(bytes)?;
        match parse_result {
            Status::Complete(header_len) => {
                let method = req.method
                    .ok_or_else(|| anyhow::anyhow!("Missing HTTP method"))?
                    .to_string();
                
                let uri = req.path
                    .ok_or_else(|| anyhow::anyhow!("Missing HTTP path"))?
                    .to_string();
                
                let mut header_vec = Vec::new();
                for header in req.headers.iter() {
                    let name = header.name.to_string();
                    let value = std::str::from_utf8(header.value)
                        .with_context(|| format!("Invalid header value for {}", name))?
                        .to_string();
                    header_vec.push((name, value));
                }
                
                let body = if bytes.len() > header_len {
                    bytes[header_len..].to_vec()
                } else {
                    Vec::new()
                };
                
                Ok(Self {
                    method: Some(method),
                    uri: Some(uri),
                    headers: header_vec,
                    body,
                })
            }
            Status::Partial => {
                anyhow::bail!("Incomplete HTTP headers");
            }
        }
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = Some(method.to_string());
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_string());
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers.extend(headers);
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn to_hyper_request(self, base_uri: Option<&str>) -> Result<hyper::Request<http_body_util::Full<bytes::Bytes>>> {
        let method_str = self.method
            .ok_or_else(|| anyhow::anyhow!("Missing HTTP method"))?;
        
        let method = Method::from_str(&method_str)
            .with_context(|| format!("Invalid HTTP method: {}", method_str))?;
        
        let uri_str = if let Some(uri) = &self.uri {
            if uri.starts_with("http://") || uri.starts_with("https://") {
                uri.clone()
            } else if let Some(base) = base_uri {
                let base_uri = Uri::from_str(base)?;
                let base_path = base_uri.path();
                let base_scheme = base_uri.scheme_str().unwrap_or("http");
                let base_authority = base_uri.authority()
                    .map(|a| a.as_str())
                    .unwrap_or("localhost");
                
                let full_path = if uri.starts_with('/') {
                    uri.clone()
                } else {
                    format!("{}/{}", base_path.trim_end_matches('/'), uri)
                };
                
                format!("{}://{}{}", base_scheme, base_authority, full_path)
            } else {
                uri.clone()
            }
        } else {
            base_uri.unwrap_or("http://localhost/").to_string()
        };
        
        let uri = Uri::from_str(&uri_str)
            .with_context(|| format!("Invalid URI: {}", uri_str))?;
        
        let mut builder = hyper::Request::builder()
            .method(method)
            .uri(uri);
        
        for (name, value) in self.headers.iter() {
            if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("connection") {
                continue;
            }
            
            builder = builder.header(name, value);
        }
        
        let body = http_body_util::Full::new(bytes::Bytes::from(self.body));
        let request = builder.body(body)?;
        
        Ok(request)
    }

    pub fn to_raw_bytes(self) -> Result<Vec<u8>> {
        let method = self.method
            .ok_or_else(|| anyhow::anyhow!("Missing HTTP method"))?;
        
        let uri = self.uri
            .ok_or_else(|| anyhow::anyhow!("Missing HTTP URI"))?;
        
        let mut bytes = BytesMut::new();
        
        // Status line
        bytes.extend_from_slice(format!("{} {} HTTP/1.1\r\n", method, uri).as_bytes());
        
        // Headers
        for (name, value) in &self.headers {
            bytes.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }
        
        // Empty line
        bytes.extend_from_slice(b"\r\n");
        
        // Body
        bytes.extend_from_slice(&self.body);
        
        Ok(bytes.to_vec())
    }
}

impl Default for HttpRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

