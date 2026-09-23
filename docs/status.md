# DuoTunnel 项目总报告

> 报告日期：2026-09-05 · 代码基线：`2a20f590c5d1cdd9122ddad2f1f6fa854ecb012e`（`codex/review-optimization-design`，工作树干净）
> 方法：多视角（agent）独立审查 —— 架构 / 协议 / Server 数据面 / Client 运行时 / ctld 控制面 / 质量与测试 / CI 与基准 / 安全与风险 —— 汇总为一份总报告。
> 证据优先级：本次实际执行的构建、测试与代码阅读 > 仓库内最新评审文档（`docs/reviews/2026-09-05.md`）> 历史评审（`docs/reviews/2026-07-26/`、`docs/archive/`）> README 自述。
>
> **2026-09-23 更新**：同基线的全量代码 review 见 [`reviews/2026-09-23.md`](reviews/2026-09-23.md)，新增 P0（控制面 watch 无心跳致静态配置下 120s/300s 后拒绝新连接）与多项出站超时/入口限流问题，已登记到 [`todo.md`](todo.md)。本报告其余内容未随之改写。

---

## 0. 执行摘要

DuoTunnel 是一个用 Rust + QUIC（quinn）实现的双向隧道代理系统，定位对标 frp，但采用中心化控制面（ctld）+ 纯数据面 server + 瘦 client 的架构。项目的工程骨架明显高于同规模原型：控制面/数据面分离、快照式热更新、协议版本与能力协商、明确的配置来源优先级、可度量的过载与准入控制、以及成体系的 CI 压测。

**一句话结论**：架构与抽象已达生产级骨架（QUIC 原生多路复用 + 集中式路由是真实优势），但"信任根基（隧道身份）、跨代一致性（H2 缓存）、控制面暴露面、admin framing"这四类正确性/安全问题尚未闭合，**当前适合内网可信流量与压测/预生产，不宜直接承载生产不可信流量**。

| 维度 | 现状 | 判断 |
|---|---|---|
| 架构设计 | crate 边界清晰、单向依赖、一个进程一个组合根 | 良好（≈3.5/5） |
| 代码质量 | 本次 `cargo check` / `cargo test` / `clippy -D warnings` 全部通过；unsafe 仅 5 处且集中 | 良好（≈3/5） |
| 协议设计 | ALPN 代际隔离 + 版本/能力协商 + rkyv 定长校验 + 控制面 DTCP v3 | 良好 |
| 性能工程 | SO_REUSEPORT、BBR、流级准入、shard、inflight、P2C、GSO/GRO、PGO/OS 调优脚本 | 良好（≈3/5） |
| 稳定性/HA | 重连退避 + LKG 快照 + 组件 supervisor + 优雅停机；但部分 drain/actor 语义未闭合 | 中等（≈2.5–3/5） |
| 安全性 | token 只存 SHA-256、日志脱敏、QUIC Retry、未认证预算；隧道身份仍为临时自签、watch 可无认证暴露 | 偏弱（≈2.5/5） |
| 可观测性 | Prometheus + 健康快照 + dial9 trace + 资源 dashboard | 良好（≈3/5） |
| 测试 | 248 个测试全过；协议矩阵集成测试 + 3k/6k/8k 压测；热路径部分模块覆盖薄、无 fuzz | 中等（≈2.5–3/5） |
| 文档 | spec/ 体系完整、评审链可追溯；README 有局部漂移（如 jemalloc/mimalloc） | 良好（≈3/5） |

---

## 1. 项目定位

- **形态**：自建隧道/内网穿透代理（frp 同类者），不依赖反向连接池，Server 可直接 `open_bi()` 向 Client 开流。
- **双向性**：同一套引擎同时支持
  - **Ingress（反向代理）**：外部 → Server 入口 → QUIC → Client → 私网服务；
  - **Egress（正向代理）**：本地应用 → Client 入口（`entry.port` / `udp_entries[]`）→ QUIC → Server → 外部服务。
- **协议覆盖**：HTTP/1.1、HTTP/2（h2c）、HTTPS/TLS-SNI 透传或终止、WebSocket、gRPC、原始 TCP、UDP（QUIC Datagram 封装）。
- **控制模型**：`duotunnel-ctld` 持有全部路由与 token 生命周期（YAML 低优先级 + SQLite 高优先级合并），Server/Client 不读路由文件。

## 2. 代码规模与技术栈

| 组件 | 路径 | Rust LOC（含内联测试） |
|---|---|---|
| 共享库 | `duotunnel-lib/` | ≈13,400 |
| 数据面服务端 | `duotunnel-server/` | ≈8,600 |
| 控制守护进程 | `duotunnel-ctld/` | ≈5,800 |
| 隧道客户端 | `duotunnel-client/` | ≈3,800 |
| CI 辅助（echo/k6/工具） | `ci-helpers/` | ≈1,100 |
| **合计** | | **≈32,700** |

文档（`docs/` + README + BENCHMARK_SPEC）≈18,400 行 Markdown；Git 历史 503 个提交（2026-02-13 起）。

**技术栈**：Rust 1.95.0（`rust-toolchain.toml`）· tokio · quinn 0.11（ALPN `tunnel-quic/v1`）· rustls 0.23 + aws-lc-rs · hyper 1.9 · rkyv 0.8（零拷贝序列化）· sqlx/SQLite · arc-swap · dashmap · metrics + Prometheus exporter · mimalloc（Server/Client）· dial9-tokio-telemetry（可选 feature）· criterion（bench）。

> 注意：README 的优化表中写的是 "jemalloc global allocator"，实际 `duotunnel-server/main.rs` / `duotunnel-client/main.rs` 用的是 `mimalloc::MiMalloc`（`mimalloc = "0.1"`）。属文档漂移，非功能问题。

发布 profile（`Cargo.toml` + `.cargo/config.toml`）：`lto = "fat"`、`codegen-units = 1`、`panic = "abort"`、`strip = debuginfo`、`-C target-cpu=native`（CI 用 `CARGO_ENCODED_RUSTFLAGS` 覆盖为可移植）。

## 3. 架构总览

### 3.1 Crate 拓扑

```
                    ┌─────────────────┐
                    │  duotunnel-ctld │  控制面：路由 + token + watch + SQLite
                    └────────┬────────┘
                             │ TCP watch (:7788, rkyv/DTCP v3)
              ┌──────────────┴──────────────┐
              ▼                             ▼
      ┌────────────────┐            ┌─────────────────┐
      │ duotunnel-server│            │ duotunnel-client│
      └────────────────┘            └─────────────────┘
              └───────────┬────────────────┘
                          ▼
                   ┌──────────────┐
                   │duotunnel-lib │  协议/传输/代理/插件/LB/infra
                   └──────────────┘
```

依赖规则：Server/Client → lib；只有 ctld 拥有存储层（`storage/` 私有模块）；lib 不依赖任何二进制 crate。

### 3.2 进程内分层（三端一致）

`bootstrap/`（CLI + 配置装配，组合根）→ `runtime/`（启动编排、supervisor/engine、健康与指标）→ 业务运行时（`ingress/`、`egress/`、`control/`、`tunnel/`）→ `duotunnel-lib` 能力对象。

Server 侧 `ServerState` 是唯一能力面，内部三块：

- **IngressRuntime**：`ArcSwap<RuntimeGeneration>`、`ListenerManager`、`PluginRegistry`、peek buffer pool、上游健康表；
- **ConnectionRuntime**：分片 `ClientRegistry`（actor + 快照选路）；
- **ControlRuntime**：ctld watch 客户端、token 映射、revision/revocation 状态、LKG 快照。

Client 侧 `ClientApp` 向 `RuntimeEngine` 注册 `ClientService`：`TunnelPoolService`（必选）、`EgressListenerService`（`entry.port`）、`UdpEgressListenerService`（`udp_entries[]`）、健康/指标服务。

### 3.3 关键设计决策

1. **快照替换而非原地修改**：路由配置以 `ArcSwap<RuntimeGeneration>` 整体发布；请求路径 pin 住自己那一代，热更新不产生字段级可见性。
2. **代际隔离在握手层**：ALPN 携带 wire generation（`tunnel-quic/v1`），破坏性布局变更直接让 QUIC 握手失败，而不是等 rkyv 校验出错。
3. **行为开关走能力位**：`Login`/`LoginResp` 交换 `protocol_version` + `capabilities`（当前 `CAP_NONE`），只追加字段、必须能力门控。
4. **错误语义与文本解耦**：`LoginResp.retryable` 是机器可读字段，Client 不得解析错误文本（避免把后端瞬时故障误判为 token 拒绝）。
5. **SSOT 收敛**：ctld 合并 YAML（默认层）与 SQLite（覆盖层 + tombstone），materialized 结果不视为 override。

## 4. 协议与接口契约

### 4.1 数据面（Client ↔ Server）

- 帧格式：`[MessageType u8][len u32 BE][rkyv payload]`；类型：`Login=0x01`、`LoginResp=0x02`、`RoutingInfo=0x10`、`Ping/Pong`、`ConfigPush=0x06`。
- 上限分层：通用 `MAX_MESSAGE_BYTES = 10 MiB`；未认证阶段 `MAX_LOGIN_BYTES = 64 KiB`；`RoutingInfo` 8 KiB；UDP datagram 1200 B。
- `RoutingInfo` 携带 `proxy_name / src_addr / src_port / protocol / host`，QUIC 流首帧即路由信息。
- 版本策略：`PROTOCOL_VERSION = 2`、`MIN_SUPPORTED_VERSION = 2`；新客户端向后兼容（`min(ours, theirs)`），旧客户端显式拒绝。

### 4.2 控制面（ctld ↔ Server，DTCP v3）

- 信封：magic `"DTCP"` + `u16` wire version + `kind u8` + `u32` payload 长度；负载为 rkyv。
- 消息：`WatchRequest`（带 token、last revision/hash）→ `ConfigEvent::Snapshot | Delta` → `ApplyResponse`（`Applied / Duplicate / Rejected / ResyncRequired`）。
- `ConfigSnapshot` 内容：ingress listeners、client groups、egress upstreams、egress vhost rules、token cache；`ConfigDelta` 以 `base/target revision + hash` 做 CAS 式应用。
- **内容哈希**：对规范化（排序）快照做 SHA-256，前缀 `duotunnel-control-protocol-v3\0`，用于幂等与防篡改检查。
- **语义校验**：`validate_config_snapshot` 检查重复端口/组名/upstream/host/token hash、悬空引用、空地址；apply 失败必须返回 stable reason。

## 5. 数据面调用链

### 5.1 Ingress（Server → Client → 私网）

```
外部 TCP → SO_REUSEPORT accept worker（HTTP/TCP/UDP 各自循环）
 → IngressDispatcher 六阶段：
     1) sniff（≤4096B / sniff_timeout，防 Slowloris）
     2) ConnectionModule::pre_admission（按 order 排序）
     3) TunnelService::admission（准入，可拒绝）
     4) RouteResolver（VhostPlugin）→ Route{group_id, proxy_name}
     5) IngressProtocolHandler（h1 / h2c / tls / tcp_pass）
     6) logging（panic 隔离，不让插件炸掉 accept worker）
 → ClientRegistry::select_healthy（轮转起始 shard + 全局最低 inflight + 平局轮转）
 → open_bi_guarded（快路径 now_or_never；慢路径 per-connection pending semaphore + 超时）
 → 写 RoutingInfo 首帧 → QUIC bidi stream
 → Client: accept_bi 循环 → recv_routing_info → ProxyEngine → LocalProxyMap → 本地服务
```

### 5.2 Egress（Client → Server → 外部）

```
本地应用 → Client entry listener（TCP/UDP，可选）
 → 协议嗅探 / Host 提取 → 对照 LoginResp 下发的 egress_rules 白名单（大小写/端口规范化 + 通配匹配）
 → 选择池中 QUIC 连接（shard 快照 + 排除重试）→ 首帧 RoutingInfo
 → Server: tunnel_handler → ProxyEngine(ServerEgressMap)
 → 上游组（round-robin / inflight / P2C 策略）+ DNS 缓存 → 外部服务
```

### 5.3 隧道管理

Server 的 `handlers/quic.rs` 负责：QUIC Retry（地址校验）→ 未认证预算（默认 64，超限直接 `refuse()`）→ 单一 pre-auth deadline（默认 10s，覆盖握手/accept_bi/读 Login/鉴权）→ token 鉴权 → 注册 `ClientRegistry` → 进入 `accept_bi` 循环接收 Server 发起的反向流。

## 6. 控制面与热更新

- **配置源**：`YamlConfigSource`（watch 文件）+ `SqliteConfigSource`（轮询 source_revision，只读观察者），各自暴露 `subscribe()` 与 `subscribe_degraded()`。
- **合并**：资源键级 merge（不是字段级深合并）；SQLite upsert 覆盖 YAML，tombstone 隐藏，清除 override 后恢复 YAML 值；token 始终 SQLite 所有。
- **事务边界**：admin 变更（token 轮换、配置 override）在单个 SQLite 事务内同时写 override/effective state/revision；内存快照与 watch 通知只在 commit 后更新。
- **幂等**：`admin_idempotency` 表存 request key/fingerprint/status/response（token 明文不落库，只有 redacted marker + 有界内存缓存）；30 天保留。
- **Server 侧应用**：`ControlClientService` 维护 watch 循环，处理 Snapshot/Delta/Resync/Duplicate；应用前做 hash 与语义预检；应用失败不污染旧状态；同步维护 LKG（Last Known Good，16 MiB、格式版本 3、控制协议版本绑定、过期与未来时钟偏移检查）以便断连/重启恢复。
- **运行时发布**：listener 由 `ListenerManager` 按 port/kind 做代际化 prepare/commit/rollback/drain（bind 失败不提交任何变更）；路由由 `ArcSwap` 原子替换；`RuntimeGeneration` 带 `epoch + sequence`。

## 7. 并发与性能工程

| 机制 | 位置 | 作用 |
|---|---|---|
| SO_REUSEPORT + backlog 4096 | `transport/listener.rs`、`listener_mgr.rs` | 多核 accept 扩展，每 worker 独立绑定 |
| BBR（可配 cubic/new_reno） | `transport/quic.rs` | 高带宽波动链路吞吐 |
| 流控窗口 4/32/8 MiB | `transport/quic.rs` | 高 BDP 链路不 stall |
| per-connection stream semaphore + 全局 pending 指标 | `transport/open_bi.rs` | 慢路径准入，区分 rejected/timed out/connection lost |
| sharded ClientRegistry / EntryConnPool | `server/ingress/registry.rs`、`client/tunnel/conn_pool.rs` | actor 变更 + 无锁快照读 |
| inflight / P2C / shard 选择 | `lib/lb/*` | 动态负载均衡，避免固定 shard 独占 |
| BytesMut 缓冲池（thread-local + 全局 ArrayQueue） | `engine/copy.rs` | relay 零拷贝、无未初始化内存 |
| Peek buffer pool | `lib/infra/peek_buf.rs` | 协议嗅探复用固定缓冲 |
| GSO/GRO、UDP 收发缓冲 8 MiB | `transport/quic.rs` | 批量 UDP I/O（Linux ≥5.4） |
| 运行拓扑推导 | `lib/infra/runtime.rs` | `effective_runtime_parallelism()` 受 cgroup CPUQuota 约束，派生 worker/accept/shard/connection 数 |
| PGO / OS 调优脚本 | `scripts/pgo-build.sh`、`scripts/tune-os.sh` | +10–20% 吞吐；rmem/somaxconn/THP/governor/NIC ring |
| panic = abort + mimalloc | `Cargo.toml`、main.rs | 小二进制、低碎片 |

**度量与压测**：k6（基础 ramp、body_size、3k/6k/8k 固定速率、多 host 50 域、含 frp 基线）+ `bench-tool.py`（psutil 1s 采样，CPU/内存/网络/上下文切换/TCP/UDP 错误等）+ 静态 dashboard（gh-pages），CI 每次 main 推送自动发布。

## 8. 可靠性、优雅停机与可观测性

- **Server supervisor**：组件有 `Starting/Running/Stopping/Stopped/Failed` 状态机，限制重启预算（2 次 + 250ms 退避），进程 owner 统一取消。
- **Client supervisor**：每连接一个 supervisor，`JitterBackoff`（初值 1s、上限 60s），只有 `FailureClass::Fatal` 终止循环；短会话保持退避、稳定会话与完成业务后重置。
- **Client 健康**：`min_ready_tunnels`（默认 1）驱动 `/healthz`；低于 desired 记 degraded；池 actor 存活与 entry listener 就绪参与判定。
- **优雅停机**：共享 `CancellationToken`，Client 先退池再 drain（30s 应用级 backstop / 15s 会话 drain / QUIC 20s+15s），Server 按 generation 关闭 listener 并等待连接任务。
- **指标**：Prometheus exporter（`metrics`），连接/认证/请求/open_bi 等待/overload/UDP 丢弃/LKG 降级/未认证拒绝等；client 追加 active/desired/degraded/ready/pool_actor_alive gauge。
- **追踪**：`dial9-tokio-telemetry` feature（8k 压测链路），支持 trace 解析与 viewer。

## 9. 测试、CI 与基准

**本地验证（本次实际执行）**：

| 命令 | 结果 |
|---|---|
| `cargo check --workspace --locked` | 通过（37.9s，全 5 个成员） |
| `cargo test --workspace --locked` | **248 通过 / 0 失败**（lib 164、server 35、client 27、ctld 22） |
| `cargo clippy --workspace --locked --all-targets -- -D warnings` | 通过（0 warning） |

**CI（`.github/workflows/ci.yml`）**：

- `unit-tests`：`cargo llvm-cov --workspace` + clippy `-D warnings` + `cargo udeps` + `cargo audit`（依赖安装均有 action 固定/版本管理）；
- `build` / `build-dial9`：release 与带符号/telemetry 两种产物，artifact 共享给后续 job；
- `integration-test`：`ci-test-client` 协议矩阵（内网 HTTP/H2c/WS/gRPC、外部 beeceptor/websocket.org/grpc-echo、双向并发、10 次稳定性抽样）；
- `bench-basic / bench-3k / bench-6k / bench-8k`：k6 固定速率 + 资源采样 + 可选 frp 对照 + 8k dial9 trace；
- `publish-bench`：合并 artifact 发布 gh-pages。

**手动入口**：`workflow_dispatch` 可开关各 job，调 `worker_threads`（并行度锚点，`0`=auto）、`stress_core_target_rate`、`stress_cpu_quota`（默认 100% ≈ 1 CPU）、CPU 隔离模式等。GPU 无关。

**本地集成**：`ci-helpers/local-test/examples/simple-tunnel/test.sh` 一键起 ctld+server+client 并验证双向；`ci-helpers/local-test/test.sh` 为拓扑脚本。

## 10. 安全态势

**已具备**：

- Token 只以 SHA-256 摘要形式存在于内存/SQLite；`AuthError` 与 `Login` 的 `Debug` 均做 token 脱敏（`dt_masked_<8hex>`）。
- 未认证 QUIC 连接并发预算（默认 64）+ 强制 Retry 地址校验，超限 `refuse()` 不消耗 task/stream/crypto 状态；拒绝日志 1s 限频。
- 认证前使用单一 deadline（10s）而非逐步超时，封堵"每步卡满"的慢速占用。
- 入站嗅探超时防 Slowloris；UDP datagram 独立 1200B 上限；rkyv `CheckBytes` 反序列化校验。
- CA 文件读取带 owner/权限/symlink 检查（`pki.rs`）；egress 白名单双层防御（Client 池索引 + Server egress map）。
- watch 支持可选 `watch_token`（`--ctld-token` / `DUOTUNNEL_CTLD_TOKEN`），admin socket 为 0600 Unix socket + 有界 framing。

**未闭合（最新评审 `docs/reviews/2026-09-05.md` 确认，当前分支仍是待实施状态）**：

| 编号 | 问题 | 影响 |
|---|---|---|
| F1 | H2c route/sender cache 键只有 `sequence`，缺 `epoch` | 换库/重建控制面后，同序号的旧缓存可能复用错误代路由 |
| F2 | 隧道 QUIC 身份 = 每次启动生成的临时自签证书，server 无固定 cert/key 加载入口 | Client 无法稳定验证 Server 身份，"生产用真实证书"缺可执行步骤 |
| F3 | 持久 CA 损坏会被静默重新生成并覆盖文件 | 信任不连续、部分写入可能留下不配对文件 |
| F4 | README 快速开始使用 `watch_addr: 0.0.0.0:7788` 且无 token | 非回环部署下 watch 快照可被任意访问（token cache 含组与状态） |
| F5 | h2c handler 用 `split(':')` 解析 authority，`[::1]:8080` 被截断 | IPv6 主机 vhost 路由失败 |
| F6 | H2 单流错误触发内外两层 sender 缓存无差别清退 | 迟到错误可能移除新 sender；GOAWAY 被当成连接级致命错误 |
| F7 | `panic = "abort"` 与 supervisor `catch_unwind` 恢复承诺矛盾 | release 下 panic 直接结束进程，进程内恢复语义不成立 |
| F8 | admin framing 手写解析：重复 CL、缺失长度、`header_end + expected_len` 未 checked | 极端长度 debug 溢出/release 回绕（影响面受 Unix socket 0600 约束） |

另：热路径核心（copy/open_bi/sniff/H1 driver 等）测试较薄，无 fuzz/loom/miri；README 与实现存在局部漂移（allocator 名称等）。

**历史 P0 已修复（本次代码核对）**：relay buffer 未初始化 UB（`engine/copy.rs` 现用 `BytesMut` + `read_buf`，无 `set_len`）；H1 对 204/304/HEAD 的 RFC 9112 §6.1 framing（`protocol/driver/h1.rs` 已按语义分流并带测试）；`open_bi` 慢路径改为 per-connection semaphore 准入（`transport/open_bi.rs`）。

## 11. 当前待办与优先级（摘自 `docs/todo.md` S1–S9）

| 顺序 | 方向 | TODO |
|---|---|---|
| S1 | CA 安全加载/显式初始化 + 独立稳定的 tunnel 身份 | 176、166、152 |
| S2 | 请求固定 generation，所有相关缓存使用 `epoch + sequence` | 175、52 |
| S3 | 统一 authority 契约（修 IPv6）；H1 超头数明确 431 | 177、158、167 |
| S4 | H2 错误作用域、两层实例身份、重试安全统一设计 | 168、68 |
| S5 | admin 协议适配与业务 mutation 分离，严格有界 framing | 163 |
| S6 | watch 安全监听（回环默认/非回环强制认证）+ 显式 authority reset | 171、178 |
| S7 | actor 失败传递到进程 owner；结构化阶段 outcome | 154、165、170 |
| S8 | TCP relay 核心收敛；容量按真实生命周期管理 | 169、142、153 |
| S9 | 有测量证据的热点优化；带租约的关联视图与多目标 HA | 140、156–159、172 |

## 12. 成熟度评估

参考 `docs/archive/review-2026-07-26/05-maturity-assessment.md` 的 9 维评分口径（1=原型 … 5=业界标杆），结合本次代码基线更新：

- **架构 3.5 / 代码 3.0 / 抽象 3.5 / 性能 3.0 / 稳定性 2.5→3.0 / 安全 2.5 / 可观测 3.0 / 测试 2.5→3.0 / 文档 3.0**。
- 定位：**"工程化良好的准生产系统"**。QUIC 架构红利真实存在（新流 ~0 RTT、无连接池、原生迁移），但信任根基、跨代一致性和故障语义是阻止上生产的关键。
- 与 pingora 的差距集中在"多核模型、统一 Session 抽象、生命周期管理、测试/模糊测试"，均为工程实现问题，路径清晰、可收敛（无需 io_uring/thread-per-core 重写）。
- 与 frp 的差距在生态与长期生产验证，属时间/采用度问题，不是设计问题。

## 13. 建议行动（按依赖排序）

1. **先修信任与正确性**（S1–S4）：隧道身份文件加载 + CA 不覆盖；generation 贯穿 H2 缓存；authority 解析统一；H2 失败域建模。每一项都带可复现验收（跨 epoch 同序号、IPv6、损坏 CA、迟到 RST 等）。
2. **收敛控制面边界**（S5–S6）：admin 走受限 HTTP 解析器或成熟库；watch 默认回环、非回环强制认证并设计显式 epoch reset 许可。
3. **修正故障模型**（S7）：在 `panic=abort` 前提下把"进程内恢复"改为"fail-closed + 外部重启"，并把 actor 退出上升为进程级事件；错误 outcome 结构化。
4. **再做性能**（S8–S9）：先合并 TCP↔QUIC relay 核心与容量所有权，再基于同口径基准做微优化；HA 采用"稳定身份 + 带租约在线视图"，不进入每请求依赖。
5. **工程卫生**：把 README 与实现对齐（allocator、路径）；为热路径补 fuzz/属性测试；CI 已有 clippy/audit/udeps/覆盖率，可加 benchmark gate 防止性能回归。

## 14. 附：证据与复现

- 构建/测试/clippy 命令与结果见 §9；测试分布：`duotunnel-lib` 164、`duotunnel-server` 35、`duotunnel-client` 27、`duotunnel-ctld` 22。
- 代码事实核对点：`duotunnel-lib/src/models/msg.rs`（协议常量/协商）、`duotunnel-lib/src/protocol/ctld.rs`（DTCP v3/哈希/校验）、`duotunnel-lib/src/transport/open_bi.rs`（准入）、`duotunnel-lib/src/engine/copy.rs`（缓冲池）、`duotunnel-server/control/control_client.rs`（watch/LKG）、`duotunnel-server/ingress/registry.rs`（选路）、`duotunnel-ctld/src/control/{service,layer,watch}.rs`（合并/事务/发布）。
- 评审链：`docs/reviews/2026-09-05.md`（F1–F8 证据与复现）、`docs/design/optimization-design-2026-09-05.md`（逐问题推荐设计）、`docs/reviews/2026-07-26/`（专题评审）、`docs/reviews/2026-09-23.md`（最新全量代码 review）、`docs/spec/`（架构与分层规格）。
