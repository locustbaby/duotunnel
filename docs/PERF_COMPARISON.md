# DuoTunnel vs GT vs frp：代码实现性能对比

基于源码级别对比，分析 DuoTunnel 相比 GT (Go+QUIC) 和 frp (Go+TCP+yamux) 的性能差距与优化空间。

---

## 1. 数据转发核心路径 (Relay/Copy)

这是隧道代理中最影响性能的路径：所有流量都要经过 relay 函数双向拷贝。

### GT 的实现

```go
// libcs/pool/pool.go — 全局 buffer 池
const MaxBufferSize = 4 * 1024  // 4KB 固定大小

var readerPool = sync.Pool{
    New: func() interface{} {
        return bufio.NewReaderSize(nil, MaxBufferSize)
    },
}

func GetReader(rd io.Reader) *bufio.Reader {
    reader := readerPool.Get().(*bufio.Reader)
    reader.Reset(rd)
    return reader
}

func PutReader(reader *bufio.Reader) {
    reader.Reset(nil)  // 释放底层 reader 引用
    readerPool.Put(reader)
}
```

GT **不使用 `io.Copy`**。它手动实现 relay，在 pooled buffer 内预填帧头，一次系统调用写出完整帧：

```go
// server/conn.go — process() 核心 relay 循环
buf := pool.BytesPool.Get().([]byte)  // 从 sync.Pool 获取 4KB buffer
defer pool.BytesPool.Put(buf)

// 预填 10 字节帧头: 4B taskID + 2B opcode + 4B length
buf[0..4] = taskID
buf[4..6] = predef.Data
for {
    l, rErr = task.Reader.Read(buf[10:])      // 在帧头之后的位置读 payload
    buf[6..10] = length(l)                     // 填入长度
    _, wErr = c.Write(buf[:10+l])              // 单次 write: 帧头+payload
}
```

关键特征：
- **sync.Pool 复用 4KB buffer + bufio.Reader**：零 GC 压力
- **帧头+payload 单次 write**：避免 scatter/gather IO，减少系统调用次数
- **单 QUIC stream 自定义帧协议**：GT 不是每个代理连接开 QUIC stream，而是在一个 stream 上用 taskID 多路复用
- 自定义 fork 的 `bufio.LimitedReader.WriteTo()` 直接从内部 buffer 写到目标 writer
- `WriteErr` 包装区分读错误/写错误——读错误杀隧道，写错误只关任务

### frp 的实现

```go
// pkg/util/util/util.go
func Join(c1 io.ReadWriteCloser, c2 io.ReadWriteCloser) (inCount int64, outCount int64) {
    var wait sync.WaitGroup
    pipe := func(to io.ReadWriteCloser, from io.ReadWriteCloser, count *int64) {
        defer to.Close()
        defer from.Close()
        defer wait.Done()
        *count, _ = io.Copy(to, from)
    }
    wait.Add(2)
    go pipe(c1, c2, &inCount)
    go pipe(c2, c1, &outCount)
    wait.Wait()
    return
}
```

frp 的 relay 特征：
- 使用 `io.CopyBuffer` + **16KB pooled buffer**（不是默认的 32KB `io.Copy`）
- buffer pool 是**分层 sync.Pool**：5 个 tier（16KB/5KB/2KB/1KB/default），按需选择最合适的大小
- **yamux 预建连接池** (`PoolCount: 1`, `MaxPoolCount: 5`)：通过 channel 管理，非阻塞获取
- `SetKeepAlive` + `SetKeepAlivePeriod(7200s)` 维持长连接
- frp 的 Mux 层 (`pkg/util/mux/mux.go`) 做协议检测时使用 `SharedConn` — 只 peek 不消费字节
- **注意**: 因为 yamux/TLS/压缩层包装了原始 TCP conn，`io.CopyBuffer` 无法触发 splice，数据始终走 userspace

### DuoTunnel 的实现

```rust
// engine/relay.rs
pub async fn relay_bidirectional<S>(
    mut quic_recv: RecvStream,
    mut quic_send: SendStream,
    stream: S,
) -> Result<(u64, u64)> {
    let (mut stream_read, mut stream_write) = tokio::io::split(stream); // ← Arc<Mutex>
    let quic_to_stream = async {
        let bytes = tokio::io::copy(&mut quic_recv, &mut stream_write).await?;
        let _ = stream_write.shutdown().await;
        Ok::<_, std::io::Error>(bytes)
    };
    // ...
}

// proxy/base.rs — 另一套 relay，多加了 BufReader/BufWriter
let quic_to_tcp = async {
    tokio::io::copy(
        &mut tokio::io::BufReader::new(recv),       // ← 多余的 8KB buffer
        &mut tokio::io::BufWriter::new(tcp_write),   // ← 多余的 8KB buffer
    ).await
};
```

**DuoTunnel 的性能瓶颈**:

| 问题 | 影响 | GT/frp 怎么做 |
|------|------|--------------|
| `tokio::io::split` 用 `Arc<Mutex>` | 每次 IO 操作加锁/解锁 | GT: Go goroutine 天然支持半双工读写<br>frp: `io.Copy` 在独立 goroutine |
| 三套 relay 实现不统一 | base.rs 多加 BufReader/BufWriter 在 QUIC 上是反效果（QUIC 自带帧缓冲） | GT/frp: 统一 `io.Copy` 一个实现 |
| 每连接 `vec![0u8; 4096]` 新分配 | 高并发下 GC/allocator 压力大 | GT: `sync.Pool` 复用 4KB buffer<br>frp: Go runtime 管理 |
| 无零拷贝机会 | Rust `tokio::io::copy` 不触发 splice | Go `io.Copy` 自动 splice(2) |

**优化建议**:
1. 对 `TcpStream` 使用 `into_split()`（零开销）替代 `tokio::io::split()`（`Arc<Mutex>`）
2. 统一为一套 relay 实现，去掉 base.rs 的多余 BufReader/BufWriter
3. 引入 buffer pool（`thread_local!` 或 crossbeam pool）复用 read buffer

---

## 2. QUIC 传输层配置

### GT 的 QUIC 配置

```go
// GT 使用 quic-go，config.go 中：
RemoteConnections:     3,    // 预建 3 个 QUIC 连接
RemoteIdleConnections: 1,    // 保持 1 个空闲
RemoteTimeout:         45s,  // 连接超时 45s

// GT 支持 BBR 拥塞控制（通过 msquic）
OpenBBR: false  // 默认 Cubic，可选 BBR

// GT 服务端配置
Connections: 10,  // 每客户端最大 10 个隧道连接
Timeout:     90s, // 连接超时 90s
```

GT 的关键设计：
- **单 QUIC stream + 自定义帧协议**：不是每个代理连接一个 QUIC stream，而是在一个 stream 上用 taskID 多路复用所有任务。这避免了 QUIC stream 创建/关闭的开销，但也放弃了 QUIC 原生的流级别流控
- **多 QUIC 连接池**：预建 3 个 QUIC 连接，最少保持 1 个空闲 → 负载分散
- **least-connections 负载均衡**：选当前任务数最少的隧道连接（不是 round-robin）
- **BBR 拥塞控制可选**（通过 msquic 后端）：BBR 对高带宽高延迟链路远优于 Cubic
- **注意**：GT 也未设置 QUIC 窗口大小（用 quic-go 默认值），也未显式设置 TCP_NODELAY（依赖 Go 默认值，Go 1.13+ 默认启用）

### frp 的 QUIC 配置

```go
// QUICOptions 定义
type QUICOptions struct {
    KeepalivePeriod    int `json:"keepalivePeriod,omitempty"`      // default 10s
    MaxIdleTimeout     int `json:"maxIdleTimeout,omitempty"`       // default 30s
    MaxIncomingStreams  int `json:"maxIncomingStreams,omitempty"`   // default 100000
}

// frp 还有 TCP+yamux 双重选择：
TCPMux:                    true,   // 默认开启 yamux 多路复用
TCPMuxKeepaliveInterval:   30,     // yamux keepalive 30s
PoolCount:                 1,      // 预建 1 个连接
MaxPoolCount:              5,      // 服务端最大连接池 5
```

frp 的关键设计：
- QUIC `MaxIncomingStreams: 100000` — 远超 DuoTunnel 的 100
- yamux 在 TCP 之上提供多路复用，和 QUIC 是备选方案
- 连接池预建连接，减少首请求延迟

### DuoTunnel 的 QUIC 配置

```rust
// tunnel-lib/src/transport/quic.rs
transport_config.max_concurrent_bidi_streams(100_u32.into());  // ← 硬编码 100
transport_config.max_concurrent_uni_streams(100_u32.into());   // ← 硬编码 100
transport_config.keep_alive_interval(Some(Duration::from_secs(20)));
transport_config.max_idle_timeout(Some(Duration::from_secs(60).try_into().unwrap()));
// 注意：未设置以下参数
// - stream_receive_window (默认 ~48KB)
// - receive_window (默认 ~1.5MB)
// - send_window (默认 ~1.5MB)
// - initial_mtu / min_mtu
// - congestion_controller_factory (默认 Cubic)
```

**DuoTunnel 对比差距**:

| 参数 | DuoTunnel | GT | frp |
|------|-----------|----|----|
| **最大并发流** | 100 (硬编码) | 单 stream 自定义帧 | 100,000 |
| **连接数** | 1 (单连接) | 3 (连接池) | 可配置 (默认 1) |
| **stream 窗口** | ~48KB (默认) | 默认 (单 stream) | N/A |
| **连接窗口** | ~1.5MB (默认) | 默认 | N/A |
| **拥塞算法** | Cubic (不可配) | Cubic/BBR 可选 | 不可选 |
| **多路复用方式** | QUIC 原生 stream | 自定义帧协议 | yamux |
| **Idle 超时** | 60s | 90s | 30s |
| **参数可配置** | 全部硬编码 | YAML 配置 | JSON/TOML 配置 |

**优化建议**:
1. 调大流窗口和连接窗口（`stream_receive_window: 1MB`, `receive_window: 8MB`）
2. 提高 `max_concurrent_bidi_streams` 到至少 1000+
3. 所有传输参数应可配置（不应硬编码）
4. 考虑多 QUIC 连接池（类似 GT 的 RemoteConnections: 3）
5. 添加 BBR 拥塞控制选项

---

## 3. TCP Socket 优化

### GT 的做法

GT **没有显式设置 TCP_NODELAY 或 SO_KEEPALIVE**，依赖 Go 的默认行为：
- Go 1.13+ 的 `net.Dial` 默认启用 TCP_NODELAY
- Go 默认启用 TCP KeepAlive（间隔 15s）
- GT 使用 `reuseport.Listen`（SO_REUSEADDR/SO_REUSEPORT）支持多进程绑定同端口

### frp 的做法

```go
// frp 通过 golib 的 DialWithOptions 设置 keepalive
DialServerKeepAlive: 7200,  // 2小时 TCP keepalive

// frp Mux 层也设置 keepalive
func (mux *Mux) SetKeepAlive(keepAlive time.Duration) {
    mux.keepAlive = keepAlive
}
// Go 的 net 包默认 TCP_NODELAY = true
```

### DuoTunnel

**零处 socket 优化**。全项目搜索 `set_nodelay`、`set_keepalive`、`set_linger` — 无任何结果。

影响：
- **Nagle 算法延迟**: 小包（如 HTTP header、WebSocket frame）可被缓冲 40ms+
- **断线检测慢**: 没有 keepalive，TCP 连接在对端异常断开时可能永远挂起
- **TIME_WAIT 堆积**: 没有 linger 设置，高并发短连接场景可能 fd 耗尽

**优化建议**:
```rust
// 在所有 TcpStream::connect() 和 TcpListener::accept() 后：
stream.set_nodelay(true)?;
// 建议创建一个统一的 helper:
fn configure_tcp(stream: &TcpStream) -> io::Result<()> {
    stream.set_nodelay(true)?;
    // tokio TcpStream 不直接提供 keepalive API，
    // 可使用 socket2 crate 设置 SO_KEEPALIVE + 间隔
    Ok(())
}
```

---

## 4. TLS 配置与证书管理

### GT / frp 的做法

TLS 配置在启动时创建一次，通过指针/引用全局共享：

```go
// GT: TLS 配置初始化一次
tlsConfig := &tls.Config{
    InsecureSkipVerify: remoteCertInsecure,
    // ...
}
// 之后所有连接复用这个 config

// frp: 同样一次创建
// TLS config 是不可变的，所有 goroutine 共享同一个指针
```

### DuoTunnel 的问题

```rust
// tunnel-lib/src/proxy/tcp.rs:115-126 — 每次连接重建！
impl UpstreamPeer for TlsTcpPeer {
    async fn connect(&self, ...) -> Result<()> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()); // ← 克隆 ~150 个根证书
        let mut tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)  // ← 每次连接新建
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(tls_config)); // ← 每次连接 Arc::new
```

每个 TLS 连接开销：
- 克隆整个 WebPKI 根证书库（~150 证书 DER + parsed 结构）= **数十 KB 分配**
- 创建 rustls ClientConfig（包含加密套件表、验证器）
- 创建 TlsConnector

在 100 并发 TLS 连接下，这是 **数 MB 的重复分配**。

此外，`pki.rs` 的证书缓存命中时也克隆完整证书链+私钥：
```rust
// pki.rs:33 — 缓存命中仍然 clone
Some((cached.certs.clone(), cached.key.clone_key()))
```

**优化建议**:
```rust
// 方案1: TlsTcpPeer 构造时传入共享 connector
struct TlsTcpPeer {
    connector: TlsConnector,  // Arc<ClientConfig> 已经是引用计数
    target_addr: SocketAddr,
    tls_host: String,
}

// 方案2: 全局 lazy_static / OnceCell
static TLS_CONNECTOR: OnceCell<TlsConnector> = OnceCell::new();
```

---

## 5. 连接复用与池化

### GT 的连接池

```go
// GT config:
RemoteConnections:     3,   // 最多 3 个到 server 的连接
RemoteIdleConnections: 1,   // 至少保持 1 个空闲

// 连接池管理器在后台维护连接数量
// 空闲连接不足时自动补充
// 连接断开时自动重连
```

核心优势：**多 QUIC 连接消除单连接瓶颈**。QUIC 虽然多路复用，但单个 UDP socket 仍有 CPU 处理瓶颈。多连接分散负载。

### frp 的连接池

```go
// client config:
PoolCount: 1,        // 预建 1 个到 server 的连接
// server config:
MaxPoolCount: 5,     // 最多允许 5 个连接池

// frp 的 workConn 模型：
// 1. client 启动时向 server 预注册 PoolCount 个 work connection
// 2. 新的 proxy 请求到达时，server 从 pool 取 workConn
// 3. workConn 用完后不归还（一次性），client 补充新的
```

### DuoTunnel

**无连接池**。单个 QUIC 连接承载所有流量。每次新请求用 `conn.open_bi()` 创建 QUIC 流。

影响：
- 单个 QUIC 连接的 CPU 处理能力是瓶颈（所有流共享一个 UDP socket 的加解密路径）
- 没有预建连接，首请求延迟包含完整 QUIC 握手
- 客户端断线重连期间所有流量中断（没有备用连接）

**优化建议**:
1. 支持多 QUIC 连接（类似 GT 的 `RemoteConnections`）
2. 在连接池中做负载均衡（按流数量或字节数选择最闲的连接）
3. 预开若干 QUIC 流用于低延迟请求

---

## 6. 协议检测与路由

### frp 的协议检测

```go
// pkg/util/mux/mux.go — 精确的字节匹配
const (
    HTTPNeedBytesNum  uint32 = ...
    HTTPSNeedBytesNum uint32 = ...
)

// 用 SharedConn 避免数据消费
sharedConn, rd := gnet.NewSharedConnSize(conn, int(maxNeedBytesNum))
data := make([]byte, maxNeedBytesNum)
io.ReadFull(rd, data)  // 精确读取所需字节数

// 按优先级排序的匹配器
sort.Slice(lns, func(i, j int) bool {
    if lns[i].priority == lns[j].priority {
        return lns[i].needBytesNum < lns[j].needBytesNum
    }
    return lns[i].priority < lns[j].priority
})
```

frp 的优势：
- `SharedConn` 模式：peek 的数据不消费，后续 handler 可以完整读取
- 精确控制需要的字节数（不多读）
- 带优先级排序的 matcher chain

### GT 的协议检测

GT 在 Mux 层用 pooled reader peek 初始字节，使用 HTTP header 的 `Host` 进行路由。统一的 `pool.GetReader(conn)` 确保 buffer 复用。

### DuoTunnel 的协议检测

```rust
// server/handlers/http.rs:51-52 — 16KB 堆分配 peek buffer
let mut buf = vec![0u8; 16384];
let n = stream.peek(&mut buf).await?;

// server/handlers/tcp.rs:63-64 — 4KB 堆分配
let mut buf = vec![0u8; 4096];
let n = stream.peek(&mut buf).await?;

// protocol/detect.rs — 每次调用产生 String 分配
pub fn detect_protocol_and_host(data: &[u8]) -> (String, Option<String>) {
    // "h2".to_string(), "h1".to_string(), "websocket".to_string(), "tcp".to_string()
}

// listener.rs — VhostRouter.get() 每次 to_lowercase()
pub fn get(&self, host: &str) -> Option<T> {
    let host = host.split(':').next().unwrap_or(host).to_lowercase(); // ← 堆分配
}

// listener.rs — extract_host_from_http 每行 to_lowercase()
for line in data_str.lines() {
    if line.to_lowercase().starts_with("host:") { // ← 每行都堆分配
```

**DuoTunnel 的热路径分配清单**:

| 位置 | 每次调用分配 | 频率 |
|------|------------|------|
| `vec![0u8; 16384]` (http handler) | 16KB | 每个入站 HTTP 连接 |
| `vec![0u8; 4096]` (tcp handler) | 4KB | 每个入站 TCP 连接 |
| `"h2".to_string()` 等 | 2-10 bytes + heap overhead | 每个连接 |
| `host.to_lowercase()` | 整个 host 字符串 | 每个路由查找 |
| `line.to_lowercase()` | 每行 HTTP header | 每个 host 提取（N 行） |
| `Bytes::copy_from_slice` (core.rs:49) | 最多 4KB | 每个 QUIC 流 |

**优化建议**:
1. 协议返回值用 enum 替代 String（零分配）
2. `VhostRouter.get()` 用 `eq_ignore_ascii_case` 替代 `to_lowercase()`
3. `extract_host_from_http` 用 `starts_with` 的 case-insensitive 版本
4. peek buffer 用 `thread_local!` 或栈分配 `[u8; 16384]`

---

## 7. HTTP 代理实现

### frp 的 HTTP VHost

frp 在 `pkg/vhost/http.go` 中实现了完整的 HTTP/HTTPS vhost 代理：
- 基于 `net/http` 标准库的 `ReverseProxy`
- 自动处理 keep-alive、chunked encoding、WebSocket upgrade
- 超时配置 `VhostHTTPTimeout: 60s`
- 通过 `ResponseHeaderTimeout` 防止慢服务器

### DuoTunnel 的 HTTP 代理

**HTTP/1.1 不支持 keep-alive**（[http.rs:40-69](tunnel-lib/src/proxy/http.rs#L40-L69)）：
- `HttpPeer::connect` 处理一个请求就返回
- 每个 HTTP 请求 = 新开 QUIC 流 + 新 HTTP 连接

**HTTP egress 不支持 chunked**（[egress/http.rs:94-98](tunnel-lib/src/egress/http.rs#L94-L98)）：
- 只看 `Content-Length`，chunked body 被丢弃
- `parse_result.unwrap()` 对超大 header 会 panic

**H2 每请求做完整握手**（[h2_proxy.rs:35](tunnel-lib/src/proxy/h2_proxy.rs#L35)）：
- 每个 H2 请求：`open_bi() → send_routing_info → h2::handshake → send_request`
- 完整 H2 握手 = SETTINGS 帧 + WINDOW_UPDATE + ACK

**优化建议**:
1. HTTP/1.1: 在 loop 中处理多个请求，检测 `Connection: close`
2. H2: 复用 `h2::client::SendRequest` sender，多请求共享同一个 H2 会话
3. HTTP egress: 使用 `Method::from_bytes()` 支持所有方法，处理 chunked encoding

---

## 8. 配置可维护性

### GT/frp

两者都支持通过 YAML/JSON/TOML 配置所有性能参数：
- 传输层参数（超时、keepalive、buffer 大小）
- 连接池大小
- 日志级别
- 限速/限连接

### DuoTunnel

多个关键参数硬编码在源码中：

| 参数 | 硬编码位置 | 硬编码值 |
|------|-----------|---------|
| `max_concurrent_bidi_streams` | quic.rs:23 | 100 |
| `max_concurrent_uni_streams` | quic.rs:24 | 100 |
| `keep_alive_interval` | quic.rs:26 | 20s |
| `max_idle_timeout` | quic.rs:29 | 60s |
| HTTP peek buffer | http.rs:51 | 16384 |
| TCP peek buffer | tcp.rs:63 | 4096 |
| 初始读取 buffer | core.rs:47 | 4096 |
| 证书缓存 TTL | pki.rs:75 | 3600s |
| QUIC semaphore | server main | 不可配 |
| TCP semaphore | server main | 不可配 |

**优化建议**: 将所有性能参数移到配置文件中。

---

## 9. 架构设计对比：多路复用策略

这是三个项目最本质的差异，直接决定了性能特征。

| | DuoTunnel | GT | frp |
|---|-----------|---|----|
| **隧道传输** | QUIC | QUIC / TCP+TLS | TCP / TCP+TLS / KCP / QUIC / WebSocket |
| **多路复用** | QUIC 原生 stream (每代理连接一个流) | 单 QUIC stream + 自定义帧协议 (taskID) | yamux (TCP) / QUIC stream |
| **优势** | 原生流控、独立拥塞 | 极低的流创建开销、单 write 帧 | 多传输协议选择、连接池 |
| **劣势** | 流创建有 RTT 开销 | 无流级流控、自造轮子 | TCP HOL blocking (非 QUIC 时) |

**DuoTunnel 的架构优势**：

使用 QUIC 原生 stream 实际上是正确的设计选择（相比 GT 在单 stream 上自己做帧协议）：
1. **QUIC stream 有独立的流控**：慢速连接不会阻塞快速连接（GT 的单 stream 设计无法做到）
2. **无 HOL blocking**：不同 stream 的丢包不互相影响（GT 单 stream 上一个 task 的丢包影响所有 task）
3. **内核级拥塞控制**：每个 stream 受益于 QUIC 的拥塞控制，无需应用层管理

DuoTunnel 的问题不是架构选择错误，而是**实现层面的优化不足**（buffer 分配、socket 选项、窗口大小等）。

---

## 综合优化优先级

按预期性能收益排序：

| # | 优化项 | 类别 | 预期收益 | 难度 |
|---|--------|------|---------|------|
| 1 | **TCP_NODELAY 全局设置** | Socket | 延迟 -40ms（小包场景） | 低 |
| 2 | **TLS ClientConfig 全局共享** | 内存/CPU | CPU -30%（TLS 连接场景） | 低 |
| 3 | **QUIC 窗口调优** | 吞吐 | 大文件吞吐 +300%+ | 低 |
| 4 | **提高 max_concurrent_streams** | 并发 | 高并发不被限流 | 低 |
| 5 | **relay 统一 + 去掉 BufReader/Writer** | 吞吐/内存 | 减少拷贝和内存 | 中 |
| 6 | **`into_split()` 替代 `split()`** | 延迟 | 消除 relay 路径的 Mutex | 低 |
| 7 | **协议检测零分配** | 延迟/GC | 热路径零堆分配 | 中 |
| 8 | **VhostRouter 无分配查找** | 延迟 | 路由查找零堆分配 | 低 |
| 9 | **HTTP/1.1 Keep-Alive** | 吞吐 | 减少 80%+ 的流创建 | 中 |
| 10 | **H2 会话复用** | 延迟/吞吐 | H2 性能提升数倍 | 高 |
| 11 | **多 QUIC 连接池** | 吞吐/可靠性 | 消除单连接瓶颈 | 高 |
| 12 | **buffer pool (thread_local)** | 内存 | 高并发下减少分配 | 中 |
| 13 | **传输参数可配置** | 运维 | 生产环境调优能力 | 中 |
| 14 | **BBR 拥塞控制可选** | 吞吐 | 高延迟链路性能提升 | 高 |

### 快速赢面（1-4 项，共约 2 小时）

仅修改 1-4 项即可获得最大的性能提升：
- TCP_NODELAY: 5 行代码
- TLS 缓存: 重构 `TlsTcpPeer` 构造函数
- QUIC 窗口: 3 行配置
- 提高 stream 限制: 1 行改动

这四项改动加起来预期可以让 DuoTunnel 在大多数场景下达到 GT/frp **80%+** 的性能水平。


## 10. 流量翻100倍的极限性能优化方案

针对当前基于 `tokio` + `quinn` (QUIC) 开发的 Proxy Tunnel 架构，如果流量在未来翻100倍，我们将面临**连接数激增、内存分配压力大、CPU在协议解析与上下文切换上的瓶颈**等问题。以下是针对极限性能和延迟优化的核心抓手：

### 10.1. 内存与零拷贝优化 (Zero-copy & Buffer Pooling)
在目前的 `ProxyEngine::run_stream` 中：
```rust
let mut buf = vec![0u8; 4096];
let n = recv.read(&mut buf).await?.unwrap_or(0);
let initial_bytes = Bytes::copy_from_slice(&buf[..n]);
```
**问题：**高并发下，每个 Stream 动态分配 `Vec` 并随后执行 `copy_from_slice` 会导致极其严重的堆分配开销和锁竞争。
**优化：**
- 引入**内存池机制**。可以使用 `BytesMut::with_capacity` 结合全局/线程局部的内存池，或者使用 `tokio-util` 中的缓冲分发器。读取后直接使用 `buf.split_to(n).freeze()` 将所有权转移，实现**全程零拷贝 (Zero-copy)**。
- **使用全局优质分配器**：在 `Cargo.toml` 中引入 `jemalloc` 或 `mimalloc`（并在 `main.rs` 中替换默认 `GlobalAlloc`），这对 Rust 异步高压并发下的内存分配性能提升极大。

### 10.2. QUIC (Quinn) 传输层与网络栈调优
100倍流量意味着底层的 UDP 处理必须极尽高效：
- **开启 UDP GSO/GRO (Generic Segment/Receive Offload)**：尽量让网卡和内核帮我们做包的聚合和分发，极大减少系统调用的 CPU 开销。
- **SO_REUSEPORT 多路复用**：`quinn` 默认只绑定单 UDP socket。可以在 Linux 上使用 `SO_REUSEPORT`，绑定与 CPU 核心数相同的 UDP socket 实例，将不同的 socket 交由不同 Tokio worker 线程专门收发，彻底打散底层单核性能瓶颈。

### 10.3. 上游连接池复用 (Connection Pooling)
目前的 `tunnel_handler.rs` 每次接收到一个 quinn stream 就会去匹配路由和生成 Upstream 代理。
**优化：**向目标地址转发时，如果每次都需要重新建立 TCP 甚至 TLS 握手，延迟将极其严重。引入**全局 Upstream 连接池**。对于 HTTP/1.1 使用长连接池，对于 HTTP/2 直接进行 Stream 复用。充分发挥已引入依赖中 `hyper-util` 和 `hyper` 客户端自带的自带复用池机制，共享同一个 Client 实例而不要单次流孤立创建。

### 10.4. 协议探测 (Protocol Detect) 的计算优化
**优化：**对于 `is_websocket_upgrade` 和 `httparse::Request::new`，避免不必要的 Header 遍历。既然大多数长连接是透传的（TCP/TLS ClientHello），可以将判断逻辑建立一颗“快速判断树”或“Trie树”，对前几个 Byte 做 Magic Number 的模式匹配，只有部分明确需要提取 Host 的场合，才进行完整解析。

### 10.5. 降低日志与监控带来的旁路开销
**优化：**如果在高流量下按照每个 Stream 打印一条 `info` 级别日志，磁盘 I/O 和 Tracing 消费线程的锁会直接把系统拖宕机，造成极高的业务延迟。
- 将连接粒度的日志降级为 `debug!` 或 `trace!`。
- 使用 `tracing_appender::non_blocking` 进行无锁异步日志写入。
- 改用基于指标的聚合（例如 `prometheus` Counter/Histogram），而非记录流级别的详情。

### 10.6. 编译级优化 (LTO)
确保 `[profile.release]` 中设置了最佳构建参数，以换取运行时的零成本抽象最大化：
```toml
[profile.release]
lto = "fat"            # 跨 Crate 全局优化
codegen-units = 1      # 用更长的编译时间换取最高性能
panic = "abort"        # 高并发下少生成 unwind 表，节省指令缓存占用
opt-level = 3
```
