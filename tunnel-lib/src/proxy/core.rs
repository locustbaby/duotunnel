use super::peers::PeerKind;
use crate::models::msg::RoutingInfo;
use anyhow::Result;
use std::net::SocketAddr;
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Protocol {
    H1,
    H2,
    WebSocket,
    Tcp,
    Unknown,
}
pub struct Context {
    pub client_addr: SocketAddr,
    pub protocol: Protocol,
    pub initial_bytes: Option<bytes::Bytes>,
    pub routing_info: Option<RoutingInfo>,
}
pub trait ProxyApp: Send + Sync {
    fn upstream_peer(
        &self,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<PeerKind>> + Send;
}
pub struct ProxyEngine<A: ProxyApp> {
    app: A,
}

thread_local! {
    /// Per-worker-thread reusable peek buffer. Avoids zeroed() allocation on every stream.
    static STREAM_PEEK_BUF: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl<A: ProxyApp> ProxyEngine<A> {
    pub fn new(app: A) -> Self {
        Self { app }
    }
    pub async fn run_stream(
        &self,
        send: quinn::SendStream,
        mut recv: quinn::RecvStream,
        client_addr: SocketAddr,
        routing_info: Option<RoutingInfo>,
    ) -> Result<()> {
        // Take the thread-local buffer so we can use it across the await point.
        // The TL slot holds an empty Vec while this frame is live; it is restored below.
        let mut tl_buf = STREAM_PEEK_BUF.with(|cell| std::mem::take(&mut *cell.borrow_mut()));
        if tl_buf.capacity() < 4096 {
            tl_buf.reserve(4096);
        }
        // SAFETY: capacity >= 4096; recv.read() overwrites bytes before any read occurs.
        unsafe { tl_buf.set_len(4096) };
        let n = recv.read(&mut tl_buf[..]).await?.unwrap_or(0);

        let initial_bytes: bytes::Bytes = if n > 0 {
            // Copy only the actual bytes into a frozen Bytes; return buffer to TL pool.
            let b = bytes::Bytes::copy_from_slice(&tl_buf[..n]);
            tl_buf.truncate(0);
            STREAM_PEEK_BUF.with(|cell| *cell.borrow_mut() = tl_buf);
            b
        } else {
            tl_buf.truncate(0);
            STREAM_PEEK_BUF.with(|cell| *cell.borrow_mut() = tl_buf);
            bytes::Bytes::new()
        };

        let protocol = detect_protocol(n, &initial_bytes, routing_info.as_ref());
        let mut ctx = Context {
            client_addr,
            protocol,
            initial_bytes: if n > 0 { Some(initial_bytes) } else { None },
            routing_info,
        };
        let peer = self.app.upstream_peer(&mut ctx).await?;
        peer.connect(send, recv, ctx.initial_bytes).await?;
        Ok(())
    }
}

/// Determine protocol from routing_info hint or from the first bytes of the stream.
/// Consolidates the websocket upgrade check into a single httparse pass.
fn detect_protocol(n: usize, data: &[u8], routing_info: Option<&RoutingInfo>) -> Protocol {
    if let Some(ri) = routing_info {
        match ri.protocol {
            Protocol::Unknown => {}
            p => return p,
        }
    }
    if n == 0 {
        return Protocol::Unknown;
    }
    // Single httparse pass: detect both WebSocket upgrade and H1 together.
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    if req.parse(data).is_ok() {
        for h in req.headers {
            if h.name.eq_ignore_ascii_case("Upgrade")
                && std::str::from_utf8(h.value)
                    .unwrap_or("")
                    .eq_ignore_ascii_case("websocket")
            {
                return Protocol::WebSocket;
            }
        }
        return Protocol::H1;
    }
    Protocol::H1
}
