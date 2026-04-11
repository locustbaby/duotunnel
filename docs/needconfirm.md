# 待深入排查

## Egress 比 Ingress 慢的疑似原因

### Benchmark 数据

| 场景 | avg | p95 | rps |
|---|---|---|---|
| ingress 8K | 13.31ms | 48.29ms | 6763 |
| egress 8K | 28.41ms | 72.5ms | 5969 |
| ingress multihost 8K | 16.91ms | 50.53ms | 6658 |
| egress multihost 8K | 28.48ms | 70.53ms | 5942 |

p95 差距约 **20-22ms**，avg 差距约 **15ms**。

---

### CI 拓扑确认（local-test-server.yaml / bench.js）

```
echo-server (127.0.0.1:9999)
       ↑                    ↑
  [Ingress]            [Egress]
  k6 → :8081         k6 → :8082
  (server 监听)      (client entry 监听)
       ↓  QUIC :10086  ↓
     server ←→ client
```

**两个 bench case 的 upstream 都是同一个本地 `127.0.0.1:9999`，网络延迟完全对等。**

Ingress 路径：
```
k6 → server :8081 → server open_bi() → QUIC → client accept_bi
→ client Http1Driver 解析 → 127.0.0.1:9999
← Http1Driver write_response chunked → QUIC → server → k6
```

Egress 路径：
```
k6 → client entry :8082 → peek 识别协议 → client open_bi()
→ QUIC → server accept_bi → server Http1Driver 解析 → 127.0.0.1:9999
← Http1Driver write_response chunked → QUIC → client → k6
```

---

### 疑似根因：Egress 多了一次完整 HTTP parse

两条路径都经过 `Http1Driver`（chunked write_response 相同），但：

| | Ingress | Egress |
|---|---|---|
| HTTP parse 次数 | 1次（client Http1Driver） | **2次**（client entry peek + server Http1Driver 完整 parse） |
| open_bi 发起方 | server（主动 push） | client（主动 push） |
| upstream 选择开销 | select_client_for_group + inflight | UpstreamGroup round-robin |

egress 在 client entry 侧 `peek` 读了前 N 字节识别协议，然后把**整个原始 TCP stream 通过 `relay_quic_to_tcp` 透传**到 server，server 的 `Http1Driver::read_request` 再从 QUIC stream 里**重新读取并完整 parse 一遍 HTTP header**。

相比 ingress，egress 多了：
1. client entry peek（一次系统调用）
2. QUIC stream 上传输 HTTP 请求头后，server 端需要从流中读满整个 header 才能发出请求（额外的 buffering 延迟，依赖 QUIC 单向传输完成）
3. 实际上等于 HTTP header 在网络上多走了一个 QUIC 单程（ingress 是 server 收到请求后立刻 open_bi，client 直接发给 upstream；egress 是 client 发给 server，server 再发给 upstream，header 在 QUIC tunnel 里多一次完整传输才能 unblock upstream 请求）

---

### 需要确认的点

1. **是否确实是这个 extra QUIC round 导致的**：ingress 的 `open_bi` 由 server 发起，client 收到后马上就能建连 upstream；egress 的 `open_bi` 由 client 发起，但 server 要等读完 HTTP header 才能发出 upstream 请求——这中间的等待是否与 ~15ms avg 差距匹配？

2. **client entry 的 peek 用的是 `TcpStream::peek`（不消费数据），后续 `relay_quic_to_tcp` 会重发这段数据**——server 端 Http1Driver 拿到的就是完整原始请求，没有数据丢失，只是多了一次传输。

3. **chunked write_response 是否也有影响**：两条路径都走 chunked，理论上影响对等，但需确认 ingress 的 bench 确实走的是 `Http1Driver` 而非 H2 路径（`forward_h2_request`）。bench.js 里 `ingressHttpGet` 访问 `http://echo.local:8080`（明文 H1），应该走 `handle_plaintext_h1_connection` → H1 relay，不是 Http1Driver，而是 `forward_with_initial_data`（`copy_buf` 直接透传）。

4. **Ingress H1 实际走的是 `proxy/base.rs::forward_with_initial_data`（copy_buf），而不是 Http1Driver**——如果确认，那 ingress 根本没有 HTTP 层解析，是纯字节透传，而 egress 有两次 parse，差距来源就更清晰了。

---

### 建议排查步骤

1. **确认 ingress bench 走的代码路径**：在 `handle_plaintext_h1_connection` 和 `Http1Driver::read_request` 各加一个计数器或 tracing span，跑 bench 看哪个被触发
2. **确认 egress server 端走的路径**：在 `Http1Driver::read_request` 加 tracing，确认 egress 请求确实经过两次 parse
3. 如果第 4 点确认（ingress 是 copy_buf，egress 是两次 parse），根因就是 **egress 多了 HTTP 语义层**——优化方向是在 client entry 侧直接解析并转发，而非原始字节透传
