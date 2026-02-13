use super::{ProtocolDriver, ProxyRequest};
use anyhow::{Context, Result};
use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderMap, Method, Response, Uri, Version};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use quinn::{RecvStream, SendStream};
use crate::protocol::http_utils::{sanitize_request_headers, sanitize_response_headers};

pub struct Http1Driver {
    send: SendStream,
    recv: Option<RecvStream>,
    scheme: String,
    authority: String,
    initial_bytes: Option<Bytes>,
}

impl Http1Driver {
    pub fn new(
        send: SendStream, 
        recv: RecvStream, 
        scheme: String, 
        authority: String, 
        initial_bytes: Option<Bytes>
    ) -> Self {
        Self {
            send,
            recv: Some(recv),
            scheme,
            authority,
            initial_bytes,
        }
    }

    pub async fn finish(&mut self) -> Result<()> {
        self.send.finish()?;
        Ok(())
    }

    pub fn into_parts(mut self) -> (SendStream, Option<RecvStream>) {
        (self.send, self.recv.take())
    }
}

#[async_trait]
impl ProtocolDriver for Http1Driver {
    async fn read_request(&mut self) -> Result<Option<ProxyRequest>> {
        let mut recv = self.recv.take().context("RecvStream already consumed")?;

        // 1. Read Headers (or use initial bytes)
        let mut first_buf = vec![0u8; 8192];
        let mut pos = 0;
        
        // 1. Fill with initial bytes
        if let Some(data) = self.initial_bytes.take() {
            let n = std::cmp::min(data.len(), first_buf.len());
            first_buf[..n].copy_from_slice(&data[..n]);
            pos = n;
        }

        // 2. Loop until headers parsed
        // 2. Loop until headers parsed
        loop {
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers);
            
            // httparse reads from the slice.
            match req.parse(&first_buf[..pos]) {
                Ok(httparse::Status::Complete(_n)) => {
                    // We need to keep 'req' accessible or extract fields NOW.
                    // But 'req' borrows 'headers' which is local to loop.
                    // We must break and re-parse?
                    // Or extract fields here and break.
                    // BUT 'req' fields borrow from 'first_buf'.
                    // We can break, and then re-parse outside loop knowing it will succeed.
                    break;
                }
                Ok(httparse::Status::Partial) => {
                    if pos >= first_buf.len() {
                        return Err(anyhow::anyhow!("Headers exceeding buffer size"));
                    }
                    // Read more
                    match recv.read(&mut first_buf[pos..]).await? {
                        Some(n) => pos += n,
                        None => {
                            if pos == 0 {
                                return Ok(None); // EOF at start
                            }
                            return Err(anyhow::anyhow!("Unexpected EOF in headers"));
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        // Re-parse to extract fields (since loop scope dropped req)
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        let body_start = match req.parse(&first_buf[..pos])? {
            httparse::Status::Complete(n) => n,
            _ => unreachable!("Headers already validated as complete"),
        };
        
        let n = pos; // Total bytes read (headers + partial body)


        let method_str = req.method.context("no method")?;
        let path = req.path.context("no path")?;
        
        // Construct URI
        let uri_str = format!("{}://{}{}", self.scheme, self.authority, path);
        let uri: Uri = uri_str.parse()?;

        let method = match method_str {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "PATCH" => Method::PATCH,
            _ => Method::GET, // Fallback
        };

        let mut header_map = HeaderMap::new();
        for h in req.headers.iter() {
            if !h.name.is_empty() {
                if let (Ok(name), Ok(value)) = (
                    http::header::HeaderName::from_bytes(h.name.as_bytes()),
                    http::header::HeaderValue::from_bytes(h.value),
                ) {
                    header_map.insert(name, value);
                }
            }
        }

        // Sanitize headers for H1
        sanitize_request_headers(&mut header_map);

        // Prepare Body Stream
        let remaining_first = first_buf[body_start..n].to_vec(); // Clone remaining bytes

        let content_length: usize = header_map
            .get(http::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let stream = futures_util::stream::try_unfold(
            (recv, Some(Bytes::from(remaining_first)), 0usize, content_length),
            |(mut recv, first_chunk, mut read, total)| async move {
                // 1. Process initial chunk
                if let Some(chunk) = first_chunk {
                     let len = chunk.len();
                     if len > 0 {
                         read += len;
                         return Ok(Some((hyper::body::Frame::data(chunk), (recv, None, read, total))));
                     }
                }
                
                // 2. Read from stream
                // Simple logic: if content-length known, read until limit.
                // If 0, and not chunked, assume empty?
                // proxy.rs logic was: if content_length > 0, read. Else empty.
                // Note: proxy.rs also had infinite read fallback? Let's check proxy.rs.
                
                if total > 0 && read >= total {
                    return Ok(None);
                }
                
                let mut buf = vec![0u8; 8192];
                match recv.read(&mut buf).await {
                    Ok(Some(n)) => {
                        read += n;
                        let data = Bytes::copy_from_slice(&buf[..n]);
                        Ok(Some((hyper::body::Frame::data(data), (recv, None, read, total))))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
                }
            },
        );

        let body = if content_length > 0 {
            http_body_util::StreamBody::new(stream).boxed_unsync()
        } else {
            // Check if we should read until EOF? 
            // Current proxy.rs says: `if content_length > 0 { ... } else { empty }`
            // But it also had an `else` block for streaming if Connection: close?
            // Let's assume standard behavior: if no CL and no Transfer-Encoding, body is empty.
            http_body_util::Empty::new().map_err(|e| match e {}).boxed_unsync()
        };

        Ok(Some(ProxyRequest {
            method,
            uri,
            headers: header_map,
            version: Version::HTTP_11,
            body,
        }))
    }

    async fn write_response(&mut self, response: Response<Incoming>) -> Result<()> {
        let status_line = format!("HTTP/1.1 {} {}\r\n", 
            response.status().as_u16(), 
            response.status().canonical_reason().unwrap_or("OK")
        );
        self.send.write_all(status_line.as_bytes()).await?;

        let sanitized_headers = sanitize_response_headers(response.headers());
        for (name, value) in &sanitized_headers {
            let value_str = value.to_str().unwrap_or_else(|_| "");
            let header_line = format!("{}: {}\r\n", name, value_str);
            self.send.write_all(header_line.as_bytes()).await?;
        }
        
        self.send.write_all(b"Transfer-Encoding: chunked\r\n").await?;
        self.send.write_all(b"\r\n").await?;

        let mut body = response.into_body();
        let mut accumulated_trailers = HeaderMap::new();

        loop {
            match body.frame().await {
                Some(Ok(frame)) => {
                    if let Some(chunk) = frame.data_ref() {
                        if !chunk.is_empty() {
                            let chunk_header = format!("{:x}\r\n", chunk.len());
                            self.send.write_all(chunk_header.as_bytes()).await?;
                            self.send.write_all(chunk).await?;
                            self.send.write_all(b"\r\n").await?;
                        }
                    }
                    if let Ok(trailers) = frame.into_trailers() {
                        accumulated_trailers.extend(trailers);
                    }
                }
                Some(Err(e)) => return Err(anyhow::anyhow!("error reading response frame: {}", e)),
                None => break,
            }
        }

        self.send.write_all(b"0\r\n").await?;

        if !accumulated_trailers.is_empty() {
            for (name, value) in &accumulated_trailers {
                let value_str = value.to_str().unwrap_or_else(|_| "");
                let trailer_line = format!("{}: {}\r\n", name, value_str);
                self.send.write_all(trailer_line.as_bytes()).await?;
            }
        }

        self.send.write_all(b"\r\n").await?;
        Ok(())
    }
}
