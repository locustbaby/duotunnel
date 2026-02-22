/// Tunable buffer sizes for the proxy/protocol layer.
///
/// These control how many bytes are allocated for protocol detection, HTTP
/// header parsing, and body streaming. All defaults preserve the previous
/// hard-coded values, so existing callers are fully backward-compatible.
#[derive(Debug, Clone)]
pub struct ProxyBufferParams {
    /// Stack-allocated peek buffer used at entry listeners to detect the
    /// protocol (TLS/H2/H1/TCP) before routing. Larger values allow detecting
    /// protocols with longer preambles, but increase per-connection stack
    /// pressure. Default: 16384 bytes.
    pub peek_buf_size: usize,

    /// Heap-allocated buffer for reading the initial HTTP request headers.
    /// Requests with headers larger than this limit are rejected.
    /// Default: 8192 bytes.
    pub http_header_buf_size: usize,

    /// Chunk size used when streaming an HTTP request or response body.
    /// Smaller values reduce memory per in-flight stream; larger values
    /// improve throughput for bulk transfers. Default: 8192 bytes.
    pub http_body_chunk_size: usize,
}

impl Default for ProxyBufferParams {
    fn default() -> Self {
        Self {
            peek_buf_size: 16384,
            http_header_buf_size: 8192,
            http_body_chunk_size: 8192,
        }
    }
}
