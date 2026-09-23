# DuoTunnel 文档地图

先看哪份、以哪份为准：

| 想知道 | 看这里 | 性质 |
|---|---|---|
| 项目现在什么状态、成熟度、风险 | [`status.md`](status.md) | 当前状态总报告（整体替换更新，不新建） |
| 接下来做什么、优先级 | [`todo.md`](todo.md) | **只放开放项**；顶部 S 表为执行顺序 |
| 做过什么、为什么关掉 | [`done.md`](done.md) | 已完成 / 已否决项总账 |
| 系统怎么工作 | [`spec/`](spec/) | 活规格，随代码更新，以代码为准 |
| 某项能力打算怎么建 | [`design/`](design/) | 前瞻设计，实施后在文内标注落地状态 |
| 某次评审的证据与结论 | [`reviews/`](reviews/) | 时间点快照，不回改；结论入账到 todo/done |
| 反直觉的网络/系统编程实践 | [`guide/`](guide/) | 原理与经验 |
| 历史材料 | [`archive/`](archive/) | 已被取代，只读，仅供溯源 |

## spec/ — 活规格

- [`overview.md`](spec/overview.md) — 设计目标、与 frp 对比、指标、设计原则（入门从这里开始）
- [`architecture.md`](spec/architecture.md) — 跨 crate 拓扑、数据面流程、模块地图（**索引中心**）
- [`architecture-guidelines.md`](spec/architecture-guidelines.md) — 编码与重构准则
- [`duotunnel-lib.md`](spec/duotunnel-lib.md) · [`server-runtime.md`](spec/server-runtime.md) · [`client-runtime.md`](spec/client-runtime.md) · [`duotunnel-ctld-runtime.md`](spec/duotunnel-ctld-runtime.md) · [`duotunnel-ctld-storage.md`](spec/duotunnel-ctld-storage.md) — 分 crate 规格
- [`parameters.md`](spec/parameters.md) — 超时 / 上限 / 缓冲参数总表

## design/ — 方案设计

- [`README.md`](design/README.md) — D1–D10 索引与依赖图
- D1 [HttpFilter](design/01-httpfilter-layer.md) · D2 [LB 质量](design/02-lb-quality.md) · D3 [最终用户认证](design/03-end-user-auth.md) · D4 [可信 TLS/ACME](design/04-trusted-tls-acme.md) · D5 [限流/Admission](design/05-rate-limit-admission.md) · D6 [客户端 IP 与可观测](design/06-client-ip-and-observability.md) · D7 [多 Endpoint](design/07-multi-endpoint.md) · D9 [运行时可靠性](design/09-runtime-reliability.md) · D10 [性能加固](design/10-performance-hardening.md)
- [`plugins.md`](design/plugins.md) — Ingress/Egress 插件化 v2（已落地，现行架构依据）
- [`optimization-design-2026-09-05.md`](design/optimization-design-2026-09-05.md) — 09-05 复核问题 F1–F8 的推荐设计

## reviews/ — 评审快照（新 → 旧）

- [`2026-09-23.md`](reviews/2026-09-23.md) — 全量代码 review（只看代码），R1–R19
- [`2026-09-05.md`](reviews/2026-09-05.md) — 代码与 TODO 独立复核，F1–F8
- [`2026-07-26/`](reviews/2026-07-26/README.md) — 专题评审系列 + 决策记录 D-1~D-12；任务进度见 [`15-task-breakdown.md`](reviews/2026-07-26/15-task-breakdown.md)

## archive/ — 历史

- [`review-history-through-2026-09-04.md`](archive/review-history-through-2026-09-04.md) — 历次评审纪要合集
- [`history-2026-04~06.md`](archive/history-2026-04~06.md) — 04~06 月优化提案与 wstunnel/pingora/rathole 对比（5 份合并）
- [`architecture-refactor-history.md`](archive/architecture-refactor-history.md) — Pingora 式重构：发现 → 方案 → 任务
- [`2026-05-29-review.md`](archive/2026-05-29-review.md) — 逐文件审查
- [`future_research_directions.md`](archive/future_research_directions.md) — 长期研究方向（MP-QUIC、io_uring、CoDel …）
- [`quic-topology-endpoint-actor-plan.md`](archive/quic-topology-endpoint-actor-plan.md) — 不做 endpoint actor 重写的决策记录
- [`review-2026-07-26/`](archive/review-2026-07-26/) — 07-26 系列中已定案或被取代的分析（01 热路径、03 io_uring、04 代码质量、05 成熟度、06 压测方法、10 LB 抽象、12 商业对比、13 协议版本化）

## 维护约定

- 新评审写成 `reviews/YYYY-MM-DD.md`（单文件优先）；结论必须入账到 `todo.md`，不要在评审文件里维护进度。
- `todo.md` 条目关闭后移到 `done.md`，保留原 ID。
- `status.md` 整体替换更新，不另建新报告。
- 规格与代码冲突时以代码为准，并修正规格。
