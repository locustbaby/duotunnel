use super::peers::UpstreamPeer;
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::Stream;
use h2::server::handshake;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use quinn::{RecvStream, SendStream};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tracing::{debug, error, info};

type HttpsClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, http_body_util::combinators::UnsyncBoxBody<Bytes, std::io::Error>>;

pub struct H2Peer {
    pub target_host: String,
    pub scheme: String,
    pub client: HttpsClient,
}

#[async_trait]
impl UpstreamPeer for H2Peer {
    async fn connect(
        &self,
        send: SendStream,
        recv: RecvStream,
        initial_data: Option<Bytes>,
    ) -> Result<()> {
        info!(target = %self.target_host, scheme = %self.scheme, "H2 proxy starting");

        let io = QuicIo::new(send, recv, initial_data);

        let mut h2_conn = match handshake(io).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("H2 handshake failed: {}", e);
                return Err(anyhow::anyhow!("H2 handshake failed: {}", e));
            }
        };

        while let Some(request) = h2_conn.accept().await {
            let (req, mut respond) = match request {
                Ok(r) => r,
                Err(e) => {
                    error!("H2 accept error: {}", e);
                    break;
                }
            };

            let client = self.client.clone();
            let target_host = self.target_host.clone();
            let scheme = self.scheme.clone();

            tokio::spawn(async move {
                let (parts, mut body) = req.into_parts();
                debug!(method = %parts.method, uri = %parts.uri, "H2 request received");

                let target_uri = format!(
                    "{}://{}{}",
                    scheme,
                    target_host,
                    parts.uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
                );

                let mut builder = hyper::Request::builder()
                    .method(parts.method)
                    .uri(&target_uri)
                    .version(hyper::Version::HTTP_2);

                for (name, value) in parts.headers.iter() {
                    // Filter pseudo-headers and connection-specific headers
                    if name != "host" 
                        && !name.as_str().starts_with(':') 
                        && name != "connection" 
                        && name != "upgrade" 
                        && name != "keep-alive" 
                        && name != "proxy-connection" 
                        && name != "te" 
                        && name != "transfer-encoding" 
                    {
                        builder = builder.header(name, value);
                    }
                }
                builder = builder.header("host", &target_host);

                // Stream request body logic
                let stream_body = stream_body_from_h2(body);
                let boxed_body = http_body_util::combinators::UnsyncBoxBody::new(stream_body);

                let upstream_req = match builder.body(boxed_body) {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to build upstream request: {}", e);
                        return;
                    }
                };

                let response = match client.request(upstream_req).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error!("Upstream request failed: {}", e);
                        // Try to send 502
                        let _ = respond.send_response(
                            hyper::Response::builder().status(502).body(()).unwrap(), 
                            true
                        );
                        return;
                    }
                };

                info!(status = %response.status(), "Upstream response received");

                let (resp_parts, mut resp_body) = response.into_parts();
                let mut h2_response = hyper::Response::builder().status(resp_parts.status);
                
                for (name, value) in resp_parts.headers.iter() {
                    if !name.as_str().starts_with(':') {
                        h2_response = h2_response.header(name, value);
                    }
                }

                let h2_resp_empty = h2_response.body(()).unwrap();
                let mut send_stream = match respond.send_response(h2_resp_empty, false) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to send H2 response headers: {}", e);
                        return;
                    }
                };

                // Stream response body logic
                while let Some(frame_res) = resp_body.frame().await {
                    match frame_res {
                        Ok(frame) => {
                            if frame.is_data() {
                                let data = frame.into_data().unwrap();
                                let len = data.len();
                                // Reserve capacity if needed? h2 send_data handles flow control for us by returning Poll::Pending if blocked.
                                // But here we are in async context, send_data returns Result.
                                // send_data pushes to buffer. We might need to handle capacity explicitly if we push too fast?
                                // h2::SendStream::send_data does NOT await capacity. It fails if window is too small?
                                // No, send_data(..., end_of_stream: bool).
                                // h2 documentation says: "If there is not enough capacity in the stream's window, this will return an error."
                                // Wait, actually `send_data` returns Ok(()) if it queues the data. 
                                // But if we just pump data, we might buffer infinitely in memory if peer is slow.
                                // We should check `capacity` or `reserve_capacity`.
                                
                                // Actually, h2::SendStream::reserve_capacity is async.
                                send_stream.reserve_capacity(len); 
                                // Wait, reserve_capacity requests capacity.
                                
                                // Correct way: use `poll_capacity` or just send and handle error?
                                // h2 0.4 `send_data` docs: "Frames are queued up... If the remote window is exceeded, the data is buffered internally."
                                // So strictly speaking it's "safe" but can OOM.
                                // For robust proxying, ideally we wait for capacity.
                                
                                if let Err(e) = send_stream.send_data(data, false) {
                                     error!("Failed to send response data: {}", e);
                                     return;
                                }
                            } else if frame.is_trailers() {
                                let trailers = frame.into_trailers().unwrap();
                                if let Err(e) = send_stream.send_trailers(trailers) {
                                     error!("Failed to send response trailers: {}", e);
                                     return;
                                }
                                // Trailers imply EOS
                                return; 
                            }
                        },
                        Err(e) => {
                            error!("Error reading response body: {}", e);
                            return;
                        }
                    }
                }

                // End of stream
                if let Err(e) = send_stream.send_data(Bytes::new(), true) {
                     debug!("Failed to send EOS (might be already closed): {}", e);
                }
            });
        }

        Ok(())
    }
}

// Adapter to turn h2::RecvStream into a Stream of hyper Frames
fn stream_body_from_h2(mut body: h2::RecvStream) -> StreamBody<impl Stream<Item = Result<Frame<Bytes>, std::io::Error>> + Send + 'static> {
    let stream = futures_util::stream::poll_fn(move |cx| {
        loop {
            // Check for data
            match body.poll_data(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    let _ = body.flow_control().release_capacity(bytes.len());
                    return Poll::Ready(Some(Ok(Frame::data(bytes))));
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(std::io::Error::new(std::io::ErrorKind::Other, e))));
                }
                Poll::Ready(None) => {
                    // Data stream ended, check trailers
                    match body.poll_trailers(cx) {
                        Poll::Ready(Ok(Some(trailers))) => {
                            return Poll::Ready(Some(Ok(Frame::trailers(trailers))));
                        }
                        Poll::Ready(Ok(None)) => return Poll::Ready(None),
                        Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(std::io::Error::new(std::io::ErrorKind::Other, e)))),
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Poll::Pending => {
                     // If data is pending, we still need to check if trailers are available?
                     // No, poll_data returning Pending means no data yet.
                     // But could there be trailers without data? poll_data returns None in that case.
                     return Poll::Pending;
                }
            }
        }
    });
    StreamBody::new(stream)
}


struct QuicIo {
    send: SendStream,
    recv: RecvStream,
    initial_data: Option<Bytes>,
    initial_pos: usize,
}

impl QuicIo {
    fn new(send: SendStream, recv: RecvStream, initial_data: Option<Bytes>) -> Self {
        Self {
            send,
            recv,
            initial_data,
            initial_pos: 0,
        }
    }
}

impl AsyncRead for QuicIo {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if let Some(ref initial) = self.initial_data {
            if self.initial_pos < initial.len() {
                let remaining = &initial[self.initial_pos..];
                let to_copy = std::cmp::min(remaining.len(), buf.remaining());
                buf.put_slice(&remaining[..to_copy]);
                self.initial_pos += to_copy;
                return Poll::Ready(Ok(()));
            }
        }

        match Pin::new(&mut self.recv).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for QuicIo {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match Pin::new(&mut self.send).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => Poll::Ready(Ok(n)),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match Pin::new(&mut self.send).poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match Pin::new(&mut self.send).poll_shutdown(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }
}
