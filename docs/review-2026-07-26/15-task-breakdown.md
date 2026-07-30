# 2026-07 审阅后续实施任务拆分

> 本文按当前 review 分支工作树的实际代码状态重新整理 `docs/review-2026-07-26/` 的后续工作。
> “已实现”表示代码已在当前工作树完成；“待验收”表示仍缺少故障注入、长稳、跨进程或性能证据。

## Changes review follow-up

- Admin mutation 的业务错误现在使用 `AdminErrorKind`，HTTP status 只在 CLI/socket adapter 层映射。
- Dial9 单文件仍为 512 MiB，但轮转总容量提高到 2 GiB；8K trace case 会校验 server/client 日志确认 runtime 已启动 Dial9。
- Admission cancellation test 使用 oneshot startup handshake，不再依赖固定次数的 `yield_now()`。
- CPU contract 增加 `load_cpu_count`，用于发现 load 侧只剩单核的最小 runner 问题。
- buffer 参数文档明确 H2 对 `http_header_buf_size` 和 `http_body_chunk_size` 不消费；T6 的实际收益仍需真实 runner before/after 数据。
- 直接 `tokio::spawn` 仅继续审计真正的长生命周期/请求路径任务；测试 task 和有明确生命周期边界的 H2 driver 不因观测而强行改写。

## 当前进度

| 领域 | 当前状态 | 说明 |
| --- | --- | --- |
| 控制面拓扑 | 已实现 | 正式拓扑为 `duotunnel-ctld → duotunnel-server → duotunnel-client`。 |
| YAML/SQLite 分层 | 已实现基础闭环，待故障验收 | source list 校验、YAML 解析/规范化、SQLite override/tombstone/clear、只读 `SqliteConfigSource` 订阅、同事务物化 routing/config_state/revision/hash、启动初始化、source-specific degraded 持久化和失败重试已实现；legacy `server_config` fallback、迁移标记和重复启动保护已补，SQLite 故障注入与跨进程验收仍待补。当前 coordinator 支持一个 YAML source + 一个 SQLite source，Etcd 等未来 source 仍需显式注册。 |
| Snapshot/Delta Watch | 核心协议已实现，待跨进程验收 | 已有 DTCP envelope、numeric wire version、实际 wire size、Snapshot/Delta、ACK、Resync、target hash 自校验、批量 apply、共享 ACK snapshot、连接预算、握手超时和 epoch authority-reset 边界；连接断线矩阵及 LKG 磁盘故障仍待验收。 |
| Admin mutation 幂等 | 已实现，待运行态验收 | 需要 `Idempotency-Key`；配置/revoke 响应可从 SQLite 重放；create/rotate 的 bearer token 只保存在进程缓存，SQLite 仅持久化脱敏提交标记，重启后同 key 返回 410，避免重复 mutation 和明文 token 落盘。 |
| RuntimeGeneration | 核心 apply 已实现，待故障注入 | 已有不可变 generation（含 epoch）、LKG preflight/durability、listener prepare/rollback、token fence 和 fence-on-uncertainty；CommitUncertain supervisor 状态和跨进程故障证据仍待补。 |
| 生命周期/readiness/UDP budget | 生命周期边界已收敛，待长稳 | server 单一 signal owner、异常 QUIC/listener cleanup、drain timeout、启动 listener gate、ctld/server 聚合 readiness 已实现；UDP 预算和 full relay admission 仍按 T8/T9 排期。 |
| crate/目录/CLI 命名 | 已实现 | 当前正式 crate、binary、环境变量和 CI 使用 `duotunnel-*` / `DUOTUNNEL_*`。 |
| CI 8K case | 拓扑已迁移 | 继续只测性能，不在 8K case 中加入热更新或 Delta 专项验证。 |
| M1 性能基线 | CI 口径已实现，真实 runner 待验收 | `bench-basic`、3K/6K case 和 8K trace 都记录绝对 k6 窗口、CPU contract、配置快照、dropped iterations 和 gate 报告；相同 runner 的重复运行、baseline artifact 和跨 runner 真实数据仍待验收。 |

## 性能与延迟范围

本轮任务中，直接涉及请求路径性能或尾延迟的只有 T5–T9，但性质不同：

- **T5 是测量基础，不是优化**：固定 cpuset、有效配置、采样窗口和 before/after 口径。
- **T6 是数据面参数接线**：buffer/relay/header/body 参数已经贯通，仍需用 T5 证明具体取值是否改善吞吐、p99 或 RSS。
- **T7 是函数级优化入口**：使用现有 dial9 trace、Criterion 和系统资源指标定位真实热点；不需要额外引入独立 profile 测试框架。每次只接受一个有 before/after 证据的局部改动。
- **T8 是容量与尾延迟治理**：registry 容量和跨 shard 选择影响排队、负载倾斜及 p99，不等同于正常路径加速。
- **T9 是过载时延控制**：admission 防止慢客户端/慢后端拖垮进程，不能把它当作正常路径的速度优化，也不能用一个全局计数器替代各协议资源预算。

Delta、配置 merge、协议清理和 RuntimeGeneration 主要改善控制面一致性、恢复和运维行为；它们不计入 8K 请求路径性能收益。8K case 继续只做性能测量，不执行配置 mutation 或 Delta 专项验证。

## P0：先完成控制面和运行时实现

### T0 · EffectiveConfig 原子提交与 schema migration

把 `config_layers`、物化 routing、`config_state` 和 `control_revision` 收敛到统一 SQLite transaction。source 变化必须经过 merge、normalize、引用/容量校验和完整 Snapshot hash；commit 成功后才更新内存 snapshot 和 watch channel。

同时增加显式 schema migration version，迁移失败不得启动 watch/admin 服务。旧 routing 表只能作为一次性迁移输入，不能成为新的隐式 authority。

依赖：无。T1/T2/T3 都依赖 T0。

### T0.1 · ctld 启动初始化 barrier

启动顺序固定为：数据库迁移 → 加载所有 source → merge/validate → 原子提交 EffectiveConfig/revision → 创建 ControlService → 启动 admin/watch → readiness=true。YAML 初始失败、SQLite 不可用或 schema 不匹配时不得报告 ready。

### T1 · Control wire envelope 与 Watch 状态机

目标：先实现固定 magic、numeric wire version、message kind、payload length 的 control envelope，再证明 Snapshot/Delta 实现不会因跳跃、重复、ACK 丢失或 hash 不一致导致 server 静默分叉。

范围：

- 首次 Snapshot、持续 Delta、ACK、Duplicate、Rejected、ResyncRequired；
- A→B→C 连续更新，以及 ACK 等待期间合并为 A→C；
- server 重启、ctld 重启、断线重连和 ACK 丢失；
- base revision/hash mismatch、target hash mismatch、同 revision 不同 hash；
- Delta 大于 Snapshot 时降级为 Snapshot；
- 使用实际 wire payload size 选择 Delta 或 Snapshot，并拒绝超过 message limit 的事件；
- ctld 生成 Delta 后先在本地 apply、validate 并校验 target hash；
- Snapshot 路径与 Delta 应用后的 canonical hash 一致。

验收：server 最终只接受完整且 hash 正确的 generation；任何 gap/corruption 都进入明确的 resync 或 rejected 状态，不产生部分配置。

依赖：T0。建议拆成 `feat(control): add wire envelope` 和 `test(control): cover watch state machine` 两个提交。

### T2 · Source manager 与 YAML/SQLite Merge

目标：实现 source list 的明确校验和分层配置语义，并用 ctld 重启、watch 和 admin API 级别测试保证行为。未知 source、重复 SQLite source、priority 冲突都必须显式报错；YAML hash 与解析必须使用同一份 bytes。

范围：

- YAML 新增/修改/删除；
- SQLite upsert、tombstone、clear override；
- SQLite 同 key 覆盖 YAML，YAML 被覆盖时不产生 effective revision；
- YAML 删除后无 override 的资源消失，clear 后 YAML 资源重新出现；
- token 只由 SQLite 管理，rotate 在一个 SQLite transaction 内完成；
- 无效 YAML 保留最后有效 layer/effective config，并进入 degraded；
- SQLite 不可用时运行态保留当前配置，重启时按启动策略失败；
- 空 ingress、只有 egress、只有 token 等初始化边界；
- 旧数据库迁移后 routing 表按 SQLite override 解释。
- host/key normalization、重复 key、tombstone invariant 和引用关系校验；
- effective hash 统一定义为 canonical `ConfigSnapshot` hash，routing/source hash 使用不同字段名。

验收：每次 effective hash 未变化不产生 server revision；变化时 Snapshot/Delta 内容、revision 和 SQLite 状态一致。

依赖：T0。建议拆成实现提交和 `test(ctld): verify layered config lifecycle`。

### T3A · ConfigApplyCoordinator 与 revision/epoch 状态机

统一 candidate generation、apply id、完整 `ControlRevision`（包含 epoch）和 `CommitUncertain` 状态。不同 epoch 不得走普通 Apply，只能通过显式 authority reset 的完整 Snapshot 接受；Delta 不得跨 epoch。

### T3B · Listener prepare/commit/rollback

所有 listener 先 prepare，任一失败释放全部 prepared resource；commit 后不可逆失败进入 `CommitUncertain`、readiness=false 和 forward-fix，不伪装成成功回滚。旧 listener、worker 和过期 completion 必须有 owner、revision 和 deadline。

### T3C · TokenFenceLease 与全路径 admission fence

将 token revoke 的 `Fenced → Draining → Closed` 语义覆盖 login、reverse stream、UDP session 和 server-initiated stream。ACK 前必须进入 Fenced，QUIC close 只是后续 drain，不是 fence 本身。

### T3D · LKG/durability failure semantics

在 runtime side effect 前完成 LKG 编码、大小和 hash 校验；commit 后持久化失败不回滚已运行 generation，而是进入 `durability_degraded` 并阻止错误恢复旧 authority。LKG 选择必须校验同一 epoch。

### T3E · Lifecycle/shutdown ownership

只保留一个 signal owner；ConfigApply、listener shutdown、QUIC 异常退出和所有 child task 使用统一 lifecycle owner。进入 Draining 后 readiness=false 且禁止新 work；所有异常退出都走 cleanup/join 路径。

### T3F · RuntimeGeneration 故障与恢复验收

目标：在 T3A–T3E 完成后，证明 apply 失败不会发布半代配置，也不会让旧 generation 在安全边界上重新开放。

范围：

- 新配置 schema、引用、端口冲突和证书错误；
- listener bind/pre-bind 失败、旧 listener drain 超时、快速 A→B→C coalesce；
- token revoke/rotate 与 listener apply 同时发生；
- generation 构建失败、磁盘满、LKG 损坏和 LKG 双代恢复；
- worker panic、component unexpected exit、reload 与 shutdown 交错；
- 应用失败后 readiness、config fence、旧 generation 和后续 retry 状态。

验收：失败时无部分发布；安全 fence 不被错误解除；成功 retry 后能恢复；无法恢复时 readiness 明确失败且无孤儿 worker。

依赖：T3A–T3E。建议独立提交 `test(server): fault-inject runtime generation apply`。

### T4 · M0 Rollout/长稳验收

目标：补齐文档中 M0 代码闭环之后仍缺的系统级证据。

范围：

- watch 跨至少 1000 次 revision；
- 10 次 reload 与长连接并行；
- 断网、ctld 重启、server 重连、磁盘满和进程异常退出；
- 多 client group 的 revoke/rotate；
- 跨版本矩阵只验证当前协议的明确拒绝/恢复行为，不恢复旧业务 V1/V2 分叉。

验收：输出运行日志、最终 effective hash、应用/拒绝原因、generation 和资源上限指标；作为单独 CI/集成测试 job，不混入 8K 性能数字。

依赖：T0、T1、T2、T3A–T3F。

## P1：建立可信性能基线，再做优化

### T5 · CI 采样窗口与有效配置遥测

对应 TODO-140。

- 将 benchmark 进程、echo、load generator、collector 的 CPU 分配写入 case；
- 记录 p50/p95/p99/p99.9、完成率、错误率、CPU/RSS/FD/context switch 和 UDP drop；
- 启动日志输出最终 buffer、QUIC/TCP window、worker/shard、admission 和 queue 限制；
- 固定 H1/H2/L4/UDP 场景矩阵和结果 artifact；
- 8K 仍只做性能测量，不执行配置 mutation。

当前实现：`benchmark-env.sh` 在 cgroup v2/isolate 模式下为 server、client、load/echo 分配
不重叠 `AllowedCPUs`，写入 CPU contract；FRPS 使用 server CPU、FRPC 使用 client CPU，bench-basic 的 collector 绑定到 load CPU 且由 EXIT trap 清理；`bench-tool.py snapshot/attach/gate` 保存脱敏配置
快照、进程/cgroup/环境信息、分位数和 dropped iterations，并可对显式 baseline 做 p95/p99/p99.9、
RPS、错误率和 dropped-iteration 回归判定；`bench-basic`、`run-bench-case.sh` 和
`run-trace-8k.sh` 已统一使用该路径；isolate 模式会清除 scope 的 `CPUQuota`，runtime
按进程 `Cpus_allowed_list` 计算 worker/shard 上限。真实 GitHub runner 仍需验证 cgroup/systemd 权限、
物理核分配和重复运行变异系数；没有 baseline artifact 时不声称完成 before/after 优化证据。

验收：同一 case 在相同 cpuset 和有效配置下可复现；任何优化前后都有同口径 before/after 数据。

依赖：可与 T0–T4 并行；最终性能优化必须依赖 T5。T5 不依赖 T4。

### T6 · 贯通实际 buffer 配置

对应 TODO-141。

relay、HTTP header/body、QUIC stream sniff 和相关连接路径已统一接入解析后的 buffer 参数；保留默认 wrapper，配置值现在贯通 bridge、generic TCP/TLS relay、client entry、server egress、H1 parser/body reader 和 ProxyEngine。focused tests 已证明非默认值抵达 copy reader 和 H1 driver 配置。当前 Hyper H2 builder 仍使用协议库自身的 framing/buffering 策略，不假装支持独立 header/body buffer 调参；checked budget、reload 双代峰值和性能 before/after 仍属于后续 T5/T8/T9 验收，不在本 task 内改变默认值。

依赖：T5；建议独立提交，避免和其他热路径优化混合。

### T7 · Dial9-gated 热点治理

对应 TODO-145、TODO-144、TODO-137、TODO-143。TODO-145/hotpath-rs 不作为前置依赖，现有 dial9 负责第一轮 CPU/task/scheduler 定位。

当前 8K trace 已有 dial9 采集能力；本轮没有把 dial9 扩展到 basic、3K 或 6K。变化集中在 T5 的 CPU/采样/gate 口径，8K 的 dial9 覆盖范围保持不变。

顺序固定为：

1. 先用 dial9 trace、Criterion 和系统级资源指标确认热点；
2. 每次只做一个局部优化；
3. 用 T5 的吞吐、分位、RSS、CPU 和错误率复测；
4. 未证明瓶颈的多 Endpoint、自定义 runtime、H2 sender pool 和 UDP wire 重构不启动。

候选顺序：buffer/relay 接线 → UDP encode/copy/session → H1 scratch/body → H2 sender → metrics/DNS。每项都必须能回滚，并保留 profile artifact。

依赖：T5、T6；每个候选项独立立项，不整体启动 T7。

## P1/P2：单独排期的容量与 LB 任务

### T8 · Server registry 容量与跨 shard 选择

包含 TODO-146 和 review 08 的选择分片问题。

- 先补 active/available/exhausted 指标和 checked capacity；
- 按当前决策，4096 上限先作为容量治理与可观测性问题，不盲目扩大；
- 在多 shard benchmark 中验证同一 group 的 client 是否被错误集中到单 shard；
- 只有 profile/测试证明后，再调整选择算法或容量结构。

checked capacity、active/available/exhausted 指标和耗尽时不驱逐已实现；跨 shard correctness 与公平分布 benchmark 仍依赖 T5，多核扩展前必须完成 correctness test。

### T9 · 分层 active-stream admission

对应 TODO-142。global/group RAII controller、显式 Global/Group 拒绝原因和并发测试已实现；connection-local 与 protocol-specific guards 仍保持独立。实际生产接入仍待定义各协议的预算域、路由后 group identity、response-body 生命周期和拒绝语义。不能把 `max_pending_streams` 误用为 active relay 上限，也不能在 permit 生命周期未覆盖完整请求/relay 前接入生产路径。

当前已安全接入一个明确资源域：每个 QUIC client connection 的 UDP session。`UdpSessionManager` 用独立的 `AdmissionController` 替换原 per-connection session semaphore，`SessionEntry` 持有 RAII permit 直到 session 移除；进程级 UDP semaphore、UDP queue semaphore 和其他协议预算仍保持独立。创建排队、三秒操作超时、连接/空闲淘汰、QUIC shutdown 和失败路径均通过 entry drop 释放 permit。

HTTP request、raw relay、reverse stream 和 UDP queue 仍是 deferred，不把它们混成一个错误的生命周期计数器。每个 deferred 域必须先定义 owner、取消/超时释放、拒绝语义、指标和 success/reject/body-or-relay-completion/cancellation/timeout/shutdown/retry 测试，之后才能接入。T9 当前应标为“UDP session production integration + other domains deferred”，不能标为全量 active-stream admission 完成。

阈值和默认策略仍依赖 T5 的慢客户端/慢后端基线；本次 UDP 接入复用已有 `MAX_UDP_SESSIONS_PER_CONNECTION` 安全上限，不改变默认值。

### T10 · HttpFilter/LB 扩展地基

仅在产品继续需要通用 L7 LB 能力时启动：先统一 `LoadBalancer` seam 和每请求 filter，再按需求增加 XFF/Forwarded、重试预算、outlier、最终用户认证等。当前内网穿透定位下不作为 M0/M1 阻断项。

依赖：T3、T5；产品范围确认后排期。

## 明确暂缓

- io_uring：已拒绝，不创建实现任务；
- 自定义 runtime、per-core、多 Endpoint：只有 T5 + profile 指向 endpoint I/O/锁才启动；
- UDP 高 PPS/大包和 server UDP ingress：当前产品范围不纳入本轮；
- TODO-151 按 tenant/server 下发不同配置：保留为独立需求，不混入全局 EffectiveConfig；
- 旧业务 V1/V2 codec：不恢复长期兼容分支，当前协议版本只使用数字 wire version，旧 codec 如有需要仅在迁移隔离模块中短期存在。

## 推荐提交顺序

```text
T1 control watch tests
  ├─ T2 layered config integration tests
  └─ T3 runtime fault-injection tests
       └─ T4 M0 rollout/soak CI

T5 cpuset + effective-config telemetry
  ├─ T6 buffer wiring
  ├─ T7 profile-gated optimizations
  ├─ T8 registry capacity/cross-shard correctness
  └─ T9 admission benchmark and implementation

T10 HttpFilter/LB extensions (product-scope gated)
```

第一批建议先做 T0–T3E，再做 T1/T2/T3F 验收。T5 可并行推进；T5 是后续所有性能结论的前置，不和控制面验收混成一个大提交。
