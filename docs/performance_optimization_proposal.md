# Performance Optimization Proposal: L4 & L7 Refactoring Plan

This document details the concrete code modifications, architectural trade-offs, and expected performance impacts of the proposed optimization plan for the `duotunnel` project. It is structured to serve as a technical specification for subsequent implementation.

---

## 1. L7 Zero-Copy Body Streaming (HTTP/1.x Drivers)

### 1.1 The Issue
Currently, when proxying HTTP/1.x request and response bodies, the proxy reads bytes from the QUIC stream (`quinn::RecvStream`) into a temporary stack-allocated buffer (8KB) and then performs a memory copy and heap allocation to wrap it in a `Bytes` struct.

- **Request Body Egress:** [http.rs:L182-L215](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/egress/http.rs#L182-L215)
- **Request Body Driver (Ingress):** [h1.rs:L230-L250](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/protocol/driver/h1.rs#L230-L250)

### 1.2 Refactoring Plan
Replace `recv.read()` with `recv.read_chunk()`. Quinn's `read_chunk` returns a `quinn::Chunk` that wraps the packet buffer directly inside a reference-counted `bytes::Bytes` object. This eliminates the intermediate 8KB buffer copy and heap allocation.

#### Code Modification: Request Body Driver in [h1.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/protocol/driver/h1.rs#L230-L250)

```diff
-                     let to_read = state.remaining.min(state.scratch.len());
-                     match recv.read(&mut state.scratch[..to_read]).await {
-                         Ok(Some(n)) => {
-                             state.remaining -= n;
-                             let data = Bytes::copy_from_slice(&state.scratch[..n]);
-                             if state.remaining == 0 {
-                                 if let (Some(tx), Some(recv)) =
-                                     (state.reclaim_tx.take(), state.recv.take())
-                                 {
-                                     let _ = tx.send(Reclaim {
-                                         recv,
-                                         overflow: Bytes::new(),
-                                     });
-                                 }
-                             }
-                             Ok(Some((hyper::body::Frame::data(data), state)))
-                         }
-                         Ok(None) => Ok(None),
-                         Err(e) => Err(std::io::Error::other(e)),
-                     }
+                     let to_read = state.remaining;
+                     match recv.read_chunk(to_read, true).await {
+                         Ok(Some(chunk)) => {
+                             let len = chunk.bytes.len();
+                             state.remaining -= len;
+                             let data = chunk.bytes; // Zero-copy reference
+                             if state.remaining == 0 {
+                                 if let (Some(tx), Some(recv)) =
+                                     (state.reclaim_tx.take(), state.recv.take())
+                                 {
+                                     let _ = tx.send(Reclaim {
+                                         recv,
+                                         overflow: Bytes::new(),
+                                     });
+                                 }
+                             }
+                             Ok(Some((hyper::body::Frame::data(data), state)))
+                         }
+                         Ok(None) => Ok(None),
+                         Err(e) => Err(std::io::Error::other(e)),
+                     }
```

### 1.3 Why It Works & Alternatives
- **Why:** Quinn receives UDP datagrams into its memory pool. Using `read_chunk` slices this memory and increments the reference count of the `Bytes` structure, bypassing the user-space copy.
- **Alternatives:** We could use a larger stack buffer (e.g., 64KB) to reduce syscalls, but that would still copy memory. `read_chunk` is the most optimal way to achieve true zero-copy in Quinn.
- **Impact:**
  - **Memory:** Zero allocations per chunk.
  - **CPU:** Saves memory-copy overhead; improves L2/L3 cache coherence.
  - **Performance Fit:** Excellent for high-bandwidth downloads and uploads.

---

## 2. L7 Zero-Copy Chunked Response Writer

### 2.1 The Issue
In [h1.rs:L301-L306](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/protocol/driver/h1.rs#L301-L306), the driver serializes HTTP response chunks by writing the chunk length in hex, putting the chunk body, and appending `\r\n` into a temporary `BytesMut` buffer before writing it to the QUIC stream:
```rust
chunk_buf.clear();
write!(chunk_buf, "{:x}\r\n", chunk.len()).unwrap();
chunk_buf.put_slice(chunk); // Memory copy!
chunk_buf.put_slice(b"\r\n");
self.send.write_all(&chunk_buf).await?;
```

### 2.2 Refactoring Plan
Avoid copying the chunk body. Stack-allocate a small array (32 bytes) for the chunk length prefix, write that prefix, write the chunk body directly, and write the chunk suffix `\r\n` sequentially.

#### Code Modification in [h1.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/protocol/driver/h1.rs#L295-L317)

```diff
-         let mut chunk_buf = BytesMut::with_capacity(8192 + 24);
          loop {
              match body.frame().await {
                  Some(Ok(frame)) => {
                      if let Some(chunk) = frame.data_ref() {
                          if !chunk.is_empty() {
-                             chunk_buf.clear();
-                             write!(chunk_buf, "{:x}\r\n", chunk.len()).unwrap();
-                             chunk_buf.put_slice(chunk);
-                             chunk_buf.put_slice(b"\r\n");
-                             self.send.write_all(&chunk_buf).await?;
+                             let mut prefix = [0u8; 32];
+                             let mut cursor = std::io::Cursor::new(&mut prefix[..]);
+                             write!(cursor, "{:x}\r\n", chunk.len()).unwrap();
+                             let prefix_len = cursor.position() as usize;
+                             
+                             self.send.write_all(&prefix[..prefix_len]).await?;
+                             self.send.write_all(chunk).await?;
+                             self.send.write_all(b"\r\n").await?;
                          }
                      }
```

### 2.3 Why It Works & Alternatives
- **Why:** Quinn's `SendStream` performs user-space stream buffering. Writing in three steps doesn't result in three UDP packets; Quinn coalesces the writes into a single UDP transmission.
- **Alternatives:** Use vectored writes (`write_vectored`), but it has higher API complexity in async traits. Sequential writes on buffered stream streams are simpler and zero-copy.
- **Impact:** Eliminates payload copying on HTTP chunked responses. Excellent for large data downloads.

---

## 3. Buffer Pool Sharing and Capacity Laxity

### 3.1 The Issue
The relay copy loops pool buffers using `take_buffer` and `return_buffer` in [copy.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/copy.rs#L16-L62). However, it strictly drops and reallocates buffers if their capacity does not exactly match the requested size:
```rust
fn take_buffer(buffer_size: usize) -> Vec<u8> {
    if let Some(mut buf) = LOCAL_POOL.with(|pool| pool.borrow_mut().pop()) {
        if buf.capacity() == buffer_size { // Strict check
             // ...
```
Also, [PeekBufPool](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/infra/peek_buf.rs#L39) zero-fills buffers on every reuse via `.resize(..., 0)`.

### 3.2 Refactoring Plan
1. Allow using a buffer if its capacity is greater than or equal to the requested size.
2. In `PeekBufPool`, maintain the vector length up to capacity, avoiding the safe but slow `.resize` zero-fill on reuse.

#### Code Modification in [copy.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/copy.rs#L16-L62)

```diff
  fn take_buffer(buffer_size: usize) -> Vec<u8> {
      if let Some(mut buf) = LOCAL_POOL.with(|pool| pool.borrow_mut().pop()) {
-         if buf.capacity() == buffer_size {
+         if buf.capacity() >= buffer_size {
              if buf.len() != buffer_size {
                  buf.resize(buffer_size, 0);
              }
              return buf;
          }
      }
      let global = global_pool();
      while let Some(mut buf) = global.pop() {
-         if buf.capacity() == buffer_size {
+         if buf.capacity() >= buffer_size {
              if buf.len() != buffer_size {
                  buf.resize(buffer_size, 0);
              }
              return buf;
          }
      }
      vec![0; buffer_size]
  }
  
  fn return_buffer(buf: Vec<u8>, buffer_size: usize) {
-     if buf.capacity() != buffer_size {
+     if buf.capacity() < buffer_size {
          return;
      }
```

### 3.3 Why It Works & Alternatives
- **Why:** Re-slicing or resizing a `Vec<u8>` to a smaller length is a constant-time O(1) operation that changes only the metadata (`len`) without modifying the allocated capacity or freeing the memory.
- **Alternatives:** Use a global alloc pool like `bytes::BytesMut` or jemalloc arenas. However, fixing the capacity check is a zero-dependency change that preserves thread-locality.
- **Impact:** Prevents allocator fragmentation and thrashing under mixed proxy configurations (e.g., when some tunnels run at 32KB and others at 64KB).

---

## 4. TCP Stream Splitting Lock Elimination

### 4.1 The Issue
The bidirectional TCP relay in [bridge.rs:L10](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/bridge.rs#L10) splits stream handles using `tokio::io::split()`:
```rust
let (a_read, a_write) = tokio::io::split(stream_a);
let (b_read, b_write) = tokio::io::split(stream_b);
```
`tokio::io::split()` is generic over `AsyncRead + AsyncWrite` but introduces a shared mutex lock (`BiLock`) between the read and write halves. When the network is highly active in both directions (full duplex), tasks on separate CPU cores contend for this lock.

### 4.2 Refactoring Plan
Specialize the TCP relay code path to use `TcpStream::into_split()`, which splits the socket at the OS level into two separate file descriptors/handles, allowing independent read and write tasks to execute concurrently without lock contention.

#### Code Modification in [relay.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/relay.rs#L48-L60)

```diff
  pub async fn relay_tcp_bidirectional(
      quic_recv: RecvStream,
      quic_send: SendStream,
      stream: TcpStream,
  ) -> Result<(u64, u64)> {
-     let (stream_read, stream_write) = tokio::io::split(stream);
+     let (stream_read, stream_write) = stream.into_split();
      let quic_to_stream =
          copy_buffered_then_shutdown(quic_recv, stream_write, DEFAULT_RELAY_BUF_SIZE);
      let stream_to_quic = copy_buffered_then_finish(stream_read, quic_send, DEFAULT_RELAY_BUF_SIZE);
      let (a, b) = tokio::try_join!(quic_to_stream, stream_to_quic)?;
      debug!(quic_to_stream = a, stream_to_quic = b, "relay completed");
      Ok((a, b))
  }
```

### 4.3 Why It Works & Alternatives
- **Why:** `TcpStream::into_split()` utilizes Tokio's split socket internals that leverage the OS kernel's support for concurrent read/write operations on the same socket file descriptor.
- **Alternatives:** Keep the generic `AsyncRead + AsyncWrite` but specialize TCP streams via trait detection. Specalization (`relay_tcp_bidirectional`) is already present and is the cleanest path.
- **Impact:** Eliminates mutex locks on L4 TCP forwarding. Prevents multi-core CPU bottlenecking under high-bandwidth full-duplex loads.

---

## 5. HTTP/2 over QUIC Double Multiplexing (H2c Egress)

### 5.1 The Issue (The Double-Framing / HoL Bottleneck)
In [h2_proxy.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/proxy/h2_proxy.rs#L53-L109), multiple downstream H2/gRPC streams are multiplexed onto a single cached H2 connection over a **single** QUIC stream.
This is a classic "Double Multiplexing" bottleneck:

```
[Downstream H2 Streams] ──► [Multiplexed over ONE QUIC Stream] ──► [QUIC Tunnel Connection]
```

- **Pros:** Ultra-low latency for small requests (no new QUIC stream creation, no duplicate `RoutingInfo` overhead).
- **Cons:** Any packet loss triggers QUIC stream retransmission, causing Head-of-Line (HoL) blocking for *all* multiplexed HTTP/2 streams on that QUIC stream. In addition, they all share a single QUIC stream flow-control window.

### 5.2 Refactoring Plan
Introduce a configuration flag `multiplex_mode` in [http_pool.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/config/http_pool.rs) to select the forwarding mode:

1. **`h2_over_quic` (Default / API optimized):** Keeps H2 streams multiplexed on a single cached H2 session over a single QUIC stream (best for low latency, small unary requests).
2. **`native_quic` (Bandwidth optimized):** Bypasses the H2 cache and opens a native QUIC stream for each request.

#### Architectural Proposal: Egress Routing Selector in `h2_proxy.rs`

```rust
pub async fn forward_h2_request_selective<B>(
    client_conn: &Connection,
    sender_cache: &H2Sender,
    routing_info: RoutingInfo,
    request: Request<B>,
    mode: MultiplexMode,
) -> Result<Response<BoxBody>> {
    match mode {
        MultiplexMode::H2OverQuic => {
            // Existing logic: Use single H2 session over a shared QUIC stream
            forward_h2_request(client_conn, sender_cache, routing_info, request).await
        }
        MultiplexMode::NativeQuic => {
            // Bandwidth logic: Open a dedicated QUIC stream for this request
            let (mut send, recv) = client_conn.open_bi().await?;
            send_routing_info(&mut send, &routing_info).await?;
            
            // Perform standard single-request H2/H1 handshake on this dedicated stream
            let io = TokioIo::new(QuinnStream { send, recv });
            let (mut sender, conn_driver) = H2ClientBuilder::new(TokioExecutor::new())
                .handshake(io)
                .await?;
            
            tokio::spawn(async move {
                let _ = conn_driver.await;
            });
            
            send_via(sender, request).await
        }
    }
}
```

### 5.3 Feasibility & Expected Impact
- **Feasibility:** High. Bypassing the cached sender and opening a new stream uses the existing native QUIC pipeline.
- **Expected Impact:**
  - **gRPC API Gateway:** Keeping `h2_over_quic` maintains sub-millisecond response latency.
  - **File Transfer/High Bandwidth:** Switching to `native_quic` eliminates Head-of-Line blocking and window bottlenecks, allowing BBR congestion control to scale up to maximum physical bandwidth.

---

## 6. User-Space Buffered Relay Architecture (2-Copy vs. 1-Copy)

### 6.1 Kernel-Level eBPF Limitations
While kernel-level optimizations like `bpf_sock_splice_pair()` can bypass the TCP/IP network stack for local loopback connections (TCP-to-TCP), they cannot accelerate the core proxy relay inside `duotunnel`.

In `duotunnel`, the data path crosses a **QUIC Stream (UDP) ◄─► TCP Stream** boundary.
1. The kernel is unaware of individual QUIC streams (which are user-space constructs inside the `quinn` library). It only sees a single UDP socket.
2. Direct socket redirection at the kernel level is impossible because redirecting the UDP socket would dump all multiplexed stream data into a single TCP socket, breaking protocol framing.
3. Therefore, the data **must** traverse user-space to be demultiplexed by the QUIC engine.

### 6.2 The "Courier & Mailbox" Analogy: Why 2-Copy is Optimal
When performing user-space copying between the QUIC stream and the TCP socket, we are faced with two design choices:

1. **Synchronous Rendezvous (1-Copy / No Buffer):**
   - The reader thread blocks until the writer thread is ready to transmit. 
   - *Analogy:* A courier delivering a package must wait at the door until the recipient is physically present to sign for it. If the recipient is busy (e.g., parsing headers or scheduling tasks), the courier is blocked.
   - This destroys concurrent execution, pipelining, and packet coalescing.

2. **Asynchronous Buffered Relay (2-Copy / Buffered):**
   - We read data into a temporary buffer in user-space (Copy 1: Kernel -> User) and write it out asynchronously (Copy 2: User -> Kernel).
   - *Analogy:* Installing a mailbox at the door. The courier drops off the package and immediately leaves. The recipient collects the packages in batches.
   - Pipelining allows the sender to keep sending without waiting for the recipient to finish processing. Wakes and CPU interrupts are batched, leading to massive throughput gains.

### 6.3 Design Impact
To leverage the 2-copy architectural advantages in `duotunnel`:
- **Buffer Pool Refactoring:** The pool optimizations described in **Section 3** are critical. We must ensure the user-space "mailbox" (buffer) is cheap to retrieve and reuse (eliminating strict capacity checks and zero-fill overhead).
- **User-space Spin-polling (Optional):** For latency-sensitive microservices, the copy loop can perform a brief spin-lock (e.g., 50 microseconds) polling the read stream before yielding the task. This keeps the thread active, avoiding the latency penalty of thread sleep and wake triggers.
