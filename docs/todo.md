# Tunnel TODO

> 2026-09-23 整理：本文件只保留**未关闭事项**。已完成（✅ Done/Implemented/Fixed）、已废弃（❌ Discarded/Rejected）及已作出最终决策的条目一律迁出到 [`done.md`](done.md)（含证据与结论摘要）。运行时/性能现状总览见 [`status.md`](status.md)，不在本文重复。
>
> 2026-09-05 复核；代码基线 `7f8fe625bff1f6aa5c7e98924e4f946196a433a0`。2026-09-23 补充全项目代码 review（[`reviews/2026-09-23.md`](reviews/2026-09-23.md)，基线 `2a20f59`），新增/重开条目见下方对应小节。

[详细优化方案](design/optimization-design-2026-09-05.md) · [代码证据与复现](reviews/2026-09-05.md) · [设计文档索引](design/README.md) · [旧评审记录](archive/review-history-through-2026-09-04.md)

## 当前执行依据

每个问题根据真实调用链、领域职责、并发与资源生命周期、兼容迁移和验证成本综合决策。具体方案与不采用其他方案的理由见详细设计；测试是验证约束的手段。历史 Phase 0–3 划分已不代表当前执行优先级（原文归档于 [`done.md`](done.md#历史路线图存档phase-03-框架--mermaid-依赖图--2026-07-审计索引)），以本节和各条复核状态为准。

| 顺序 | 问题及推荐方向 | TODO |
|---|---|---|
| S0 | 控制面 watch 无心跳，静态配置 120s/300s 后新连接被拒（2026-09-23 P0，见 review R1） | 179 |
| S1 | CA 加载不覆盖；显式初始化；独立的 tunnel 稳定身份 | 176、166、152 |
| S2 | 请求固定 generation，所有相关缓存使用 epoch + sequence | 175、52 |
| S3 | 统一 authority 契约，修 IPv6；H1 超头数明确 431 | 177、158、167 |
| S4 | H2 错误作用域、两层实例身份、重试安全统一设计 | 168、68 |
| S5 | admin 协议适配与业务 mutation 分离，严格有界 framing | 163 |
| S6 | watch 安全监听可先独立实施；远程身份与显式 authority reset | 171、178 |
| S7 | actor 失败传递到进程 owner；结构化阶段 outcome | 154、165、170 |
| S8 | TCP relay 核心收敛；按真实生命周期管理容量 | 169、142、153 |
| S9 | 有测量证据的热点优化；带租约关联视图与多目标 HA | 140、156–159、172 |

状态约定：**已确认**表示代码可证明问题；**已复现**表示已有可执行证据；**候选**须先取得需求或测量依据；**已实现/原判断不成立**不继续按旧方案实施。推荐设计尚需在实现提交中通过其验收条件。

ID 修正：保留原 TODO-148（listener ownership）、TODO-149（batching）；重复的 Tick Timer 改为 TODO-173，Count-Min Sketch 改为 TODO-174。2026-09-05 新增 175–178，2026-09-23 新增 179–199，均不复用旧 ID。

> **已知 ID 冲突（未改号，仅记录）**：`TODO-51` 被使用两次——`done.md` 中的 “Server Auth Path: Resolve Name by Token, Then Push Rules”（已完成）与本文件「Control Plane & Config」小节中的 “LocalTokenCache incremental updates”（未完成）。按本次整理规则不重新编号，读者需以标题区分。

## ID 映射（折叠合并，不改号）

以下条目的独立内容已并入目标 ID，原 ID 仅保留一行指针，避免重复维护：

| 原 ID | 并入 | 说明 |
|---|---|---|
| TODO-52 | TODO-175 | H2 请求级 generation 与缓存 —— 内容见 TODO-175 |
| TODO-68 | TODO-168 | Ingress 请求生命周期收敛 —— 内容并入 TODO-168 的共享转发策略设计 |
| TODO-152 | TODO-166 | 不安全 TLS 状态可观测性 —— 随隧道稳定身份一并验收 |

## 专题簇（交叉引用，避免重复展开）

- **隧道身份 / CA 信任**：TODO-166（P1，稳定 TLS 身份）、TODO-176（P1，CA 加载失败不覆盖）、TODO-152（已折叠→166）、TODO-99（证书 watch 热重载，另见 D4 指针）、TODO-27（身份/会话恢复/0-RTT，需求待细化）。
- **watch 信任模型**：TODO-171（P1，watch 安全默认与远程信任）、TODO-178（P1，显式 authority reset）；另见本文件 R2（watch 通道明文 TCP，2026-09-23 新增，已作为跟进点附在 171/178 内）。
- **H2 sender / 缓存**：TODO-175（P1，完整 generation 身份贯穿 H2 缓存，含原 TODO-52 内容）、TODO-168（P1，H2 错误作用域与实例安全失效，含原 TODO-68 内容）、TODO-139（benchmark-gated，H2 sender 重建风暴，含 R5 握手超时跟进）、TODO-143（研究，adaptive 小池）。
- **inflight / 准入**：TODO-142（分层 active-stream admission，含原 TODO-80 残留的进程级总量兜底缺口）、TODO-109（InflightTable 计数器合并）、TODO-135（inflight 近似读语义）、TODO-146（Server registry slot 容量，含 T8 跨 shard correctness）；已关闭成员 TODO-80/TODO-110/TODO-111/TODO-134 见 `done.md`。

---

## 🔐 2026-09-23 Review 新增（未追踪项，见 [`reviews/2026-09-23.md`](reviews/2026-09-23.md)）

> 5 路并行代码 review（只看代码不看文档），基线 `2a20f59`。R2 已并入 TODO-171/178 簇；R5 已并入 TODO-139；R6 已并入 TODO-142/80 簇说明；均不新建 ID。R10 触发对 TODO-CR-AUDIT-18 / TODO-141 的重开（见下方对应小节）。

### [TODO-179] 控制面 watch 心跳缺失，静态配置下 server 自判过期拒绝新连接
* **Priority**: P0 | **Status**: 已确认（2026-09-23，review R1） | **Track**: Control Plane & Config
* **证据**：`duotunnel-ctld/src/control/watch.rs:133` 只在 `changes.changed()` 时发送；全仓 `watch_tx.send` 仅 `service.rs:478` 一处，且只在 resource_version 变化时触发。server 新鲜度只在 `control_client.rs:394/604/679/785` 收到事件时刷新；阈值 `runtime/health.rs:7-9` = 60s 降级 / 120s 安全过期 / 300s 过期。`recv_config_event` 无读超时，TCP 无 keepalive。
* **后果**：配置长期不变时，120s 后新 tunnel client 认证被拒（`ingress/handlers/quic.rs:252`），300s 后所有新公网连接被拒、`/healthz` not ready；ctld 半开死亡时 server 永不察觉。
* **方案**：watch 协议加 Ping/Pong（或周期性 Duplicate 快照，间隔 < 30s）；server 端读超时即重连。
* **验收**：静态配置长时间运行下新连接不被误拒；ctld 半开断连能在读超时内被 server 感知并重连。

### [TODO-180] 上游 TCP connect / TLS 握手无超时
* **Priority**: P1 | **Status**: 已确认（review R3 ✅ / R4 ✅） | **Track**: Transport & Performance
* **证据**：`lib/proxy/tcp.rs:165`（connect）、`lib/proxy/tcp.rs:188`（TLS 握手）、`server/egress/mod.rs:244,313,347`。黑洞地址会挂起约 130s（TCP 默认重传超时），占用 stream 许可与 task；TLS 握手无超时与 connect 问题叠加。
* **方案**：为 egress TCP connect 与 TLS 握手统一接入可配置超时（复用现有 timeout 基础设施 `infra/timeout.rs`），超时按 `ErrorSource::Upstream` 分类并快速失败释放 permit/task。
* **验收**：黑洞/慢速上游在配置超时内失败退出，不长期占用 stream 许可；正常握手不受影响。

### [TODO-181] client 反向拨号目标无本地白名单
* **Priority**: P1 | **Status**: 已确认（review R7 ✅） | **Track**: Core Proxy & Protocol
* **证据**：`client/tunnel/client.rs:63-65`（`LocalProxyMap` 完全来自 `resp.config`）、`client/ingress/app.rs:141-241`。拨号地址完全由 server 下发，server/ctld 失陷即可把 client 变成内网跳板。
* **方案**：client 侧增加可选本地拨号目标白名单/规则（复用现有 egress 规则模型），默认关闭或宽松，可配置为严格模式用于高安全环境。
* **验收**：白名单开启时，server 下发的越界目标被本地拒绝并有日志/指标；默认行为不回归。

### [TODO-182] 502 响应体泄露内部拓扑
* **Priority**: P1 | **Status**: 已确认（review R8 ✅） | **Track**: Core Proxy & Protocol
* **证据**：`lib/proxy/http.rs:110,123` → `protocol/driver/h1.rs:121 write_502` 返回上游 `host:port` 与 IO 错误细节。
* **方案**：对外 502 响应改为通用文案，详细原因只进结构化日志/指标（呼应 TODO-CR-AUDIT-22 已建立的公开错误边界模式）。
* **验收**：外部 502 响应不包含上游地址/端口/底层 IO 错误文本；日志侧保留完整诊断信息。

### [TODO-183] DNS 无负缓存且 `EgressDnsCache` 无容量上限
* **Priority**: P2 | **Status**: 已确认（review R9 ✅） | **Track**: Transport & Performance
* **证据**：`lib/infra/dns_cache.rs:191-195` —— 首次解析失败不入缓存，每次请求都要等 5s timeout；缓存 map 无容量上限。
* **方案**：加短 TTL 负缓存减少重复 5s 等待；为缓存加容量上限与淘汰策略，避免海量域名场景内存无界增长。
* **验收**：连续对同一失败域名的请求不再每次等待完整 5s；缓存条目数有界。

### [TODO-184] TLS ClientHello 只解析首个 record + 4KB 预算，大 ClientHello 丢 SNI
* **Priority**: P2 | **Status**: 已确认（review R11） | **Track**: Transport & Performance
* **证据**：`lib/protocol/sniff.rs:174,290`。PQ 密钥交换等大 ClientHello 跨多个 TLS record 时，SNI 提取失败并静默降级为 TCP 透传。
* **方案**：支持跨多 record 组装 ClientHello（在明确的字节预算上限内），预算不足时按已知策略降级，但需要可观测（指标/日志），不能完全静默。
* **验收**：常见大 ClientHello（含 PQ 套件）场景下 SNI 正确提取；超预算的降级路径可观测。

### [TODO-185] HTTP header 超过 64 个被判非 HTTP，静默降级 TCP 透传
* **Priority**: P2 | **Status**: 已确认（review R12） | **Track**: Transport & Performance
* **证据**：`lib/protocol/sniff.rs:234,263`。呼应 TODO-167（H1 header 上限应返回 431）——本项是嗅探阶段的姊妹问题：嗅探阶段的 header 超限直接判非 HTTP 走 TCP 透传，而不是像已建立连接后的 431。
* **方案**：嗅探阶段对"疑似 HTTP 但 header 数超限"的情况增加可观测的降级路径（指标/日志），并与 TODO-167 的 431 语义保持行为一致性评估。
* **验收**：超限 header 的 HTTP 请求降级行为可观测；不引入新的误判（合法大 header 请求）。

### [TODO-186] 单 group 固定一个 preferred shard，单租户吞吐锁在约 1/N 核
* **Priority**: P2 | **Status**: 已确认（review R13 ✅） | **Track**: Code Quality, Safety, and Registry
* **证据**：`lib/lb/shard.rs`、`server/ingress/registry.rs:484`。与 TODO-146 的跨 shard 公平性验收（T8）相关但不同：本项是单租户高吞吐场景下的 shard 溢出策略缺失。
* **方案**：在 TODO-146/T8 的跨 shard correctness 验收基础上，评估单 group 溢出到多 shard 的路由策略（需先有压测证据证明是真实瓶颈）。
* **验收**：单租户高 QPS 场景下吞吐不再固定卡在 ~1/N 核；不破坏现有 shard 隔离语义。

### [TODO-187] 通配符 vhost 动态签证书仅 4 并发信号量，可放大为 CPU DoS
* **Priority**: P2 | **Status**: 已确认（review R14） | **Track**: Core Proxy & Protocol
* **证据**：`lib/infra/pki.rs`。任意 SNI 在通配符 vhost 下都会触发动态签证书，并发限流仅 4，攻击者可通过大量不同 SNI 触发 CPU 消耗。呼应 TODO-79（通配符证书预签名与握手缓存）——本项是其安全动机的具体化。
* **方案**：为动态签证书增加按来源/SNI 的速率限制，与 TODO-79 的预签名/缓存方向合流评估。
* **验收**：单来源高频不同 SNI 请求不能无限放大证书签发 CPU 消耗。

### [TODO-188] 单连接 Fatal 登录失败级联杀死整个连接池并退出进程
* **Priority**: P2 | **Status**: 已确认（review R15） | **Track**: HA, Overload & Observability
* **证据**：`client/tunnel/supervisor.rs:114`、`pool.rs:37`。
* **方案**：区分单连接 fatal 与整池不可用；单连接 fatal 不应默认级联退出整个进程，需明确 owner 级失败传递策略（与 TODO-165 的 actor 失败传递设计原则一致）。
* **验收**：单条连接因 fatal 登录失败退出时，其余连接与进程本身不受影响，除非确认为整池级故障。

### [TODO-189] `/metrics`、`/healthz` 硬编码 `0.0.0.0` 且无认证
* **Priority**: P2 | **Status**: 已确认（review R16 ✅） | **Track**: Control Plane & Config
* **证据**：`server/ingress/handlers/metrics.rs:22`、`client/runtime/app.rs:150`。
* **方案**：bind 地址可配置（默认可保持 loopback 或显式声明），并为 `/metrics` 增加可选认证（Bearer/Basic 均可），与 admin socket 的权限模型保持一致的最小暴露原则。
* **验收**：默认部署下 `/metrics`、`/healthz` 不再无条件暴露在所有网卡；开启认证后未授权请求被拒绝。

### [TODO-190] admin CLI `percent_decode` 非 UTF-8 感知，中文名乱码
* **Priority**: P2 | **Status**: 已确认（review R17） | **Track**: Control Plane & Config
* **证据**：`ctld/bootstrap/cli.rs:278`。
* **方案**：percent-decode 后按 UTF-8 校验/替换非法序列，而非字节级直通。
* **验收**：包含多字节字符的 client/group 名称经 CLI 创建、查询后往返一致。

### [TODO-191] `Cargo.lock` 未提交，CI 无 `--locked`
* **Priority**: P2 | **Status**: 已确认（review R18 ✅） | **Track**: Future/Research & CI
* **证据**：`.gitignore:2` 忽略 `Cargo.lock`；CI 未使用 `--locked`。
* **方案**：提交 `Cargo.lock`，CI 构建/测试统一加 `--locked`，避免依赖漂移导致的不可复现构建。
* **验收**：CI 使用固定依赖版本；意外的依赖版本变化会在 `--locked` 下显式失败而非静默漂移。

### [TODO-192] 重连恒推全量 Snapshot，大规模滚动重启时惊群
* **Priority**: P2 | **Status**: 已确认（review R19） | **Track**: Control Plane & Config
* **证据**：`ctld/control/watch.rs:104`。
* **方案**：评估是否可在满足 base revision/hash 匹配时走 Delta（当前架构下首次连接/重连固定走 Snapshot 是有意设计，见 `design/09-runtime-reliability.md` §3.3），若要优化需先有滚动重启惊群的量化证据。
* **验收**：先出具大规模滚动重启场景下 ctld CPU/带宽的压测数据，再决定是否改变当前"重连必发 Snapshot"的简单性设计。

### [TODO-193] P3 清理项（2026-09-23 review 死代码/遗留清单）
* **Priority**: Low | **Status**: 已确认，清理项 | **Track**: Code Quality, Safety, and Registry
* **清单**：
  - `engine/bridge.rs` 的 `relay*`/`QuicBiStream` 死代码；
  - `egress/http.rs::forward_http`（不支持 chunked、无超时，呼应 TODO-147/TODO-180）；
  - `protocol/detect.rs::detect_protocol_and_host` 死代码；
  - QUIC `idle_timeout_secs`/`keepalive_secs` 无下限校验（0 = 关闭 idle 超时）；
  - `plugin/dispatcher.rs:94-118` admission 失败时跳过访问日志；
  - `ctld/control/service.rs:742` admin 响应缓存"淘汰最旧"实为 HashMap 任意项；
  - `engine/copy.rs` 缓冲池不按 size 分桶；
  - `lib/lb/inflight.rs` 对象池只 pop 不 push，回收未生效（已核实，见 `done.md`，此处统一收口跟踪修复）。
* **方案**：按清单逐项清理或明确标注保留原因；不需要独立设计文档。

---

## 🧭 控制面与能力设计新增项（T3F·T4·D1·D2·D3·D6）

> 见 [`reviews/2026-07-26/15-task-breakdown.md`](reviews/2026-07-26/15-task-breakdown.md)、[`16-industrial-implementation-design.md`](reviews/2026-07-26/16-industrial-implementation-design.md) 与 [`design/README.md`](design/README.md)。T0–T3E 的核心实现（`RuntimeGeneration`/`ConfigApplyCoordinator`/`TokenFenceLease`/listener prepare-commit-rollback）已用 CodeGraph 核对代码存在并接线，记入 `done.md`；D9/M0 与 D10/P0 已落地（`design/README.md` 确认）。D4/D5/D7 与既有 TODO-99/TODO-142/TODO-24 重叠，已作为指针并入，不新建 ID。

### [TODO-194] RuntimeGeneration 故障注入验收（T3F）与 M0 长稳验收（T4）
* **Priority**: Medium | **Status**: 待补系统级证据 | **Track**: Control Plane & Config
* **背景**：T0–T3E 的实现已确认存在（`duotunnel-server/bootstrap/mod.rs` 的 `RuntimeGeneration`、`control/control_client.rs` 的 `apply_snapshot`/`fence_revoked_sessions`、`ingress/listener_mgr.rs` 的 `OperationFence`/prepare-commit-rollback、`runtime/health.rs` 的 `begin/finish/hold/fail_config_apply`），但只有单元测试覆盖。
* **T3F 范围**：新配置 schema/引用/端口冲突/证书错误；listener bind/pre-bind 失败、旧 listener drain 超时、快速 A→B→C coalesce；token revoke/rotate 与 listener apply 并发；generation 构建失败、磁盘满、LKG 损坏与双代恢复；worker panic、component 非预期退出与 reload/shutdown 交错。
* **T4 范围**：watch 跨至少 1000 次 revision；10 次 reload 与长连接并行；断网、ctld 重启、server 重连、磁盘满和进程异常退出；多 client group 的 revoke/rotate。
* **验收**：失败时无部分发布；安全 fence 不被错误解除；成功 retry 后能恢复；无法恢复时 readiness 明确失败且无孤儿 worker；输出运行日志、最终 effective hash、应用/拒绝原因、generation 和资源上限指标，作为独立 CI/集成测试 job，不与 8K 性能数字混合。

### [TODO-195] D1：统一 HttpFilter 层（XFF 透传 + 每请求可观测 + 插件插入点）
* **Priority**: 高（由 D6 驱动） | **Status**: 仅完成设计，未实现 | **Track**: Core Proxy & Protocol
* **依赖**：D9/M0（已落地）。
* **内容**：见 [`design/01-httpfilter-layer.md`](design/01-httpfilter-layer.md)。在 ProtocolDriver 层之上建立统一请求级 filter 层，承载 XFF 客户端 IP 透传、每请求可观测埋点，并为 D3（认证）/D5（限流）预留插入点；等价于 TODO-77（统一多协议 Session）+ TODO-68/168 的落地形态之一。
* **验收**：见设计文档验收条件；先以 06 号报告的 microbench + TODO-140 基线验证热路径新增开销不劣化。

### [TODO-196] D2：统一 LB + 健康/outlier 剔除 + 重试预算
* **Priority**: 高 | **Status**: 仅完成设计，未实现 | **Track**: Core Proxy & Protocol
* **依赖**：D9 owned state + D1 outcome 通道；依赖 D6 的按后端数据设阈值。
* **内容**：见 [`design/02-lb-quality.md`](design/02-lb-quality.md)，合流 [`reviews/2026-07-26/08-server-one-to-many-fanout.md`](reviews/2026-07-26/08-server-one-to-many-fanout.md)、[`09-lb-grade-capability-gap.md`](reviews/2026-07-26/09-lb-grade-capability-gap.md) 的选择缺陷修复；brownout 剔除与防重试雪崩。代码现状：`HttpConnector` 已有基础 `mark_upstream_healthy/unhealthy`（`lib/proxy/http_connector.rs:51-61`）与 P2C 选路（TODO-71，已完成），但无 outlier 检测、健康剔除窗口与统一重试预算——本项是在此基础上的能力扩展，非重复。
* **验收**：见设计文档验收条件。

### [TODO-199] 每后端熔断（circuit breaker）
* **Priority**: 低于重试预算，可后置 | **Status**: 仅在设计文档中提及，无实现、无跟踪 ID | **Track**: Core Proxy & Protocol
* **背景**：`design/02-lb-quality.md` §5.3 指出重试预算（全局、简单）应先行，每后端熔断（状态更多：半开/滑窗）可后置——`OutlierDetector` 的"剔除"已提供后端级短路的雏形，熔断是其强化版本。[`reviews/2026-07-26/09-lb-grade-capability-gap.md`](reviews/2026-07-26/09-lb-grade-capability-gap.md) 第 41 行、§"预算与熔断"（211-212 行）将其列为 C 类"弹性"能力缺口之一：当前只有固定 3 次重试（`h1/mod.rs:54`、`tls/mod.rs:148`）+ h2c→h1 回退，无重试预算、无熔断、无自适应并发、无对冲。
* **方案**：在 TODO-196（D2 统一 LB）的 `OutlierDetector` 基础上，按半开/滑窗窗口实现每后端熔断状态机；先完成全局重试预算（TODO-196 范围内），再评估是否需要独立的每后端熔断。
* **验收**：见 `design/02-lb-quality.md` §5.3 的设计条件；无独立测量门槛前不视为阻断项。

### [TODO-197] D3：最终用户认证插件插入点（可选，默认关闭）
* **Priority**: 可选 / opt-in（内网场景非必需） | **Status**: 仅完成设计，未实现 | **Track**: Core Proxy & Protocol
* **依赖**：D1（插入点）+ D4（mTLS 场景）。
* **内容**：见 [`design/03-end-user-auth.md`](design/03-end-user-auth.md)。本轮只要求 D1 留出插入点，不实现 OIDC/JWT/mTLS 具体逻辑。
* **验收**：验证抽象层"插得进去"，不要求默认路径有任何行为变化（默认不认证、零配置开销）。

### [TODO-198] D6：客户端真实来源 IP 透传 + 按后端可观测
* **Priority**: 能力线最高优先（M0 之后） | **Status**: 仅完成设计，未实现 | **Track**: Core Proxy & Protocol
* **依赖**：D9 + D1 + MetricsSink。
* **内容**：见 [`design/06-client-ip-and-observability.md`](design/06-client-ip-and-observability.md)。当前后端看不到真实来源 IP（XFF 未透传），且缺少按后端归因的指标，是 D2 设阈值的前置数据来源。
* **验收**：见设计文档验收条件；产出的按后端数据需能直接喂给 D2 的健康剔除判据。

---

## Control Plane & Config

### [TODO-84] Event-driven Control Plane DB Synchronization
* **Priority**: Medium | **Status**: TODO | **Track**: Control Plane & Config
* **Problem**:
  `duotunnel-ctld` 使用 1500ms 的强轮询 `db_poll_task` 来同步数据更改。
* **Implementation plan**:
  在控制面自己的成功 DB mutation 路径中，于事务提交后发布带 resource version 的事件（必要时使用 outbox/sequence 表保证重启恢复）；`ControlService` 继续以现有 watch channel 向 server 推送 Patch/Snapshot。不要把 SQLite WAL 或文件系统 `notify` 当作正确性来源：事件可能合并、遗漏或无法区分写入语义。保留低频 reconciliation poll 作为外部写入与故障恢复 fallback，直到所有写入都统一经过发布路径。

### [TODO-99] TLS certificate watch and hot reload
* **Priority**: Medium | **Status**: TODO | **Track**: Control Plane & Config
* **Problem**:
  更新证书目前需要物理重启 DuoTunnel 进程。
* **Fix**:
  配合 `notify` 监听本地证书文件的改变，在不切断存量连接的情况下动态 Swap acceptor。
* **对应设计 D4**：内网场景只需 BYO 证书 + 热重载（本项）；ACME 自动签发降为按需，见 [`design/04-trusted-tls-acme.md`](design/04-trusted-tls-acme.md)，未单独立项。

### [TODO-51] LocalTokenCache incremental updates
* **Priority**: Low | **Status**: TODO | **Track**: Control Plane & Config
* **Fix**:
  引入 `WatchEvent::TokenDelta { added, removed }` 实现局部的增量缓存 patch，而不用在每次小变动时全量 clone 重建大 HashMap。

### [TODO-CR5] Config stream model
* **Priority**: Low | **Status**: TODO | **Track**: Control Plane & Config
* **Fix**:
  从 pull 式的 snapshot 加载机制转向响应式的 `Stream<Item = RoutingSnapshot>` 订阅管道。

### [TODO-102] Verify aws-lc-rs ALPN feature consistency in hyper-rustls
* **Priority**: Low | **Status**: TODO | **Track**: Control Plane & Config
* **Fix**:
  对齐依赖包。检查并确保 `hyper-rustls` 和 Quinn 均只调用同一个 `aws-lc-rs` 密码引擎，防止编译进双份不同的加密组件，减小最终二进制的体积与常驻内存。

### [TODO-PARAM-1] Unified parameter configuration schema
* **Priority**: Medium | **Status**: TODO | **Track**: Control Plane & Config
* **Fix**:
  根据 [parameters.md](spec/parameters.md) 进一步细化和合并 timeout、重连退避机制等字段。

### [TODO-151] Tenant-scoped server configuration
* **Priority**: Medium | **Status**: TODO | **Track**: Control Plane & Config
* **Problem**:
  当前 ctld 向每个 server 下发同一份完整的全局配置；server 本地通过 Snapshot/Delta 维护完整配置状态。尚不支持按租户、区域或 server identity 过滤配置，因此所有 server 最终看到的 routing 配置相同。
* **Design direction**:
  为 server 建立稳定 identity，并在配置资源上增加 tenant/scope 关联。ctld 根据 server 注册信息生成目标 server 专属的 Snapshot/Delta；server 只应用属于自身作用域的配置，同时保持 base revision、content hash、ACK 和重同步语义正确。需要明确租户与 server 的绑定、跨租户资源引用、默认配置、迁移/解绑行为，以及多 server 场景下的权限隔离和测试矩阵。
* **备注**：`reviews/2026-07-26/15-task-breakdown.md` 明确将此列为独立需求，不混入全局 EffectiveConfig。

### [TODO-164] Duplicate ingress listener port validation in config validate()
* **Priority**: Low | **Status**: 已复核：防御式校验建议 | **Track**: Control Plane & Config
* **Problem**:
  `duotunnel-lib/src/config/file.rs::ServerConfigFile::validate()` 没有直接检查 ingress listener 重复端口。但 YAML/SQLite override 合并后的 control-plane 路径会经过 `normalize_and_validate_routing`，最终 snapshot 也会再次校验，因此当前重复 listener 会在 bind 前被明确拒绝，不是无配置层校验而只能依赖 `EADDRINUSE`。
* **Fix（建议）**:
  可在 `ServerConfigFile::validate()` 增加 HashSet 端口查重，提升独立调用方的错误提示一致性；动态规则仍必须在合并后的最终 routing 层校验。该项属于防御式校验和可维护性改进，非当前运行时阻断问题。来源：2026-08-21 多视角专项评审及 2026-08-24 代码复核。

---

## HA, Overload & Observability

### [TODO-CR-AUDIT-18] Sniffer Slowloris Vulnerability in Protocol Detection — ⚠️ 重开（2026-09-23）
* **Priority**: High | **Status**: 部分修复；QUIC stream 二次 sniff 路径仍硬编码 5s，未接入 `sniff_timeout_ms` | **Track**: HA, Overload & Observability
* **原修复**: 已 `tokio::time::timeout(sniff_timeout, ...)` 包裹 ingress accept 路径的首次 sniff（client entry / server `IngressDispatcher`），默认 5s，可配置。
* **重开原因（review R10，2026-09-23）**：`duotunnel-lib/src/proxy/core.rs:82` 的 `ProxyEngine::run_stream` 在 QUIC stream 内做二次 sniff 时仍写死 `std::time::Duration::from_secs(5)`，未读取任何 `sniff_timeout_ms` 配置，与 TODO-141 "buffer/relay 参数已全路径贯通"的表述不符（已核对代码：截至 `2a20f59` 该行仍是字面量）。
* **Fix**:
  让 `core.rs::run_stream` 的 sniff timeout 也接入统一的 sniff timeout 配置（与 ingress 路径共用同一来源），而不是独立的硬编码常量。

### [TODO-CR-AUDIT-21] SIGTERM Graceful Drain —— 残留缺口（核心修复已完成，见 `done.md`）
* **Priority**: Medium | **Status**: 核心 drain 已实现；以下缺口仍未做 | **Track**: HA, Overload & Observability
* **残留缺口**:
  1. 无应用层 GOAWAY——QUIC close 会 abort 全部 stream，"drain 后 close" 只是近似，drain 窗口内对端仍可能 open_bi 成功却在 close 时被 abort；
  2. server 侧 drain 计数只覆盖公网 ingress TCP + pending open_bi，client-entry 方向的反向 egress stream 无计数；
  3. 每 stream 短任务与 healthz 每请求任务仍是有意保留的 detached 设计，非缺陷；
  4. 停机路径只有编译与手工验证，无集成测试。
* **Fix**:
  逐项评估是否需要修复（1/2 是行为缺口，4 是验收缺口），3 保持现状。

### [TODO-140] Establish attributable performance baselines and effective-config telemetry
* **Priority**: High | **Status**: P0 prerequisite | **Track**: HA, Overload & Observability
* **Problem**:
  现有 benchmark 与 `/metrics` 已覆盖部分请求、连接和 `open_bi` 指标，但缺少按流量类型可复现的 p99/p99.9、完成率、UDP drop、CPU/GB 与阶段延迟矩阵；不同 cgroup、runtime 和配置默认值也难以从一次结果中复原。
* **Implementation plan**:
  定义 H1/H2 小请求、单/多 H2 connection、1/10/100 MiB L4、受控 RTT/丢包、UDP PPS 和 1→N 核扩展场景。每次输出 achieved RPS/PPS、dropped iterations、错误率、p50/p95/p99/p99.9、open_bi outcome、CPU/RSS/FD/context switch 与 UDP socket drop/`RcvbufErrors`；分阶段采样 sniff、route、connection selection、open stream、first byte 与 relay。启动日志必须打印最终生效的 runtime/accept worker、connection/shard、QUIC/TCP window、buffer、admission 和 pending 限制。以 `QPS(N)/(N × QPS(1))` 记录多核效率。
* **对应 T5（`reviews/2026-07-26/15-task-breakdown.md`）**：CI 采样窗口/有效配置遥测的实现（`benchmark-env.sh`、`bench-tool.py`）已落地，真实 GitHub runner 的 cgroup/systemd 权限、物理核分配与重复运行变异系数仍待验收；无 baseline artifact 前不得声称完成 before/after 优化证据。

### [TODO-145] Integrate hotpath-rs for benchmark-scoped function profiling
* **Priority**: Medium | **Status**: TODO | **Track**: HA, Overload & Observability
* **Problem**:
  现有 benchmark 能看到端到端延迟、吞吐和部分 runtime 指标，但难以直接归因到具体函数边界。很多性能 TODO（HTTP egress scratch、relay buffer、H2 sender、UDP PPS、EntryConnPool actor）都需要先确认热点是否真的落在目标路径上。
* **Implementation plan**:
  以可选 Cargo feature 接入 `hotpath`，优先只启用函数耗时 profiling：在 duotunnel-server + duotunnel-client runtime 入口创建 guard，并对 QUIC stream open、sniff、route lookup、HTTP egress、TCP relay、UDP datagram encode/decode、H2 sender rebuild 和 EntryConnPool mutation 等少量关键边界加 `#[measure]`。先产出静态 JSON 报告并接入现有 benchmark artifact；不要默认打开 `hotpath-alloc` 或 `hotpath-cpu`，前者需先验证与现有 `mimalloc` 全局 allocator 的关系，后者需要独立 profiling profile/debug symbols，不能混入常规 release/CI 结果。
* **Adoption stages**:
  Stage 1 uses only `functions-timing` + `threads` with `HOTPATH_OUTPUT_FORMAT=json`, `HOTPATH_OUTPUT_PATH`, `HOTPATH_REPORT` and `HOTPATH_FOCUS`, so benchmark artifacts answer "which measured boundary got slower" without changing allocator/runtime behavior. Stage 2 may add `channel!`, `future!`, `stream!`, `mutex!` and `rw_lock!` only for suspected contention or backpressure points such as EntryConnPool, control watch, H2 sender rebuild and UDP session paths; wrapper macros can change named endpoint/lock types, so use `hotpath::wrap::*` deliberately and keep the profiled build semantically identical to the normal build. Stage 3 may add TUI/live inspection for local debugging and PR comment-style CI comparison after the benchmark matrix is stable.
* **Do not copy blindly**:
  Treat external hotpath guides as practice references, not exact API contracts. Prefer current 0.21.x names (`HOTPATH_OUTPUT_FORMAT`, `HOTPATH_OUTPUT_PATH`, `HOTPATH_REPORT`, `HOTPATH_ALLOC_METRIC`, `HOTPATH_ALLOC_CUMULATIVE`) over older names such as `HOTPATH_OUTPUT` or `HOTPATH_MEMORY_MODE`. Do not add a custom Prometheus reporter unless the current crate API explicitly supports that integration; the first production-grade output path should remain static JSON artifacts plus existing DuoTunnel metrics. CI should compare controlled head/base artifacts and post a report before it becomes a blocking gate.

### [TODO-142] Add global and group-level active-stream admission control
* **Priority**: High | **Status**: Partial / UDP session integrated, other domains deferred | **Track**: HA, Overload & Observability
* **Problem**:
  当前 `open_bi` 有每 connection semaphore、pending queue 上限和 slowpath，但没有跨 connection 的 global active-relay 预算或按 client group 的公平预算。慢后端/慢客户端可在多个连接上同时占用 task 与 buffer，直到局部限制才生效。**含原 TODO-80 的残留缺口**：其进程级总量兜底闸门被移除后一直没有替代，归并到本项的分层模型中。
* **Implementation plan**:
  当前已完成可复用 global/group RAII controller、显式 Global/Group rejection scope 和并发 invariant 测试，并将 controller 接入一个边界清晰的资源域：每个 QUIC client connection 的 UDP session。`UdpSessionManager` 的 `SessionEntry` 持有 permit，覆盖创建排队、三秒 session operation timeout、连接/空闲淘汰、失败、QUIC shutdown 和 reply pump 结束；原进程级 UDP semaphore 与 UDP queue semaphore 保持独立。HTTP request、raw relay、reverse stream 和 UDP queue 不共享该计数器。

  其他域只有在明确 owner、路由/认证 group identity、拒绝语义、取消/超时释放和完整测试后接入：HTTP permit 必须覆盖 response body 完成，raw relay 必须覆盖双向 relay task，reverse stream 必须覆盖 drain/cancel，queue budget 必须覆盖 queued envelope 的消费或丢弃。H1 可返回 503，其他协议需定义可观测的关闭/错误策略，并记录 reject、queue wait、permit-held duration。阈值仍以 TODO-140 慢客户端/慢后端基线为依据，不能把 `max_pending_streams` 当 active relay 上限，也不能把本次 UDP session 接入误标为全协议完成。
* **补充（review R6 ✅，2026-09-23）**：公网 HTTP/TCP accept 目前无并发上限（`lib/transport/accept.rs:39` 每连接无条件 spawn；QUIC/metrics/UDP 均已有上限）。建议作为本项接入的下一个资源域，或补一个独立的公网入口全局信号量 + per-IP 限流（`reviews/2026-09-23.md` 建议推进顺序第 3 步）。
* **D5 对应关系**：capacity fairness across client group 即本项的 group-level 预算；`design/05-rate-limit-admission.md` 的设计内容已并入本项范围，不单独立项。

### [TODO-88] Coarse Monotonic Clock for High-Frequency Telemetry
* **Priority**: Medium | **Status**: TODO | **Track**: HA, Overload & Observability
* **Problem**:
  即使有 vDSO 优化，在高频（每秒百万包）数据包中继流中调用 `Instant::now()` 获取指标时间仍会占据不少的 CPU 时间比例。
* **Fix**:
  设计一个微秒级更新的 thread-local 或全局粗粒度单调时钟缓存（Coarse Monotonic Clock），用于高频遥测下的时间戳计算，降低对 OS 内核的访问频次。

### [TODO-CR-AUDIT-3] QuicConnectionFatal 的宏观责任划分缺陷
* **Priority**: Medium | **Status**: TODO | **Track**: HA, Overload & Observability
* **Fix**:
  对 `QuicConnectionFatal` 异常进行细化归类，结合上下文流向区分其具体是属于 `Upstream` 还是 `Downstream`，防止由于网络异常误报核心故障。

### [TODO-CR4] Decouple observability from business hot paths
* **Priority**: Low | **Status**: TODO | **Track**: HA, Overload & Observability
* **Fix**:
  减少在热路径上直接调用 metrics。利用 trace 事件以非阻塞的 channel 异步收集指标，保证不在 Tracing 锁下更新 metrics 计数器。

---

## Core Proxy & Protocol

### [TODO-77] Unified multi-protocol session handling inspired by Pingora
* **Priority**: Medium | **Status**: TODO | **Track**: Core Proxy & Protocol
* **Proposed Architectural Directions**:
  * **方案 A**: 使用类似 Pingora 的 `DownstreamSession` Enum 封装 H1, H2, WS。利用 hyper 的底层 `http1::handshake` 获取 Upstream，用 low-level API 控制写入，支持灵活重试及 WebSockets 降级。
  * **方案 B**: 极度简化的、针对 DuoTunnel 特化的 L4 级透传 async 方法。
* **关联**：见上方 TODO-195（D1 HttpFilter 层）——该设计是本项的一种落地形态。

### [TODO-67b] Move keep-alive loop into Session layer
* **Priority**: Medium | **Status**: TODO | **Track**: Core Proxy & Protocol
* **Problem**:
  H1 Keep-Alive 逻辑与 upstream 描述和建连代码耦合严重。
* **Fix**:
  创建 `H1Session` / `H2Session` 等生命周期宿主，使重试判定与会话逻辑拥有清晰的作用域。

### [TODO-62] Full per-peer protocol capability memory
* **Priority**: Medium | **Status**: TODO | **Track**: Core Proxy & Protocol
* **Problem**:
  对上游节点的协议能力缺乏可靠的缓存记忆。遇到 ALPN 或 h2c 回退时，每次新请求都会试探并出错，带来严重的瞬时延迟和请求毛刺。
* **Fix**:
  实现一个全局 TTL 协议记忆组件（例如 `ArcSwap<HashMap<PeerKey, ProtocolCapability>>`），记录可用 ALPN 结果。当下游/上游 TLS 降级时立即刷新，避免再次探测产生黑洞。

### [TODO-78] L7 HTTP Connector integration with EgressDnsCache
* **Priority**: High | **Status**: TODO | **Track**: Core Proxy & Protocol
* **Fix**:
  让 Hyper 的 L7 HttpConnector 在建连时不再阻塞进行同步解析，改用自定义解析器注入 `EgressDnsCache`。

### [TODO-79] Wildcard Certificate Pre-signing & Handshake Cache for MITM
* **Priority**: Medium | **Status**: TODO | **Track**: Core Proxy & Protocol
* **Fix**:
  引入预生成通配符 CA 证书的机制，并在后台异步签署、缓存它们，解决实时生成 rcgen 对 CPU 的重度挤占问题。
* **安全动机补充（review R14，2026-09-23）**：见 TODO-187——当前每 SNI 动态签证书仅 4 并发限流，可被大量不同 SNI 放大为 CPU DoS；本项落地时需同时覆盖按来源的速率限制。

### [TODO-100] HTTP/2 over QUIC Selective Native Multiplexing Mode
* **Priority**: Medium | **Status**: TODO | **Track**: Core Proxy & Protocol
* **Fix**:
  增加配置化多路复用选项。支持多流 H2 复用单一 QUIC 流（unary gRPC 延迟优），或对于大文件传输直接开启独立原生 QUIC 流（避免 H2 窗口阻塞）。

---

## Zero-Copy & Buffer Pooling

### [TODO-149] Batching candidates on the hot path (benchmark-gated)
* **Priority**: Low until the baseline is trustworthy | **Status**: Recorded 2026-07-26, **do not start yet** | **Track**: Zero-Copy & Buffer Pooling
* **Context**:
  Assessed the "batch everything" lens (writev / io_uring / SIMD) against this codebase. Most of it is already in place or already decided: `httparse` is SIMD internally, quinn batches UDP via GSO/GRO, io_uring is rejected (decision D-12), and the chunked response path already vectorizes its stream writes. Magnitude check: batching a few stream writes is worth ~1–3 µs/req against a 25–55 µs/req L7 cost — a second-order effect next to the per-request allocations (TODO-97 neighbours, review 01 §3.4/§4.1) and the structural serialization points (review 02). Full reasoning in `docs/archive/review-2026-07-26/01-hotpath-analysis.md` §4.8.
* **Candidates**:
  1. **Relay read batching** — `read_chunk` (singular) → `read_chunks(&mut [Bytes])`, which quinn 0.11.9 provides (`recv_stream.rs:215`) and which nothing in the repo uses. Highest-frequency loop in the system, but 64 KiB buffers already amortize much of it and the win depends on how often more than one chunk is actually available; profile the chunk-arrival distribution first.
  2. **Merge response head with the first body frame** — the Content-Length and close-delimited branches write the header separately and then one `write_chunk` per frame, so a small GET response costs two stream writes. Same open question as `docs/guide/counter_intuitive_network_practices.md` §1.4 (contiguous assembly vs vectored); measure both.
  3. **UDP per-packet allocation and per-packet `send_datagram`** — already folded into TODO-144; the clearest of the three, since one allocation per packet is unambiguous waste.
  4. **Per-core counters (K1)** — the same principle applied to cache lines rather than syscalls; already scheduled as review 02 Phase A, and this lens argues for keeping it ahead of the others.
* **Gate**:
  Blocked on the cpuset baseline work (review 06, review 02 §6). This is not bureaucratic: every multi-core number taken before 2026-07-26 was measured with public ingress pinned to a single thread (TODO-148), so there is currently no baseline that could tell whether any of these helps.

### [TODO-98] Bind buffer lifecycle to async tasks (Cache hit improvement)
* **Priority**: Medium | **Status**: Research / benchmark-gated | **Track**: Zero-Copy & Buffer Pooling
* **Problem**:
  多线程 Tokio 的 work-stealing 会使 task 在不同 worker 上恢复；thread-local pool 因此可能在新 worker miss。把 buffer 放入 task state 可减少一次 pool 交接，但 future 的堆内存和 CPU cache line 不会随 task 物理迁移，不能据此承诺 L1/L2 locality 改善。
* **Implementation plan**:
  先完成 TODO-97 的安全 `BytesMut/read_buf` 缓冲模型，再在 relay benchmark 中比较 thread-local reuse、task-owned buffer 和有界全局 fallback 的 allocator 事件、P99 与吞吐。仅在 pool miss 是可观测瓶颈时保留 task-owned buffer；保持取消安全和每个 relay 两个方向 buffer 的明确上限。

### [TODO-136] Safely remove sniff-buffer zero initialization with `BytesMut` / `read_buf`
* **Priority**: Medium | **Status**: Research / benchmark-gated | **Track**: Zero-Copy & Buffer Pooling
* **Problem**:
  当前 `PeekBufPool` 用已初始化的 `Vec<u8>` 为 `SniffRuntime::sniff` 提供可写 slice，因此在 buffer 短于目标长度或冷启动时会清零。它是安全的，但在高连接创建率下会消耗内存带宽；不能以未初始化的 `Vec<u8>` 加 `set_len` 取代，因为 `AsyncReadExt::read` 需要有效的已初始化 slice。
* **Implementation plan**:
  先在连接建立压测中测量 memset 占比。若确认为热点，再将 pool 与 `SniffPrefix::Pooled` 的所有权模型整体迁移为长度为零的 `BytesMut`：使用 `read_buf` 追加已初始化字节，检测时只借用 filled region，并在 prefix 的最后一个 owner drop 后安全回收容量。必须覆盖 partial read、detector 多轮读取、prefix advance、跨 Tokio worker 迁移和取消；不要引入仅靠 thread-local 归还假设的跨 await 缓冲池。

### [TODO-22 / TODO-34 / TODO-86] 消除中继路径上的 Generic tokio::io::split 锁竞争
* **Priority**: Medium | **Status**: 🚧 Partial / Production TCP Specialized | **Track**: Zero-Copy & Buffer Pooling
* **Problem**:
  中继核心接口使用的是 Tokio 提供的通用 `tokio::io::split(stream)`。该通用接口在内部使用 `BiLock<Mutex>` 锁来在 Generic 抽象上模拟全双工，高负载下会导致严重的多核 CPU 锁争用。
* **Fix**:
  针对具体的套接字类型进行直接的类型特化。TCP 连接直接使用 `stream.into_split()`（操作系统层面的 FD 拆分），QUIC 连接直接使用拥有的 `SendStream` 和 `RecvStream`。当前 QUIC↔TCP production helpers and `relay_tcp_*` paths are specialized; remaining generic `tokio::io::split` usage is retained only for generic fallback/test-only relay helpers.

### [TODO-101] Optional user-space spin-polling for copy loops
* **Priority**: Low | **Status**: TODO | **Track**: Zero-Copy & Buffer Pooling
* **Problem**:
  Tokio 的默认 `epoll` 线程唤醒存在 10–20 微秒的固有延迟。
* **Fix**:
  支持配置自旋。在空闲时先调用 `.try_read()` 轮询自旋数十微秒（如 50us），若实在无包再释放控制权挂起，以硬件换极致低时延。

---

## Transport & Performance

### [TODO-141] Propagate relay and HTTP body buffer configuration end-to-end — ⚠️ 部分重开（2026-09-23）
* **Priority**: High | **Status**: 绝大部分已实现；QUIC stream 二次 sniff timeout 仍未贯通配置（见 TODO-CR-AUDIT-18） | **Track**: Transport & Performance
* **Problem**:
  `ProxyBufferParams` 已提供 relay/header/body 参数，但部分 bridge、generic TCP/TLS relay、client entry、server egress 和 H1 reader 仍使用固定值，调参无法覆盖完整数据路径。
* **Implementation**:
  已在不改变默认值的前提下增加带参数 relay API，并将其接入 client entry、server ingress/server egress、generic TCP/TLS relay 和 ProxyEngine QUIC stream sniff；`HttpConnector`、`HttpPeer`、`Http1Driver` 和 `forward_http` 现在消费 header/body 参数。旧无参数函数保留为默认 wrapper。focused unit tests 验证非默认 relay capacity、H1 buffer configuration 和 sniff pool 配置的贯通。16/32/64/128 KiB 的吞吐、P99、CPU/GB 和 RSS 对比仍由 T5 性能基线单独执行。
* **重开原因（review R10，2026-09-23）**：`lib/proxy/core.rs:82` 的二次 sniff timeout 仍是字面量 `Duration::from_secs(5)`，未接入任何 buffer/timeout 参数贯通链路——与本项"端到端贯通"的表述有出入。跟进动作见 TODO-CR-AUDIT-18。

### [TODO-137] Benchmark-gated H1 egress scratch-buffer and response-line optimization
* **Priority**: Low | **Status**: Research | **Track**: Transport & Performance
* **Problem**:
  `egress/http.rs::forward_http` 为请求头创建 8 KiB `BytesMut`，并为响应头创建独立 buffer；响应状态行经 `write!` 格式化。是否为真实热点尚无分配/CPU profile 证据，且 `write!` 的格式串是编译期解析，不存在报告所称的运行时格式串解析。
* **Implementation plan**:
  先在 H1 小请求高 QPS profile 中分离 allocator、header parse、upstream I/O 与 response write 的占比。仅当 scratch allocation 可见时，设计取消安全、容量有上限的 scoped `BytesMut` reuse；不能因为 Tokio task 可跨 worker 迁移就简单依赖 thread-local pool。若响应行格式化进入 profile，再以 `status.as_str()` 和 `extend_from_slice` 取代通用 fmt，并保持现有 header buffer 的单次写入语义。
* **关联（review P3，2026-09-23）**：`egress/http.rs::forward_http` 被 review 标注为不支持 chunked、无超时的遗留路径，见 TODO-193 清理清单与 TODO-147/TODO-180。

### [TODO-138] Benchmark QUIC-to-TCP small-chunk aggregation
* **Priority**: Low | **Status**: Research | **Track**: Transport & Performance
* **Problem**:
  `copy_quic_to_shutdown` 正确地通过 `RecvStream::read_chunk` 将 Quinn 的 `Bytes` 直接交给 TCP writer，避免用户态中间拷贝。若真实流量呈现大量小 chunk，可能增加 TCP `write_all` 调用次数；但 `BufWriter` 会重新引入拷贝、改变 flush 延迟，内核 TCP 本身也会聚合发送。
* **Implementation plan**:
  先记录 chunk-size 分布、write syscall 次数、吞吐与 P99，在 bulk 与 latency-sensitive 两种负载下比较。只有 bulk profile 证明 syscall 成本主导时，才加入有明确容量和 flush 边界的可选聚合模式；默认保留现有零拷贝路径。

### [TODO-144] Profile and redesign the UDP PPS data plane when warranted
* **Priority**: Medium | **Status**: Research / UDP-gated | **Track**: Transport & Performance
* **Problem**:
  UDP 路径当前每包使用 rkyv envelope encode/decode，并在 session reply path 分配 payload/`Bytes`；每个 session 也维护 socket、reply pump 与定时清理。它保证了独立 upstream source-port 语义，但高 PPS、短 session 工作负载可能受 allocator、FD、task 和 wall-clock 调用限制。
* **Implementation plan**:
  先在 TODO-140 的 PPS/session-density 基线中拆分 encode/decode、copy、DashMap、socket/task、timer 和 drop 成本。若确认瓶颈，再设计版本化紧凑 header、borrowed/`Bytes` decode view、粗粒度时间和带上限的 session shard；共享 socket 只能作为会改变 upstream source-port 语义的显式模式，不能替换默认语义。不要把 UDP datagram payload 上限硬编码为 1200 bytes，须依据协商的 QUIC datagram/path MTU 处理。

### [TODO-105] Enable TCP Autotuning by defaulting buffer sizes to None
* **Priority**: High | **Status**: Ready for implementation | **Track**: Transport & Performance
* **Problem**:
  在 `duotunnel-lib/src/transport/tcp_params.rs` 中，`recv_buf_size` 和 `send_buf_size` 默认被设置为 `Some(4 * 1024 * 1024)`；`TcpConfig::default()` 会透传这两个值，所有未显式配置的 TCP 路径都会调用 `setsockopt`。这会固定 socket buffer 的策略，放弃由 Linux 的 `tcp_rmem` / `tcp_wmem` 随 RTT 与 BDP 调节的默认能力，并为大量空闲连接保留过高的缓冲上限。
* **Implementation plan**:
  将 `TcpParams` 默认值改为 `None`，让 `TcpConfig::default()` 自然继承；保留配置文件中显式 `recv_buf_size` / `send_buf_size` 的覆盖语义。补充默认值、显式覆盖和 `apply()` 不调用对应 `setsockopt` 的测试，并在 Linux 上对低 RTT 与高 BDP 两组负载做吞吐/内存回归。不要修改 QUIC 的 UDP buffer 参数，它们是独立的收包队列调优项。

### [TODO-73] Plugin-based IPv6 support and DNS Hijacking connection interceptor
* **Priority**: Medium | **Status**: TODO | **Track**: Transport & Performance
* **Fix**:
  实现插拔式的 `Ipv6FirstResolver` 插件，以及可在 admission 阶段重定向 DNS 端口流量的劫持模块。

### [TODO-CR-AUDIT-2] 缓存行填充与堆内存分离的开销权衡 (False Sharing vs Heap Allocation)
* **Priority**: Low | **Status**: TODO | **Track**: Transport & Performance
* **Problem**:
  使用 `CachePadded<AtomicUsize>` 包裹 `Arc` 可以防 False Sharing，但引发了多余的堆分配。

### [TODO-CR-AUDIT-7] 高频请求生命周期内 Engine 对象的动态实例化开销
* **Priority**: Low | **Status**: TODO | **Track**: Transport & Performance
* **Problem**:
  在每次 TCP 连接请求时都会动态 new 出 `ClientApp` 与 `ProxyEngine` 对象，引起轻微的堆内存抖动。

### [TODO-109] Optimize InflightTable atomic operations by merging counters
* **Priority**: Medium | **Status**: Semantics and benchmark gated | **Track**: Transport & Performance
* **Problem**:
  `InflightSlot` 维护了 `pending_opens` 和 `active_streams` 两个独立的原子变量；当前选择与 slowpath 只读取二者之和。`promote()` 因而会执行一次减法和一次加法，`inflight_load()` 会执行两次读取。
* **Implementation plan**:
  可以改为 `total_inflight: AtomicUsize`，使 `promote()` 无需再改计数，load 只读取一个原子，Drop 在任何 phase（即时失败、等待超时、取消或正常关闭）递减后都 `notify_one()`：slowpath 等待的正是总 inflight 下降。先覆盖即时成功、即时失败、等待超时、取消和并发选择的 invariant/notification 测试，再以 flamegraph/基准确认是否值得合并。原子操作数量减少不等于端到端性能按相同比例提升。

### [TODO-24] Multi-endpoint + SO_REUSEPORT UDP research
* **Priority**: Low | **Status**: Research | **Track**: Future/Research & CI
* **Fix**:
  仅在压测证明单个 Quinn endpoint UDP driver 单核打满、其他核空闲，且连接池读路径、注册 shard , H2 sender 缓存、egress reject 索引和 UDP 拷贝链均不是主瓶颈后，再研究 multi-endpoint + `SO_REUSEPORT` UDP。当前明确不推进 thread-per-core：它会丢失 Tokio work-stealing，显著抬高 Quinn/Hyper 生态改造成本，并且不适合 DuoTunnel 常见的 N 对 M 汇聚隧道流量。研究路径需包含在前端挂载轻量级 eBPF (XDP / Socket Redirect) 程序，根据 QUIC CID 路由数据包，解决 SO_REUSEPORT 因连接迁移/NAT重绑定导致的路由失效和丢包问题。
* **D7 对应关系**：`design/07-multi-endpoint.md` 的 Phase B 多 Endpoint 实验设计已并入本项范围（P2 / profile-gated，依赖 D9 M0 + D10 可信 profile），不单独立项。

---

## Code Quality, Safety, and Registry

### [TODO-83] Deconstruct duotunnel-lib into targeted sub-crates
* **Priority**: Medium | **Status**: TODO | **Track**: Code Quality, Safety, and Registry
* **Problem**:
  `duotunnel-lib` 库趋向庞大混乱，混合了协议、中继以及 Client/Server 的具体实现。
* **Fix**:
  拆分为 `tunnel-proto` (协议帧), `tunnel-engine` (复制中继) 与 `tunnel-plugins` (接口插件)。

### [TODO-106] Shard EntryConnPool write actor by shard_id to scale write throughput
* **Priority**: Medium | **Status**: Research / benchmark-gated | **Track**: Code Quality, Safety, and Registry
* **Problem**:
  目前 `EntryConnPool` 中所有的 `Push`/`Remove` 写操作均串行发送给单个 MPSC 通道后台 Actor 进行更改。在极端网络闪断和海量连接重连时，单个 Actor 可能会因消息积压成为写吞吐瓶颈。
* **Decision and implementation plan**:
  当前 Actor 只承载冷路径 mutation，读路径已经通过 `ArcSwap` 分片快照无锁执行；因此不应仅凭设计推断拆分。先在重连风暴压测中采集 MPSC queue depth、push/remove acknowledgement latency、Actor CPU 和 snapshot clone 时间。若 Actor 确认占主导，再按 `stable_id` 选择每 shard channel/Actor，并让每个 Actor 独占其 `PoolShard` 与 slot free-list（TODO-111，已关闭见 `done.md`）；`Remove` 直接从 `stable_id` 重算 shard，避免跨 shard 搜索。TODO-108 应随此改造合并完成，不作为单独优化；TODO-107 的预构造 handle 会增加去重和 slot 回滚协议，除非 profile 显示 spawn/alloc 是主因，否则继续延后。

### [TODO-107] Offload connection handle spawning from EntryConnPool actor
* **Priority**: Medium | **Status**: Deferred pending profile evidence | **Track**: Code Quality, Safety, and Registry
* **Problem**:
  目前的 `EntryConnPool` 在处理 `Push` 消息时，在 Actor 线程内执行了 `inflight_table.alloc_slot()` 以及 `ConnectionHandle::spawn`（涉及创建信号量 `Semaphore` 等堆内存分配动作），增加了单线程 Actor 的负载与延迟。
* **Decision**:
  预构造 handle 会引入重复 Push 的去重、slot 回滚和 actor 关闭时的资源归还协议；在 reconnect 冷路径上不应先支付这份复杂度。仅在 TODO-106 的压测证明 `alloc_slot` / `ConnectionHandle::spawn` 是主导耗时后再设计两阶段 reserve/commit 协议。

### [TODO-146] Make Server ClientRegistry slot capacity explicit and observable
* **Priority**: High | **Status**: Partially implemented; cross-shard acceptance pending | **Track**: Code Quality, Safety, and Registry
* **Problem**:
  `ClientRegistry::new` currently creates `new_inflight_table(4096)` regardless of configured QUIC stream limits, runtime parallelism, expected client connection count, shard count, or deployment size. Every registered client connection consumes one slot; after 4096 live registrations, authentication succeeds but registration fails with `inflight slot table exhausted`. The limit is absent from configuration, startup telemetry, capacity documentation and `/metrics`.
* **Implementation plan**:
  Choose one explicit model: a validated `max_client_connections` capacity, a capacity derived from a documented connection budget, or a safely segmented/growing slot table whose existing slot references remain stable. The current implementation uses a validated explicit capacity and exposes capacity, active, available, high-water mark and exhaustion count at startup/metrics. Boundary tests cover exactly-at-capacity, one-over-capacity and unregister/reuse; cross-shard fairness and broader actor lifecycle acceptance remain pending. Coordinate allocator ownership changes with TODO-111 (closed, see `done.md`), but do not make correctness depend on the benchmark-gated EntryConnPool actor sharding in TODO-106.
* **T8 对应关系（`reviews/2026-07-26/15-task-breakdown.md`）**：checked capacity、active/available/exhausted 指标、耗尽不驱逐已实现；跨 shard correctness 与公平分布 benchmark 仍依赖 T5，多核扩展前必须先完成 correctness test——即本项"cross-shard acceptance pending"部分。另见 TODO-186（review R13，单 group 固定 shard 的溢出问题）。

### [TODO-155] Naming and dead-field cleanup
* **Priority**: Low | **Status**: 已确认：清理项 | **Track**: Code Quality, Safety, and Registry
* **Problem**:
  评审发现的命名/字段小瑕疵：`NegotiatedProtocol { version, capabilities }` 装的是协商结果而非协议（宜叫 NegotiatedSession）；`MessageType` 数值乱序（0x10 夹在 0x04–0x06 之间）易误读；`SelectedConnection.negotiated` 是带解释的 dead_code 字段；`ComponentHandle._name` 未使用；`supervise_component` 一处 error! 缩进错位。
* **Fix（建议）**:
  逐项清理；注意 wire 类型（MessageType 数值）不能改值只能改注释，内部类型可直接重命名。`stable_id` 命名问题见此前记录，不在此重复。来源：2026-08-21 深度评审。

---

## Future/Research & CI

### [TODO-CR-AUDIT-20] Fuzz Testing for Sniffing and Lock-Free Structures
* **Priority**: Medium | **Status**: TODO | **Track**: Future/Research & CI
* **Problem**:
  DuoTunnel 核心逻辑直接暴露在未经校验的物理协议嗅探数据下，并且用到了许多复杂的无锁结构（如 `ArcSwap`/`InflightTable`），缺少模糊测试以确保鲁棒性。
* **Fix**:
  集成 `cargo-fuzz` 框架，为嗅探器和并发无锁表单独设计模糊测试靶标。

### [TODO-156] Optional hot-path micro-optimizations (xxhash, UDP Bytes)
* **Priority**: Low | **Status**: 已确认：benchmark-gated 优化候选 | **Track**: Future/Research & CI
* **Problem**:
  两处可选优化：① `lb/shard.rs::stable_shard_index` 用 SipHash（DefaultHasher），抗碰撞但较慢——当前按连接调用影响极小，但注意 DefaultHasher 跨版本稳定性无保证，绝不可用于任何持久化场景；② `UdpDatagramEnvelope.payload: Vec<u8>` 每个 datagram 一次堆分配+拷贝，换 `Bytes` 可减少编码路径拷贝（1200B 小包场景收益有限）。
* **Fix（建议）**:
  仅在 dial9/Criterion profile 证明相关路径是瓶颈后再做（符合本仓库既定门槛）。若引入 xxhash/fxhash，需在文档标注其非抗碰撞、仅限进程内分片使用。来源：2026-08-21 深度评审。

### [TODO-160] Mid-stream transformation hook for ConnectionModule
* **Priority**: Low | **Status**: 已确认：架构扩展项 | **Track**: Future/Research & CI
* **Problem**:
  `ConnectionModule` 只有 `pre_admission`（准入前）和 `on_complete`（结束后）两个生命周期端点钩子；想做请求头改写、响应注入这类流中变换的插件没有挂点——header 清洗目前硬编码在 h1 driver 内部（sanitize_request_headers）。当前业务不需要，但这是最可能撞墙的扩展点缺口。
* **Design direction（待确认）**:
  参考 Pingora HttpBase/HttpModule 的 request_filter/response_filter 语义，在 ProtocolDriver 层增加可选的变换钩子；注意与现有 capability bits 协商机制对齐，避免未协商特性静默生效。先等真实需求出现再设计，避免过度抽象。来源：2026-08-21 抽象与可读性评审。

### [TODO-57] quinn stream-level lock research
* **Priority**: Low | **Status**: Research | **Track**: Future/Research & CI

### [TODO-25] io_uring instead of epoll
* **Priority**: Low | **Status**: Deferred | **Track**: Future/Research & CI

### [TODO-55] quinn ConnectionDriver debug_span per-poll overhead
* **Priority**: Low | **Status**: Deferred pending evidence | **Track**: Future/Research & CI

### [TODO-28] Kernel-level zero-copy relay
* **Priority**: Medium | **Status**: TODO | **Track**: Future/Research & CI
* **Fix**:
  在单纯 TCP 的 Passthrough 通路上，在 Linux 下利用 splice/sendfile进行试验性零拷贝加速。

### [TODO-29] Dynamic buffer/window tuning
* **Priority**: Medium | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-30] Upstream pre-warming
* **Priority**: Low | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-31] VhostRouter wildcard trie/radix tree
* **Priority**: Medium | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-37] Seamless graceful handover / hot upgrades
* **Priority**: Medium | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-39] TCP Fast Open for egress connections
* **Priority**: Medium | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-40] Buffer slab allocator / arena
* **Priority**: Medium | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-42] Kernel bypass for QUIC
* **Priority**: Low | **Status**: Research | **Track**: Future/Research & CI

### [TODO-43] HugePages support
* **Priority**: Low | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-46] Dynamic TCP congestion control and socket tuning
* **Priority**: Low | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-47] Memory-efficient load balancing ring
* **Priority**: Low | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-CI-1] CI connection matrix
* **Priority**: Low | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-15] egress_http_post phase boundary annotation
* **Priority**: Low | **Status**: TODO | **Track**: Future/Research & CI

### [TODO-173] Coarse-grained shared Tick Timer (Pingora Fast-Timeout style)
* **Priority**: Low | **Status**: TODO | **Track**: Future/Research & CI
* **Fix**:
  实现 10ms 或 50ms 粗粒度的定时器轮（Timer Wheel），支持多 Future Waker 的共享 tick 唤醒，避免在高并发连接下频繁增删 Tokio 计时堆的 CPU 锁争用。

---

## Ingress Security / Performance Tuning / L7 Protocol / Runtime & Scalability

### [TODO-148] Listener runtime ownership guard against regression
* **Priority**: High | **Status**: 原始 bug 已于 2026-07-26（PR #58）修复，见 `done.md`；本项仅保留仍开放的回归防护 | **Track**: Runtime & Scalability
* **Follow-up (open)**:
  Nothing prevents the next `tokio::spawn` in a config-apply path from re-introducing the listener-runtime-ownership bug (see `docs/reviews/2026-07-26/02-scalability-and-cpu-affinity.md` §2.0 for the original evidence). Worth a guard: assert at listener startup that the current runtime is the proxy runtime, or add an integration assertion that shutdown completes well inside the systemd stop timeout. Also worth re-measuring multi-core ingress scaling now that the path is no longer single-threaded — historical benchmarks ran with `CPUQuota=100%`, which masked the bottleneck entirely.

### [TODO-147] Chunked request bodies are rejected with 411
* **Priority**: Medium | **Status**: Open (capability gap introduced by the PR #58 smuggling fix) | **Track**: L7 Protocol
* **Problem**:
  `Http1Driver::read_request` frames request bodies by `content-length` only. Before 2026-07-26 a `Transfer-Encoding: chunked` request body was silently ignored and its bytes were parsed as the next request on the stream — a real smuggling primitive, now closed by rejecting TE bodies with `411 Length Required`. The rejection is correct but leaves a functional hole: `curl -T -`, `fetch` with a `ReadableStream` body, `docker push`, and `git http-backend` all send chunked request bodies and now fail.
* **Fix**:
  Implement chunked transfer decoding in the driver (parse chunk sizes, surface chunks as body frames, handle trailers) so unknown-length uploads are relayed instead of refused. Until then the limitation should be visible: consider a dedicated metric for TE-rejected requests so the gap shows up in operations rather than as unexplained client failures.

### [TODO-174] DDoS-Resistant Count-Min Sketch Ingress Rate Limiter
* **Priority**: Medium | **Status**: TODO | **Track**: Ingress Security
* **Fix**:
  在 Ingress Listener 阶段引入内存有界的 Count-Min Sketch 限流矩阵。对 Peer IP 进行快速多重 Hash 映射，避免在大流量防刷时因维护海量 IP Session Map 导致 OOM（内存溢出）。

### [TODO-150] In-depth TCP Autotuning & QUIC Buffer Tuning
* **Priority**: Medium | **Status**: TODO | **Track**: Performance Tuning
* **Fix**:
  在 Linux 生产环境下，禁用显式的 4MB TCP 缓冲大小，启用 Linux 的自适应 Autotuning，同时调优 QUIC 发送/接收窗口上限与流控制门限。

---

## 📋 2026-09-05 / 2026-08-21 复核批次（紧凑字段模板：复核状态 / 方案 / 依据）

> 以下条目沿用原评审的紧凑格式（未强行套用 Priority/Track 字段，避免编造未出现过的信息）。按原文件相对顺序保留；簇内条目已在上方「专题簇」小节给出跨引用。

### [TODO-20] Bytes::copy_from_slice -> split_to().freeze() (消减 HTTP 驱动拷贝)

**复核状态**：部分已实现；剩余优化须定位具体复制（2026-09-05）。

H1 body_prefix 已使用 split_to().freeze()，不能以全量替换 copy_from_slice 作为方案。剩余序列化/缓冲复制逐处测量，比较引用计数持有大底层 allocation 的内存成本；验证首包、分片、取消和回收。

### [TODO-27] 身份、会话恢复与 0-RTT（隧道身份/CA 簇）

**复核状态**：需求需分别定义（2026-09-05）。

持久证书改善身份稳定，但不自动开启 0-RTT。当前连接路径等待完整握手，未发现 early-data 接线。先实施 166；会话票据、服务端 resumption 状态与 early-data 重放策略需要独立协议设计和真实握手测试，不能以保存证书/票据声称完成。

### [TODO-108] EntryConnPool 定向 remove

**复核状态**：候选（2026-09-05）。

当前 push 接收调用方 shard_id 并取模，不能按 stable_id 哈希重算归属。若 profile 证明扫描成本显著，使用真实登记映射或在 handle/remove 中携带登记 shard，覆盖替换、重复注销与未知 ID。保持单 actor 写所有权，不以此强制实施 actor 分片。

### [TODO-135] Inflight 近似读语义（inflight/准入 簇）

**复核状态**：已确认（2026-09-05）。

pending 与 active 两次 relaxed 读取不是同一时刻快照，并发多个 promote/释放时不能承诺最多低估 1。用于负载均衡可接受近似时应明确用途；硬准入使用独立准确预算。109 若统一 total，先验证生命周期与通知，而非仅减少 atomic 数量。

### [TODO-36] Finish static dispatch cleanup

**复核状态**：候选，撤回全部静态分派目标（2026-09-05）。

只有 profile 确认 boxed peer 的分配/分派影响时才比较 enum、泛型与现有 trait object；同时评估代码体积、编译成本和扩展边界。没有测量依据不重构整条管线。

### [TODO-85] Listener reconcile 验收

**复核状态**：原同步阻塞描述过时（2026-09-05）。

当前已有异步 listener 管理。保留实际验收：端口变更/失败绑定/并发 reload/取消期间的资源回收与 generation 发布顺序；不重新造 AsyncListenerReconciler。

### [TODO-CR-AUDIT-5] 配置容量算术与组合预算

**复核状态**：原三项乘法风险已过时（2026-09-05）。

当前 client 容量有上限 262144，旧 max_streams × connections × 2 表达式已不存在。剩余工作是跨连接、stream、buffer、UDP 队列的可解释内存预算，不能继续按旧表达式报告漏洞。与 153/142/CR-AUDIT-6 统一有效配置验收。

### [TODO-CR-AUDIT-6] 资源配置组合校验

**复核状态**：部分已实现（2026-09-05）。

QUIC VarInt/idle 转换已有错误返回及边界测试，不再按 unwrap panic 记录。后续校验真实跨字段预算、实现上界与有效值可观测性；协议合法不代表内存可承受。不要机械规定 shards <= connections 而不检查调度语义。

### [TODO-153] UDP 容量配置与生命周期预算

**复核状态**：已确认（2026-09-05）。

三个硬编码上限确实存在，但只暴露数字不能保证资源有界。保持默认值，统一校验单连接 session、全局 session、排队 envelope 的组合预算；明确 permit 从何时取得，到出队、超时、取消、连接关闭何时释放。与 TODO-142 共用预算原则而非万能 limiter；并发压测验证峰值、释放和公平性。详见设计 §8–9。

### [TODO-154] Server registry actor 失败传递

**复核状态**：已确认，原自动恢复方案撤回（2026-09-05）。

现有 actor 操作有 fail-closed 防护；release 配置 panic=abort，catch_unwind 不能提供生产 panic 恢复。即使 unwind，外部读快照也不足以证明可重建完整写侧状态。推荐 owner 追踪非预期退出并使进程失败，由外部 supervisor 重启；readiness 不可被迟到更新恢复。覆盖正常关闭、channel 异常与 release 子进程故障。与 165 共用失败契约，不共享业务 actor 状态。详见设计 §7。
* **关联（review R15，2026-09-23）**：TODO-188（client 侧单连接 Fatal 级联杀死整池）与本项 server 侧的失败传递原则一致，可复用同一设计语言（明确 owner、fail-closed、外部 supervisor 重启）。

### [TODO-157] UDP 编码所有权与分配优化

**复核状态**：候选，撤回栈缓冲零分配承诺（2026-09-05）。

payload Vec、序列化 AlignedVec、转 Bytes 的复制链存在；Quinn 排队发送需要拥有数据，任务栈缓冲不能直接复用给未完成发送。先测 PPS、分配与排队内存；评估拥有 AlignedVec 的 Bytes owner，验证依赖能力后再实现，暂不改 wire format。同时按协议与连接 MTU 较小值限长，区分单包 TooLarge 与连接失败，检测 UDP 截断。验收含丢包、限长、队列饱和与会话继续工作。详见设计 §9。

### [TODO-158] 统一 host 匹配并去除冗余 unsafe

**复核状态**：已确认冗余；零分配收益待测（2026-09-05）。

当前 canonicalizer 已分配 String，随后 ASCII 栈拷贝和 from_utf8_unchecked 并未消除该分配。先随 177 统一 request authority 与配置 pattern 的语义，使用一个 &str 匹配核心移除重复扫描及 unsafe；保留非 ASCII、端口忽略与 wildcard 语义。借用/Cow 优化在 profile 后单独验证，不再叠加栈拷贝。详见设计 §5。

### [TODO-159] RoutingInfo 首包写入优化

**复核状态**：候选；额外 flush 判断不成立（2026-09-05）。

当前 send_message 有组包分配和 write_all，但没有原记录所称额外 flush。先测每流组包和首包延迟，再比较合并缓冲与 vectored 写；vectored 必须正确处理短写、跨 slice 推进、零写与取消，不能调用一次就假定全部发出。保持线协议与首包顺序。详见设计 §9。

### [TODO-161] Sniff 策略配置需求

**复核状态**：候选；每连接 detector 分配判断不成立（2026-09-05）。

SniffRuntime::new 保存策略和借用的静态 detector 集合，不会每次构造分配列表。只有实际需要可配置 detector 顺序/策略时才从 bootstrap 注入不可变配置，覆盖歧义协议优先级与慢速输入；不要为虚构分配引入插件 registry。预留 token 字段单独依据真实调用方决定去留。

### [TODO-162] 按不变量拆分 control_client

**复核状态**：已确认维护成本；增量实施（2026-09-05）。

提取 revision policy、LKG adapter、watch transport 与 apply/security orchestration；保持单一 apply owner 负责 listener、token fence、generation 发布顺序。模块默认私有，不让多个子模块各自发布状态。先等价移动，再在 175/178 对应提交修改契约；不让全面重构阻塞确定的缓存 bug。return_buffer 只在保持池上限/淘汰行为前提下简化。详见设计 §3、§11。

### [TODO-163] ctld admin framing 与业务适配边界

**复核状态**：已确认，P1（2026-09-05）。

本地 socket 有权限和超时保护，现有 CLI POST 带 Content-Length: 0，不能宣称正常 CLI 已被截断。但解析器接受重复/非法长度、缺失长度，并有 header_end + length 溢出风险。推荐 ctld 私有 admin adapter 使用 httparse 处理语法及有界 framing 状态机；mutation 接收类型化请求，保留幂等指纹和 410 行为。拒绝重复 CL、TE、溢出，POST 强制 CL；保留 256KiB/5s 上限、单请求后关闭。覆盖逐字节输入、超时、尾随数据及非法请求零 mutation。为何暂不引入完整 Hyper server 见设计 §6。

### [TODO-165] Client pool actor 失败传递

**复核状态**：已确认，撤回局部快照重建（2026-09-05）。

EntryConnPool 的 JoinHandle 未由运行时监督，pool 被 TCP、UDP、pool service 多处持有；替换局部 actor 不会自动修复旧 Arc 引用。release abort 与 154 相同。推荐进程 owner 收到不可恢复 actor 退出后取消服务并非零退出，外部重启；不承诺透明恢复已有流。验证 channel 关闭、旧 handle 拒绝服务、正常退出不误报及 release 故障。详见设计 §7。

### [TODO-166] Tunnel QUIC 稳定 TLS 身份（隧道身份/CA 簇，含原 TODO-152）

**复核状态**：已确认，P1（2026-09-05）。

server 当前无条件生成临时 localhost 身份，client 已支持 CA/server_name。推荐 bootstrap 加载经验证的 tunnel cert/key，身份配置独立于 QUIC 流控参数；仅底层安全文件读取与 ingress PKI 共享，不能复用 ingress CA 单例。显式 files/development 模式、错误不降级；样例同步迁移。真实握手验证 SAN、信任链、错配和重启一致性；依赖 176 的安全材料能力。详见设计 §1。

**原 TODO-152（不安全 TLS 状态可观测性）并入内容**：横幅不能修复缺乏稳定身份的问题。随本项同步迁移样例与严格验证配置，并记录有效 insecure 状态与固定低基数指标。显式开发模式可告警，生产配置不可静默 fallback。验收覆盖启动配置、错误 CA/SAN 与重启后身份一致。

### [TODO-167] H1 header 上限明确返回 431

**复核状态**：原伪挂起结论不成立；状态码改进待实施（2026-09-05）。

实际 httparse 超过 64 个 header 返回 TooManyHeaders，并非 Partial，当前驱动返回 400；已有探针证实 64/65/128 边界。保持 64 上限，按解析错误映射 431，不以 CRLFCRLF 猜测 Partial 原因，也不直接扩大栈数组。头字节上限只计 header，不把首包 body 混入；验证 Complete 和 Partial 两条边界、分片与大 body 前缀。详见设计 §5。
* **关联（review R12，2026-09-23）**：TODO-185——嗅探阶段（未建立连接前）header 超限走的是静默降级 TCP 透传，与本项已建立连接后的 431 语义不同层次，需评估一致性。

### [TODO-168] H2 错误作用域与实例安全失效（H2 sender/缓存簇，含原 TODO-68）

**复核状态**：已确认，P1（2026-09-05）。

内层 send 错误清 sender，外层 H2c/TLS 还会移除路由 sender cache；外层仅按 QUIC stable_id 判断，旧请求失败可能删除同一 QUIC 上的新 H2 实例。缓存为 downstream connection 局部，重建 H2 不会重新做 QUIC ALPN。推荐 lib 返回请求/H2/QUIC 失败作用域，server 共享转发策略持有明确 SenderLease；外层 Arc 身份比较和内层 CAS 均保留。失效与重试独立，继续限制安全方法+空 body，总 deadline/permit 覆盖完整请求。用真实双流 RST、GOAWAY、driver 退出和迟到旧错误证明行为，不能仅 mock 错误字符串。详见设计 §4。

**原 TODO-68（Ingress request lifecycle convergence）并入内容**：已有错误后失效逻辑，但任意请求失败都驱逐 sender 并非最终契约。共享转发策略同时表达失败作用域、实例身份、安全重试、总 deadline 与 body 完成；H2c/TLS 接入同一规则，保留协议适配差异。详见优化设计 §4。

### [TODO-169] TCP↔QUIC relay 核心收敛

**复核状态**：已确认（2026-09-05）。

bridge 与 relay 的 TCP 核心重复；收敛为一个私有实现，旧入口暂作薄 wrapper。保留泛型 TLS 路径，不能为统一签名丢掉 TCP 专用 into_split。首包目前未计入返回统计，等价迁移先保持；计数语义如需改变另作明确行为提交。测试双向内容、首包顺序、半关闭、背压、错误取消和统计。详见设计 §8。

### [TODO-170] 结构化阶段 outcome

**复核状态**：已确认（2026-09-05）。

PhaseOutcome 丢失错误类型，server 从字符串倒推指标。推荐在真正阶段终止边界产生有限错误类别与 source，日志 adapter 映射固定标签；保留展示文本，明确插件接口迁移。错误作用域、阶段和重试 attempt 不应混为一个枚举。body 尚未完成不能提前宣告成功；验证文案变化不改指标、取消归类和基数上界。详见设计 §8。

### [TODO-171] watch 监听安全默认与远程信任（watch信任模型簇）

**复核状态**：已确认，P1（2026-09-05）。

非回环无 token 会放行，README 示例会引导该配置；回环也不是同机用户级鉴权。bootstrap 校验得到 WatchEndpointPolicy，默认回环，非回环 token 缺失/空白拒绝启动；同步 README/配置样例。远程 bearer 明文仍不安全，采用 TLS 或受信安全隧道并验证远端身份。监听修复可先实施，不等待 178，但不能把它标为完整远程安全。测试 IPv4/IPv6 回环、通配绑定、空 token 与握手失败。详见设计 §2。
* **跟进（review R2，2026-09-23）**：watch 通道当前是明文 TCP（`watch.rs:49,90`、`control_client.rs:459`，两目录零 tls/rustls 引用），`watch_token`、完整路由拓扑、token 哈希均明文传输。修复方向：复用 `infra/pki.rs` 加 TLS，建议 mTLS——与本项"远程 bearer 明文仍不安全"的结论一致，作为具体落地动作纳入本项范围，不单独立项。

### [TODO-172] Server HA 关联登记与可用路径

**复核状态**：需求确认；推荐分阶段方案（2026-09-05）。

保留 server↔group/client 关联视图需求。稳定 server_id + incarnation + event_seq + 租约，周期全量 reconciliation 修复事件丢失；presence 与 routing revision/hash 分离，ctld 重启为 unknown。第一期 group 级视图；client 级身份必须先定义认证来源。client 双目标各自 ServerSession/pool/退避/egress config，不能共享最后一次 LoginResp 覆盖的全局规则，也不能一台 fatal 取消全部目标。LB 必须能按 group 导流，或保证两个 server 均有目标 group；视图本身不提供可用路径。UDP 会话固定目标，存量流不承诺迁移。完整 151 不是前置；client 接收 LoginResp 而非 ctld Snapshot ACK。故障矩阵、预算和取舍详见设计 §10。

### [TODO-175] 完整 generation 身份贯穿 H2 缓存（H2 sender/缓存簇，含原 TODO-52）

**复核状态**：待实施（2026-09-05）。

P1。RuntimeGeneration 包含 epoch，但 H2c/TLS 缓存仅使用 sequence。authority A/1 切 B/1 可复用旧路由/sender。推荐不可变 GenerationKey(epoch, sequence)，请求固定 generation，正负路由与 sender 缓存均接线；限制容量且旧请求完成不可淘汰新代。无需全局清缓存广播。长 H2c 连接跨 epoch 同 sequence 的真实测试是验收门槛；静态键冲突已确认，完整网络复现尚待实现。详见设计 §3。

**原 TODO-52（H2 请求级 generation 与缓存）并入内容**：每个请求固定一个 RuntimeGeneration，连接可以长存活但新请求必须看到新 generation；不能将 RoutingSnapshot 永久固定在整条 H2 连接。缓存用完整 epoch+sequence，包含负缓存及迟到请求的淘汰规则。详见设计 §3。

### [TODO-176] CA 加载失败不得覆盖已有身份（隧道身份/CA 簇）

**复核状态**：待实施（2026-09-05）。

P1，已复现。RootCa::load_or_generate 遇到已有损坏文件会生成并覆盖。拆成只读验证加载与显式初始化，失败字节不变；初始化排他创建，不以两次 rename 冒充成套原子更新。共享材料能力而非共享 ingress/tunnel 信任域。覆盖缺单文件、损坏、权限、错配和初始化竞争。详见设计 §1。

### [TODO-177] H2c authority IPv6 一致性

**复核状态**：待实施（2026-09-05）。

P1，已复现。[::1]:8080 经 split(':') 得到 [，无法匹配已有 IPv6 route。引入明确的请求 authority 与配置 host-pattern 解析边界，共享 canonical host 匹配，保留端口忽略/大小写/非 ASCII 当前语义。请求侧不接受配置 wildcard。覆盖 URI authority/Host 来源、IPv6 括号、端口与非法输入。详见设计 §5。

### [TODO-178] 显式配置 authority reset（watch信任模型簇）

**复核状态**：待实施（2026-09-05）。

P1，策略缺口已确认。当前每次 watch reconnect 都允许不同 epoch 的完整 Snapshot，重连并不等于管理员授权换 authority。推荐绑定已认证控制端，并提供持久、幂等的 expected_old_epoch + target_epoch reset 操作；不同 epoch 默认保留 LKG 并报告异常。迁移需允许合法新 authority 恢复，防止无限拒绝；先修独立缓存键 175。覆盖迟到快照、断连重启、重复 reset 和错误身份。详见设计 §2。
* **跟进（review R2，2026-09-23）**：同 TODO-171，watch 通道明文 TCP 的修复（TLS/mTLS）与本项的 authority reset 授权模型是同一信任边界的两面，实现时应一并设计"谁能触发 reset"与"reset 请求本身如何被认证/加密"。
