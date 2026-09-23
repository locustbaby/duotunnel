# Pingora 风格重构史（2026-04 ~ 2026-05，历史存档）

> **时间范围**：2026-04-24 ~ 2026-05-29（同一条重构主线的三个阶段）。
> **合并来源**（原文件已删除，git 历史可查）：
> `archive/CODE_REVIEW_done.md`（代码质量 review 发现）、
> `archive/2026-05-22-review.md`（对标 Pingora 的综合优化方案）、
> `archive/pingora-tasks.md`（任务化拆解，TODO-63~73，主线文档）。
> 三份文档是同一条 "对照 Pingora 做架构重构" 主线的不同阶段：CODE_REVIEW 先发现问题 →
> 05-22 综合方案给出对标设计 → pingora-tasks 把方案拆成可执行 TODO 并持续跟踪到完成。
> 本文件以 pingora-tasks 的任务清单为主体，状态已对照 [`../todo.md`](../todo.md) /
> [`../done.md`](../done.md) 重新核实；背景动机与设计理由从另外两份文档中提炼，
> 与 done.md 已有记录重复的叙述不再保留。**当前实施状态以 todo.md / done.md 为准**。

---

## §1 背景：为什么要对标 Pingora

### 1.1 代码质量审计发现的核心问题（CODE_REVIEW_done.md）

审计范围 `tunnel-lib` / `server` / `client` 全部 `.rs`，核心方法论：先梳理业务流转，再看代码抽象是否匹配物理世界逻辑。发现的关键问题：

- **`RoutingInfo` 语义错位**：把"连接/路由级"语义（该发给哪个 `proxy_name`）和"请求级"语义（`src_addr`/`host`）混在同一个结构体里，且在 QUIC stream 刚建立时一次性发送。这直接挡住了 H2 多路复用——一条 QUIC stream 上跑几十万个 HTTP 请求时，请求级字段无处安放。正确做法是把 `src_addr`/`host` 剥离到 L7 Header（`X-Forwarded-For` 等），`RoutingInfo` 只保留连接级的 `proxy_name`/`protocol`。
- **Server 控制面职责越位**：Server 必须知道 Client 侧定义的 `proxy_name`，导致新增本地服务要双端改路由表；对于自带寻址语义的 L7 流量，`host` 已经足够寻址，`proxy_name` 是冗余指令。理想方向是 L7 流量交给 Client 做"边缘路由"（host → local backend），L4 流量才需要 `proxy_name`。
- **`ProxyApp` 命名误导**：听起来像业务应用生命周期框架，实际只是按 `RoutingInfo`/`host` 路由返回 `PeerKind` 的路由匹配器（本质应叫 `UpstreamResolver`/`RouteMatcher`）。
- **`PeerKind` 分派不一致**：`Tcp`/`Http`/`H2` 走裸 `connect_inner`，只有 `Dyn(Box<dyn UpstreamPeer>)` 走 trait——而 `Dyn` 其实是 client 侧 `MitmH2Peer` 借道的"半活后门"，不是死代码。这是 §2 TODO-63 的直接起因。
- **`anyhow` 全局黑盒错误**：无法区分 upstream/downstream/internal、无法区分可重试/不可重试。最典型的真实 bug：H1/H2 长连接被服务端 idle 超时关闭是正常现象，但 `HttpPeer::connect_inner` 把错误一股脑上抛，用户看到偶发 502。这是 §2 TODO-65 的直接起因。
- **H2c per-route sticky sender cache** 当时已存在但只保存裸 `quinn::Connection`，无法携带 `inflight`/`conn_id`/错误分类；cache hit 时不检查 `close_reason()`，client 断开要等 `forward_h2_request` 失败才发现。这是 §2 TODO-69 的直接起因。
- 已完成项（合入 `done.md`，此处不再复述细节）：`recv_typed_message<T>` 封装（CR-NEW-A）、`RouteTarget` 类型化替代匿名元组（CR-NEW-B）、`lib.rs` API 导出梳理（CR-NEW-D）、`PeekBufPool` 共享工具（CR-NEW-E）、`TcpPeer`/`TlsTcpPeer` 合并（CR-NEW-C）、`Protocol` 枚举化（CR1）、relay 层归一化（CR2）、URL 解析归一化（CR3）。
- **仍开放**：`CR4`（观测性与业务热路径解耦）、`CR5`（配置从 pull 快照演进为 `Stream<Item = RoutingSnapshot>`）——现追踪为 `TODO-CR4`/`TODO-CR5`，均为 Priority Low / Status TODO，详见 todo.md。

### 1.2 对标 Pingora 的综合方案要点（2026-05-22-review.md）

- **Workspace 架构问题**：`server` 直接依赖 `tunnel-store` 把 SQLite 编译进网关可执行文件，阻碍边缘节点水平扩展；`tunnel-lib` 混杂了嗅探缓冲、HTTP 连接池、QUIC wire 协议、metrics 于一体。提出的方向——`server` 通过无状态 `ControlPlaneClient` 与控制面通信、`tunnel-lib` 拆分为 `tunnel-proto`/`tunnel-engine`/`tunnel-plugins` 子 crate——**该 workspace 级拆分未被采纳为独立执行项**：`duotunnel-ctld → duotunnel-server → duotunnel-client` 的常驻拓扑已经实现了"边缘无状态"的核心诉求（见 done.md TODO-82：server 不再编译 SQLite 驱动，只消费 ctld 下发快照）；`tunnel-lib` 拆子 crate 仍在 todo.md 作为独立项保留（TODO-83，Priority Medium，Status TODO），未与本主线合并推进。
- **`server/listener_mgr.rs` 事件驱动重构建议**：当时描述为"同步阻塞的 `sync_listeners`，用单个 `parking_lot::Mutex` 全量 reconcile，大批量 reload 时顺序 spawn 数百个 task 可能阻塞调度线程"，建议改为 `watch` channel 驱动的异步 reconciler。**该描述已过时**：todo.md TODO-85 复核记录明确"原同步阻塞描述过时，当前已有异步 listener 管理"。现状见 §4（及 TASK 3 的独立验证：generation/reservation 化的 `plan_sync_with` 已经是异步、原子化的 reconciler，早已不是本节描述的阻塞模型）。
- **其余零散优化点**（`TcpPassHandler`/`H1Handler` 冗余堆分配、Peek buffer 拷贝、O(N)→O(1) 负载均衡扫描、DNS 解析路径、Serialized upstream dialing、client 模块化重命名、FD/Stream 限制可观测性、debounce publish 与 DB 轮询反应器、单趟 preface 解析）**已在文档中标记 [Completed] 或与后续 §1 系列文档的结论重复**（O(N)→O(1) 负载均衡即 P2C，见 TASK1 历史文档 §1.5；DNS/FD 可观测性等已被 review-2026-07-26 系列与 todo.md 吸收），此处不再重复陈述。

---

## §2 主线任务清单（TODO-63 ~ TODO-73）

> 推荐落地顺序（原文档记录）：66 → 69 → 72 → 64 → 67b → 68 → 71。除标注"开放"外均已完成。

| TODO | 标题 | 状态（已核实） |
|---|---|---|
| [TODO-63](#todo-63) | Peer 描述符化，消灭 `PeerKind + Dyn` 双轨 | ✅ Done |
| [TODO-64](#todo-64) | `ClientId`/`GroupId`/`ProxyName`/`ReuseHash` newtype 收尾 | ⏳ **开放**（TODO） |
| [TODO-65](#todo-65) | 热路径结构化错误，替换 `anyhow` 黑盒 | ✅ Done |
| [TODO-66](#todo-66) | 统一 `HttpConnector` + H1/H2 降级记忆 | ✅ Done |
| TODO-67 | ~~`Service<A>` + `ServerApp` 抽象~~ | 部分达成，剩余部分降级为 TODO-67b |
| [TODO-67b](#todo-67b) | H1 keep-alive loop 下沉到 Session 层 | ⏳ **开放**（TODO） |
| [TODO-68](#todo-68) | Ingress request lifecycle 收敛 | ⏳ **开放**（部分实现，后续并入 TODO-168） |
| [TODO-69](#todo-69) | h2c per-route sticky cache 失效重选 + failover | ✅ Done |
| [TODO-70](#todo-70) | Server 端 snapshot 持 `Arc<SelectedConnection>` | ✅ Done |
| [TODO-71](#todo-71) | P2C pick 算法 | ✅ Done |
| [TODO-72](#todo-72) | Client 端连接池去重 + 重试 exclude set | ✅ Done |
| TODO-73 | 不抄 Pingora 的部分（决策记录，非实施任务） | FYI，见 §3 |

### <a name="todo-63"></a>TODO-63 — Peer 描述符化

**问题**：`PeerKind` 混用三种分派方式——`Tcp`/`Http(Box<HttpPeer>)`/`H2(Box<H2Peer>)` 走裸 `connect_inner`，`Dyn(Box<dyn UpstreamPeer>)` 走 vtable，而 `Dyn` 实际是 client `MitmH2Peer` 借道的后门。Peer 还直接持有 `hyper` client，把"上游描述符"和"连接执行"职责混在一起。

**方案**（对标 Pingora `Peer` trait + `PeerOptions`，但不直接照搬全 trait 化）：新增纯值对象 `PeerSpec`/`BasicPeerSpec`/`HttpPeerSpec`/`MitmPeerSpec`，`UpstreamResolver::upstream_peer` 改为返回 `PeerSpec`，"怎么连、怎么复用"移交 TODO-66 的 connector。

**结果**：执行链路已从 `UpstreamResolver -> PeerKind` 切到 `UpstreamResolver -> PeerSpec -> connect_peer`；client 侧 MITM 路径改用显式 `PeerSpec::MitmH2`，不再依赖 `PeerKind::Dyn` 后门；`PeerKind::Dyn`/`UpstreamPeer` 活跃路径已移除。

### <a name="todo-64"></a>TODO-64 — ID newtype 收尾（开放）

**问题**：`server/registry.rs`、`client/conn_pool.rs`、`tunnel-store/src/rules.rs`、h2c `route_cache` 等热路径仍用裸 `String client_id`/`group_id`，typo 编译期不报错；对 connector 而言 `ReuseHash`（address+scheme+TLS+ALPN 一起 hash 成 `u64` key）仍有价值但不是 TODO-66 落地的前置条件。

**方案**：新增 `tunnel-lib/src/ids.rs` 定义 `ClientId`/`GroupId`/`ProxyName`（`Arc<str>` 包装，zero-cost clone）/`ReuseHash`；wire/config 边界暂保留 `String`，只在内存热路径接入；`tunnel-store` 等存储层按 schema 兼容性单独迁移。

**现状**：仓库内仍无 `tunnel-lib/src/ids.rs`；registry/conn_pool/h2c route_cache 仍是裸 `String`。**未开始**，todo.md 中 TODO-64 依旧列为独立 TODO。

### <a name="todo-65"></a>TODO-65 — 热路径结构化错误

**问题**：`anyhow::Error` 全局使用，无法区分错误来源/可重试性；`open_bi_guarded` 内部虽有 `OpenBiOutcome` 观测但返回给调用方的仍是 `anyhow::Result`；`client/entry.rs` 重试循环无法分辨 QUIC open 的 fatal/transient。

**方案**（对标 Pingora `Error { etype, esource, retry }`，其中 `RetryType::ReusedOnly` 是长连接代理的关键语义：只有"复用连接的首个请求失败"才允许重试）：新增 `tunnel-lib/src/error.rs`，Phase 1 只替换三条热路径：`open_bi.rs`、HTTP upstream request、h2c ingress fail response；`ctld_proto.rs`/`tunnel-store` 等外围边界暂不强推。

**结果**：`ProxyError`/`ErrorKind`/`ErrorSource`/`RetryType` 已覆盖 `open_bi_guarded`（区分 `QuicStreamLimit`/`QuicConnectionLost`/`QuicConnectionFatal`）、server/client `UpstreamResolver`、H1/H2c/TLS/TCP ingress 热路径，并接入共享指标 `duotunnel_proxy_errors_total{protocol,type,source,retry}`。外围路径（config/store/ctld）仍用 `anyhow`，不在本次范围内。

### <a name="todo-66"></a>TODO-66 — 统一 `HttpConnector`

**问题**：H1 keep-alive loop（`http.rs`）和 H2 的 `serve_h2_forward`（`h2.rs`）走不同路径，`HttpPeer`/`H2Peer` 字段重复；无 per-peer 协议偏好记忆——upstream 不支持某协议时直接 502，不会记住失败结果（对应旧 todo.md TODO-62 的诉求）。

**方案**（对标 Pingora `Connector<C>` 共享 H1/H2 pool + 全局 `PreferredHttpVersion` map）：Phase 1 保留 hyper client，新增 `HttpConnector` 包装 `HttpsClient`/`H2cClient` + `prefer_h1: DashMap<ReuseHash, Instant>`（TTL 记忆，H2c 探测失败后记住走 H1）；Phase 2（只有 profile 证明 hyper pool 不够时才做）才考虑自研 `H1Session`/`H2Session` pool。

**结果（Phase 1 完成）**：`HttpConnector` 统一封装三端；`server/egress.rs`、`client/app.rs`、client MITM H2 均已切换；H1/H2 两条实时请求路径共享同一份 `prefer_h1` 记忆；cleartext 空 body 请求 h2c 失败后自动回退 H1 一次，非空 body 只做偏好记忆不冒险重放。H1 keep-alive loop 仍留在 `HttpPeer::connect_inner`，未下沉到 session 层——尾巴拆到 TODO-67b。Phase 2 未启动（无 profile 证据）。

### TODO-67 → TODO-67b — keep-alive loop 下沉（开放）

TODO-67（`Service<A>`/`ServerApp` 抽象）目标已通过 plugin 系统（`IngressProtocolHandler`/`IngressDispatcher` 六相位管线）达成，`accept.rs::run_accept_worker` 已是统一 accept 抽象，`transport/listener.rs::start_tcp_listener` 旧抽象保留但未被调用。唯一没做的：H1 的 keep-alive loop 仍写在 `HttpPeer::connect_inner`（peer 层），而不是 Pingora 式的 `ServerApp`/Session 层；`was_reused` 状态无处记录，TODO-65 的 `RetryType::ReusedOnly` 无处使用。**方案**：随 `H1Session::run_loop` 一并下沉。**现状：TODO，未开始。**

### <a name="todo-68"></a>TODO-68 — Ingress request lifecycle 收敛（开放）

二次核对后否决了"把 `UpstreamResolver` 扩展成 Pingora 风格 `ProxyHandler`"的方向——会把 ingress plugin 与 egress core 重新耦合，且 h2c 的并发 request future 不能退化成"天然无锁"的 `&mut Ctx`。真正痛点集中在 h2c：`first_authority`/`route_cache`/`sender_cache` 是 handler 内部 ad-hoc 状态，上游失败统一 502，错误分类/retry/failover 与 TLS/H1 路径不一致。方案是在 `server/plugins/h2c/` 内聚出 `H2cConnState`，不碰 `proxy/core.rs`。

**现状**：todo.md 复核记录"部分实现，后续并入 TODO-168（2026-09-05）"——已有错误后失效逻辑，但"任意请求失败都驱逐 sender"并非最终契约；共享转发策略仍需同时表达失败作用域、实例身份、安全重试、总 deadline 与 body 完成。**开放，且已改道并入 TODO-168 而非继续独立推进。**

### <a name="todo-69"></a>TODO-69 — h2c per-route sticky cache 失效重选 + failover

**问题**：h2c 的真实需求不是"每连接一个 selected"，而是同一 h2c 连接可能承载多个 authority/route，cache key 必须是 `RouteTarget`；cached value 只存 `quinn::Connection`，缺 stale 检测；失败路径统一删除 sender 返回 502，无"可重试一次并重选 client"的语义。**不能引入单 `ctx.selected` fast path**——会把同一连接上不同 authority 的错误复用到第一个 route，破坏 multihost。

**结果**：`CachedSender` 收口为 `{ selected: Arc<SelectedConnection>, sender }`；cache hit 前先查 `close_reason()`，stale 立即失效重选而非等 `forward_h2_request` 报错；按 `conn_id`/`selected` 条件失效避免并发误删；空 body（`is_end_stream()`）请求失败后安全重试一次，非空 body 只做条件失效不冒险重放（必须先有显式 replayability 语义）；补充 `duotunnel_h2c_errors_total`/`duotunnel_h2c_retry_total` 观测指标。per-route state 仍是 handler 内部 `Mutex<HashMap<...>>` 组合，未收敛成独立 `H2cConnState`/`DashMap` 结构（该收尾工作并入 TODO-68 → TODO-168）。

### <a name="todo-70"></a>TODO-70 — Server 端 snapshot 对齐 client

**问题**：`server/registry.rs` 的 `snapshot: ArcSwap<Vec<SelectedConnection>>`，pick 时 `.cloned()` 整个 struct 要 3 次 `Arc::clone`（`conn_id`/`conn`/`inflight` 各一次）；client 侧 `conn_pool.rs` 已经是 `ArcSwap<Vec<Arc<PooledConnection>>>`，只需 1 次。

**结果**：server 侧 snapshot 已改为 `ArcSwap<Vec<Arc<SelectedConnection>>>`，`ClientGroup::build_snapshot`/`select_healthy`/`ClientRegistry::select_client_for_group` 全部返回 `Arc<SelectedConnection>`，h2c cached sender 与 TLS plugin 同步适配。

### <a name="todo-71"></a>TODO-71 — P2C pick 算法

**问题**：`pick_least_inflight` 是 $O(N)$ 线性扫描（`items.iter().filter(healthy).min_by_key(inflight)`），group 内 client 数 > 10 时开始可观测。对标 Pingora 的 Ketama/P2C：选择质量接近 least-loaded 但开销固定。

**结果**：已在 `tunnel-lib/src/inflight.rs` 实现通用有界 `pick_p2c_inflight`，并为 `Server::ClientGroup::select_healthy`、`client::EntryConnPool::next_conn_excluding` 抽出 $O(1)$ 快路径（小列表仍 $O(N)$ 扫描，大列表走 P2C 随机两选一）。**注**：`pingora-tasks.md` 原文将本项标记为 TODO/低优先级"待池规模增长后启用"，但代码已提前落地并合入 done.md，本文件以代码现状为准。

### <a name="todo-72"></a>TODO-72 — Client 端连接池去重 + 重试 exclude set

**问题**：`EntryConnPool::push` 用 `g.iter().any(...)` 线性查重（池小可接受，不紧急）；`client/entry.rs` 重试循环可能在归还 inflight 后重选同一条刚失败的连接，且不区分错误类型一律重试。

**结果**：`push` 已加 `HashSet<stable_id>` O(1) 去重；重试循环带 exclude set，避免本轮重复命中刚失败的连接；按 `QuicStreamLimit`/`QuicConnectionLost`/`QuicConnectionFatal` 细分类，connection-level 失败时驱逐 stale pool entry。"未来多 server 节点分组" 超出当时范围，另见 server HA 相关的 TODO-172（client-server association registry，已在 2026-09 系列记录中单独跟踪）。

---

## §3 TODO-73：不抄 Pingora 的部分（决策记录）

调研 Pingora 时明确记录**不引入**的模式，避免后来者重复调研走弯路：

1. **worker/fork/listenfd 跨进程 fd 传递热升级** — duotunnel 不做 CDN 规模，`CancellationToken` 协作式 shutdown 已够用。
2. **`pingora-cache`（HTTP 响应缓存）** — tunnel 本身不缓存响应体，无此业务需求。
3. **`pingora-ketama`（一致性哈希环）** — duotunnel 单 group 通常几到几十个 client，P2C（TODO-71）已够用，一致性哈希环维护成本不成比例。
4. **`tinyufo`（S3-FIFO + TinyLFU 缓存）** — duotunnel 现有 in-memory 缓存都不需要淘汰语义（连接在线就在，一次性生命周期结束自然回收）；将来做 upstream DNS/auth token/TLS 证书缓存且 QPS 极高时才值得引入。
5. **`ShutdownWatch` + SIGHUP 传 fd 的无缝升级** — overkill，duotunnel 允许短暂连接断开。
6. **`ServiceWithDependents`/`ServiceHandle` 服务启动依赖图** — duotunnel service 数量少，简单 spawn 即可。
7. **`ServerApp` 式一个 app 挂多个 listener**（如 `add_tcp(:80) + add_tls(:443)`）— duotunnel 的协议分派已在 `IngressDispatcher` 按 sniff 结果做，不需要多 listener-per-app。

---

## §4 仍开放的项目汇总

| 项目 | 编号 | 状态 |
|---|---|---|
| ID newtype（`ClientId`/`GroupId`/`ProxyName`/`ReuseHash`）收尾 | TODO-64 | 未开始 |
| H1 keep-alive loop 下沉到 Session 层 | TODO-67b | 未开始 |
| Ingress request lifecycle 收敛（h2c `H2cConnState`） | TODO-68 | 部分实现，已并入 TODO-168 继续 |
| 观测性与业务热路径解耦（tracing 事件 + 独立 subscriber） | TODO-CR4 | 未开始；已有前车之鉴——早期把 Prometheus 更新塞进 `tracing_subscriber::on_event` 同步锁内导致 8k QPS 下 tokio worker 争锁、QUIC keepalive 超时、client 集体断连（`cea0261` 已回滚），正确做法是 `on_event` 内只做非阻塞 channel send，由后台 task 消费更新 |
| 配置源 pull → `Stream<Item = RoutingSnapshot>` 演进 | TODO-CR5 | 未开始，建议排在 TODO-53 Milestone D 之后（会同时改配置加载路径） |
| `tunnel-lib` 拆分为 `tunnel-proto`/`tunnel-engine`/`tunnel-plugins` | TODO-83 | 未开始，独立于本主线 |
