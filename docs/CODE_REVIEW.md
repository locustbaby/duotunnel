# DuoTunnel 代码审查报告：性能 & 正确性分析

对比项目：GT (Go+QUIC)、frp (Go+TCP+Yamux)

---

## CRITICAL (必须修复)

### 1. H2 流控缺失 — 可导致 OOM / DoS
**文件**: `tunnel-lib/src/proxy/h2.rs:142-164`

`reserve_capacity()` 被调用但从未 await `poll_capacity()`。慢速客户端会导致 h2 库无限缓冲数据，最终 OOM。

```rust
// 当前代码 — 只是 hint，不会阻塞等待
send_stream.reserve_capacity(len);
send_stream.send_data(data, false)?; // 如果窗口不够，数据会无限缓冲
```

**修复**: 在 send_data 前 await flow control capacity，或限制缓冲大小。

> GT/frp: Go 的 `x/net/http2` 在 Write 路径内部自动阻塞等待流控窗口。

---

### 2. Round-Robin 负载均衡是坏的
**文件**: `server/registry.rs:56`

```rust
self.clients.iter().nth(idx)
```

`DashMap::iter()` **不保证稳定的迭代顺序**（依赖 hash bucket 布局）。问题：
- 两次连续调用可能返回相同的客户端（不是真正的 round-robin）
- `iter().nth(idx)` 是 O(n)，对多客户端组退化为 O(n²)
- 并发修改时可能跳过或重复访问客户端

**修复**: 用 `Vec<(String, Connection)>` + `RwLock` 替代 DashMap，保证稳定的索引顺序。

> GT: `sync.RWMutex` 保护的稳定 Slice + atomic index。frp: `sync.Mutex` + list。

---

### 3. 重复客户端注册竞态条件
**文件**: `server/handlers/quic.rs:98-106`

```rust
if let Some(existing_conn) = state.registry.get_client_connection(&login.client_id) {
    existing_conn.close(0u32.into(), b"duplicate client");
    state.registry.unregister(&login.client_id);
}
// ← 此处其他线程可能注册相同 client_id
state.registry.register(login.client_id.clone(), group_id.clone(), conn.clone());
```

`get_client_connection` 和 `unregister` 不是原子操作。两个相同 client_id 的并发连接可能同时通过检查。

**修复**: 使用 DashMap 的 `entry()` API 做原子的 check-and-replace。

---

### 4. 每个 H2 请求做完整握手
**文件**: `tunnel-lib/src/proxy/h2_proxy.rs:35`

```rust
let (mut sender, conn) = handshake(executor, io).await?; // 每次调用都做 H2 握手
```

每个 HTTP/2 请求都：新开 QUIC 双向流 → H2 握手（SETTINGS 交换 + 窗口更新）→ 发送请求。

**修复**: 复用 H2 会话，多个请求共享同一个 H2 connection。

> frp: 预建立 yamux 多路复用会话。

---

## HIGH (显著影响性能)

### 5. 全局缺失 TCP_NODELAY
**整个项目零处调用 `set_nodelay(true)`**

Nagle 算法对小包引入高达 **40ms** 延迟。这可能是当前最大的单一延迟来源。

```rust
// 修复：在所有 TcpStream::connect() 和 accept() 后加一行
tcp_stream.set_nodelay(true)?;
```

> GT/frp: 所有 TCP socket 无条件设置 `TCP_NODELAY(true)`。

---

### 6. TLS 配置每次连接重建
**文件**: `tunnel-lib/src/proxy/tcp.rs:115-126`

每个 `TlsTcpPeer::connect()` 都会：
1. 克隆整个 WebPKI 根证书库（~150 个证书，数百次堆分配）
2. 新建 `rustls::ClientConfig`
3. 新建 `TlsConnector`

```rust
// 修复：在 TlsTcpPeer 构建时创建一次
struct TlsTcpPeer {
    connector: TlsConnector, // Arc<ClientConfig> 内部已经是引用计数
    // ...
}
```

> GT/frp: TLS 配置全局共享一个 `Arc`。

---

### 7. QUIC 窗口大小未调优
**文件**: `tunnel-lib/src/transport/quic.rs:22-31`

未设置 `stream_receive_window`、`receive_window`、`send_window`。Quinn 默认值：
- 连接级窗口 ~1.5MB → 100 并发流平均每流仅 **15KB** 流控窗口
- 对大文件传输严重瓶颈

```rust
// 修复
transport_config.stream_receive_window(1024 * 1024u64.into());      // 1MB per stream
transport_config.receive_window(8 * 1024 * 1024u64.into());          // 8MB per connection
transport_config.send_window(8 * 1024 * 1024);                       // 8MB send
```

> GT: 配置了更大的 QUIC 窗口。

---

### 8. HTTP/1.1 不支持 Keep-Alive
**文件**: `tunnel-lib/src/proxy/http.rs:40-69`

`HttpPeer::connect` 只处理一个请求就返回。HTTP/1.1 keep-alive 的后续请求被静默丢弃。每个 HTTP 请求都需要新开 QUIC 流。

**修复**: 在 connect 中循环处理多个请求，直到连接关闭或 `Connection: close`。

---

### 9. 每次重连创建新 QUIC Endpoint
**文件**: `client/main.rs:92`

重连循环每次创建新的 `quinn::Endpoint`（绑定新 UDP socket）。快速重连下可能泄漏 fd。

**修复**: 在重连循环外创建一次 Endpoint，循环内只做 `endpoint.connect()`。

---

### 10. ServerConfig 每个 egress 流深拷贝
**文件**: `server/handlers/quic.rs:137`

```rust
let config = Arc::new(state.config.clone()); // 每个流都深拷贝 HashMap、Vec<String> 等
```

Config 在启动后不可变，应该只 `Arc` 包装一次。

---

### 11. TLS handler 客户端选择绑定到连接
**文件**: `server/handlers/http.rs:88-92`

TLS 连接上所有 H2 请求都路由到同一个客户端。如果该客户端中途断开，整个连接的后续请求全部失败。对比 plaintext H2 handler 正确地做了 per-request 路由。

---

## MEDIUM

### 12. `tokio::io::split` 带 Mutex 开销
**文件**: `engine/relay.rs:14`, `engine/bridge.rs:9-10`

泛型 `tokio::io::split` 用 `Arc<Mutex>` 包装，每次 read/write 都经过锁。`TcpStream` 本身有零开销的 `into_split()`。

---

### 13. 三套重复的 relay 实现

| 文件 | 函数 | 差异 |
|------|------|------|
| `engine/relay.rs` | `relay_bidirectional`, `relay_with_initial` | 无缓冲 |
| `engine/bridge.rs` | `relay`, `relay_quic_to_tcp`, `relay_with_first_data` | 无缓冲，用 `into_split` |
| `proxy/base.rs` | `forward_to_client`, `forward_with_initial_data` | 多了 BufReader/BufWriter |

`base.rs` 在 QUIC 流上多加 BufReader/BufWriter 是**反效果**（QUIC 传输层已有缓冲，多一层 8KB buffer 只增加内存和拷贝，不提升吞吐）。

---

### 14. Relay 错误被静默吞掉
**文件**: `relay.rs:32`, `bridge.rs:28`

```rust
Ok((r1.unwrap_or(0), r2.unwrap_or(0)))
```

无法区分"正常传输 0 字节"和"连接被 reset"。

---

### 15. 初始字节读取仅 4KB
**文件**: `tunnel-lib/src/proxy/core.rs:47-49`

单次 `recv.read()` 可能不足以捕获完整 HTTP 头。大 Cookie/Auth 头超过 4KB 会导致协议检测或 WebSocket 升级检测失败。同时存在双重分配（Vec + Bytes::copy_from_slice）。

---

### 16. 热路径中 String 分配
**文件**: `tunnel-lib/src/protocol/detect.rs:5-42`

每次调用 `detect_protocol_and_host` 都会 `"h2".to_string()`、`"h1".to_string()`。`VhostRouter.get()` 每次做 `to_lowercase()` + `split(':')`。

**修复**: 使用枚举替代字符串，或返回 `&'static str`。

---

### 17. Client entry listener 无背压
**文件**: `client/entry.rs:30-35`

不像 server 端有 semaphore 限流，client entry listener 允许无限连接。

---

### 18. 模糊的回退路由
**文件**: `client/app.rs:106-114`

`name.contains(proxy_name)` 子串匹配 + `HashMap::values().next()` 顺序不确定。

---

### 19. DNS 无缓存
**文件**: `client/app.rs:170-171`

每个 WebSocket/TCP 上游连接触发 DNS 查询。

---

### 20. 死代码 connection_pool.rs
**文件**: `client/connection_pool.rs` 存在但未被导入使用。

---

## 额外发现：安全 & 协议合规问题

### 21. HTTP egress: `parse_result.unwrap()` 会 panic
**文件**: `tunnel-lib/src/egress/http.rs:91`

当 HTTP 请求超过 8192 字节时，httparse 返回 `Status::Partial`，`.unwrap()` 直接 panic 导致进程崩溃。

---

### 22. HTTP egress: Chunked Transfer-Encoding 完全被忽略
**文件**: `tunnel-lib/src/egress/http.rs:94-98`

只处理 `Content-Length` body。chunked 编码的请求 body 被静默丢弃（content_length 默认为 0）。

---

### 23. HTTP egress: 未知 HTTP method 静默变为 GET
**文件**: `tunnel-lib/src/egress/http.rs:88`

`PROPFIND`、`MKCOL` 等 WebDAV 方法或自定义方法被当作 GET。应该用 `Method::from_bytes()`。

---

### 24. HTTP egress: Rewriter host_rewrite 完全无效
**文件**: `tunnel-lib/src/egress/http.rs:69-75`

Rewriter 设置的 Host header 被后面的代码无条件覆盖。`with_host()` 功能实际上是坏的。

---

### 25. bincode 反序列化不安全输入
**文件**: `tunnel-lib/src/models/msg.rs:120`

bincode 不是为不可信输入设计的。结构体内的 `String` 字段（如 `client_id`）用 `len: u64 + data` 编码，攻击者可伪造超大 len 导致巨额分配。当前未配置 `with_limit()`。

---

### 26. 地址解析: `:443` 子串匹配误判 HTTPS
**文件**: `tunnel-lib/src/transport/addr.rs`

```rust
let is_https = addr.starts_with("https://") || addr.contains(":443");
```

`example.com:4430` 或 `my443host:80` 都会被误判为 HTTPS。

---

### 27. 地址解析: IPv6 地址解析错误
**文件**: `tunnel-lib/src/transport/addr.rs`

`http://[::1]:8080` 经过 `split(':')` 后 host 变成 `"["`，port 解析失败。

---

### 28. Token 验证非常量时间
**文件**: `server/config.rs:148`

`expected == token` 会在第一个不同字节处短路，存在时序侧信道。应用 `subtle::ConstantTimeEq`。

---

### 29. 证书缓存无限增长
**文件**: `tunnel-lib/src/infra/pki.rs`

过期条目永不驱逐。千个不同主机名的证书永驻内存。且缓存命中时完整克隆证书链+私钥，应用 `Arc`。

---

### 30. HTTP 响应 header 含非 ASCII 值时中断
**文件**: `tunnel-lib/src/egress/http.rs:162-165`

`value.to_str()?` 对非可见 ASCII 字符返回 Err，导致整个响应被中断。应用 `value.as_bytes()`。

---

## 与 GT / frp 实现对比总结

| 维度 | DuoTunnel | GT (Go+QUIC) | frp (Go+TCP) |
|------|-----------|-------------|-------------|
| **Buffer 回收** | 每连接新分配 `vec![]` | `sync.Pool` 复用 | `sync.Pool` 复用 |
| **TCP_NODELAY** | ❌ 未设置 | ⚠️ 依赖 Go 默认值 | ⚠️ 依赖 Go 默认值 |
| **TLS 配置** | ❌ 每连接重建 | ✅ 全局共享 | ✅ 全局共享 |
| **Round-Robin** | ❌ 坏的 (DashMap) | ✅ 稳定 Slice+Index | ✅ 稳定 List+Mutex |
| **H2 流控** | ❌ 缺失(OOM) | ✅ 自动处理 | ✅ yamux 自动 |
| **连接复用** | ❌ 每请求新握手 | ✅ 复用会话 | ✅ 预建立连接池 |
| **HTTP Keep-Alive** | ❌ 不支持 | ✅ 支持 | ✅ 支持 |
| **QUIC 窗口** | ❌ 默认值(太小) | ⚠️ 也是默认值 | N/A |
| **DNS 缓存** | ❌ 无 | ✅ 有 | OS 缓存 |
| **优雅关闭** | ❌ 无 | ✅ context.Cancel | ✅ Close() |
| **IO 缓冲** | ❌ 不一致 | ✅ 统一 bufio | ✅ 统一 buffered |
| **连接超时** | ❌ 无 | ✅ 10s dial | ✅ 可配置 |

---

## 优先修复顺序（按 ROI 排序）

| 优先级 | 修复项 | 难度 | 预期收益 |
|--------|--------|------|----------|
| P0 | 设置 TCP_NODELAY | 1h | 延迟降低 40ms+ |
| P0 | 修复 round-robin (Vec+RwLock) | 2h | 正确性 + O(1) |
| P0 | 缓存 TLS ClientConfig | 1h | CPU 大幅降低 |
| P1 | 调大 QUIC 窗口 | 30min | 吞吐提升数倍 |
| P1 | 修复 H2 流控 | 3h | 消除 OOM 风险 |
| P1 | 统一 relay 实现 | 2h | 去掉多余缓冲/锁 |
| P2 | HTTP/1.1 Keep-Alive | 4h | 显著降低流创建开销 |
| P2 | H2 会话复用 | 4h | H2 性能大幅提升 |
| P2 | 修复重复客户端竞态 | 1h | 正确性 |
| P3 | Endpoint 复用 | 30min | 避免 fd 泄漏 |
| P3 | 协议枚举替代字符串 | 1h | 热路径零分配 |
| P3 | Entry listener 背压 | 30min | 防资源耗尽 |
