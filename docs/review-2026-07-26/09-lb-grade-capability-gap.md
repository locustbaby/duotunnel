# 顶级 LB 视角的能力缺口分析（2026-07-26）

## 背景

用户把 DuoTunnel 定位为对标顶级负载均衡器（LB）。这个定位是成立的：DuoTunnel
本质是一台 **QUIC 承载的 L7 反向代理 / 边界网关**——它做协议嗅探、vhost 路由、
在多个后端（client 连接 / egress upstream）间做选择、TLS 终止、协议转发。因此可以
用顶级 LB（nginx、HAProxy、Envoy、Cloudflare pingora、Google Maglev/Katran、
AWS ALB/NLB）的能力标准去衡量它。

前 8 篇文档已覆盖：热路径成本（01）、多核扩展（02）、io_uring（03）、代码质量
（04）、成熟度（05）、压测方法论（06）、安全（07）、1-to-many 选择缺陷（08）。
**本文回答用户的新问题**：从顶级 LB 角度**还有哪些维度需要考虑**，DuoTunnel 在
这些维度上现状如何。方法同前——以 HEAD 代码为准，每条附 `file:line`。

> **定位修订（2026-07-26 复审）**：目标场景是**企业内部服务，不暴露到公网**。
> 因此本文所有"关键度"按**内网 L7 LB 生产可用**重排：
> **XFF（后端要看到真实来源 IP）> 每后端可观测 > 认证（可选插件，默认关闭）**。
> 最终用户认证**不再是必做项**——默认不启用（保持当前零配置行为），只要求抽象层
> 能把它作为插件插进去（见 10 §6.1 / design/03）。技术发现与证据**一条未改**，
> 变的只是优先级与叙事。

## 问题陈述

1. 一台顶级 L7 LB **应当具备哪些能力维度**（给出一张可复用的能力清单/评估框架）？
2. DuoTunnel 在每个维度上**现状是什么**（有 / 部分 / 无，代码证据）？
3. 哪些缺口对"对标顶级 LB"是**关键路径**，如何排序补齐？

---

## 结论速览：LB 能力矩阵

评级：✅ 达标 / 🟡 部分（有基础但不完整）/ ❌ 缺失 / ⚪ 不适用。
"关键度" = 对"作为**企业内网** L7 LB 生产可用"的重要性（定位修订后重排；面向公网
才需要的能力——可信 TLS、访问者鉴权、抗 DDoS——一律降档为"可选/按需"）。

| 维度 | 顶级 LB 期望 | DuoTunnel 现状（证据） | 评级 | 关键度 |
| --- | --- | --- | --- | --- |
| **A 负载均衡质量** | P2C/最少连接、加权、一致性哈希/会话亲和、子集化(subsetting)、局部性/分区感知 | client 选择 P2C（但 08 的分片缺陷）；egress upstream 纯**轮询**（`upstream.rs:90-109`）；无权重/无亲和/无子集化/无局部性 | 🟡 | 高 |
| **B 健康与故障检测** | 主动探活（proactive）+ 被动异常剔除(outlier)，按错误率/连续 5xx 剔除，慢启动(slow-start) | egress：连接失败被动剔除 10s + 主动**恢复**探测（`upstream.rs:23-88`）；**仅按 TCP connect 失败**，不看 HTTP 错误率；无慢启动；client 侧仅 `close_reason` + 10s `purge_dead`（`supervisor.rs:233`） | 🟡 | 高 |
| **C 弹性（重试/熔断/自适应）** | 重试预算(retry budget)、每尝试超时、熔断、自适应并发、对冲(hedging) | 有固定 3 次重试（`h1/mod.rs:54`、`tls/mod.rs:148`）+ h2c→h1 回退；**无重试预算**（storm 风险）、无熔断、无自适应并发、无对冲 | 🟡 | 高 |
| **D 过载与公平** | 全局/每租户限流、优先级、公平队列、快速拒绝 | per-connection inflight 慢路径 + pending 上限；**无限流、无每租户公平**（一个 group 可饿死全体，TODO-142） | 🟡 | 高（内网仍需：防单 group 饿死全体，属**容量公平**而非防攻击） |
| **E 协议完备与防滥用** | H1 边界(100-continue/CONNECT/pipelining)、H2 防滥用(rapid-reset/CVE-2023-44487)、请求走私防护、HTTP/3 | H1 keep-alive/trailers ✅；**chunked 违规**（01§3.2）；**`Expect:100-continue` 会挂**（见 §5.3）；`serve_h2_forward` **禁用 max_streams**（`h2.rs:81`）；走私未防（07§4.7） | 🟡 | 高（**协议正确性**：chunked/100-continue）/ 中（**防滥用**：内网半受信可放宽上界） |
| **F L7 可观测** | 每后端 RED、每路由分位延迟、访问日志、X-Forwarded-For/Forwarded、trace 透传/request-id | Prometheus 计数（auth/请求/open_bi）；**无每后端 RED、无每路由分位、无 XFF/客户端 IP 透传、无 trace/request-id 注入**（见 §5.1） | ❌ | **最高**（内网定位下的第一优先：后端要看到真实来源 IP + 按后端归因） |
| **G 流量管理** | 加权灰度/金丝雀、镜像(shadow)、Header/路径改写、重定向 | 仅 host 改写（TLS）+ hop-by-hop 清洗；**无灰度/镜像/路径改写/重定向**；路由仅 vhost 精确+通配 | ❌ | 中 |
| **H 连接与服务发现** | 上游连接池、happy-eyeballs/IPv6、预热、动态发现(EDS/SRV)、DNS TTL | hyper H1/H2 池 ✅；DNS 缓存+轮询+stale（`dns_cache.rs`）；**无 happy-eyeballs/IPv6**（TODO-73）、**无预热**（TODO-30）、**无动态发现**（静态 config + ctld 推送） | 🟡 | 中 |
| **I 运维与生命周期** | 优雅排空、零停机热升级(fd 交接)、证书热重载、配置原子切换 | 配置热重载 ✅、ArcSwap 原子切换 ✅；优雅排空**不完整**（CR-AUDIT-21）；**无零停机升级**（TODO-37）、**无证书热重载**（TODO-99） | 🟡 | 中 |
| **J TLS/加密 grade** | SNI 路由、到后端 mTLS、cipher 策略、OCSP stapling、0-RTT/tickets、证书轮换 | SNI 路由 ✅、TLS 终止+动态签发 ✅、上游 TLS ✅；**无到后端 mTLS、无 cipher 策略面、无 OCSP、0-RTT 未持久化**（TODO-27） | 🟡 | 低-中（内网自签/内部 PKI 已够；**浏览器可信证书/ACME 非必需**） |

**一句话**：DuoTunnel 具备一台 L7 LB 的**骨架能力**（路由、选择、TLS 终止、协议
转发、被动+主动健康探测），但在**LB 质量（A/B/C/D）**与**L7 可观测（F）**这两组
"顶级 LB 的核心区分项"上只到"部分"或"缺失"。其中 **F（客户端 IP 透传 + 每后端
可观测）是最刺眼的功能性缺口**——它让隧道后的后端既看不到真实客户端、也无法按
后端归因监控，这是任何生产 L7 LB 的底线能力。**在"企业内网、不暴露公网"的定位下
它进一步升为第一优先**：内网后端的访问日志/审计/按 IP 灰度都依赖真实来源 IP，而
"防外部攻击者"那一类能力（访问者鉴权、可信 TLS、抗 DDoS）反而退居可选。

**内网定位下的优先级（明确排序）**：
**XFF（后端要看到真实来源 IP）> 每后端可观测 > 认证（可选插件，默认关闭）**。
LB 质量三项（§4.2 outlier / §4.3 重试预算 / §4.1 XFF·可观测）**优先级不变**，
且 §4.1 的两项升到最前；**最终用户认证默认不启用**，只要求抽象层能插入（10 §6.1）。

> 与既有文档的关系：A 的 client 选择缺陷见 **08**；D 的未认证/限流与 **07§3.1**
> 重叠、并入 **TODO-142**；E 的 chunked/走私见 **01§3.2 / 07§4.7**；本文聚焦
> **尚未被前 8 篇覆盖的 LB 专属维度**：egress LB 算法（A）、健康模型（B）、
> 重试预算（C）、客户端 IP 透传与可观测（F）、H2 防滥用默认值（E）。

---

## 4. 关键缺口深挖（按关键度）

### 4.1 F — 无客户端 IP 透传 / 无每后端可观测（最刺眼功能缺口）`[新发现]`

**现象与证据**：
- 全仓无 `X-Forwarded-For` / `Forwarded` / `X-Real-IP` 注入（grep 全库仅命中无关
  符号）。`sanitize_request_headers`（`http_utils.rs:75-111`）只**删除** hop-by-hop
  头，从不**添加**转发头。
- 真实客户端地址其实**已经拿到并跨隧道传了**：`RoutingInfo.src_addr/src_port`
  （`msg.rs:98-104`）在 ingress 建流时携带，client 侧 `handle_work_stream` 解析成
  `client_addr`（`handler.rs:24-28`）——但它只用于 `ProxyEngine` 的
  `Context.client_addr`（日志/选路），**从未写入转发给后端的 HTTP 头**。
- 可观测：`runtime/metrics.rs` 只有全局计数（auth/requests by protocol+status/
  open_bi）；**无按 upstream/后端的请求数·错误率·延迟（RED）**，**无按路由/vhost
  的分位延迟直方图**，**无 request-id 生成、无 trace-context（traceparent）注入**。

**根因**：数据面把 L7 转发当"字节/请求搬运"，客户端身份信息在 `RoutingInfo` 里止步
于选路与日志，没有下沉到"改写请求头"这一步；指标维度停留在进程级而非后端/路由级。

**影响**：
- 后端**完全看不到真实客户端 IP**（只看到 client 进程的本地连接地址）——后端的
  访问日志、地理策略、按 IP 限流、风控、审计**全部失效**。这是 L7 LB 的底线能力缺失。
- 无法回答"哪个后端在变慢/报错"——故障定位只能靠端到端指标，违反 TODO-140 的
  可归因诉求。

**方案**：
1. **客户端 IP 透传**：在做 L7 的一侧（ingress=client 的 `Http1Driver`/H2 转发、
   egress=server）注入 `X-Forwarded-For`（追加而非覆盖）+ `X-Forwarded-Proto` +
   可选 `Forwarded`（RFC 7239）。源数据现成（`RoutingInfo.src_addr`）。
2. **request-id / trace**：无 `x-request-id` 时生成一个并回填响应；透传/续接
   `traceparent`（W3C Trace Context）。
3. **每后端/每路由指标**：`upstream`/`route` 作为 label 的 RED + 分位直方图
   （注意 07/TODO-CR4 的热路径 metrics 开销——用低基数 label + 采样）。

**论证 / 备选**：
- XFF 追加而非覆盖——LB 惯例是 append，保留链路；覆盖会丢失多级代理信息。
- 用 `Forwarded`(RFC 7239) 还是 `X-Forwarded-For`？两者都发：XFF 兼容性最好，
  `Forwarded` 更规范；成本仅多一个 header。
- 信任边界：**必须在入口剥离外部伪造的 XFF 再追加**，否则客户端可伪造前缀
  （安全项，见 Corner Cases）。
- 指标 label 基数：per-upstream（有界，配置已知）安全；**per-host/per-client-IP
  会爆基数**，禁用；路由 label 用 vhost 规则名而非原始 Host。

**场景覆盖 & Corner Cases**：
- **TCP passthrough / WebSocket / 原始 TCP**：无 HTTP 头可注入——用 PROXY protocol
  v2（HAProxy 事实标准）传客户端地址，或明确文档化"仅 HTTP 模式透传 IP"。
- **XFF 伪造**：外部请求自带 `X-Forwarded-For` 时，入口必须先剥离/信任策略校验
  再追加真实源，否则被伪造。需配置"受信前置代理"名单。
- **H2/gRPC**：注入到 H2 header（伪头之外的普通 header），`:authority` 已被 TLS
  handler 改写，注意不要覆盖。
- **大量 header 已满**：注入前检查 header 数/大小上限，避免超出后端限制。

**取舍**：注入 header = 每请求少量额外分配与写入（~100ns 级）；换取 L7 LB 底线
能力。指标维度增加 scrape/内存成本——用有界 label 控制。

**收益 / 改动量 / 影响面**：功能性补齐（后端可见客户端、可按后端归因）；改动集中在
两处 L7 转发点（`Http1Driver` 请求构造 + server egress `forward` / H2 service）+
metrics label 扩展，~1-2 天；影响所有 HTTP 转发路径，需回归 header 正确性。

---

### 4.2 A/B — Egress upstream LB 是"轮询 + 仅连接级健康"，缺 LB 质量要素 `[新发现]`

**现象与证据**（`crates/duotunnel-core/src/proxy/upstream.rs`）：
- **算法 = 轮询**：`next_healthy()`（:90-109）用 `counter.fetch_add` 做 round-robin，
  跳过不健康项；**非** P2C、**非**最少连接、**无权重**。（对比：client 选择用的是
  P2C，两条 LB 路径算法不一致。）
- **健康 = 仅 TCP connect 结果**：`mark_unhealthy`（:23）在**连接失败**时调用，
  剔除 10s（`Instant::now()+10s`，:28）；同时 spawn 一个**主动恢复探测**任务
  （:32-73），每 2s 试连一次、最多 5 次、成功即提前恢复。
- **恢复 = 时间到期即自动健康**：`is_healthy`（:81-88）= `now >= expires_at`——
  即使探测没成功，**10s 一到就自动重新可选**，失败的话下次请求再剔除 → **flapping**。

**根因**：健康模型建立在"TCP 可连"这一最弱信号上；对"能连上但持续返回 5xx / 超时"
的后端（brownout，生产最常见的故障态）**完全无感**——它一直被当健康、持续接流。
恢复用固定 10s TTL 而非"探测成功才恢复 + 指数退避"，导致坏后端周期性 flap 回池。

**影响**：
- **灰度/异常后端无法被剔除**：返回 500 的后端照常轮询接流；
- **无慢启动**：恢复的后端立即拿满额 RR 份额（冷连接池/冷缓存被打满，thundering）；
- **无权重**：异构后端（大小机器）无法按容量分配。

**方案**：
1. **被动异常剔除(outlier)**：按"连续 N 次 5xx/超时"或"错误率 > 阈值"剔除，而非
   仅 connect 失败；错误信号从 `HttpConnector::request`/relay 结果回传。
2. **指数退避 + 探测确认恢复**：剔除时长随连续失败指数增长；**必须探测成功才恢复**，
   而非时间到期自动恢复。
3. **慢启动**：恢复后的后端权重在窗口内线性爬升。
4. **加权 + 可选 P2C**：与 client 选择统一到 P2C/加权最少连接。

**论证 / 备选**：
- 为何要 outlier 而非仅 connect 健康——生产故障多为 brownout（可连接但坏响应），
  connect 健康对此零覆盖，这是 Envoy outlier detection 存在的根本原因。
- 为何"探测确认恢复"而非"TTL 到期恢复"——TTL 恢复会周期性把坏后端放回，制造
  可预测的错误尖峰；探测确认 + 退避是 HAProxy/Envoy 的标准做法。
- 保留主动恢复探测（现有 :32-73 已有雏形）是对的方向，缺的是"退避 + 不到期不恢复"。
- 备选"纯主动健康检查"（周期探所有后端）：更强但更重；对隧道后端可先用
  被动 outlier + 恢复探测，主动全量探测作为可选增强。

**场景覆盖 & Corner Cases**：
- **全部后端不健康**：`next_healthy` 兜底 `self.next()`（:108）返回一个"明知不健康"
  的后端硬试——保留（fail-open 比无处可去好），但应记指标。
- **单后端 group**：剔除=无处可去，outlier 阈值需避免把唯一后端剔没（保留 fail-open）。
- **探测任务与过期状态**：`mark_unhealthy` 每次 spawn 一个 detached 探测（:32）——
  并入 TODO-96 结构化任务。`contains_key`(:25) 并不是正确去重：`is_healthy` 在 TTL
  到期后仅返回 true、不删除 entry；若主动探测持续失败，后续 `mark_unhealthy` 会因
  旧 key 仍存在而永久跳过，无法刷新剔除时间。应改为带 generation 的
  `Healthy/Ejected/HalfOpen` 状态机与 single-flight probe，详见 14 §5.2 / D2。
- **DNS 变化**：探测用的地址在 DNS 轮换后可能过期；恢复探测应重解析（现 :51-52 有
  lookup 分支）。
- **健康状态跨 snapshot 重建丢失**：`ServerEgressMap` 在路由热重载时重建
  `UpstreamGroup`（`crates/duotunnel-server/egress/mod.rs:26`）→ **unhealthy 状态被清空**，坏后端
  热重载后短暂重新接流。需在重建时迁移健康状态或接受一次探测窗口。

**取舍**：outlier + 退避 + 慢启动增加状态与复杂度；换取对 brownout 的真实防护。
可分档落地（先 outlier + 退避，再慢启动，再加权）。

**收益 / 改动量 / 影响面**：从"只防宕机"升级到"防 brownout + 平滑恢复"；改动集中在
`upstream.rs`（健康模型重写）+ 错误信号回传接线，~2-3 天；影响 egress 所有 HTTP
上游；需补 outlier/退避/慢启动/flap 单测。**优先级建议高于 A 的加权/亲和**——
brownout 防护是生产可用性的硬需求，加权是优化项。

---

### 4.3 C — 无重试预算：固定 3 次重试在后端故障时放大 3× `[新发现]`

**现象与证据**：`h1/mod.rs:53-103` 每请求最多 3 次尝试（失败即 unregister 换连接
重试）；`tls/mod.rs:147-225` H2 路径同样 `max_attempts=3`；`http_connector.rs` 有
h2c→h1 一次回退。**没有任何全局/时间窗口的重试预算**（retry budget）。

**根因**：重试上限是"每请求独立计数"，缺"全局重试率上限"这层保护。

**影响**：后端集体 brownout 时，**每个请求都重试到 3 次** → 对后端的实际请求量
放大到 3×，把濒死后端彻底打死（retry storm / 重试雪崩）——这是重试预算要解决的
经典失效模式。与 4.2 的"坏后端不被剔除"叠加会更严重（重试还是打到坏后端）。

**方案**：引入重试预算（Envoy 模型）：全局维护"重试数 / 总请求数"比值，超过阈值
（如 20%）时**停止新的重试**（首次请求仍放行）。每尝试独立超时。可选对冲（hedging）
留待 F 的分位可观测就绪后再评估。

**论证 / 备选**：重试预算 vs 固定次数——固定次数在稳态无害、在故障态是放大器；
预算把"系统级重试量"作为受控资源，是唯一能防雪崩的机制。熔断(circuit breaker)是
互补手段（按后端短路）；两者可先做预算（全局、简单）再做熔断（每后端、状态更多）。

**场景覆盖 & Corner Cases**：
- **幂等性**：当前仅以 body 是否为空决定能否重放，并不等价于幂等；空 body
  POST/PATCH/DELETE 仍可能在上游已执行后重试。预算之外还必须按 method、失败发生在
  pre/post-dispatch 的位置和显式 Idempotency-Key/策略决定，详见 14 §5.1 / D2。
- **重试与 unregister 叠加**：`h1` 失败即 `unregister` 连接（:97），重试换连接是对的；
  预算限制的是"换后还继续重试"的次数总量。
- **预算计数的一致性开销**：全局比值计数是热路径原子——用 per-core 近似聚合（同 02 K1）。

**取舍**：预算会让部分请求"少重试一次"即失败（fail-fast）——这正是防雪崩要的取舍。
阈值可配 + 指标可见。

**收益 / 改动量 / 影响面**：消除重试雪崩这一生产杀手；改动为一个全局预算门 + 接入
两处重试循环，~1 天；影响所有 L7 重试路径。

---

### 4.4 E — 客户端可达的 H2 服务未设显式防滥用上界 `[新发现，与 07 交叉]`

**现象与证据**：`serve_h2_forward`（`h2.rs:80-82`）显式
`.max_concurrent_streams(None)` **禁用**并发流上限；server TLS 终止的 H2
（`tls/mod.rs:230`）**未设**任何 `max_concurrent_streams`/header/keepalive 上界，
依赖 hyper 默认。

**根因**：H2 server 构建未按"面向不可信下游"的标准加固；`None` 明确关掉了主要节流阀。

**影响**：H2 rapid-reset（CVE-2023-44487 类）/ 流洪泛 / CONTINUATION 洪泛的暴露面。
`serve_h2_forward` 主要在 MITM/上游方向（半受信），但 `tls/mod.rs` 的 H2 是
**面向公网客户端**的——它的加固程度取决于 hyper 版本默认值，未显式设定即"看运气"。

**方案**：所有面向不可信方的 H2 server 显式设置：`max_concurrent_streams`（如 256）、
`max_header_list_size`、`keep_alive_interval/timeout`；**核对 hyper 版本是否含
rapid-reset 缓解**（reset 计数），不满足则升级或显式限制。移除 `serve_h2_forward`
的 `None`，改为有界值。

**论证 / 备选**：显式上界 > 依赖默认——默认值随版本漂移，安全属性不能"看运气"；
`None` 在面向不可信方时是明确错误。

**场景覆盖 & Corner Cases**：gRPC 长流需要较高并发流上限——上界要可配，避免误伤
合法高并发；内网受信部署可放宽。

**取舍**：上界过低伤合法并发、过高留滥用面；给可配默认（256）+ 文档。

**收益 / 改动量 / 影响面**：关闭 H2 滥用面；改动为两处 H2 builder 配置 + 版本核对，
~0.5 天。**并入 07 的安全批次**。

---

### 4.5 E — `Expect: 100-continue` 会导致下游挂起 `[新发现]`

**现象与证据**：`Http1Driver::read_request`（`h1.rs:103-261`）解析完 header 后**立即**
按 `Content-Length` 构造 body 流并开始读下游 body（:173-253）；全程**不检查
`Expect: 100-continue`、从不向下游发送 `100 Continue` 中间响应**。

**根因**：H1 driver 未实现 100-continue 协商。

**影响**：遵循 RFC 7231 §5.1.1 的客户端会**先发 header（带 `Expect:100-continue`）、
等 `100 Continue` 再发 body**；而 driver 在等 body、客户端在等 100 → **双向死等**
直到 `KEEPALIVE_IDLE_TIMEOUT`(60s，`http.rs:13`) 超时。大文件上传/很多 HTTP 客户端
库默认带此头。

**方案**：解析到 `Expect: 100-continue` 时，先向下游写 `HTTP/1.1 100 Continue\r\n\r\n`
再读 body（或按策略先探上游可用性再放行）。

**论证 / 备选**：直接回 100 最简单且合规；更严谨的是"转发 Expect 给上游、按上游
100/417 决定"——但对透明 L7 转发，先回 100 再转发是常见且安全的近似。

**场景覆盖 & Corner Cases**：`Expect` 值非 100-continue（如 `417` 预期）要按规范
处理；H2 无此机制（不受影响）；已在读 body 后到达的重复 Expect 忽略。

**取舍**：先回 100 可能让上游最终拒绝时 body 已被下游发出（浪费一次传输）——对代理
可接受。

**收益 / 改动量 / 影响面**：消除一类下游挂起；改动为 `read_request` 中加一次条件写，
~0.5 天；影响 H1 上传路径，需加 100-continue 单测。

---

## 5. 其余维度（部分/缺失，紧凑列出）

> 本节关键度已按"企业内网、不暴露公网"重排：**XFF > 每后端可观测 > 认证（可选）**。

- **最终用户认证（访问者鉴权）**：无（只认隧道 agent token，`token.rs`）。
  **在本定位下降级为"可选插件、默认关闭"**——内网服务本就不面向匿名公网访客，
  强制鉴权只会增加配置负担；对该项的**唯一要求是抽象可插入**：`ConnectionModule`
  （IP 策略，连接级，seam 已足够）+ 未来的 `HttpFilter`（OIDC/JWT，请求级，10§6.1）。
  即**不实现、但不能挡住实现**。详细蓝图见 `design/03-end-user-auth.md`（opt-in）。
- **A 会话亲和 / 一致性哈希 / 子集化**：无。有状态后端（需 sticky）当前不支持；
  大规模后端无子集化会导致每个 LB 与全部后端建连（连接爆炸）。属**功能扩展**，
  非缺陷；按需求引入 `hash`/`sticky` LB 策略。
- **D 限流 / 每租户公平**：无（`grep` 全库无 rate-limit 实现）。与 **07§3.1**
  未认证 DoS、**TODO-142** 分层 admission 同一治理面：入口全局预算 + 每 group 预算 +
  每连接预算三层，H1 拒绝 503。**高关键度**——内网定位下它的理由从"抗攻击"变为
  **容量公平**（一个 group 的暴增不能饿死其余租户），仍属生产底线。
- **G 流量管理**（灰度/镜像/改写/重定向）：仅 host 改写。灰度/金丝雀需"加权路由"
  （依赖 A 的加权）；镜像/改写/重定向为独立特性，按产品需求排期，非生产底线。
- **H happy-eyeballs / IPv6**（TODO-73）：egress connect 单地址顺序尝试，无双栈竞速；
  DNS 缓存有多地址轮询（`dns_cache.rs`）但 connect 不并发。中等。
- **H 服务发现**：静态 upstream + ctld 推送路由；无 EDS/SRV/DNS-based 动态发现。
  对"集中控制面"定位可接受，若要对标 Envoy 需 xDS 类接口。
- **I 优雅排空**（CR-AUDIT-21 / TODO-96）、**零停机热升级**（TODO-37，pingora 有、
  DuoTunnel 无）、**证书热重载**（TODO-99）：见 05/07 与 todo，运维维度补齐。
- **J 到后端 mTLS / cipher 策略 / OCSP / 0-RTT 持久化**（TODO-27）：上游 TLS 有，
  但无客户端证书（mTLS）、无 cipher 套件策略面、无 OCSP stapling。**内网定位下降为
  低-中**：自签证书 / 内部 PKI 已满足内网信任模型，ACME/浏览器可信不是必需；
  mTLS 若要做，也属"可选插件"那一档（同上条认证）。
- **E HTTP/3 面向客户端**：QUIC 仅用作**隧道传输**，不面向下游客户端提供 H3；
  若要做 H3 LB 是独立大特性。⚪ 视定位而定。

---

## 6. 实施顺序与依赖

```mermaid
flowchart TD
    subgraph 第一批: LB 底线能力（与安全/正确性同期）
        F1[4.1 客户端 IP 透传 XFF/Forwarded]
        F2[4.1 每后端/每路由 RED + 分位]
        D[D 限流/每租户公平 = 07§3.1 + TODO-142]
        H2H[4.4 H2 显式防滥用上界 = 07 批次]
        EXP[4.5 Expect:100-continue]
    end
    subgraph 第二批: LB 质量（依赖可观测基线）
        B[4.2 outlier 剔除 + 退避 + 探测确认恢复]
        SS[4.2 慢启动]
        C[4.3 重试预算]
        W[A 加权 + 统一 P2C]
    end
    subgraph 第三批: 特性扩展（按产品需求）
        G[G 灰度/镜像/改写]
        AFF[A 会话亲和/一致性哈希]
        HE[H happy-eyeballs/IPv6]
        MTLS[J 到后端 mTLS/OCSP]
    end
    08[08 修复 client 选择分片缺陷] --> W
    F2 -->|按后端归因数据| B
    F2 -->|分位可观测| C
    B --> SS
    W --> G
    T140[TODO-140 基线] --> B
    T140 --> C
```

**顺序理由**：
- **第一批（LB 底线）**与 01/07 的 P0 同期：客户端 IP 透传与每后端可观测是 L7 LB
  的底线功能，且 F2 的按后端数据是第二批 outlier/重试预算的**判据前置**；限流/H2
  加固直接并入安全批次。**内网定位下 F1/F2 是第一批里的第一名**（后端看不到真实
  来源 IP 是当前最痛的功能缺失）；最终用户认证**不在任何批次内**——它是默认关闭的
  可选插件，只需第一批的抽象为它留好插入点。
- **第二批（LB 质量）依赖第一批的可观测**：没有每后端错误率/延迟，outlier 剔除与
  重试预算无从设阈值（呼应 TODO-140）；加权/统一 P2C 需先修 **08** 的选择缺陷，
  否则在错的选择域上加权无意义。
- **第三批**是产品特性，按需求排期，非生产底线。

## 7. 验收

- [ ] 后端访问日志能看到真实客户端 IP（XFF 追加、伪造被剥离）；
- [ ] `/metrics` 有 per-upstream RED 与 per-route 分位；
- [ ] 注入 brownout（后端返回 500/超时）后，坏后端被 outlier 剔除且退避恢复，
  流量不再打到它（当前做不到）；
- [ ] 后端集体 brownout 下，对后端的重试放大 ≤ 预算阈值（当前 3×）；
- [ ] `Expect: 100-continue` 上传不再挂起；
- [ ] 面向公网的 H2 有显式并发流上界且核对了 rapid-reset 缓解。
