# 性能瓶颈分析：Client Entry 与 Server Egress

## 1. Client Entry 缺少 SO_REUSEPORT 与多 Loop

### 问题位置

`client/entry.rs:28`

### 分析

Client 的 Entry 端口（8082）在实现上比 Server 的 Ingress 端口（8080）简陋：

**单 Loop**：`start_entry_listener` 内部只有一个 `loop { listener.accept().await }`。在高并发（8000 RPS）下，单个 Accept 任务在处理连接握手和分配 Token 时会成为瓶颈。

**缺少 SO_REUSEPORT**：直接使用了 `TcpListener::bind`。对比 Server 端使用了 `tunnel_lib::build_reuseport_listener`（`tunnel-lib/src/transport/listener.rs:12`），后者设置了：
- `socket.set_reuse_address(true)`
- `socket.set_reuse_port(true)`（Unix）
- `socket.listen(4096)`（大 backlog 队列）

**影响**：Client Entry 用 Tokio 默认 bind，backlog 默认值远小于 4096。在突发流量下 SYN 包会被内核直接丢弃（而不是排队等待），同时由于无 `SO_REUSEPORT`，无法利用多核并行受理连接，p95 响应时间显著变差。

---

## 2. Ingress 与 Egress 路径的不对称：Http1Driver 性能问题

### 路径 A：Ingress（k6 → Server:8080 → Client → Backend）

- **Server 入口端**（`server/handlers/http.rs:386`）：plaintext H1/WS 走 `proxy::forward_with_initial_data`，这是**纯字节转发**（copy_buf），不解析应用层协议。
- **Client 出口端**：对接本地 Backend 时使用 `HttpPeer`，在此处使用 `Http1Driver` 做 L7 解析。

### 路径 B：Egress（k6 → Client:8082 → Server → Backend）

- **Client 入口端**（`client/entry.rs:105`）：使用 `relay_quic_to_tcp`，同样是字节转发。
- **Server 出口端**（`server/egress.rs:110-116`）：对 `Protocol::H1 | H2 | Unknown` 均强制走 `PeerKind::Http(HttpPeer)`，触发 `Http1Driver`。

> 例外：WebSocket 在 `server/egress.rs:85-108` 有专门分支，走 `TcpPeer` 字节转发。

### Http1Driver 慢的根本原因

**强制无条件 Chunked 编码**（`tunnel-lib/src/protocol/driver/h1.rs:290`）：

```rust
header_buf.put_slice(b"transfer-encoding: chunked\r\n\r\n");
```

这行是无条件写入的——无论后端原始响应是 `Content-Length`、`Transfer-Encoding: chunked` 还是其他，都会强行覆盖为 chunked。导致：
1. 所有响应必须被 frame 化再重新 chunk 编码，每个 chunk 有额外的 hex 长度行 + CRLF 开销。
2. 若后端已返回 `Content-Length`，客户端同时看到两种长度语义存在协议合规风险。

**解析→重构→重新编码开销**：`Http1Driver` 将原始 HTTP 请求解析为结构体，再重新编码输出，相较于 Ingress 端的"直接转发原始字节"模式，CPU 开销显著更高。

**Keep-alive idle 占用**（`tunnel-lib/src/proxy/http.rs:14`）：`HttpPeer` 维护了 60 秒的 idle timeout 循环（`KEEPALIVE_IDLE_TIMEOUT`），Egress 路径的连接无法立即释放，持续占用资源。

---

## 总结

| 论点 | 准确性 | 备注 |
|---|---|---|
| Client Entry 无 SO_REUSEPORT | ✅ 正确 | 也缺少大 backlog（默认远小于 4096）|
| Client Entry 单 Loop | ✅ 正确 | Tokio 单任务 accept |
| Server Ingress 字节转发 | ✅ 正确 | `forward_with_initial_data` |
| Server Egress 强制 HttpPeer/H1Driver | ✅ 正确 | H1\|H2\|Unknown 全走 L7 解析 |
| H1Driver 无条件写 chunked | ✅ 正确 | `h1.rs:290`，无条件覆盖原始 TE |
| WebSocket 走 TcpPeer 字节转发 | ✅ 正确 | egress.rs 有专门分支 |
| Egress Keep-alive idle 占用 | ✅ 补充发现 | 60s idle timeout，连接不立即释放 |
