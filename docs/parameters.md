# Parameter Reference: Timeouts, Limits, and Buffers

Full request path: `k6 → TCP (entry) → client → QUIC → server → TCP (upstream)`

---

## 1. 入口与受理层 (Ingress & Entry Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **accept_workers** | `server/listener_mgr.rs`: `sync_listeners` / `client/entry.rs` | 默认: 4 (client `DEFAULT_ACCEPT_WORKERS`; server `Option<usize>`, None → 4) | `entry.accept_workers` (client) / `server.accept_workers` (server) | Accept 串行化；突发流量下建连延迟 (Sync 延迟) | 指标: `connection_latency`; 代码见 `listener_mgr.rs:163` |
| **Listen backlog** | `tunnel-lib/transport/listener.rs`: `listen(4096)` | 4096 | ❌ (硬编码) | 内核丢弃新连接；报 `ECONNREFUSED` | 命令: `netstat -s \| grep "SYNs to LISTEN sockets dropped"` |
| **EMFILE backoff** | `entry.rs`: `EMFILE_BACKOFF_MS` | 100ms | ❌ (client 常量) / `overload.emfile_backoff_ms` (server) | errno 24 (Too many open files) 时暂停 Accept | 日志: `entry accept: too many open files, backing off` |
| **peek_buf_size** | `PeekBufPool::new(size)` | 16 KiB | `proxy_buffers.peek_buf_size` | 缓冲区不足会导致协议识别失败 (Protocol::Unknown) | 日志: `detected protocol: Unknown`; 代码见 `core.rs:48` |
| **http_header_buf_size** | `Http1Driver` header 解析缓冲 | 8 KiB | `proxy_buffers.http_header_buf_size` | HTTP header 最大尺寸；过小会拒绝带大 Cookie 的请求 | 代码见 `proxy_buffers.rs:18` |
| **entry.port / http_entry_port** | client `EntryConfig.port` / client 顶层 `http_entry_port` | None / 未启用 | `entry.port` / `http_entry_port` | Client 本地 TCP/HTTP 入口端口；未配置则不起入口 | `config/client.yaml:19` |
| **metrics_port** | server/client main | None | `metrics_port` (client 顶层 / server `server.metrics_port`) | Prometheus 指标端点；None = 不暴露 | `server/main.rs:285` |

---

## 2. 隧道与传输层 (QUIC / Tunnel Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **max_concurrent_streams** | `quinn::TransportConfig`: `max_concurrent_bidi_streams` | **server default: 1000**；**client default: 100** (code) / 1000 (示例 yaml) / CI: 1000 | `quic.max_concurrent_streams` | `open_bi()` 等待空闲槽位；压力过大导致超时失败。**注意 client 代码 default 与 yaml 示例不一致** | 指标: `open_bi_wait_ms`; 代码见 `quic.rs:30`, `client/config.rs:43` |
| **open_stream_timeout** | `client/entry.rs`: `tokio::time::timeout` / server 对称 | 5s / CI: 5s | client: `reconnect.open_stream_timeout_ms`；server: `server.open_stream_timeout_ms` | 超过此值放弃当前 QUIC 连接并尝试下一个；最终 client 报超时。⚠️ `client/config.rs:177` docstring 写 3000ms，与实际 default 5000ms 不符（源码注释 bug） | 日志: `open_bi timed out after ...`; 见 `entry.rs:104` |
| **stream_window** | `quinn::TransportConfig`: `stream_receive_window` | 4 MiB | `quic.stream_window_mb` | 单个流的流量窗口，耗尽时发送端挂起 (L4 背压) | 指标: `quic_stream_data_blocked` |
| **connection_window** | `quinn::TransportConfig`: `receive_window` (连接聚合流控) | 32 MiB | `quic.connection_window_mb` | 一条 QUIC 连接上所有流共享的总接收窗口；小于 N × stream_window 会提前阻塞 | 代码见 `transport/quic.rs:21` |
| **send_window** | `quinn::TransportConfig`: `send_window` | 8 MiB (client 回退到 `connection_window_mb`) | `quic.send_window_mb` (仅 client) | 本端发送缓冲上限；非对称链路（上下行差异大）时可独立设置 | 代码见 `client/config.rs:67-71` |
| **keepalive_secs** | `quinn::TransportConfig.keep_alive_interval` | 20s | `quic.keepalive_secs` | QUIC 心跳 PING 间隔；必须 < `idle_timeout_secs` 否则空闲连接会被关 | 代码见 `transport/quic.rs:35` |
| **idle_timeout_secs** | `quinn::TransportConfig.max_idle_timeout` | 60s | `quic.idle_timeout_secs` | 空闲 QUIC 连接被关闭的阈值；同步日志阻塞时可能先触发 | §4 Logging Latency 相关 |
| **connections** | `client/worker.rs`: 启动 supervisor 的数量 | 1 / CI: 4 | `quic.connections` | 总吞吐能力 = connections × max_concurrent_streams | 见 `client/app.rs` 的 slot 启动逻辑 |
| **congestion_controller** | `quinn::BbrConfig` / `CubicConfig` / `NewRenoConfig` | bbr | `quic.congestion` | 丢包重传与吞吐爬坡算法；bbr 适合高带宽波动链路；未知值 fallback 到 quinn 默认（NewReno） | 代码见 `quic.rs:41-54` |
| **login_timeout_secs** | `server/handlers/quic.rs:34` | 10s | `server.login_timeout_secs` | 服务端对 client QUIC 登录握手超时；与 client `reconnect.login_timeout_ms` (5000ms) **不对称** | 代码见 `tunnel-store/src/server_config.rs:143` |

---

## 2.5 过载保护 (Overload Protection)

当 QUIC 流槽位接近 `max_concurrent_streams` 上限时，两侧在 `open_bi()` 前通过 `maybe_slow_path` 主动让渡或短暂睡眠，避免 open_bi 堆积超时。Client 侧计数器挂在 `EntryConnPool` 的每条 QUIC 连接上 (`client/conn_pool.rs:7`)，Server 侧挂在 `ClientRegistry` 的 `SelectedConnection` 上 (`server/registry.rs:16`)。

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **overload.mode** | `client/entry.rs:31` `maybe_slow_path`; `server/handlers/mod.rs:12` 同名 | `inflight_slowpath` / `burst` | `overload.mode` | `burst` 直接 bypass 阻塞逻辑，任由 QUIC 层排队或 `open_bi` 超时 | 搜代码路径 `OverloadMode::Burst` |
| **overload.inflight_yield_threshold** | 同上 `maybe_slow_path` inflight 比较 | 800 (两侧一致) | `overload.inflight_yield_threshold` | 在途流 ≥ 该值时每次 `open_bi` 前 `tokio::task::yield_now()`；值过低会频繁让出 runtime 拉低吞吐 | 指标: `inflight` / `max_concurrent_streams` 占比 |
| **overload.inflight_sleep_threshold** | 同上 `maybe_slow_path` inflight 比较 | 950 (两侧一致) | `overload.inflight_sleep_threshold` | 在途流 ≥ 该值时进入 backoff 循环（具体由 `backoff_strategy` 决定） | 指标: p99 latency 与 inflight 曲线同步上抬 |
| **overload.inflight_sleep_ms** | 同上 `maybe_slow_path` 总时长预算 | 2ms (两侧一致) | `overload.inflight_sleep_ms` | backoff 循环的**总超时预算**；超过后放行到 `open_bi`（与 `backoff_strategy` 配合） | tracing span 中 `maybe_slow_path` 前后时间差 |
| **overload.inflight_yield_pct** | 同上，**优先级高于**绝对阈值 | 0.80 (两侧一致) | `overload.inflight_yield_pct` | 相对 `max_concurrent_streams` 的比例阈值；设置后覆盖 `inflight_yield_threshold` | `client/config.rs:126` / `server_config.rs:84` |
| **overload.inflight_sleep_pct** | 同上 | 0.95 (两侧一致) | `overload.inflight_sleep_pct` | 同上，覆盖 `inflight_sleep_threshold` | 同上 |
| **overload.backoff_strategy** | `tunnel-lib/src/overload.rs` `maybe_slow_path` | `exponential` (默认) / `fixed` / `none` | `overload.backoff_strategy` | `exponential`: 每轮重查 inflight，从 `budget/16` 翻倍到 `budget/4`，槽位一空立刻返回；`fixed`: 直接 sleep 一整个 budget；`none`: 不等，交给 QUIC 背压 | 代码见 `exponential_backoff` |
| **overload.emfile_backoff_ms** (仅 server) | `server/handlers/tcp.rs:19` 与 `http.rs:19` accept 循环 | 100ms | `overload.emfile_backoff_ms` | EMFILE 时暂停 accept 的时长；偏低会 CPU 打满，偏高丢连接 | 日志: `too many open files, backing off` |

---

## 3. 业务转发层 (Proxy / Data Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **relay_buf_size** | `relay_inner` 中的 `BufReader::with_capacity` | 64 KiB / 范围 >=4K | `proxy_buffers.relay_buf_size` | **内存风险**: 1w 并发 = 1.25GB RAM 消耗 (双向 Buffer) | `top/htop` 观察 RSS 增长速度; 见 `relay.rs:25` |
| **http_body_chunk** | `Http1Driver` / `H2Peer` 读块大小 | 8 KiB | `proxy_buffers.http_body_chunk_size` | 影响 L7 转发的系统调用频率及单次 IO 耗时 | 代码见 `h1.rs` 和 `h2_proxy.rs` |
| **max_idle_per_host** | `hyper::client::pool::Config` | **代码 default: 128**；yaml 示例: 10 | `http_pool.max_idle_per_host` | 超过负载时，闲置连接被关闭，新请求需重新建连 (TCP Handshake)。⚠️ 代码 default 与 `config/server.yaml:43` 示例值不一致 | 见 `server/egress.rs` 的 pool 初始化 |
| **http_pool.idle_timeout_secs** | `HttpClientParams.pool_idle_timeout_secs` | None (yaml 示例: 90) | `http_pool.idle_timeout_secs` | 池内空闲连接最大存活时间；None = 不主动关闭 | `tunnel-lib/src/config/http_pool.rs:13` |
| **http_pool.tcp_keepalive_secs** | `HttpClientParams.tcp_keepalive_secs` | 15s | `http_pool.tcp_keepalive_secs` | egress 池连接的 TCP_KEEPALIVE 间隔 | `tunnel-lib/src/config/http_pool.rs:15` |

---

## 3.5 TCP Socket 选项 (TCP Plane)

所有经由 `TcpParams::apply` 的 socket 都会被设置。Server 应用于 ingress TCP；client 应用于 upstream TCP。

| 参数 (Parameter) | 消费者 / 逻辑位置 | 默认值 | YAML 路径 | 阈值影响 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **tcp.nodelay** | `TcpStream::set_nodelay` | true | `tcp.nodelay` | 关闭后小包会被 Nagle 合并，增加 40ms 延迟 | — |
| **tcp.recv_buf_size** | `setsockopt SO_RCVBUF` | 4 MiB | `tcp.recv_buf_size` | 高 BDP 链路下过小会压制吞吐 | `ss -ti` 看 rcv_space |
| **tcp.send_buf_size** | `setsockopt SO_SNDBUF` | 4 MiB | `tcp.send_buf_size` | 发送阻塞（POLLOUT 等待） | 同上 `snd_cwnd` |
| **tcp.keepalive** | `setsockopt SO_KEEPALIVE` | true | `tcp.keepalive` | 长连接对端崩溃后能被内核感知 | — |
| **tcp.user_timeout_ms** | `setsockopt TCP_USER_TIMEOUT` (Linux) | 30000ms | `tcp.user_timeout_ms` | 未确认的数据超过此时长则关闭连接；0 = 禁用 | 代码见 `transport/tcp_params.rs:37` |

---

## 3.6 TLS / PKI (Client 到 Server)

| 参数 (Parameter) | 消费者 / 逻辑位置 | 默认值 | YAML 路径 | 阈值影响 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **tls_skip_verify** | `client/main.rs:427` | false (yaml 示例: true) | `tls_skip_verify` (client) | 跳过服务端证书校验，仅供开发 | — |
| **tls_ca_cert** | `client/main.rs:435` | None | `tls_ca_cert` (client) | 自定义 CA 路径；为空时走系统根 | — |
| **tls_server_name** | `ClientConfigFile::tls_server_name()` | None → fallback 到 `server_addr` | `tls_server_name` (client) | SNI 覆盖；与证书 CN 不匹配时握手失败 | `client/config.rs:308` |
| **allow_insecure_fallback** | `client/main.rs:460` | false | `allow_insecure_fallback` (client) | 系统无根证书时是否允许降级为 insecure；生产应保持 false | — |
| **pki.cert_cache_ttl_secs** | `PkiParams`, `init_cert_cache` | 3600s | `server.pki.cert_cache_ttl_secs` | 自生成证书缓存存活；过短会在高并发下反复签发消耗 CPU | `tunnel-lib/src/infra/pki.rs:11` |

---

## 3.7 Client Reconnect (Backoff / Handshake 超时)

Client → Server QUIC 建连与登录握手的时间预算。`initial_delay_ms ≤ max_delay_ms` 为硬约束。

| 参数 (Parameter) | 消费者 / 逻辑位置 | 默认值 | YAML 路径 | 阈值影响 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **reconnect.initial_delay_ms** | 重连指数退避起点 | 1000ms | `reconnect.initial_delay_ms` | 与故障恢复速度成反比 | — |
| **reconnect.max_delay_ms** | 重连退避上限 | 60000ms | `reconnect.max_delay_ms` | 对端长时间宕机时的最大等待窗口 | — |
| **reconnect.grace_ms** | 取消后重连前的 grace | 100ms | `reconnect.grace_ms` | 避免取消→立即重连抖动 | — |
| **reconnect.connect_timeout_ms** | QUIC 连接建立超时 | 10000ms | `reconnect.connect_timeout_ms` | 链路慢或 MTU 问题时会频繁触发 | — |
| **reconnect.resolve_timeout_ms** | DNS 解析超时 | 5000ms | `reconnect.resolve_timeout_ms` | DNS 故障时阻塞 reconnect 循环 | — |
| **reconnect.login_timeout_ms** | Client 侧 login 握手超时 | 5000ms | `reconnect.login_timeout_ms` | 与 server `login_timeout_secs` (10s) **不对称**；短的一侧先触发 | 日志: `login timed out` |
| **reconnect.startup_jitter_ms** | 启动抖动窗口 | 300ms | `reconnect.startup_jitter_ms` | 集群冷启动时避免 thundering herd | — |
| **reconnect.open_stream_timeout_ms** | `open_bi()` 等待流槽超时 | 5000ms | `reconnect.open_stream_timeout_ms` | 见 §2 同项说明 | — |

---

## 3.8 Server 专属 (进程级基础配置)

| 参数 (Parameter) | 消费者 / 逻辑位置 | 默认值 | YAML 路径 | 阈值影响 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **tunnel_port** | `server/handlers/quic.rs:11` | 必填 | `server.tunnel_port` | QUIC 监听端口 | `server/config.rs:211` 有 validate |
| **h2_single_authority** | `server/main.rs:408` | true | `server.h2_single_authority` | H2 跨 vhost 共用 authority；关闭后每 host 独立池 | — |
| **database_url** | standalone 模式 SQLite 路径 | (empty) | `server.database_url` | ctld 模式下不使用；standalone 模式必填 | — |
| **max_connections** / **max_tcp_connections** | ⚠️ **未消费** | yaml 注释 10000 | `config/server.yaml:20-21` | **YAML 注释里有但代码里 grep 不到任何消费者**，是历史遗留/未实现 | 应从 YAML 模板中删除 |

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

## 5. Pingora 参数系统对照 (Design Reference)

对照 Cloudflare [pingora](https://github.com/cloudflare/pingora) (`pingora-core/src/server/configuration/mod.rs` `ServerConf`, `connectors/mod.rs` `ConnectorOptions`, `upstreams/peer.rs` `PeerOptions`, `listeners/l4.rs` `TcpSocketOptions`) 的设计，总结 duotunnel 当前参数系统的差距。

### 5.1 架构分层对比

Pingora 采用**三层覆盖**：

```
ServerConf           ← 全局进程级（YAML/CLI）
  ↓ from_server_conf()
ConnectorOptions     ← 派生到 connector
  ↓
PeerOptions          ← 每个 upstream / 每次连接独立覆盖
TcpSocketOptions     ← 每个 listener 独立覆盖
HttpServerOptions    ← 每个 service 独立覆盖
```

duotunnel 当前只有**全局一份**（`ClientConfigFile` / `ServerConfigFile`），`UpstreamDef` 里只有 `servers + lb_policy`，无 per-upstream 的 tcp/http_pool/timeout 覆盖。

### 5.2 参数 / 模式差距清单

| Pingora 参数 / 模式 | duotunnel 对应物 | 差距 / 建议 |
| :--- | :--- | :--- |
| `ServerConf.version: usize` | ❌ | YAML 没有 schema 版本，破坏性改动无阻断点 |
| `graceful_shutdown_timeout_seconds`, `grace_period_seconds` | ❌ | 没有 graceful shutdown 时间预算；只在 `reconnect.grace_ms` 有局部 grace |
| `ServerConf.max_retries` (proxy 可重试错误上限) | ❌ | `reconnect.*` 只有退避，没有总重试次数上限 |
| `HttpServerOptions.keepalive_request_limit` | ❌ | H1 keep-alive 循环无限复用，长连接会累积碎片内存 |
| `PeerOptions.max_h2_streams` | QUIC 有（`max_concurrent_streams`），H2 egress 无 | egress 侧 H2 没有 per-upstream 流数限制 |
| `PeerOptions.h2_ping_interval` | QUIC `keepalive_secs` 有，H2 egress 无 | egress H2 长连接没有主动探活 |
| `PeerOptions.connection_timeout / read_timeout / write_timeout / idle_timeout` | 散落在 `reconnect.*` / `http_pool.*` | duotunnel 命名按"动作"（reconnect/login/open_stream）而非"语义"（read/write/idle），上游方向的 read/write timeout 完全缺失 |
| `TcpSocketOptions.so_reuseport` | ❌（`todo.md` TODO-23） | pingora 已有，duotunnel 还在 TODO |
| `TcpSocketOptions.tcp_fastopen` / `dscp` / `ipv6_only` | ❌ | 没有 TFO、DSCP、IPv6 独占等 |
| `ServerConf.threads` / `listener_tasks_per_fd` / `work_stealing` | 仅 `accept_workers` | duotunnel 没有"每 service 独立 runtime"的概念 |
| `ServerConf.max_blocking_threads` / `blocking_threads_ttl_seconds` | ❌ | tokio 阻塞池未暴露 |
| `upstream_connect_offload_threadpools` | ❌ | 没有"连接建立 CPU 隔离"的概念 |
| `#[non_exhaustive]` + `Option<T>` 表示"未配置" | 部分已用 | duotunnel 存在 `u64=0` / `Option<u64>` 混用（如 `overload.inflight_sleep_ms: u64`、`tcp.user_timeout_ms: u32`，0 语义模糊） |
| `ConnectorOptions::from_server_conf` 派生 | `impl From<&XxxConfig> for XxxParams` | **duotunnel 这层甚至更干净**（trait 化 vs 手写），保留 |
| `validate(self) -> Result<Self>` 链式 | `validate(&self) -> Result<()>` 收集 errors vec | **duotunnel 更好**（一次报出所有错），保留 |
| CLI `Opt::merge_with_opt(&mut ServerConf)` | 仅 `TUNNEL_CLIENT__` 环境变量覆盖少数字段 | 没有完整 CLI override 层 |
| `overload.inflight_{yield,sleep}_pct` 百分比覆盖 | ✅ 已有 | **duotunnel 独有的好设计**，pingora 没有 |

### 5.3 当前文档化已发现的不一致

1. **`max_concurrent_streams` client 代码 default=100，yaml 示例=1000**（`client/config.rs:43` vs `config/client.yaml:25`）
2. **`max_idle_per_host` 代码 default=128，yaml 示例=10**（`http_pool.rs:14` vs `config/server.yaml:43`）
3. **`open_stream_timeout_ms` 源码 docstring 写 3000ms，实际 default=5000ms**（`client/config.rs:177`）
4. **`login_timeout` 两侧不对称**：server 10s，client 5s
5. **`max_connections` / `max_tcp_connections`** YAML 注释里有但代码无消费者，是死配置

---

## 6. Future Roadmap & Design TODOs

### TODO: Error Code Design and Propagation
目前请求失败（如 `open_bi` 超时、后端 EOF）仅在日志中记录，无法回传给 k6 主动识别错误码。
- **目标**: 设计跨隧道的 `ErrorCode`，将后端 502/504 等细节透传至入口处。

### TODO: Unified Parameter Configuration Design (v1)

对照 §5 pingora 设计，按三步改造：

**Step 1 — YAML schema 版本化 + 清理死配置**
- 顶层加 `version: 1` 字段（参考 `ServerConf.version`），便于未来做破坏性 migration
- 删除 `config/server.yaml` 里的 `max_connections` / `max_tcp_connections` 注释（无消费者）
- 统一所有 timeout 的单位后缀（`_ms` 或 `_secs`），目前混用：`connect_timeout_ms`、`login_timeout_secs`、`idle_timeout_secs`、`open_stream_timeout_ms`
- 修复 client `max_concurrent_streams` default（100→1000）与 yaml 示例一致

**Step 2 — 抽出 `TimeoutConfig` (语义化拆分)**

目前 `reconnect.*` 混了 DNS / TCP connect / QUIC handshake / login / stream-open 多类超时。参考 pingora `PeerOptions` 按语义拆成四元组：

```rust
pub struct TimeoutConfig {
    pub connect_timeout_ms: Option<u64>,       // TCP/QUIC 建连
    pub handshake_timeout_ms: Option<u64>,     // TLS/Login 握手
    pub read_timeout_ms: Option<u64>,          // ⚠️ 新增：上游读超时
    pub write_timeout_ms: Option<u64>,         // ⚠️ 新增：上游写超时
    pub idle_timeout_ms: Option<u64>,          // 空闲回收
}
```
两侧 server/client 都复用同一套，`reconnect.*` 只保留纯退避参数（`initial_delay_ms / max_delay_ms / grace_ms / startup_jitter_ms`）。

**Step 3 — Per-upstream override**

`UpstreamDef` 下沉配置容器，全局值作为 fallback：

```rust
pub struct UpstreamDef {
    pub servers: Vec<ServerDef>,
    pub lb_policy: String,
    #[serde(default)]
    pub tcp: Option<TcpConfig>,           // per-upstream SO_* 覆盖
    #[serde(default)]
    pub http_pool: Option<HttpPoolConfig>,
    #[serde(default)]
    pub timeouts: Option<TimeoutConfig>,
    #[serde(default)]
    pub max_h2_streams: Option<usize>,    // 对齐 pingora
    #[serde(default)]
    pub h2_ping_interval_ms: Option<u64>, // 对齐 pingora
}
```

这一步直接为 `todo.md` **TODO-62（per-peer 协议自适应 H2/H1 探测+记忆）** 铺路——upstream 维度天然是协议探测结果的承载容器。

**Step 4 — Server→Client 配置下发（原 TODO 目标）**
统一命名后，`ClientRegistry` 可以把 `overload.*` / `quic.*` 的"推荐值"通过现有的控制流推给 client，避免两侧 yaml 漂移。依赖 Step 1-3 完成。
