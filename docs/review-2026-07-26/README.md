# DuoTunnel 代码审阅与优化调研（2026-07-26）

> 方法论：**以当前 HEAD 代码为唯一事实来源**，不转述历史文档结论；每条发现附
> `file:line` 证据，并标注与 `docs/todo.md` 的关系（`[已追踪 TODO-xx]` /
> `[新发现]` / `[需确认]`）。分析场景锚定 CI 8000 QPS 压测（server/client 各
> 1 CPU cgroup 配额）。
>
> **文档结构**：每篇均按统一报告模板组织——**背景 → 问题陈述 → 结论速览 →
> （每条发现）现象与证据 → 根因 → 方案 → 论证/备选对比 → 场景覆盖 & Corner
> Cases → 取舍 → 收益/改动量/影响面 → 实施顺序与依赖**。

审阅覆盖：`tunnel-lib` / `tunnel-store` / `server` / `client` /
`tunnel-service` / `ci-helpers`（~21k 行 Rust）+ `../pingora` 实证对照 + CI 编排。

---

## 文档索引

| # | 文档 | 回答的核心问题 |
| --- | --- | --- |
| 01 | [热路径与瓶颈分析](./01-hotpath-analysis.md) | 8k QPS 数据通路逐段成本、正确性问题（含 UB）、优化优先级 |
| 02 | [多核线性扩展与绑核设计](./02-scalability-and-cpu-affinity.md) | 如何随核数线性扩展且延迟平稳？actor mode 是否合适？绑核怎么做？ |
| 03 | [io_uring 可行性评估](./03-io-uring-assessment.md) | pingora 用 io_uring 吗？Rust 生态约束？该不该切？ |
| 04 | [代码质量与抽象审查](./04-code-quality-review.md) | unsafe/死代码/重复/抽象边界/测试覆盖 |
| 05 | [项目成熟度评估](./05-maturity-assessment.md) | 是否达到成熟高水平网络应用？对标 pingora/frp 的差距 |
| 06 | [CI 压测方法论改进](./06-bench-methodology.md) | 4c 争抢根因、用例定位、度量口径、microbench |
| 07 | [安全性评估](./07-security-assessment.md) | 认证/DoS/密钥/走私/多租户隔离 |
| 08 | [Server 一对多扇出审查](./08-server-one-to-many-fanout.md) | 一 server 服务同 group 多 client：选择/LB 设计是否有缺陷？ |
| 09 | [顶级 LB 能力缺口分析](./09-lb-grade-capability-gap.md) | 对标顶级 LB 还差哪些能力（健康检测/重试/限流/客户端IP透传/可观测）？ |
| 10 | [LB 适配抽象充分性](./10-lb-extensibility-abstraction.md) | 09 的缺口能否"加适配项"补上？现有 trait/seam 够不够？缺哪些抽象？ |
| 11 | [透传模式进度（TCP/L7/UDP）](./11-passthrough-modes.md) | 三类透传实现到什么程度、还差多少、怎么补？ |
| 12 | [商业产品对比与缺口](./12-commercial-landscape-gap.md) | cloudflared/ngrok/frp 等对比，DuoTunnel 缺哪些能力、哪些该补 |
| 13 | [协议版本化与运维补遗](./13-protocol-versioning-and-ops-addendum.md) | 完整性补遗：线协议无版本协商（阻碍滚动升级）+ 供应链/日志隐私 |

---

## 执行摘要

**总评：≈2.9/5，"工程化良好的准生产项目"。** 架构选型先进（QUIC 红利真实存在）、
抽象骨架已达生产水准；但带着一条正确性尾巴（1 处 UB、优雅停机不完整、过载竞态）、
若干安全硬伤（未认证限流缺失、CA 私钥落盘无权限）、以及热路径测试盲区——这三项
决定了**当前不宜直接承载生产不可信流量**。补齐后可稳定到 3.5-4。

**关于线性扩展（用户核心关切）**：目标是 `QPS(N) ≈ N×QPS(1)` 且 p99 平稳。
**二轮修正后的结论**：主障碍是 **quinn 单 Endpoint 的收包（S1）——不是 tokio 调度器**。
核对 quinn 分工可知：**Connection 的处理本来就已并行**（每 Connection 独立 task），
单点只在 **endpoint driver 的收包**那一步；因此多 Endpoint 的意义是"并行收包"，
而非"让 Connection 上不同核"（02 §2.1）。tokio work-stealing 的争抢真实但量级
**小于** S1/S2/K1（02 §3.5）。
**推论**：先解 S1（Phase B 多 Endpoint）+ S2（hyper 池分片）+ K1（per-core 计数），
**三者都不需要更换运行时模型**；per-core runtime（Phase C）降为**最后手段**，
仅在上述落地后压测仍不线性时启动。
**actor mode 的判断不变**：管"写"是对的（registry/conn_pool 已用），不应推广到数据面。

**关于 4c 测不了 8k**：根因是**物理超订**（k6≥2核 + server1 + client1 + echo +
collector > 4 核）叠加 **CPUQuota 的 100ms 周期性冻结**。
**二轮修正（D-6）**：8k 用例的**目的不变**——它就是用来量最高 QPS 的，**不**改成
过载行为测试。要解的不是目标而是**数字不可信**：解药是 **cpuset 隔离**
（`AllowedCPUs` 替代 `CPUQuota`），并接受"4c 上测得的上限 ≠ 系统上限"，
真实容量数字迁到更大 runner。

**关于 io_uring**：不切。pingora 没用；Rust 生态（tokio/quinn/hyper 全 readiness
模型）使其"无法完全替换"的判断正确；QUIC 的 UDP 已被 quinn 的 GSO/GRO 批量化，
收益 5-15% 却需重写 I/O 层 + 部署受限（seccomp）。同等预算应投给绑核/per-core 化。

**关于 server 一对多（08）**：多 client 能注册、能故障切换，但**负载均衡在多核
（shard_count>1）下是坏的**——注册按全局轮询把一个 group 的 client 撒到各 shard，
选择却只从 group 哈希出的**单一 shard** 取（`registry.rs:282` vs `:308-311`），
导致约只有 1/N 的 client 承载流量，其余空转。**CI 因 shard_count=1 永远测不到**，
且这正是 02 多核化会立即放大的问题——必须在多核化前修。另有 slot 表硬编码 4096
的总连接天花板（TODO-146）。

**关于对标顶级 LB（09）**：DuoTunnel 具备 L7 LB 骨架（路由/选择/TLS 终止/转发/
主动+被动健康探测），但在**LB 质量**与**L7 可观测**上是核心短板：egress upstream
是**纯轮询 + 仅连接级健康**（对后端 brownout 无感，`upstream.rs:90-109`）、
**无重试预算**（后端故障时 3× 放大）、**无限流/每租户公平**、**无客户端 IP 透传
（XFF）**——隧道后的后端看不到真实客户端 IP，访问日志/风控/按 IP 限流全失效,
这是最刺眼的功能缺口。

**关于能否"加适配项"补上（10）**：插件体系在**连接级/选路/DNS/指标后端/协议分发**
抽象充分（加 impl 即可）；但**缺一层每请求 HTTP 过滤（HttpFilter）**——它参照
Pingora 却只做到连接级 `ConnectionModule`，没有 Pingora `ProxyHttp` 的请求/响应
hook。结果是客户端 IP 透传/header 改写/镜像/重定向/请求级限流**都无处插入**。
建议**把引入 `HttpFilter` 层作为一切 L7-LB 能力的公共地基优先做**，而非逐个打补丁；
同时把三处各写各的选择（server RR / client P2C / 本地 upstream）统一到 `LoadBalancer`
seam（08 的修复即落在这里）。

**关于三种透传（11）**：都已跑通，完成度参差——**TCP(L4)~85%**（最完整，只差
PROXY protocol 让后端看到客户端 IP）、**L7~75%**（多协议已通，带 chunked/100-continue
正确性尾巴 + 缺 L7 filter 层）、**UDP~60%**（最小可用原型，session 无上限/1200 硬
编码/每包分配等生产化问题未做）。

**关于商业产品对比（12）**：landscape 除 cloudflared/ngrok 外还有 frp、rathole、
inlets、zrok/OpenZiti、boringproxy、chisel、Tailscale Funnel 等。DuoTunnel 的缺口
**集中在"边缘产品层"而非"隧道传输层"**——传输用 QUIC 反而**领先**同类。缺的是
SaaS 边缘那一层：**最终用户认证/零信任、公网可信 TLS(ACME)+域名、限流/IP 策略/WAF、
流量检查、全球边缘/DDoS**。其中"安全暴露四件套"（最终用户认证、可信 TLS、限流、
LB 质量+客户端 IP 透传）是"能否替代 ngrok/cloudflared 去公网暴露"的分水岭，且大多
能落在 10 号规划的抽象上；全球边缘/DDoS 属自托管形态的结构性劣势，不必强求。

---

## 全部发现清单（按优先级）

### P0 — 正确性/安全，上线前必须闭合（不依赖压测/架构改造）

> **✅ 本批已全部实施并合入 main（2026-07-26，PR #58）。** 实施过程中经三轮对抗式
> review（并发/内存、HTTP 协议合规、安全/协商），又查出 8 项后续问题一并修复——其中
> 两项是本批自己引入的回归：未认证配额一度可被 64 个伪造源 IP 包确定性锁死；
> Content-Length 修复因 `MapFrame` 不转发 `size_hint` 而**完全没生效**。
> 另有一项超出原清单的重大发现：ctld 模式下 listener 被 spawn 到单线程 runtime，
> 既导致停机死锁（CI stop 92s → 1s），也让整条公网 ingress 无法多核（新增串行点 S0，
> 见 [02 §2.0](./02-scalability-and-cpu-affinity.md)）。
> 逐条落地细节见 [`docs/todo.md`](../todo.md) 各条目的 Outcome。

| 项 | 文档 | 证据 | todo 关系 |
| --- | --- | --- | --- |
| ✅ relay buffer 未初始化内存 UB | 01 §3.1 | `engine/copy.rs:17-44,127` | TODO-97 **已闭合** |
| ✅ 响应无条件 chunked + 204/304/HEAD 违规致 keep-alive 失步 | 01 §3.2 | `http_utils.rs:112-115`+`h1.rs:291,322` | **已闭合**（初版修复无效，见上方说明） |
| ✅ 未认证连接无限流 → DoS/槽位耗尽 | 07 §3.1 | `handlers/quic.rs:47`+`registry.rs:89` | **已闭合**（含地址验证前置） |
| ✅ CA 私钥落盘无权限控制 | 07 §3.3 | `infra/pki.rs:225` | **已闭合**（含 O_NOFOLLOW/属主校验） |
| ✅ 认证内部错误回传未认证方 | 07 §4.2 | `handlers/quic.rs:152-157` | CR-AUDIT-22 **已闭合**（+ `retryable` 字段） |
| ✅ open_bi pending 上限竞态 + 全局/单连接语义混用 | 01 §3.3 | `open_bi.rs:57-67`+`overload.rs:53` | TODO-80 **已闭合**；进程级兜底转 TODO-142 |
| ✅ 优雅停机不完整（漏 QUIC/stream/UDP/H2）+ 孤儿 spawn | 05 M1 | `client.rs:97`等 | CR-AUDIT-21 + TODO-96 **已闭合**（残留缺口见 todo） |
| ✅ 请求走私面（CL/TE 冲突未拒绝） | 07 §4.7 | `h1.rs:173-177` | **已闭合**；代价见 TODO-147（chunked 请求体被拒） |
| ✅ `Expect: 100-continue` 下游挂起（60s 超时） | 09 §4.5 | `h1.rs:103-261` | **已闭合**（其他 Expect 回 417） |
| ✅ 面向公网 H2 未设显式防滥用上界（rapid-reset） | 09 §4.4 | `h2.rs:81`+`tls/mod.rs:230` | **已闭合**；顺带修 TLS 侧 ALPN 分派缺失 |
| ✅ 线协议无版本协商 → 无法滚动升级/灰度（新旧混跑批量断连） | 13 §4 | `msg.rs:41,56,119`+`quic.rs:83` | **已闭合**（ALPN 世代化为 `tunnel-quic/v1`） |

### P1 — 性能/测量，达成可信基线与确定性优化

| 项 | 文档 | todo 关系 |
| --- | --- | --- |
| CI cpuset 隔离 + 度量口径修正（offset/分位/配置快照） | 06 §2, 02 §6 | 关联 TODO-140 |
| worker 绑核 + per-core 计数 | 02 §5 Phase A | **新设计** |
| client 每连接独立 Endpoint | 02 Phase B | 关联 TODO-24（client 侧可先行） |
| L7 三重 BoxBody 去重 + body read_chunk 零拷贝 | 01 §4.1/§3.4 | **新发现** |
| TcpParams 默认 None（启用 autotuning） | 01 §4.4 | TODO-105 |
| 死代码删除（forward_http 175行）+ 重复 helper | 04 §2.1 | 修正 TODO-137 对象 |
| criterion microbench（解锁 benchmark-gated TODO 群） | 06 §2.4 | 关联 TODO-145 |
| VhostRouter::get 零分配 + 去无收益 unsafe | 01 §4.3, 04 §1 | **新发现** |

### P2 — 架构/抽象，长期演进

| 项 | 文档 | todo 关系 |
| --- | --- | --- |
| per-core 运行模式（NoSteal 等价）**降为最后手段**（D-4），条件触发 | 02 Phase C | **新设计** |
| upstream 级 L4/L7 开关（原 passthrough vhost）**⏸️ 已搁置**（D-7） | 01 §4.2 | 仅记录需求 |
| 统一 Session 抽象（简化 H1 body reclaim） | 04 §3.2 A4 | TODO-77 |
| 配置去重下沉 + 统一参数校验 | 04 §2.2-2.3 | CR-AUDIT-6 |
| tunnel-lib 拆分（先 tunnel-proto，利于 fuzz） | 04 §3.2 A6 | TODO-83 |
| sniff/codec/udp fuzz 靶 | 07 §4.5 | CR-AUDIT-20（建议升优先级） |
| server 多 endpoint（eBPF CID steering） | 02 Phase D | TODO-24（维持研究门槛） |

### 一对多 & LB 能力缺口（对标顶级 LB，详见 08 / 09）

| 项 | 文档 | 证据 | 关键度 / todo 关系 |
| --- | --- | --- | --- |
| **client 选择分片错配**：一 group 流量集中到 1/N client（多核下），CI 测不到 | 08 F1 | `registry.rs:282` vs `:308-311` | **高·多核化前置**·新发现 |
| registry slot 表硬编码 4096（1-to-many 总连接天花板） | 08 F2 | `registry.rs:89` | TODO-146 |
| **客户端 IP 透传缺失（XFF/Forwarded）**：后端看不到真实客户端 | 09 §4.1 | `http_utils.rs:75-111`（只删不加） | **高·LB 底线**·新发现 |
| 每后端/每路由 RED + 分位延迟缺失 | 09 §4.1 | `runtime/metrics.rs` 仅全局 | **高**·关联 TODO-140/CR4 |
| egress upstream = 轮询 + 仅连接级健康（对 brownout 无感、无慢启动） | 09 §4.2 | `upstream.rs:23-109` | **高**·新发现 |
| 无重试预算（后端故障 3× 放大 / 重试雪崩） | 09 §4.3 | `h1/mod.rs:54`+`tls/mod.rs:148` | **高**·新发现 |
| 无限流 / 无每租户公平 | 09 §5(D) | 全库无实现 | **高**·并入 07§3.1 + TODO-142 |
| 加权/会话亲和/子集化、灰度/镜像/改写、happy-eyeballs、到后端 mTLS | 09 §5 | 见 09 | 中·特性扩展按需 |

### 需引入的抽象 & 透传/产品化（详见 10 / 11 / 12）

| 项 | 文档 | 现状 | 建议 |
| --- | --- | --- | --- |
| **每请求 HttpFilter 层**（承接 XFF/改写/镜像/重定向/请求级限流/最终用户认证） | 10 §6.1 | ❌ 缺失（`ConnectionModule` 仅连接级，无请求 hook） | **一切 L7-LB/产品化能力的公共地基，优先做** |
| 三处选择统一到 `LoadBalancer` seam（08 修复落此） | 10 §6.2 | 🟡 trait 在，2/3 选择绕过 | 统一 + widen Target（weight/inflight/health/key） |
| OutlierDetector + RetryBudget（借 filter outcome 通道） | 10 §6.3 | ❌ 健康/重试内联硬编码 | 抽组件 + 请求结果回传 |
| 收敛双指标路径到 `MetricsSink` | 10 §6.4 | 🟡 与 `runtime/metrics` 宏并存 | 统一 + 带 upstream/route label |
| TCP 透传：PROXY protocol v2（后端见客户端 IP） | 11 §4 | ❌ | 小改，配置开关 |
| L7 透传：正确性 + HttpFilter 承接 | 11 §5 | 🟡 ~75% | 依赖 01 正确性 + 10§6.1 |
| UDP 代理：session 上限 + MTU 大包 + 每包零拷贝 | 11 §6 | 🟡 ~60% MVP | 并入 TODO-144 |
| 产品化"安全暴露四件套"（最终用户认证 / 可信 TLS-ACME / 限流·IP策略 / LB质量+XFF） | 12 §6.2 | ❌ 基本缺失 | 若定位 ngrok/cloudflared 替代则必补；多落在 10 的抽象上 |

---

## 里程碑路线（详见 05 · 到达生产级的路径）

```
M1 正确性/安全闭合 ──▶ M2 可信测量+线性扩展 ──▶ M3 抽象收敛
   ✅ 已完成 2026-07-26   ◀── 当前所在          (可长期演进)
   (PR #58)              (性能可证明 ≥0.8 线性)     成熟度 ~4.0
   成熟度 ~3.5            成熟度 ~3.8
```

**M1 完成情况（2026-07-26，PR #58）**：P0 清单 11 项全部闭合，另修 8 项 review 追加问题。
遗留的已知缺口都已显式记录而非默认解决：无应用层 GOAWAY、反向 stream 无 drain 计数、
chunked 请求体被 411 拒绝（TODO-147）、metrics/healthz 端口仍无认证。

**M2 的第一步已被 M1 意外推进**：S0（listener runtime 归属）修复后公网 ingress 才真正
跑在多线程 runtime 上——但**这也意味着此前所有多核压测数字都不足以支撑扩展性结论**。
M2 的顺序因此不变且更紧迫：先做 [06](./06-bench-methodology.md) / [02 §6](./02-scalability-and-cpu-affinity.md#6-ci-4c-runner-的绑核配比立即可做解决争抢)
的 cpuset 隔离建立可信基线，再谈 Phase B（多 Endpoint）。

**顺序铁律**：M1 未闭合前，任何 benchmark-gated 微优化都不应启动（与 todo.md 的
evidence-driven 原则一致）；CI 可信基线（M2 首步）建立前，所有 CI 数字不可作为
优化判据。

---

## 决策记录（2026-07-26 二轮复审，已确认）

以下决策由作者拍板，**已回写进各文档**；本节是唯一权威索引。

| # | 决策 | 结论 | 影响的文档 |
| --- | --- | --- | --- |
| D-1 | **产品定位** | **企业内部服务，不暴露公网**。据此"安全暴露四件套"整体降级，全球边缘/DDoS 明确不做 | 09 / 10 / 12 / design/ |
| D-2 | **最终用户认证** | **默认不启用**，作为**可选插件**按需接入；只要求抽象层能插进去，不要求现在实现 | 09 / 12 / design/03 / design/README |
| D-3 | **主串行点判定** | **是 quinn 单 Endpoint 收包（S1），不是 tokio 调度器**；work-stealing 争抢量级小于 S1/S2/K1 | **02 §2.1 / §3.5** |
| D-4 | **Phase 顺序** | 由 A→B→C 改为 **B（多 Endpoint）优先 → A + B′ → 压测 → C（最后手段、条件触发）** | **02 §4 / §10** |
| D-5 | **自定义 runtime** | 可行且 **quinn/hyper 零改动**（~500-800 行）；A 与 C 重合、B 与 C 正交。若走 C，路径简化为 **B → C（吸收 A+S2+K1）** | **02 §3.5.1** |
| D-6 | **8k 压测用例** | **目的不变**（测最高 QPS），**不**改为过载行为测试；只做 cpuset 隔离让数字可信，并接受"4c 上限 ≠ 系统上限" | **06 §2.2** |
| D-7 | **L4/L7 开关** | **搁置**，仅记录需求。设计上开关应放在 **upstream(proxy) 定义**上（`mode: l4\|l7`），非 vhost 级 | **01 §4.2 / 11 §4.1.1** |
| D-8 | **ctld watch 鉴权** | **已有鉴权**（`watch.rs:87-89` + `subtle::ConstantTimeEq`），原"待确认"作废；仅剩"非 loopback 强制 token"的启动校验建议（低危） | **07 §4.6** |
| D-9 | **client group 规模** | 每 group 仅 **2–3 个 client**。TODO-146（4096 slot）**降级**为日志+指标；08 F1 的危害定性为**负载不均/单点瓶颈**而非可用性损失 | **08** |
| D-10 | **chunked 根因** | 是 **BoxBody 类型擦除后的保守 framing**，**不是**"H1 转 H2 所以必须删 CL"；`size_hint().exact()` 本可用。修复需 driver 记录原始 method 以判 HEAD | **01 §3.2** |
| D-11 | **UDP 范围** | 现状**只有 egress 方向**，server 侧无 UDP ingress listener——"对齐 TCP listener"是**新功能**。近期只补 session 上限；高 PPS/大包不做 | **11 §6** |
| D-12 | **io_uring** | **Rejected**。pingora 用的就是 tokio（非自定义 runtime），没用 io_uring 与 runtime 无关，是生态 readiness 契约所致 | 03 |

## 待与作者确认的问题（grill list）

> 下列问题中，#4/#6/#9/#11/#12 已由上表决策关闭，保留仅为追溯。

1. **passthrough vhost（01 §4.2）**：内网穿透场景是否普遍不需要按请求改写
   header/按 `:authority` 路由？若是，L4 快路径可让转发容量近似翻倍——这是否
   符合产品定位？（取舍：放弃到后端的 hyper 连接复用）
2. ~~**per-core 运行模式（02 Phase C）**~~ → **已由 D-4/D-5 关闭**：降为条件触发的
   最后手段；若走则路径为 B → C（吸收 A+S2+K1），quinn/hyper 零改动。
3. ~~**8k 用例定位（06 §2.2）**~~ → **已由 D-6 关闭**：目的不变，只做 cpuset 隔离。
4. ~~**ctld watch 鉴权（07 §4.6）**~~ → **已由 D-8 关闭**：已有 token 鉴权 +
   常数时间比较；仅剩"非 loopback 强制 token"的启动校验建议。
5. **兼容性红线**：正确性修复（chunked/CL 处理）会改变线上可观测行为，是否有
   现网客户端依赖当前 chunked 行为需要灰度？
6. **io_uring（03）**：确认放弃后，TODO-25 是否可从 "Deferred" 正式改为
   "Rejected（除非上游提供后端）"？
7. **一对多选择缺陷（08 F1）**：确认修复方向（选择跨 group 全 shard 做 P2C，保留
   突变分片）——是否接受为多核化的硬前置？单 group 预期最大 client 数量级（决定
   4096 slot 上限如何治理）？
8. **客户端 IP 透传（09 §4.1）**：后端是否需要看到真实客户端 IP？若需要，HTTP 用
   XFF、TCP passthrough 是否引入 PROXY protocol v2？受信前置代理名单如何界定
   （防 XFF 伪造）？
9. **LB 质量 vs 定位（09）**：DuoTunnel 是定位"内网穿透隧道"（后端可信、LB 质量
   要求低）还是"通用 L7 LB"（需 outlier/重试预算/限流/加权）？这决定 09 第二批
   的取舍与排期。
10. **HttpFilter 地基（10 §6.1）**：是否认可"先引入每请求过滤层作为公共地基、
    再逐一挂 XFF/认证/限流/改写"这一顺序？它与 TODO-77（统一 Session）合流，是否
    一并规划？
11. **UDP 生产化范围（11 §6）**：UDP 代理目标是 DNS/小包够用，还是要支持高 PPS/
    大包（>1200，需按 MTU 分片）？这决定 TODO-144 的投入量级。
12. **产品定位（12）**：目标是"更好的自托管隧道（对标 frp/rathole）"还是"自托管的
    ngrok/cloudflared 替代（需安全暴露四件套）"？这是决定 09/10/12 大批工作是否上马
    的总开关。全球边缘/DDoS 明确不做（自托管结构性劣势）？

---

## 附：本次审阅对 todo.md 的校正

- **TODO-137** 的优化对象 `egress/http.rs::forward_http` 是死代码（04 §2.1）——
  实际热路径是 `Http1Driver::write_response`，建议改写该条目对象。
- **TODO-80** 除已记录的 check-then-act 竞态外，还存在全局计数 vs 单连接阈值的
  语义错配（01 §3.3），修复方案需一并处理。
- **新增未被 todo 覆盖**：chunked 协议正确性（01 §3.2）、body 双拷贝（01 §3.4）、
  未认证连接限流（07 §3.1）、CA 私钥权限（07 §3.3）、请求走私防护（07 §4.7）、
  BoxBody 三重装箱（01 §4.1）、VhostRouter 分配+unsafe（01 §4.3 / 04 §1）、
  **client 选择分片错配（08 F1，多核 LB 失衡、CI 测不到）**、
  **客户端 IP 透传缺失 + 每后端可观测（09 §4.1）**、
  **egress 轮询+仅连接级健康、无重试预算（09 §4.2/§4.3）**、
  **`Expect:100-continue` 挂起、H2 未设防滥用上界（09 §4.5/§4.4）**。
- **TODO-142（分层 admission）** 应吸收 09 的限流/每租户公平（D 维度）与 07§3.1
  的入口全局预算，作为同一治理面。
