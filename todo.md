# Tunnel TODO

---

## [perf] 3K QPS CPU 打满问题排查（bb15326）

**基准数据**：commit `bb15326`，GitHub Actions runner 2核

### 现象

| 指标 | 1K QPS | 2K QPS | 3K QPS |
|------|--------|--------|--------|
| system CPU | ~50% | ~95% | 100% |
| server CPU（单核归一化） | ~62% | ~62% | - |
| TIME_WAIT | 线性上升，峰值 6000+ | | |
| disk write (server) | 随 QPS 线性涨 | | |
| major page fault | 启动时 4086，运行期持续 | | |
| involuntary context switch | 峰值 1300+/s | | |

### 根本原因分析

**1. 每个请求新建 TCP 连接（最主要）**

两段都没有连接复用：

```
k6 ──(新TCP, noConnectionReuse=true)──► server
                                           │QUIC
                                        client ──(新TCP, HttpPeer 无 keep-alive)──► http-echo
```

- k6 配置了 `noConnectionReuse: true`（`ci-helpers/k6/bench.js:267`），每请求强制新建连接
- `HttpPeer::connect_inner()` 处理完一个请求就调 `driver.finish()` 关闭，
  hyper 的连接池（`pool_max_idle_per_host=10`）形同虚设
- 3K QPS = 每秒 6000 次 TCP 建连+关闭，全部进入 TIME_WAIT（2MSL ≈ 60s）
- TIME_WAIT 堆积到 6000+ 意味着内核同时维护 6000 条状态记录

**2. 大量短命 task 调度开销**

- 每个 QUIC stream spawn 一个 tokio task，3K QPS = 每秒 3000 个 task 创建销毁
- 2个 worker thread 高频调度这些短命 task → involuntary context switch 高
- H2 path 里每个 upstream 连接额外 `tokio::spawn` 管理 task，task 数翻倍

**3. 热路径上的 `info!` 日志**

- `handle_work_stream` 入口 + `HttpPeer::connect_inner` 各一条 `info!`
- 3K QPS = 每秒 6000 次 tracing subscriber 锁争用
- 多线程抢锁 → 线程挂起等待 → involuntary context switch
- 每条日志最终 write syscall → disk IO 随 QPS 线性增长

**4. 每请求高频内存分配 → page fault**

每个请求在 `Http1Driver::read_request()` 里：
- `vec![0u8; 8192]` — first_buf
- `BytesMut::with_capacity(8192)` — body stream 里每次循环
- `header_buf`、`chunk_buf` 各一次

3K QPS = 每秒 >15000 次 heap 分配，新分配的页首次写触发 minor page fault，
内核处理缺页也消耗 CPU（内核态时间）。

**5. 为什么 system CPU 打满**

```
system CPU = 用户态（实际转发逻辑）+ 内核态

内核态 CPU 来源：
  6000次/s TCP 建连关闭  → 内核 TCP 状态机
  15000次/s page fault   → 内核缺页处理
  6000次/s 日志 write    → syscall
  大量 context switch    → 调度器开销

= 2核 CPU 大量跑在内核态，实际转发逻辑反而没时间跑
```

### 优化方向

见下方各 TODO 条目。

---

## [perf/TODO-0] tcp_tw_reuse 缓解 TIME_WAIT（治标）

**优先级**: 低，治标不治本，配合 TODO-1 一起用

`net.ipv4.tcp_tw_reuse = 1` 允许内核在新出站连接时复用处于 TIME_WAIT 状态的
socket，需要双方都开启 TCP timestamp（默认开启）。

### 局限性

- 只对**出站**连接有效（client → upstream 这段），server 侧无效
- 不减少 TIME_WAIT 数量，只是让端口可以被复用，防止端口耗尽
- TIME_WAIT 条目本身还在，内核维护开销不变
- 根本解法是 TODO-1（keep-alive），连接不关就没有 TIME_WAIT

### CI 里加的方式

在 `ci.yml` Start backend servers 之前加：

```yaml
- name: Tune kernel for benchmark
  run: |
    sudo sysctl -w net.ipv4.tcp_tw_reuse=1
    sudo sysctl -w net.ipv4.ip_local_port_range="1024 65535"
```

---

## [perf/TODO-1] HttpPeer HTTP/1.1 keep-alive loop

**文件**: `tunnel-lib/src/proxy/http.rs`
**优先级**: 高，预期收益最大

### 问题

`HttpPeer::connect_inner()` 每个 QUIC stream 只处理一个请求然后关闭（代码里有 TODO 注释）。

### 目标

同一个 QUIC stream 上循环处理多个 HTTP/1.1 请求，复用同一条 upstream TCP 连接：

```
现在：QUIC stream → 请求1 → finish() → TIME_WAIT → 新QUIC stream → 请求2 ...
优化：QUIC stream → 请求1 → 请求2 → 请求3 → ... → stream关闭时才释放upstream连接
```

### 实现思路

`connect_inner` 里把单次 `read_request` 改为 loop：
- 读请求 → 用 hyper client 转发 → 写响应 → 继续读下一个请求
- `read_request` 返回 `Ok(None)` 时退出（QUIC stream 关闭）
- 检查响应/请求的 `Connection: close` header，有则退出循环
- `Http1Driver` 需要支持在同一个 send/recv 上多次调用 read/write（现在 recv 被 `take()` 消耗了）

### 预期收益

- TIME_WAIT: 6000+ → 接近 0（client 侧）
- system CPU @3K QPS: 预期降 20-30%
- involuntary context switch 明显减少

---

## [perf/TODO-2] 热路径日志降级

**文件**: `client/proxy.rs`, `tunnel-lib/src/proxy/http.rs`
**优先级**: 中，改动极小

将热路径上的 `info!` 改为 `debug!`：

```rust
// client/proxy.rs handle_work_stream 入口的 info!
// tunnel-lib/src/proxy/http.rs connect_inner 里的 info!
```

### 预期收益

- 生产/bench 下默认 WARN level，这两处完全静默
- 消除日志锁争用，减少 involuntary context switch
- disk write 随 QPS 线性增长的问题消失

---

## [perf/TODO-3] 调整 tracing 位置和级别

**文件**: `client/proxy.rs`, `tunnel-lib/src/proxy/http.rs`, 以及其他热路径文件
**优先级**: 中

### 问题

热路径上存在不合适的日志级别，bench/生产环境下产生无效开销：
- 每请求必经路径上的 `info!` 在高 QPS 下变成性能瓶颈
- tracing subscriber 内部有锁，多线程高频调用产生锁争用 → involuntary context switch
- 最终 write syscall 导致 disk IO 随 QPS 线性增长

### 需要审查的位置

- `client/proxy.rs` — `handle_work_stream` 入口的 `info!`（每请求触发）
- `tunnel-lib/src/proxy/http.rs` — `connect_inner` 里的 `info!`/`debug!`（每请求触发）
- server/client 各 handler 里连接建立/关闭的日志级别
- 确认哪些 span/event 在 release 模式下会被编译掉

### 原则

- 每请求必经路径：`debug!` 或去掉，默认不输出
- 连接建立/关闭（低频）：`info!` 合理
- 错误路径：`warn!`/`error!` 保留
- 性能敏感的循环内部：用 `trace!` 或条件编译去掉

---

## [perf/TODO-4] first_buf 改为栈分配

**文件**: `tunnel-lib/src/protocol/driver/h1.rs`
**优先级**: 低

```rust
// 现在：heap 分配，每请求 minor fault
let mut first_buf = vec![0u8; 8192];

// 改为：栈分配，零 heap 分配
let mut first_buf = [0u8; 8192];
```

### 预期收益

- 减少每请求 heap 分配次数
- minor page fault 降低
- 需要同步修改 `pos` 相关逻辑，影响面小
