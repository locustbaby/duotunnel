# Parameter Reference: Timeouts, Limits, and Buffers

Full request path: `k6 → TCP (entry) → client → QUIC → server → TCP (upstream)`

---

## 1. 入口与受理层 (Ingress & Entry Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **accept_workers** | `server/listener_mgr.rs`: `sync_listeners` / `client/entry.rs` | 默认: 4 | `entry.accept_workers` | Accept 串行化；突发流量下建连延迟 (Sync 延迟) | 指标: `connection_latency`; 代码见 `listener_mgr.rs:163` |
| **Listen backlog** | `tunnel-lib/transport/listener.rs`: `listen(4096)` | 4096 | ❌ (硬编码) | 内核丢弃新连接；报 `ECONNREFUSED` | 命令: `netstat -s \| grep "SYNs to LISTEN sockets dropped"` |
| **EMFILE backoff** | `entry.rs`: `EMFILE_BACKOFF_MS` | 100ms | ❌ (常量) | errno 24 (Too many open files) 时暂停 Accept | 日志: `entry accept: too many open files, backing off` |
| **peek_buf_size** | `PeekBufPool::new(size)` | 16 KiB | `proxy_buffers.peek_buf_size` | 缓冲区不足会导致协议识别失败 (Protocol::Unknown) | 日志: `detected protocol: Unknown`; 代码见 `core.rs:48` |

---

## 2. 隧道与传输层 (QUIC / Tunnel Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **max_concurrent_streams** | `quinn::TransportConfig`: `max_concurrent_bidi_streams` | 1000 / CI: 1000 | `quic.max_concurrent_streams` | `open_bi()` 等待空闲槽位；压力过大导致超时失败 | 指标: `open_bi_wait_ms`; 代码见 `quic.rs:30` |
| **open_stream_timeout** | `client/entry.rs`: `tokio::time::timeout` | 5s / CI: 5s | `reconnect.open_stream_timeout_ms` | 超过此值放弃当前 QUIC 连接并尝试下一个；最终 client 报超时 | 日志: `open_bi timed out after ...`; 见 `entry.rs:104` |
| **stream_window** | `quinn::TransportConfig`: `stream_receive_window` | 4 MiB | `quic.stream_window_mb` | 单个流的流量窗口，耗尽时发送端挂起 (L4 背压) | 指标: `quic_stream_data_blocked` |
| **connections** | `client/worker.rs`: 启动 supervisor 的数量 | 1 / CI: 4 | `quic.connections` | 总吞吐能力 = connections × max_concurrent_streams | 见 `client/app.rs` 的 slot 启动逻辑 |
| **congestion_controller** | `quinn::BbrConfig` / `CubicConfig` | bbr | `quic.congestion` | 丢包重传与吞吐爬坡算法；bbr 适合高带宽波动链路 | 代码见 `quic.rs:41` |

---

## 2.5 过载保护 (Overload Protection)

当 QUIC 流槽位接近 `max_concurrent_streams` 上限时，两侧在 `open_bi()` 前通过 `maybe_slow_path` 主动让渡或短暂睡眠，避免 open_bi 堆积超时。Client 侧计数器挂在 `EntryConnPool` 的每条 QUIC 连接上 (`client/conn_pool.rs:7`)，Server 侧挂在 `ClientRegistry` 的 `SelectedConnection` 上 (`server/registry.rs:16`)。

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **overload.mode** | `client/entry.rs:31` `maybe_slow_path`; `server/handlers/mod.rs:12` 同名 | `inflight_slowpath` / `burst` | `overload.mode` | `burst` 直接 bypass 阻塞逻辑，任由 QUIC 层排队或 `open_bi` 超时 | 搜代码路径 `OverloadMode::Burst` |
| **overload.inflight_yield_threshold** | 同上 `maybe_slow_path` inflight 比较 | 800 (两侧一致) | `overload.inflight_yield_threshold` | 在途流 ≥ 该值时每次 `open_bi` 前 `tokio::task::yield_now()`；值过低会频繁让出 runtime 拉低吞吐 | 指标: `inflight` / `max_concurrent_streams` 占比 |
| **overload.inflight_sleep_threshold** | 同上 `maybe_slow_path` inflight 比较 | 950 (两侧一致) | `overload.inflight_sleep_threshold` | 在途流 ≥ 该值时进入 backoff 循环（具体由 `backoff_strategy` 决定） | 指标: p99 latency 与 inflight 曲线同步上抬 |
| **overload.inflight_sleep_ms** | 同上 `maybe_slow_path` 总时长预算 | 2ms (两侧一致) | `overload.inflight_sleep_ms` | backoff 循环的**总超时预算**；超过后放行到 `open_bi`（与 `backoff_strategy` 配合） | tracing span 中 `maybe_slow_path` 前后时间差 |
| **overload.backoff_strategy** | `tunnel-lib/src/overload.rs` `maybe_slow_path` | `exponential` (默认) / `fixed` / `none` | `overload.backoff_strategy` | `exponential`: 每轮重查 inflight，从 `budget/16` 翻倍到 `budget/4`，槽位一空立刻返回；`fixed`: 直接 sleep 一整个 budget；`none`: 不等，交给 QUIC 背压 | 代码见 `exponential_backoff` |
| **overload.emfile_backoff_ms** (仅 server) | `server/handlers/tcp.rs:19` 与 `http.rs:19` accept 循环 | 100ms | `overload.emfile_backoff_ms` | EMFILE 时暂停 accept 的时长；偏低会 CPU 打满，偏高丢连接 | 日志: `too many open files, backing off` |

---

## 3. 业务转发层 (Proxy / Data Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **relay_buf_size** | `relay_inner` 中的 `BufReader::with_capacity` | 64 KiB / 范围 >=4K | `proxy_buffers.relay_buf_size` | **内存风险**: 1w 并发 = 1.25GB RAM 消耗 (双向 Buffer) | `top/htop` 观察 RSS 增长速度; 见 `relay.rs:25` |
| **http_body_chunk** | `Http1Driver` / `H2Peer` 读块大小 | 8 KiB | `proxy_buffers.http_body_chunk_size` | 影响 L7 转发的系统调用频率及单次 IO 耗时 | 代码见 `h1.rs` 和 `h2_proxy.rs` |
| **max_idle_per_host** | `hyper::client::pool::Config` | 128 | `http_pool.max_idle_per_host` | 超过负载时，闲置连接被关闭，新请求需重新建连 (TCP Handshake) | 见 `server/egress.rs` 的 pool 初始化 |

---

## 4. 架构资源与性能深度特征 (Architecture & Resource Characteristics)

| 指标 / 瓶颈点 | 使用方 (Consumer / Logic) | 默认特征 / 复杂度 | 影响因子 (Factor) | 影响效果 (Effect) | 排查分析 (Analysis) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Routing Selection** | `ClientGroup::select_healthy` | O(N) 线性扫描 | Client 数 (N) | **CPU 尖峰**: 当分组内 Client 连接 > 500 时，单次转发选路开销显著上升 | CPU Profile (火焰图) 见 `select_healthy`; `registry.rs:104` |
| **RCU Rebuild Cost** | `ClientGroup::snapshot.store` | Vec Clone + Swap | 注册/注销频率 (Churn) | **转发抖动**: 频繁重连导致 forward 路径短时间停顿/分配大量小对象 | 日志: `registering/unregistering client` 频率 |
| **Initial Data Copy** | `ProxyEngine::run_stream` | `copy_from_slice` | 请求到达频率 | **内存毛刺**: 每请求一次额外分配；虽量小但并发高时影响 GC/RSS | 代码见 `core.rs:52` |
| **Server Overload** | `server/handlers/mod.rs:11` `maybe_slow_path` + 对称的 `client/entry.rs:30` | yield/sleep 逻辑 | In-flight 总数 vs. 阈值 `overload.inflight_{yield,sleep}_threshold` (§2.5) | **延迟突增**: 触发过载保护时系统主动挂起请求 | 指标: `inflight_requests`; `system_overload_count` |
| **Logging Latency** | `relay.rs`: `debug!` / `tracing` | 同步阻塞写入 | `log_level: info` | **吞吐硬封顶**: 同步日志写磁盘导致 Worker 线程挂起，QUIC 会发生 Idle Timeout | 火焰图见 `std::io::Write` 阻塞热点 |

---

## Future Roadmap & Design TODOs

### TODO: Error Code Design and Propagation
目前请求失败（如 `open_bi` 超时、后端 EOF）仅在日志中记录，无法回传给 k6 主动识别错误码。
- **目标**: 设计跨隧道的 `ErrorCode`，将后端 502/504 等细节透传至入口处。

### TODO: Unified Parameter Configuration Design
配置参数目前分散在 `ReconnectConfig`, `QuicConfig`, `HttpPoolConfig` 等多个结构体中。
- **目标**: 统一参数命名规范，并支持 server 端向 client 端主动推送推荐参数配置。
