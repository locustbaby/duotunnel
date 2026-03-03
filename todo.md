# Tunnel TODO

## [perf] HttpPeer HTTP/1.1 keep-alive loop

**Branch**: `perf/cpu-hotpath-3kqps`

**文件**: `tunnel-lib/src/proxy/http.rs`

### 问题

`HttpPeer::connect_inner()` 现在每个 QUIC stream 只处理一个 HTTP 请求然后调
`driver.finish()` 关闭，导致：

- client → upstream 每次都新建 TCP 连接
- TIME_WAIT 在 3K QPS 下堆积到 6000+
- 每秒大量 TCP 建连/关闭 syscall，内核态 CPU 占用高

hyper 的 `Client` 本身有连接池（`pool_max_idle_per_host=10`），但因为连接用完
就关了，池子形同虚设。

### 目标

同一个 QUIC stream 上循环处理多个 HTTP/1.1 请求，复用同一条 upstream TCP 连接：

```
现在：QUIC stream → 1个请求 → finish() → TIME_WAIT
优化：QUIC stream → 请求1 → 请求2 → 请求3 → ... → QUIC stream 关闭时才关 upstream
```

### 实现思路

`connect_inner` 里把单次 `read_request` 改为 loop：
- 读请求 → 转发 → 写响应 → 继续读下一个请求
- 直到 QUIC stream 关闭（`read_request` 返回 `Ok(None)`）才退出循环
- 需要处理 `Connection: close` header（客户端主动要求关闭时退出循环）
- `Http1Driver` 需要支持在同一个 send/recv 上多次 read/write

### 预期收益（基于 bb15326 数据）

- TIME_WAIT: 6000+ → 接近 0
- system CPU @3K QPS: ~100% → 预期降 20-30%
- context switch: 减少大量 involuntary switch
