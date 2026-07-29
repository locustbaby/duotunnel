# Parameter Reference: Timeouts, Limits, and Buffers

> Architecture, module layout, and call flows: [architecture.md](./architecture.md)

Full request path: `k6 → TCP (entry) → client → QUIC → server → TCP (upstream)`

---

## 1. 入口与受理层 (Ingress & Entry Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **accept_workers** | `duotunnel-server/ingress/listener_mgr.rs`: `spawn_single_listener` / `duotunnel-client/egress/listener.rs` | 未配置 → `effective_runtime_parallelism()`（受 cgroup `CPUQuota` 约束；CI `100%` → **1**） | `entry.accept_workers` (client) / `server.accept_workers` (server) | Accept 串行化；突发流量下建连延迟 (Sync 延迟)。每个 worker 各自 `build_reuseport_listener` 独立绑定同一端口 | resolver: `resolve_accept_workers` (`infra/runtime.rs:136`) |
| **accept loop 的 spawn 目标** (server) | `state.proxy_handle().spawn(...)` —— `duotunnel-server/ingress/listener_mgr.rs` 里 `spawn_single_listener` 的 HTTP / TCP worker 分支、UDP 分支，以及 `sync_listeners_inner` 派发 `spawn_single_listener` 本身 | 启动时一次性捕获的 `Handle::current()`（`duotunnel-server/bootstrap/mod.rs:385`、`duotunnel-server/runtime/app.rs:54`；访问器 `duotunnel-server/bootstrap/mod.rs:167`） | ❌ (代码结构) | accept loop 一律落在这个 handle 上，不再继承 `sync_listeners` 调用方所在的 runtime（ctld watch / hot-reload 路径同样如此）。关键原因是 listener socket 在里面构造，而 `TcpListener::from_std` 会把 fd 注册到**调用方 runtime 的 IO driver** 上——若继承调用方 runtime，每个公开 listener 的就绪通知就会被绑到"恰好应用了这次配置"的那个 runtime 上 | 启动日志: `listener active` |
| **Listen backlog** | `duotunnel-lib/src/transport/listener.rs:29`: `listen(4096)` | 4096 | ❌ (硬编码) | 内核丢弃新连接；报 `ECONNREFUSED` | 命令: `netstat -s \| grep "SYNs to LISTEN sockets dropped"` |
| **EMFILE backoff** | `duotunnel-client/egress/listener.rs:18`: `EMFILE_BACKOFF` (`Duration`) | 100ms | ❌ (client 常量) / `overload.emfile_backoff_ms` (server, `duotunnel-server/bootstrap/mod.rs:176`) | errno 24 (Too many open files) 时暂停 Accept | 日志: `entry accept: too many open files, backing off` |
| **peek_buf_size** | `PeekBufPool::new(size)`：client entry (`duotunnel-client/egress/listener.rs:107`)、server ingress TCP handler (`duotunnel-server/bootstrap/mod.rs:322` → `duotunnel-server/ingress/handlers/tcp.rs:54`) | 16 KiB | `proxy_buffers.peek_buf_size` | 缓冲区不足会导致协议识别失败 (Protocol::Unknown) | 日志: `detected protocol: Unknown`; 默认值见 `config/proxy_buffers.rs:18`。Peek buffer pool 始终启用并按线程保留有界空闲 buffer。⚠️ 另有两个**不受此配置影响**的独立池：`IngressDispatcher` 的 Phase 1 用硬编码 `SNIFF_LIMIT = 4096` (`plugin/dispatcher.rs:30`、`:37`)，`ProxyEngine::run_stream` 的 QUIC 流嗅探也用硬编码 4096 (`proxy/core.rs:46`) |
| **http_header_buf_size** | ❌ **无消费者**（仅存在于 `ProxyBufferConfig`/`ProxyBufferParams`） | 8 KiB (`config/proxy_buffers.rs:19`) | `proxy_buffers.http_header_buf_size` | 改这个值不生效：`Http1Driver` 的 header 缓冲与 431 阈值是硬编码 8 KiB（`protocol/driver/h1.rs:37`、`:241`、`:245`） | 见 §6 死配置清理 TODO |
| **sniff_timeout_ms** | client entry `duotunnel-client/egress/listener.rs:109`；server ingress `plugin/dispatcher.rs:38`（HTTP 入口经 `handlers/http.rs:47`）/ `handlers/tcp.rs:60`，均取 `state.sniff_timeout()` (`duotunnel-server/bootstrap/mod.rs:229`) | 2500ms | `proxy_buffers.sniff_timeout_ms` | 协议识别的最大时间预算；超时会主动断开以防御 Slowloris 攻击 | 日志: `protocol sniffing timed out`。⚠️ `ProxyEngine::run_stream` 的 QUIC 流嗅探用硬编码 5s（`proxy/core.rs:70-71`），不受此配置影响 |
| **entry.port** | client `EntryConfig.port` → `EgressListenerService` | None = 不启用 | `entry.port` | Client 本地 TCP/HTTP 入口端口（协议嗅探 + 转发） | `duotunnel-client/runtime/app.rs` |
| **entry.accept_workers** | `duotunnel-client/egress/listener.rs` / `resolve_accept_workers` | 未配置 → `effective_runtime_parallelism()` | `entry.accept_workers` | Client 入口 accept worker 数 | 同 **accept_workers** |
| **udp_entries[]** | `duotunnel-client/egress/udp_listener.rs` `UdpEgressListenerService` | `[]` (空) | `udp_entries[].port` / `udp_entries[].proxy_name` | 本地 UDP 入口；每项绑定一个 proxy_name | `duotunnel-client/runtime/app.rs` |
| **metrics_port** | duotunnel-server + duotunnel-client runtime startup | None | `metrics_port` (client 顶层 / server `server.metrics_port`) | Prometheus 指标端点；None = 不暴露 | `duotunnel-server/runtime/app.rs` |

---

## 2. 隧道与传输层 (QUIC / Tunnel Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **max_concurrent_streams** | `quinn::TransportConfig`: `max_concurrent_bidi_streams` + `max_concurrent_uni_streams` (`transport/quic.rs:47-48`) | **1000**（duotunnel-client + duotunnel-server 代码 default 与 yaml 示例一致）/ CI: 1000 | `quic.max_concurrent_streams` | `open_bi()` 等待空闲槽位；压力过大导致超时失败。同时作为 `ConnectionHandle` 上 `stream_semaphore` 的容量（`transport/connection_handle.rs:39-40`），超限直接 `RejectedOverloaded` | 指标: `open_bi_wait_ms`; 代码见 `transport/quic.rs:34`, `duotunnel-client/bootstrap/config.rs:43` |
| **open_stream_timeout** | 两侧统一由 `open_bi_guarded` 消费 (`transport/open_bi.rs:102`)，经 `OpenStreamRequest.stream_timeout` 传入 | 5s / CI: 5s | client: `reconnect.open_stream_timeout_ms`；server: `server.open_stream_timeout_ms` | 超过此值放弃当前 QUIC 连接并尝试下一个；最终 client 报超时。注意这个超时只覆盖"等流控信用"的慢路径：`open_bi()` 能立刻就绪时直接返回，不进计时 (`transport/open_bi.rs:70-85`) | 日志: `open_bi timed out after ...`; tracing span `waiting_for_stream_credit` (`transport/open_bi.rs:100`) |
| **stream_window** | `quinn::TransportConfig`: `stream_receive_window` | 4 MiB | `quic.stream_window_mb` | 单个流的流量窗口，耗尽时发送端挂起 (L4 背压) | 指标: `quic_stream_data_blocked`; 默认值 `transport/quic.rs:35` |
| **connection_window** | `quinn::TransportConfig`: `receive_window` (连接聚合流控) | 32 MiB | `quic.connection_window_mb` | 一条 QUIC 连接上所有流共享的总接收窗口；小于 N × stream_window 会提前阻塞 | 代码见 `transport/quic.rs:36` |
| **send_window** | `quinn::TransportConfig`: `send_window` | 8 MiB（未配置时回退到 `QuicTransportParams::default().send_window_bytes`，两侧一致） | `quic.send_window_mb`（client 与 server 都有） | 本端发送缓冲上限；非对称链路（上下行差异大）时可独立设置 | 代码见 `transport/quic.rs:37`, `config/quic.rs:30-33`, `duotunnel-client/bootstrap/config.rs:68-71` |
| **keepalive_secs** | `quinn::TransportConfig.keep_alive_interval` | 20s | `quic.keepalive_secs` | QUIC 心跳 PING 间隔；必须 < `idle_timeout_secs` 否则空闲连接会被关 | 代码见 `transport/quic.rs:38`（应用点 `:52`） |
| **idle_timeout_secs** | `quinn::TransportConfig.max_idle_timeout` | 180s | `quic.idle_timeout_secs` | 空闲 QUIC 连接被关闭的阈值；同步日志阻塞时可能先触发 | §4 Logging Latency 相关 |
| **connections** | `duotunnel-client/tunnel/pool.rs`: 启动 supervisor 的数量 | `0` = auto (`effective_runtime_parallelism()`；CI `CPUQuota=100%` → **1**) | `quic.connections` | 总吞吐能力 = connections × max_concurrent_streams | resolver: `resolve_connection_count` |
| **min_ready_tunnels** | `ClientHealth` + `EntryConnPool` 已提交 active connection count | **1** | `quic.min_ready_tunnels` | 连接在 cleanup 前先退池；低于该值 `/healthz` 才返回 not ready。必须 ≤ resolved `quic.connections`；低于 desired 但仍达到该值只标记 degraded | 启动日志: `client readiness initialized` |
| **shards** | `ClientRegistry` / `EntryConnPool` shard 数 | 未配置 → `effective_runtime_parallelism()`；client 上限为 `connections` | `quic.shards` | 注册表/连接池分片数；影响选路与 actor 并行度 | `resolve_shard_count`; env `DUOTUNNEL_CLIENT__quic.shards` |
| **congestion_controller** | `quinn::BbrConfig` / `CubicConfig` / `NewRenoConfig` | bbr | `quic.congestion` | 丢包重传与吞吐爬坡算法；bbr 适合高带宽波动链路；未知值 fallback 到 quinn 默认（NewReno） | 代码见 `transport/quic.rs:58-71` |
| **login_timeout_secs** | `duotunnel-server/ingress/handlers/quic.rs:144-145` → `pre_auth_deadline` | 10s | `server.login_timeout_secs` | **整个认证前阶段共享的单一 deadline**（不是每步独立超时）：QUIC handshake (`:146`)、`accept_bi` (`:155`)、收 message type (`:170`)、收 Login body (`:204`)、`auth_store().authenticate()` (`:264`) 全部用同一个 `timeout_at(pre_auth_deadline)`。因此一条连接从被 accept 到认证结束最多占用 `login_timeout_secs`，慢速对端无法靠"每步都卡到快超时"把未认证 permit 持有若干倍时长。调参时按"整段预算"而非"单步预算"理解；与 client `reconnect.login_timeout_ms` (10000ms) 对齐 | 默认值 `duotunnel-lib/src/config/file.rs:164-166`；启动校验 `:303-305`（必须 >= 1）。日志: `login handshake timed out waiting for ...` / `authentication timed out` |
| **server.max_unauthenticated_connections** | `duotunnel-server/ingress/handlers/quic.rs:48`：`Semaphore::new(state.max_unauthenticated_connections())`，accept 循环 `try_acquire_owned` (`:84`)；permit 在认证成功后 `drop` (`:309`)，任一失败路径 return 即释放 | **64** | `server.max_unauthenticated_connections` | 未认证 QUIC 连接的并发上限。取不到 permit 时调 `incoming.refuse()` (`:94`) —— quinn 在完成握手前回 CONNECTION_REFUSED，超预算的洪水不占用 task / stream / 加密状态。已认证连接不占预算（改由 registry 的 slot table 约束）。按"每组 2-3 个 client + 重连突发"取值，不要按已认证连接数放大 | 默认值 `duotunnel-lib/src/config/file.rs:167-169`；启动校验 `:306-308`（必须 >= 1）。指标: `duotunnel_unauthenticated_connections_refused_total` (`duotunnel-server/runtime/metrics.rs:39-41`)。日志（1s 限频，`REFUSAL_LOG_INTERVAL`，`quic.rs:27`）: `refusing QUIC connection: unauthenticated connection budget exhausted` |
| **QUIC Retry (地址校验)** | `duotunnel-server/ingress/handlers/quic.rs:73-83` | 强制开启 | ❌ (硬编码) | 未通过地址校验的 Incoming 一律先 `retry()`，不消耗未认证预算；不可 retry 则 `ignore()`。防止伪造源地址的 Initial 包各占一个 permit 直到握手窗口结束，从而确定性地挤掉真实 client。代价是诚实 client 每次建连多一个 RTT | 日志: `unvalidated address cannot be retried; ignoring` |
| **MAX_LOGIN_BYTES** | `duotunnel-lib/src/models/msg.rs:31`；消费点 `duotunnel-server/ingress/handlers/quic.rs:206` (`recv_message_bounded(&mut recv, MAX_LOGIN_BYTES)`) | 64 KiB | ❌ (常量) | 认证**前**读取的 Login 帧的独立大小上限，远小于通用的 `MAX_MESSAGE_BYTES`（10 MiB，`msg.rs:25`）。真实 Login 只是一个 token 加两个整数，这个上限给 token 长度留余量，同时不让未认证对端指定大额分配。`recv_message_bounded` 内部还会再和 `MAX_MESSAGE_BYTES` 取 min (`msg.rs:254`) | — |
| **udp_recv_buf_mb** | `duotunnel-lib/src/transport/quic.rs`: `build_udp_socket` → `SO_RCVBUF` | 8 MiB | `quic.udp_recv_buf_mb` | Linux 内核将请求值翻倍（受 `net.core.rmem_max` 上限约束）。过小导致高 RPS 下 UDP 丢包（`recvmsg ENOBUFS`），是 8000 RPS 延迟尖刺主因之一 | `ss -udp -e` 看 `rmem`；`/proc/net/udp` 的 `drops` 列 |
| **udp_send_buf_mb** | `duotunnel-lib/src/transport/quic.rs`: `build_udp_socket` → `SO_SNDBUF` | 8 MiB | `quic.udp_send_buf_mb` | 发送侧内核队列；过小时 quinn GSO batch 被截断，单次 `sendmmsg` 提交的包数减少，CPU 消耗上升 | `ss -udp -e` 看 `wmem` |
| **UDP session/queue hard budgets** | server `udp_datagram.rs` | 每连接 session 1024；进程 session 16384；进程 queued envelope 16384；4×256 分片仅在首包后创建 | ❌ (常量) | 任一预算满立即 drop；queue/decode drop 计 counter，session cap 当前写 debug；DNS/bind/connect/send 单步 3s；shutdown 超时 abort worker | `duotunnel_udp_datagram_dropped_total`、`duotunnel_udp_tasks_active` |
| **Reverse stream admission** | server `handlers/quic.rs` + `tunnel_handler.rs` | 每连接 256；进程 4096；RoutingInfo 8 KiB / 5s | ❌ (常量) | permit 满立即 reset/stop；routing 首帧超大或超时关闭该 stream，避免认证连接堆积 stalled task/allocation | `duotunnel_reverse_stream_rejected_total`、`duotunnel_reverse_streams_active` |


---

## 2.5 过载保护 (Overload Protection)

当 QUIC 流槽位接近 `max_concurrent_streams` 上限时，两侧在 `open_bi()` 前通过 `maybe_slow_path` (`duotunnel-lib/src/lb/overload.rs:65`) 主动等待，避免 open_bi 堆积超时。在途计数存放在 per-connection 的 `InflightTable` 槽位里：client 侧由 `EntryConnPool` 分配 (`duotunnel-client/tunnel/conn_pool.rs:87`、`:117`)，server 侧由 `ClientRegistry` 分配并挂在 `SelectedConnection.handle` 上 (`duotunnel-server/ingress/registry.rs:26`)；两侧都通过 `ConnectionHandle::inflight_table()` / `slot_id()` 访问。

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **overload.mode** | `duotunnel-client/egress/listener.rs` `maybe_slow_path`; server ingress handlers 同名路径 | `inflight_slowpath` / `burst` | `overload.mode` | `burst` 直接 bypass 阻塞逻辑，任由 QUIC 层排队或 `open_bi` 超时 | 搜代码路径 `OverloadMode::Burst` |
| **overload.inflight_yield_threshold** | `lb/overload.rs:74` inflight 比较 | 800 (两侧一致) | `overload.inflight_yield_threshold` | 在途流 ≥ 该值（但 < sleep 阈值）时，`open_bi` 前等待一次 inflight 通知或 1ms 睡眠，先到者返回 (`lb/overload.rs:91-99`)；值过低会频繁挂起任务拉低吞吐 | 指标: `duotunnel_slowpath_waiting_tasks`；`inflight` / `max_concurrent_streams` 占比 |
| **overload.inflight_sleep_threshold** | `lb/overload.rs:92`、`:105-107` inflight 比较 | 950 (两侧一致) | `overload.inflight_sleep_threshold` | 在途流 ≥ 该值时进入 backoff 循环（具体由 `backoff_strategy` 决定） | 指标: p99 latency 与 inflight 曲线同步上抬 |
| **overload.inflight_sleep_ms** | `OverloadLimits.inflight_sleep_budget` (`lb/overload.rs:60`)，`maybe_slow_path` 的 deadline (`:103`) | 2ms (两侧一致) | `overload.inflight_sleep_ms` | backoff 循环的**总超时预算**；超过后放行到 `open_bi`（与 `backoff_strategy` 配合）。为 0 时等价于 `none` (`lb/overload.rs:100`) | tracing span 中 `maybe_slow_path` 前后时间差 |
| **overload.inflight_yield_pct** | `OverloadLimits::resolve` (`lb/overload.rs:44-46`)，**优先级高于**绝对阈值 | 0.80 (两侧一致) | `overload.inflight_yield_pct` | 相对 `max_concurrent_streams` 的比例阈值；设置后覆盖 `inflight_yield_threshold`。解析后若 yield > sleep 会被夹到 sleep 值 (`lb/overload.rs:50-52`) | client `duotunnel-client/bootstrap/config.rs:135`（default `:153`）/ server `duotunnel-lib/src/config/file.rs:85`（default `:102`） |
| **overload.inflight_sleep_pct** | 同上 (`lb/overload.rs:47-49`) | 0.95 (两侧一致) | `overload.inflight_sleep_pct` | 同上，覆盖 `inflight_sleep_threshold` | client `duotunnel-client/bootstrap/config.rs:138`（default `:154`）/ server `duotunnel-lib/src/config/file.rs:88`（default `:103`） |
| **overload.backoff_strategy** | `duotunnel-lib/src/lb/overload.rs:100-124` `maybe_slow_path` | `exponential` (默认) / `fixed` / `none` | `overload.backoff_strategy` | 三种都以 `inflight_sleep_ms` 为总 deadline，每轮先重查 inflight，掉到 sleep 阈值以下立刻返回，且每轮都可被 inflight 通知提前唤醒。`exponential`: 每轮等 `min(剩余预算, 10ms)` (`:116`)——名字是历史遗留，实际是"上限 10ms 的轮询"，不是指数放大；`fixed`: 一次等完剩余预算 (`:115`)；`none`: 不等，直接交给 QUIC 背压 (`:100-102`) | 指标: `duotunnel_slowpath_waiting_tasks` (`duotunnel-server/runtime/metrics.rs:22-27`) |
| **overload.emfile_backoff_ms** (仅 server) | `duotunnel-server/ingress/handlers/tcp.rs` 与 `duotunnel-server/ingress/handlers/http.rs` accept 循环 | 100ms | `overload.emfile_backoff_ms` | EMFILE 时暂停 accept 的时长；偏低会 CPU 打满，偏高丢连接 | 日志: `too many open files, backing off` |
| **overload.max_pending_streams** | `ConnectionHandle` 上的 `pending_semaphore`（`transport/connection_handle.rs:41-42`，容量取 `max_pending_streams.max(1)`），由 `open_bi_guarded` 经 `PendingAdmission` 消费（`transport/open_bi.rs:90-99`） | 未配置 → `max(1, max_concurrent_streams / 4)` (`lb/overload.rs:53`) | `overload.max_pending_streams` | **每条 QUIC 连接**的 pending 上限（不是进程全局）：只有 `open_bi()` 未能立刻就绪、需要等流控信用的流才占 pending 槽位。超限立即 `RejectedOverloaded`（`quic_open_rejected_overloaded`，错误信息带 `limit=`），client H1 入口返回 503 + `Retry-After: 1` + `X-DuoTunnel-Reject: overload` (`duotunnel-client/egress/listener.rs:212-234`)。因此实际可 pending 总量 ≈ 连接数 × 该值 | 指标: `stream_pending_queue_depth` 仍是**进程级**纯观测 gauge，不再参与准入决策（`transport/open_bi.rs:88-89`） |

---

## 3. 业务转发层 (Proxy / Data Plane)

| 参数 (Parameter) | 消费者 / 逻辑位置 (Consumer / Used by) | 默认值 / 其他值 | YAML 路径 | 阈值影响 (Impact) | 排查手段 (Debugging / Logs) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **relay_buf_size** | `engine/copy.rs` 的池化 `BytesMut`（`copy_buffered_then_finish` / `copy_buffered_then_shutdown`）。配置值生效于 server ingress 的 `forward_prefixed_to_client` 路径（`duotunnel-server/ingress/handlers/tcp.rs:143`、`handlers/http.rs:57`、`plugins/h1/mod.rs:108`、`plugins/tcp_pass/mod.rs:99`） | 64 KiB / 范围 >=4K（`normalize_relay_buf_size` 向上夹到 `MIN_RELAY_BUF_SIZE`） | `proxy_buffers.relay_buf_size` | **内存**: buffer 始终走 thread-local (每线程 ≤8) + 全局 `ArrayQueue` (1024) 两级有界池复用，不再是每流常驻两块；`copy_buffered_then_finish` 与 shutdown 路径使用同一实现，避免小包额外 `Bytes` 分配。⚠️ TLS relay 与 bridge 路径用的是硬编码 `DEFAULT_RELAY_BUF_SIZE` 64 KiB（`engine/relay.rs:26`、`:66`、`engine/bridge.rs:14-15`、`:29`、`:59`），不受配置影响；client 侧只校验不消费（见 §6） | `top/htop` 观察 RSS；默认值 `proxy/buffer_params.rs:1`、`:22` |
| **http_body_chunk** | ❌ **无消费者**（仅存在于 `ProxyBufferConfig`/`ProxyBufferParams`） | 8 KiB (`config/proxy_buffers.rs:20`) | `proxy_buffers.http_body_chunk_size` | 改这个值不生效：`Http1Driver` 的 body scratch 是硬编码 8 KiB（`protocol/driver/h1.rs:336`） | 见 §6 死配置清理 TODO |
| **max_idle_per_host** | `hyper::client::pool::Config` | 128 | `http_pool.max_idle_per_host` | 超过负载时，闲置连接被关闭，新请求需重新建连 (TCP Handshake) | 见 `duotunnel-server/egress/mod.rs` 的 pool 初始化 |
| **http_pool.idle_timeout_secs** | `HttpClientParams.pool_idle_timeout_secs` | None (yaml 示例: 90) | `http_pool.idle_timeout_secs` | 池内空闲连接最大存活时间；None = 不主动关闭 | `duotunnel-lib/src/config/http_pool.rs:13` |
| **http_pool.tcp_keepalive_secs** | `HttpClientParams.tcp_keepalive_secs` | 15s | `http_pool.tcp_keepalive_secs` | egress 池连接的 TCP_KEEPALIVE 间隔 | `duotunnel-lib/src/config/http_pool.rs:15` |
| **H2c→H1 pin TTL** | `duotunnel-lib/src/proxy/http_connector.rs:18` `PREFER_H1_TTL` → `prefer_h1` 缓存 | 300s (硬编码) | ❌ (常量) | cleartext H2c 请求失败后，该 upstream host 在 TTL 内强制走 H1 (`http_connector.rs:49-62`、`:190-191`) | 日志: `cleartext h2c request failed; retrying once with H1` |
| **H2c→H1 pin cache cap** | `HttpConnector.prefer_h1` | 1024 entries | ❌ (常量) | 配置 churn 下仍保持内存有界；满载淘汰最旧记录 | — |
| **Upstream passive-health cap** | `UpstreamHealthRegistry` | 4096 entries / eject 10s；active probe 128 | ❌ (常量) | 状态跨 generation 保留，但按 upstream group + backend 隔离；满载淘汰最早到期项；probe 使用 owner token 与 0–499ms 确定性 jitter | 日志: `active health probe...` |
| **DNS cache TTL** | `duotunnel-client/plugins/resolver_cached/mod.rs:9` `DNS_CACHE_TTL` / `:10` `MAX_CACHE_ENTRIES` | 30s / 上限 1024 条 (硬编码) | ❌ (常量) | client 侧 upstream DNS 解析缓存；IP literal 不走缓存 | — |

---

## 3.5 TCP Socket 选项 (TCP Plane)

所有经由 `TcpParams::apply` 的 socket 都会被设置。Server 应用于 ingress TCP；client 应用于 upstream TCP。

| 参数 (Parameter) | 消费者 / 逻辑位置 | 默认值 | YAML 路径 | 阈值影响 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **tcp.nodelay** | `TcpStream::set_nodelay` | true | `tcp.nodelay` | 关闭后小包会被 Nagle 合并，增加 40ms 延迟 | — |
| **tcp.recv_buf_size** | `setsockopt SO_RCVBUF` | 4 MiB | `tcp.recv_buf_size` | 高 BDP 链路下过小会压制吞吐 | `ss -ti` 看 rcv_space |
| **tcp.send_buf_size** | `setsockopt SO_SNDBUF` | 4 MiB | `tcp.send_buf_size` | 发送阻塞（POLLOUT 等待） | 同上 `snd_cwnd` |
| **tcp.keepalive** | `setsockopt SO_KEEPALIVE` | true | `tcp.keepalive` | 长连接对端崩溃后能被内核感知 | — |
| **tcp.user_timeout_ms** | `setsockopt TCP_USER_TIMEOUT` (Linux) | 30000ms | `tcp.user_timeout_ms` | 未确认的数据超过此时长则关闭连接；0 = 禁用 | 代码见 `transport/tcp_params.rs:36-44` |

---

## 3.6 TLS / PKI (Client 到 Server)

| 参数 (Parameter) | 消费者 / 逻辑位置 | 默认值 | YAML 路径 | 阈值影响 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **tls_skip_verify** | `duotunnel-client/tunnel/client.rs` | false (yaml 示例: true) | `tls_skip_verify` (client) | 跳过服务端证书校验，仅供开发 | — |
| **tls_ca_cert** | `duotunnel-client/tunnel/client.rs` | None | `tls_ca_cert` (client) | 自定义 CA 路径；为空时走系统根 | — |
| **tls_server_name** | `ClientConfigFile::tls_server_name()` | None → fallback 到 `server_addr` | `tls_server_name` (client) | SNI 覆盖；与证书 CN 不匹配时握手失败 | `duotunnel-client/bootstrap/config.rs:392` |
| **allow_insecure_fallback** | `duotunnel-client/tunnel/client.rs` | false | `allow_insecure_fallback` (client) | 系统无根证书时是否允许降级为 insecure；生产应保持 false | — |
| **pki.cert_cache_ttl_secs** | `PkiParams`, `init_cert_cache` | 3600s | `server.pki.cert_cache_ttl_secs` | 自生成证书缓存存活；过短会在高并发下反复签发消耗 CPU | `duotunnel-lib/src/infra/pki.rs:18` |

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
| **reconnect.login_timeout_ms** | Client 侧 login 握手超时 | 10000ms | `reconnect.login_timeout_ms` | 与 server `login_timeout_secs` (10s) 对齐 | 日志: `login timed out` |
| **reconnect.startup_jitter_ms** | 启动抖动窗口 | 300ms | `reconnect.startup_jitter_ms` | 集群冷启动时避免 thundering herd | — |
| **reconnect.open_stream_timeout_ms** | `open_bi()` 等待流槽超时 | 5000ms | `reconnect.open_stream_timeout_ms` | 见 §2 同项说明 | — |

---

## 3.8 Server 专属 (进程级基础配置)

| 参数 (Parameter) | 消费者 / 逻辑位置 | 默认值 | YAML 路径 | 阈值影响 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **tunnel_port** | `duotunnel-server/ingress/handlers/quic.rs` | 必填 | `server.tunnel_port` | QUIC 监听端口 | 由 `ServerConfigFile::load` 自动校验端口合法性 |
| **h2_single_authority** | `duotunnel-server/bootstrap/mod.rs` | true | `server.h2_single_authority` | H2 跨 vhost 共用 authority；关闭后每 host 独立池 | — |
| **TOKIO_WORKER_THREADS** | `duotunnel-lib/src/infra/runtime.rs` `effective_runtime_parallelism` | 未设置 → `available_parallelism()` | ❌ (环境变量) | Tokio worker 线程数；与 cgroup `CPUQuota` 取 min | 启动日志: `effective_parallelism` |

---

## 3.9 下游 H2 / H1 服务端硬编码上界 (Downstream Hardening)

不是配置项，但直接决定服务端对下游连接的抗滥用行为，所以列在这里。全部集中在 `duotunnel-lib/src/proxy/h2.rs:18-73`，通过两个共享构造器下发：

- `hardened_h2_server_builder` (`h2.rs:45`) —— server 侧三个 H2 入口复用：h2c 明文 (`duotunnel-server/ingress/plugins/h2c/mod.rs:383`)、TLS ALPN 协商到 h2 (`duotunnel-server/ingress/plugins/tls/mod.rs:249`)、`serve_h2_forward` (`h2.rs:140`)
- `hardened_h1_server_builder` (`h2.rs:62`) —— TLS 入口的 `http/1.1` 回退路径（ALPN 协商到 http/1.1 或完全没有 ALPN，`duotunnel-server/ingress/plugins/tls/mod.rs:256`）

设计意图是**显式钉住取值**而不是继承 hyper 默认：默认值会随版本漂移，而这些是安全属性，不能依赖上游默认。凡是"钉住当前 hyper/h2 默认值"的项只能下调、不能上调 —— 各调用点原先就在继承那个默认值，调大等于放宽而不是收紧。

| 参数 (Parameter) | 消费者 / 逻辑位置 | 值 | YAML 路径 | 阈值影响 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **H2_SERVER_MAX_CONCURRENT_STREAMS** | `hardened_h2_server_builder` → `max_concurrent_streams` (`h2.rs:48`) | 200 (`h2.rs:26`) | ❌ (常量) | 单条下游 H2 连接的并发流上限，stream-flood 预算。**钉住 hyper 1.x 当前 server 默认值，不得上调** | — |
| **H2_SERVER_MAX_HEADER_LIST_SIZE** | 同上 → `max_header_list_size` (`h2.rs:49`) | 16 KiB (`h2.rs:27`) | ❌ (常量) | header-flood 上限 | — |
| **H2_SERVER_MAX_PENDING_ACCEPT_RESET_STREAMS** | 同上 → `max_pending_accept_reset_streams` (`h2.rs:50`) | 20 (`h2.rs:30`) | ❌ (常量) | rapid-reset (CVE-2023-44487 类) 预算；钉住 hyper/h2 当前默认值，防止未来默认变更悄悄放宽 | — |
| **H2_SERVER_MAX_LOCAL_ERROR_RESET_STREAMS** | 同上 → `max_local_error_reset_streams` (`h2.rs:51`) | 1024 (`h2.rs:31`) | ❌ (常量) | 本端错误引发的 reset 预算；同样是钉住默认值 | — |
| **H2_SERVER_KEEP_ALIVE_INTERVAL** | 同上 → `keep_alive_interval` (`h2.rs:54`) | 30s (`h2.rs:32`) | ❌ (常量) | H2 PING 探活间隔；需要 `timer(TokioTimer)`，否则 hyper 运行时 panic (`h2.rs:52-53`) | — |
| **H2_SERVER_KEEP_ALIVE_TIMEOUT** | 同上 → `keep_alive_timeout` (`h2.rs:55`) | 20s (`h2.rs:33`) | ❌ (常量) | PING 无响应后判定连接失效的等待时长 | — |
| **H1_SERVER_MAX_HEADERS** | `hardened_h1_server_builder` → `max_headers` (`h2.rs:67`) | 100 (`h2.rs:38`) | ❌ (常量) | header 条数上限；代价是每请求一次 header slot 堆分配，只落在 legacy-client 回退路径上，可接受 | — |
| **H1_SERVER_MAX_BUF_SIZE** | 同上 → `max_buf_size` (`h2.rs:68`) | 64 KiB (`h2.rs:39`) | ❌ (常量) | 单连接读写缓冲上限 | — |
| **H1_SERVER_HEADER_READ_TIMEOUT** | 同上 → `header_read_timeout` (`h2.rs:71`) | 10s (`h2.rs:40`) | ❌ (常量) | slowloris 预算：header 必须在此时间内读完；同样需要 `timer(TokioTimer)` (`h2.rs:69-70`) | — |
| **PROTOCOL_DRAIN_TIMEOUT** | server H2c / TLS-H2 / TLS-H1 Hyper connection | 15s | ❌ (常量) | 先 graceful shutdown；超时 drop future 强制关闭，避免 keep-alive/body 永久拖住 listener drain | typed downstream drain error |

> 与 §2 `max_concurrent_streams` (QUIC，默认 1000) 无关：那是隧道内部的 QUIC 流上限，这里是下游 HTTP/2 连接的流上限。

---

## 3.10 停机排空预算 (Graceful Shutdown Budget)

全部是硬编码常量。层级约束：**内层的排空窗口必须严格小于外层**，否则外层兜底逻辑永远等不到内层结束就被自己的超时打断，日志会只剩一条 `forcing exit` 而看不到真实卡点。

| 常量 | 位置 | 值 | 作用域 | 层级关系 | 排查手段 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **LISTENER_DRAIN_TIMEOUT** | `duotunnel-server/ingress/listener_mgr.rs:82` | 10s | 单个 ingress listener 的所有 accept worker 上报 drained | 最内层 | 日志: `listener did not report drained in time; continuing shutdown` |
| **CONN_SHUTDOWN_DRAIN_TIMEOUT** | `duotunnel-server/ingress/handlers/quic.rs:22` | 15s | 单条 QUIC 连接内在途流排空（`wait_for_resource_drain`），之后才 `conn.close()` —— CONNECTION_CLOSE 会中断所有开着的流，所以 close 必须在 drain 之后 (`quic.rs:350-358`) | > listener，< app | 日志: `closing connection: server shutting down` 带 `drained` 字段 |
| **CONN_TASK_WAIT_TIMEOUT** | `duotunnel-server/ingress/handlers/quic.rs:24` | 20s | accept 循环退出后等所有连接 task 结束（`TaskTracker::wait`），超时才 `endpoint.close()` | > 连接排空窗口（留出仍在登录握手中的 handler 的余量） | 日志: `QUIC connection tasks did not finish in time; forcing endpoint close` |
| **SHUTDOWN_DRAIN_TIMEOUT** (server) | `duotunnel-server/runtime/app.rs:18` | 30s | 进程级兜底 drain (`wait_for_resource_drain`) | 最外层 | 日志: `shutdown drain completed` / `shutdown drain timed out; forcing exit` |
| **SHUTDOWN_DRAIN_TIMEOUT** (client, 连接级) | `duotunnel-client/tunnel/client.rs:24` | 15s | 单条 client 隧道连接排空后再 `conn.close()`（与 server 侧 `CONN_SHUTDOWN_DRAIN_TIMEOUT` 对称） | < client 进程级 | 日志: `closing tunnel connection: client shutting down` 带 `drained` |
| **SHUTDOWN_DRAIN_TIMEOUT** (client, 进程级) | `duotunnel-client/runtime/app.rs:16` | 30s | client 进程级兜底 drain | 最外层 | 日志同 server 侧 `wait_for_shutdown_drain` |
| **REFUSAL_LOG_INTERVAL** | `duotunnel-server/ingress/handlers/quic.rs:27` | 1s | 未认证连接被拒的日志限频（拒绝由对端驱动，不能让 accept 循环变成日志放大器；精确速率看指标） | — | 指标: `duotunnel_unauthenticated_connections_refused_total` |

---

## 4. 架构资源与性能深度特征 (Architecture & Resource Characteristics)

| 指标 / 瓶颈点 | 使用方 (Consumer / Logic) | 默认特征 / 复杂度 | 影响因子 (Factor) | 影响效果 (Effect) | 排查分析 (Analysis) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Routing Selection** | `ClientGroup::select_healthy` (`duotunnel-server/ingress/registry.rs:54`) | 分片 + P2C：`pick_from_preferred_shards` 先试首选 shard，再回落其他 shard；每个 shard 内 `pick_p2c_inflight_owned` 只比较两个随机候选（`lb/shard.rs:19`、`:35`） | shard 数 (`quic.shards`) | 单次选路是 O(1) 期望复杂度，不随分组内 Client 连接数线性增长；负载依据是 `InflightTable` 里的在途计数 | CPU Profile 见 `pick_p2c_inflight_owned`；`duotunnel-server/ingress/registry.rs:54-70` |
| **RCU Rebuild Cost** | `build_snapshot` + 每 shard 一次 `ArcSwap::store` (`duotunnel-server/ingress/registry.rs:109`、`:168`、`:215`、`:247`) | Vec Clone + Swap，按 shard 粒度 | 注册/注销频率 (Churn) | **转发抖动**: 频繁重连导致 forward 路径短时间停顿/分配大量小对象；分片把重建成本限制在受影响的 shard 上 | 日志: `registering/unregistering client` 频率 |
| **Initial Data Copy** | `ProxyEngine::run_stream` (`proxy/core.rs:84`) | 零拷贝：`SniffPrefix::into_bytes()` 对池化前缀走 `Bytes::from_owner`（`protocol/sniff.rs:128-136`），不再 `copy_from_slice` | 请求到达频率 | 嗅探前缀交接不再产生每请求一次的额外拷贝/分配 | 代码见 `proxy/core.rs:84`, `protocol/sniff.rs:128` |
| **Server Overload** | server ingress handlers + 对称的 `duotunnel-client/egress/listener.rs` | notify/sleep 等待逻辑 (`lb/overload.rs:65`) | 该连接槽位的 in-flight 数 vs. 阈值 `overload.inflight_{yield,sleep}_threshold` (§2.5) | **延迟突增**: 触发过载保护时系统主动挂起请求 | 指标: `duotunnel_slowpath_waiting_tasks`; `stream_pending_queue_depth` |
| **Logging Latency** | `engine/relay.rs:29`: `debug!` / `tracing` | 同步阻塞写入 | `log_level: info` | **吞吐硬封顶**: 同步日志写磁盘导致 Worker 线程挂起，QUIC 会发生 Idle Timeout | 火焰图见 `std::io::Write` 阻塞热点 |

---

## 5. Pingora 参数系统对照 (Design Reference)

对照 Cloudflare [pingora](https://github.com/cloudflare/pingora) (`pingora-core/src/duotunnel-server/configuration/mod.rs` `ServerConf`, `connectors/mod.rs` `ConnectorOptions`, `upstreams/peer.rs` `PeerOptions`, `listeners/l4.rs` `TcpSocketOptions`) 的设计，总结 duotunnel 当前参数系统的差距。

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
| `graceful_shutdown_timeout_seconds`, `grace_period_seconds` | 部分（硬编码常量，见 §3.10） | 排空预算已有完整层级（listener → 连接 → 连接 task → 进程），但全是常量，没有对应 YAML 配置项 |
| `HttpServerOptions` 的 H2/H1 抗滥用上界 | ✅ 已实现（硬编码，见 §3.9） | `hardened_h2_server_builder` / `hardened_h1_server_builder` 把 rapid-reset、header-flood、slowloris 预算显式钉住，不继承 hyper 默认；尚未暴露成配置项 |
| `ServerConf.max_retries` (proxy 可重试错误上限) | ❌ | `reconnect.*` 只有退避，没有总重试次数上限 |
| `HttpServerOptions.keepalive_request_limit` | ❌ | H1 keep-alive 循环无限复用，长连接会累积碎片内存 |
| `PeerOptions.max_h2_streams` | QUIC 有（`max_concurrent_streams`），H2 egress 无 | egress 侧 H2 没有 per-upstream 流数限制 |
| `PeerOptions.h2_ping_interval` | QUIC `keepalive_secs` 有，H2 egress 无 | egress H2 长连接没有主动探活 |
| `PeerOptions.connection_timeout / read_timeout / write_timeout / idle_timeout` | 散落在 `reconnect.*` / `http_pool.*` | duotunnel 命名按"动作"（reconnect/login/open_stream）而非"语义"（read/write/idle），上游方向的 read/write timeout 完全缺失 |
| `TcpSocketOptions.so_reuseport` | ✅ 已实现 | `build_reuseport_listener` 已在 `listener.rs` 引入并应用于 ingress listener。对齐 pingora |
| `TcpSocketOptions.tcp_fastopen` / `dscp` / `ipv6_only` | ❌ | 没有 TFO、DSCP、IPv6 独占等 |
| `ServerConf.threads` / `listener_tasks_per_fd` / `work_stealing` | 仅 `accept_workers` | duotunnel 没有"每 service 独立 runtime"的概念 |
| `ServerConf.max_blocking_threads` / `blocking_threads_ttl_seconds` | ❌ | tokio 阻塞池未暴露 |
| `upstream_connect_offload_threadpools` | ❌ | 没有"连接建立 CPU 隔离"的概念 |
| `#[non_exhaustive]` + `Option<T>` 表示"未配置" | 部分已用 | duotunnel 存在 `u64=0` / `Option<u64>` 混用（如 `overload.inflight_sleep_ms: u64`、`tcp.user_timeout_ms: u32`，0 语义模糊） |
| `ConnectorOptions::from_server_conf` 派生 | `impl From<&XxxConfig> for XxxParams` | **duotunnel 这层甚至更干净**（trait 化 vs 手写），保留 |
| `validate(self) -> Result<Self>` 链式 | `validate(&self) -> Result<()>` 收集 errors vec | **duotunnel 更好**（一次报出所有错），保留 |
| CLI `Opt::merge_with_opt(&mut ServerConf)` | 仅 `DUOTUNNEL_CLIENT__` 环境变量覆盖少数字段 | 没有完整 CLI override 层 |
| `overload.inflight_{yield,sleep}_pct` 百分比覆盖 | ✅ 已有 | **duotunnel 独有的好设计**，pingora 没有 |

### 5.3 历史不一致项（已修复）

以下项曾在代码 / yaml / 文档间漂移，现已对齐：

- client `quic.max_concurrent_streams` default → **1000**
- `http_pool.max_idle_per_host` 模板示例 → **128**（与代码 default 一致）
- client `reconnect.login_timeout_ms` default → **10000ms**（与 server `login_timeout_secs` 10s 一致）
- 顶层 `http_entry_port` 废弃 → 使用 `entry.port`

---

## 6. Future Roadmap & Design TODOs

### TODO: Error Code Design and Propagation
目前请求失败（如 `open_bi` 超时、后端 EOF）仅在日志中记录，无法回传给 k6 主动识别错误码。
- **目标**: 设计跨隧道的 `ErrorCode`，将后端 502/504 等细节透传至入口处。

### TODO: Unified Parameter Configuration Design (v1)

对照 §5 pingora 设计，按三步改造：

**Step 1 — YAML schema 版本化 + 清理死配置**
- 顶层加 `version: 1` 字段（参考 `ServerConf.version`），便于未来做破坏性 migration
- 清理死配置：`proxy_buffers.http_header_buf_size` / `proxy_buffers.http_body_chunk_size` 两侧都无消费者（真实值硬编码在 `protocol/driver/h1.rs`），client 的 `proxy_buffers.relay_buf_size` 只被 `validate` 校验、无任何读取点。要么接上消费者，要么删字段——目前的状态是"改了不报错也不生效"，最容易误判
  - （`http_entry_port`、`max_connections`、`max_tcp_connections` 等历史字段已从配置结构体中移除）
- 统一所有 timeout 的单位后缀（`_ms` 或 `_secs`），目前混用：`connect_timeout_ms`、`login_timeout_secs`、`idle_timeout_secs`、`open_stream_timeout_ms`

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
两侧 duotunnel-server + duotunnel-client 都复用同一套，`reconnect.*` 只保留纯退避参数（`initial_delay_ms / max_delay_ms / grace_ms / startup_jitter_ms`）。

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
