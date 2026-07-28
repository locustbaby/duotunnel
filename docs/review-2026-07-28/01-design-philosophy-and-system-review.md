# DuoTunnel 设计哲学与整体代码复核（2026-07-28）

> 审查分支：`perf/optimize-hotpaths`。本报告将附件中的“分配器优先、无 GC 思维惯性、
> 干净热路径、无共享、单写者/RCU、数据导向、零拷贝、粗粒度合并”作为候选评估维度，
> 而不是必须服从的规则；最终取舍以协议语义、资源边界、尾延迟、故障隔离和可维护性为准。
> 本轮只做静态代码审查和方案设计，未修改业务代码，未运行测试或压测；`cargo check --workspace`
> 通过，但 `server/ingress/tunnel_handler.rs` 有一个未使用的 `warn` import 警告。

## 1. 结论摘要

当前分支已经形成较好的数据面基础：`ArcSwap` 快照、actor 写入、每连接 semaphore、
有界 UDP 队列、结构化错误、`IpAddr` 强类型和部分 Quinn `Bytes` 零拷贝路径都已落地。
但“无锁/零分配”并不自动等于更快，当前仍有三个必须先收口的设计问题：

1. **连接注册的 identity 约束不够显式**。当前生产入口在每条 QUIC connection 上生成新的
   UUID `ClientId`，因此在“ID 永不复用”的前提下，尚未证实存在实际 ABA；但 `ClientId` 是
   可从字符串构造的公开类型，registry 也没有拒绝重复注册或明确 compare-and-remove 语义，
   未来一旦复用 ID 仍会误删新连接。
2. **热路径仍叠加了业务层 allocator/cache 机制**。`copy.rs`、`PeekBufPool`、H2 sender
   cache、prefer-H1 cache 的复杂度、内存保留和锁/复制成本没有以同一基线证明收益。
3. **异步任务和 UDP 慢路径的 owner/backpressure 不够闭环**。未追踪的 H2 driver、按 session
   hash 的 worker HOL、DNS/bind/connect 在 worker 内串行等待，会把局部慢请求放大成连接级尾延迟。

因此，本分支不建议继续叠加更多微优化；应先固化 registry identity 不变量、任务治理和
可归因性能基线，再决定是否保留自定义 buffer pool、RCU cache 或更激进的零拷贝方案。

## 1.1 `main...HEAD` 变更复核补充

本节只记录相对 `main` 的新增风险，以及本次变更中确认可以保留的改动。

### P1（身份不变量需固化）— `client_to_group` 索引依赖 UUID 不复用

`server/ingress/handlers/quic.rs:339` 在认证后的每条 QUIC connection 上生成 UUID，并将其
作为 `ClientId` 贯穿 register/unregister；因此不同连接使用不同 key 时，旧 A 的 unregister
不会命中新 B。这里的风险不是 UUID 碰撞概率，而是未来调用方可能复用 `ClientId`，以及当前
registry 没有把“ID 必须唯一且只删除同一 identity”写成不变量。`server/ingress/registry.rs` 的
提前发布、无条件 remove 和重复注册行为仍需要收口：

```text
register(A) 使用同一 ClientId 重入 → 旧/新状态互相覆盖
外部构造重复 ClientId → stale unregister 可能命中新状态
PurgeDead 与重复 register 交错 → 无条件 remove 覆盖当前状态
```

当前实现可直接复用每条 connection 的 UUID `ClientId`，不必额外创建 `RegId`，但必须：
禁止重复注册同一 ID；所有异步事件携带产生事件时捕获的 `ClientId`；actor 只按该 key
compare-and-remove；失败回滚也只删除同一 key。若未来需要允许同一 ClientId 重注册，再升级
为 `ClientId + sequence` 或内部 `RegistrationId`。

### P1 — prefer-H1 TTL 过期后不会重新续租

`tunnel-lib/src/proxy/http_connector.rs:66-90` 的 `mark_prefer_h1` 发现 key 已存在就直接退出，
没有检查时间戳。key 过期后，下一次 H2C 失败仍不会更新 `Instant`，导致该 upstream 每次都
重新尝试 H2C，而不是再次进入 H1 TTL 窗口。应在 CAS 更新时区分“有效命中”和“过期条目”，
过期条目需要刷新时间戳或先删除。

### P1（性能证据待补）— ArcSwap 写入会复制整个偏好表

同一 `HttpConnector` 的新 key 写入会 clone 整个 `HashMap` 并 CAS 重试；在高基数 upstream
或故障风暴中，写侧成本从一次加锁修改变成 O(N) 分配/复制。读多写少且 key 基数低时这是
合理取舍；否则应保留分片/互斥写路径或使用受限的单写者更新。需要对失败风暴和 1024 条满表
场景采集分配次数、CPU 和 p99，不能仅凭“无锁”判定更快。

### P1（性能/内存证据待补）— QUIC 小包路径新增复制和分配

`tunnel-lib/src/engine/copy.rs:152-186` 曾把 `copy_buffered_then_finish` 改为每次读取后调用
`Bytes::copy_from_slice`，并绕过原有 pooled buffer。该策略在短小高频 payload 上增加复制和
分配，已在后续修复中恢复为与 shutdown 路径一致的池化 `BytesMut + write_all`，并移除了
`buf-pool` feature gate；后续验证重点转为同负载的 alloc、RSS 和 p99 对比。

### P1（生命周期/稳定性）— 时间缓存新增永久 OS 线程

`tunnel-lib/src/proxy/upstream.rs:24-42` 首次调用会启动永久 `time-updater` 线程，每 50ms
唤醒一次，且不受 runtime 或 shutdown 管理。它可能降低读侧时间计算成本，但也引入固定线程、
调度抖动和测试/嵌入场景的生命周期泄漏。应先比较直接 `Instant::elapsed`、runtime-owned
coarse clock 和当前线程方案；若保留，应明确线程失败、停止和进程级 owner 语义。

### P1（发布条件）— `RoutingInfo`/`UdpSessionKey` 的 wire schema 发生破坏性变化

`tunnel-lib/src/models/msg.rs:185-196` 将 `String` 改成 `IpAddr`，能消除重复 parse/format，
但 rkyv/帧布局不再兼容旧端。严格 MVP 的原子升级可以接受，但必须在握手或版本门禁中
fail-fast；不能让 mixed-version 连接静默进入数据面。

### P2 — `OpenWaitObserver` 从 boxed closure 改成函数指针

`tunnel-lib/src/transport/connection_handle.rs:7` 减少了热路径分配，但同时收窄了公开 API：
调用者不能再传递捕获环境的闭包。如果 tunnel-lib 只供 workspace 内部使用可以接受；若有外部
使用者，应保留泛型/trait 适配或在版本变更中明确记录。

### 本次变更中可保留的部分

- H2C 的 `OnceLock` 单 authority 和 `RwLock` 读写分离，语义不变且减少了不必要的独占锁。
- H2 HPACK authority 直接比较字节，避免了无意义的 lossy UTF-8 临时字符串。
- `IpAddr` 强类型化和 `ResourceGuard` 只在计数归零时唤醒等待者，方向正确；前者需配合
  wire version，后者需维持计数不变式。
- TCP wait observer 提取为函数指针属于低风险的分配优化，前提是 API 兼容性可接受。

## 1.2 争议项的方案再评估与最终取舍

前述部分问题来自之前的性能审查，并不意味着当前实现一定错误。本节按“语义正确性、当前
负载、资源边界、未来演进成本”重新判断，避免为了灵活性或无锁目标引入更复杂的结构。

| 项目 | 当前实现是否合理 | 主要边界 | 最终决策 |
| --- | --- | --- | --- |
| Registry actor + `ArcSwap` 快照 | 模型合理，当前依赖 ClientId 唯一性 | 写入串行化和快照复制有成本，但一致性更容易证明 | 保留模型，固化 ClientId identity、重复注册拒绝、prepare/commit 和失败回滚；不改成全局锁 |
| H2C `RwLock`/`OnceLock` | 对单连接、读多写少场景合理 | route/sender cache 仍有 clone、锁和淘汰成本 | 保留；只有 profile 证明读锁或写复制占比高才改 immutable map/分片 |
| prefer-H1 `ArcSwap<HashMap>` | 读热点、低频写入时合理 | 新 key 写入 O(N) clone；过期 key 当前无法续租 | 先修过期刷新；观察写入率和 key 基数，必要时改单写者或分片写路径 |
| `IpAddr` 强类型 wire model | 数据面建模和本地性能更好 | mixed-version 无法直接兼容 | MVP 原子升级可保留；滚动升级则必须增加版本/能力门禁 |
| 函数指针 wait observer | 当前只记录全局指标时最省分配 | 无法携带租户、路由、span 等调用上下文 | 内部 API 保留函数指针；公开库若需扩展，再提供显式 custom observer 适配层 |
| 永久 coarse-clock 线程 | 只有时间读取非常频繁且 profile 证明有效时才有价值 | 固定线程、50ms 更新延迟、无 shutdown owner | 默认不视为最佳实现；优先直接 `Instant` 或 runtime-owned clock，基准证明后再保留 |
| QUIC `Bytes` 混合发送 | 当前单向 relay 场景基本合理 | 小包多时复制/分配增加，大包慢消费者会 pin backing buffer | 保留混合思路，但把阈值和 pin budget 变成可测策略，不强制纯 copy 或纯 zero-copy |

### `Bytes` 小包复制与未来灵活性的详细判断

当前 `copy_buffered_then_finish` 的约束是：`AsyncRead` 单向读入、QUIC `SendStream` 单向发送，
不需要在发送后修改数据，也没有重试同一 chunk 的业务语义。在这个约束下，
`BytesMut::split().freeze()` 对大块数据可以把所有权交给 Quinn，避免额外复制；小块使用
`Bytes::copy_from_slice`，则可以立即清空并复用工作 buffer，避免慢消费者长期 pin 一个较大的
底层 allocation。这是一个合理的**混合策略**，不是“复制永远更快”。

未来确实可能需要可变 buffer 或更强的 chunk 控制，典型场景包括：

1. **内容变换**：压缩、解压、加密前处理、协议 framing、校验和或脱敏，需要在发送前修改
   bytes；此时应在 `freeze` 前保留 `BytesMut`，而不是让发送层拿到不可变 `Bytes`。
2. **重试/多路复用**：同一数据需要复制到多个下游、重试发送或延迟确认；此时 `Bytes` 的
   clone/slice 仍然可用，但必须有明确的引用计数和最大保留时间，不能无限 pin。
3. **批量与自适应分片**：根据 QUIC flow-control、RTT、下游速率或 CPU/带宽目标动态调整
   chunk 大小；固定 `buffer_size / 4` 阈值不够，需要运行时策略或配置。
4. **零拷贝输入源**：未来 reader 本身能提供拥有所有权的 `Bytes`/QUIC chunk 时，继续先读
   入通用 `BytesMut` 会多一次复制；应允许 `ChunkSource` 直接交付 immutable chunk。
5. **内存压力保护**：慢消费者、长连接或大响应导致多个 chunk 同时存活时，需要按连接/全局
   pin budget 降级为复制或暂停读取，而不是只按单个 chunk 大小判断。

这些场景并不要求现在放弃 `Bytes`。更稳妥的工业化接口是保留当前默认路径，同时抽象一个
受限的 chunk policy：

```text
ChunkPolicy {
  target_chunk_size,
  copy_threshold,
  max_pinned_bytes_per_stream,
  transform_hook / immutable-source fast path
}
```

默认 policy 仍可使用当前“低于阈值复制、高于阈值转移所有权”的实现；只有启用内容变换、
多路复用或内存压力降级时才选择其他策略。这样既保留热路径的简单性，也不把未来扩展锁死。
每种 policy 都必须记录 copy bytes、pinned bytes、chunk 数、alloc 次数和下游等待时间，
以数据决定阈值，而不是凭经验固定为四分之一。

小包的候选实现至少有三种，不能只比较“有无 zero-copy”：

| 小包路径 | 额外复制 | 外部 allocation | buffer pin | 适用情况 |
| --- | --- | --- | --- | --- |
| `write_all(&buf)` 后复用 `BytesMut` | 通常由 Quinn 写入内部发送缓冲 | 由 Quinn 决定 | 最短 | 小包占比高、下游慢、优先控制 RSS |
| `Bytes::copy_from_slice` + `write_chunk` | 业务层一次复制 | 每个 chunk 可能一次 | chunk 生命周期可控 | 需要 immutable chunk，且 allocator 对小对象表现好 |
| `split().freeze()` + `write_chunk` | 无业务层复制 | 复用原 backing allocation | 可能较长 | 大块数据、流控充足、追求吞吐 |

因此当前实现只能称为“有明确内存边界的可行混合方案”，还不能称为所有 workload 下的
最佳方案。基准应覆盖小包/大包、快/慢下游、短连接/长连接和 allocator 组合；如果
`write_all(&buf)` 在小包场景更低分配且 p99 不变，就应把小包分支切回它，而不是为了
形式上的 `Bytes` zero-copy 保留额外 allocation。

### 结论：哪些问题现在必须改，哪些不应提前改

- **现在必须改**：Registry identity 不变量/重复注册防护、prefer-H1 过期刷新、H1 非法目标 fallback、
  wire version 发布门禁。
- **先补证据再决定**：ArcSwap 全表复制、QUIC 小包复制、coarse-clock 线程、buffer pool、
  route/sender cache 的进一步无锁化。
- **当前可保留**：H2C `OnceLock`/读写锁拆分、HPACK 字节比较、归零时通知、内部函数指针回调。
- **未来扩展接口**：仅在出现内容变换、多路复用、零拷贝输入源或 pin budget 压力时，引入
  `ChunkPolicy`/custom observer，而不是现在就把所有路径改成高度可配置的抽象。

## 1.3 对《DuoTunnel 实施方案》的审阅

附件方案的优先级和总体方向正确，但以下细节需要在实施前修正，否则会把“修复 ABA”变成
新的竞态，或把“single-flight”误当成已经解决了 UDP 的跨 session HOL。

### Registry 方案需要三处结构性调整

1. **identity 必须随事件传播，不能靠当前映射推断历史身份**。当前每条 QUIC connection 都
   使用新的 UUID `ClientId`，因此可以直接复用它作为 connection identity，不必额外创建
   `RegId`。但事件必须携带产生时捕获的那个 `ClientId`；不能先按 ClientId 取“当前注册”再
   代替旧事件的身份。若未来允许 ID 重用，再升级为 `ClientId + sequence`。
2. **重复注册必须明确拒绝或定义替换语义**。在当前 UUID 不复用的生产入口下，actor 可以在
   `clients`/`group_conns` 已存在同一 ClientId 时拒绝 register；这样无需额外 generation，
   也能让 compare-and-remove 的 key 语义闭环。若要支持同 ID 重注册，才需要内部 sequence。
3. **索引发布和替换仍需事务化**。方案中“actor reply 成功后由 `register()` 写
   `client_to_group`”会让索引更新依赖外部 future 调度；更稳妥的是 actor 在 commit 时更新
   actor-owned index，或至少做带 ClientId 的 compare-and-insert。新连接插入同一
   `group/shard` 时，必须保存 predecessor 并在 commit 后无条件 retire 旧 handle。

另外，`pending_purge: AtomicUsize` 只有数量没有 identity，无法知道 mailbox 满时丢失的是谁；
至少需要 coalesced 的 `ClientId` 集合或带 identity 的有界 pending 队列。`PurgeDead` 也必须
按当前 ClientId 条件删除 map，不能在未来支持 ID 重用后无条件覆盖新状态。

方案示例中的 `reply.send(Ok(new_reg_id))` 还与现有 `Result<(), &'static str>` 签名不一致；应
如果复用 UUID `ClientId`，不需要把 `RegId` 加到返回值；但应明确 `ClientId` 是 connection
identity，并在 `SelectedConnection`/注销事件中保留同一个值。若方案选择内部 sequence，则
必须同步调整返回类型和所有调用方，不能只在伪代码层改变返回类型。

### H1 fallback 方案方向正确，但要覆盖调用边界

`try_new` 是正确的安全边界。实施时应同时修改唯一构造调用方、定义错误到协议响应的映射，
并确保 parse 失败不会继续创建 upstream 连接。更好的长期接口是让 prepare 阶段直接传入已
解析的 `Scheme/Authority`，避免每次连接重复解析；`try_new(String, String)` 可作为迁移层，
但不应再保留任何 localhost 默认值。

### prefer-H1 CAS 方案基本正确，但只适用于低写入率

附件给出的“有效 timestamp 直接 return，过期条目 CAS 刷新”修复了逻辑问题，可以保留。它仍
会在新 key 或过期 key 更新时 clone 全表，因此应保留 profile gate；如果故障风暴下写入率高，
再改为单写者或分片可变 map，不要为了避免锁而默认引入更大复制成本。

### coarse clock 的优先级应从 M0 降为 P2 性能决策

删除永久线程、回到直接 `Instant::elapsed()` 是合理的默认实现，但“10–30ns”不是跨平台保证，
不能写成无条件事实。应以目标平台 benchmark 决定；runtime-owned clock 只在 profile 证明
时间读取占比显著时引入。它不应阻塞 Registry/H1 等 correctness fix。

### buffer pool 已收敛为统一的 always-on 实现

此前 feature flag 仅用于 A/B，但造成 CI、bench 和生产构建路径不一致；当前已移除该 feature
及全部条件编译。`copy_buffered_then_finish`、`copy_buffered_then_shutdown` 和 `PeekBufPool`
均始终启用各自的有界线程本地池，避免小包额外分配和构建组合漂移。后续如引入 bulk zero-copy，
应作为独立语义 API，不得改变 L7 buffered relay 的默认路径。

### UDP Phase 1 方案目前不可直接实施

`std::sync::OnceLock::get_or_init` 不能承载异步 DNS/bind/connect；即便改用
`tokio::sync::OnceCell::get_or_try_init`，worker 如果仍 `await` 初始化 future，同一 hash shard
上的其他 session 仍会被 HOL 阻塞。single-flight 只能消除重复建连，不能自动提供跨 key 隔离。

可实施的最小闭环应是：

1. `DashMap<UdpSessionKey, Arc<SessionEntry>>` 做 identity 去重；
2. 首个 packet 只负责把建连请求投递到有界 create queue，并立即返回/排队，不能在 datagram
   worker 内等待 DNS/bind/connect；
3. 每个 `SessionEntry` 使用 `Notify`/共享 future 或 session actor 完成 single-flight，建连并发
   由全局 semaphore 限制；
4. 成功/失败回写 map 时做 identity 校验，失败后只删除当前 entry；
5. per-session pending queue、deadline、队列满丢弃和指标必须明确。

### wire version gate 应复用已有握手协商

当前代码已经存在 `PROTOCOL_VERSION`、`MIN_SUPPORTED_VERSION`、`Login.protocol_version` 和
`NegotiatedProtocol`。因此不应再平行增加一个重复的 `protocol_version` 字段。对于
`IpAddr` wire layout 变化，应选择：

- 严格 MVP：提升协议版本/最低支持版本，旧 peer 在握手阶段拒绝；或
- 滚动升级：增加 capability，只有 capability 协商成功才使用新 frame，否则走兼容编码。

测试重点应是“协商失败后不进入数据面”，而不是依赖旧 frame 被 rkyv 解码成失败；解析失败
本身不是可靠的版本门禁。

### 方案状态标记需要修正

附件实施顺序中 M0-A 至 M0-E、M3 标记为 `[x]`，但当前工作区代码仍能看到旧的
`client_to_group`、localhost fallback、永久 clock thread 和原有 wire model。除非这些修改已在
其他 commit 中完成，否则应改成 `[ ]` 或链接实际 commit，避免文档把“计划”误报为“已完成”。

### 1.4 QUIC connection identity 与 logical client identity 的边界

QUIC 本身确实有 transport Connection ID，但它的职责是让端点和中间设备路由数据包；连接迁移
时一个 QUIC connection 可以拥有多个 CID，CID 也会被轮换和 retire。它不是“认证后的 client
身份”，不应直接作为跨重连、跨进程或跨节点的业务 client key。

当前工程还暴露了 `quinn::Connection::stable_id()`，它是 Quinn 进程内用于日志/缓存等用途的
本地稳定编号，也不等同于线上 QUIC CID；适合作为本地观测或短生命周期 cache key，不应拿来
做持久化协议身份。

推荐明确拆成两层：

| 身份 | 生命周期 | 用途 |
| --- | --- | --- |
| `ConnectionId`（当前可由 UUID `ClientId` 承担） | 一次认证后的 tunnel connection | registry register/unregister、snapshot、inflight、日志和 compare-and-remove |
| `LogicalClient`（token/client group） | 可跨 reconnect 的业务身份 | 认证、授权、配置归属、配额和统计聚合 |

在当前实现中，每条 QUIC connection 在 `server/ingress/handlers/quic.rs:339` 生成新的 UUID，
因此复用 `ClientId` 作为 `ConnectionId` 是可行的，不必额外增加 `RegId`。但应把这一不变量
写进类型和 registry：重复 `ConnectionId` 注册必须拒绝；所有事件使用捕获时的原值；旧事件对
不存在的 ID 是幂等 no-op。若未来要让同一 ID 重注册，或需要 deterministic/non-random identity，
再引入 `ConnectionId + sequence`。

更长期的命名改进是把当前 per-connection `ClientId` 重命名为 `ConnectionId`，把 token/group
保留为 logical client，避免后续开发者误把“client”当作可跨连接的身份。

**当前 MVP 决策**：不新增第二个逻辑 `ClientId` 字段。registry 只需要一次 tunnel connection
的 `ConnectionId`；逻辑归属已有 `GroupId` 和 `token_hash`，足以完成选路、撤销和连接分组。
只有出现“同一逻辑客户跨多个 connection 统一限流/统计/踢下线/会话恢复”等需求时，才增加
独立的 `ClientPrincipalId`，并明确它与 `ConnectionId` 的一对多关系。

## 2. 从附件推导出的候选设计准则（非绝对规则）

附件中的原则适合作为假设，但必须补上边界条件：Rust 的 `Drop` 不保证物理页立即归还操作系统，
自定义池在大对象、跨线程迁移或特定 allocator 下可能有收益；RCU 也可能因复制大快照而比
分片锁更贵，零拷贝还可能延长大 buffer 的 pin 时间。因此每条准则都需要和当前 workload、
失败模式、部署拓扑及实现复杂度一起评估，不能凭语言或硬件直觉直接下结论。

适用于网络隧道系统的原则集合如下：

1. **Allocator sovereignty**：默认让 mimalloc/系统 allocator 管理小对象；业务池只有在
   profile 证明收益、且有 size class、容量上限、淘汰和 shutdown 语义时才启用。
2. **Share-nothing / per-core ownership**：热路径状态按 worker、connection 或 session
   切分；跨核共享只保留不可变快照、计数器或消息，不用全局锁协调每个请求。
3. **Single-writer + RCU**：配置、路由、连接索引由单写者提交不可变 generation；读者只
   加载快照，不持写锁，也不能观察半提交状态。
4. **Data-oriented locality**：连续数组、紧凑 key、cache-line 对齐和稳定 shard；避免
   每请求 String/HashMap/trait object 链，避免伪共享和无界集合。
5. **Zero-copy with bounded lifetime**：零拷贝必须同时满足 buffer 生命周期、流控和最大
   pin 时间；慢消费者不能长期占住大 buffer。小包复制有时优于切片保活大块内存。
6. **Backpressure is a contract**：每个队列、semaphore、socket/session 表都要有容量、
   满载策略、超时、丢弃/重试语义和指标；禁止“队列满就静默成功”。
7. **Structured concurrency**：任务必须挂在 connection/session/runtime owner 下，取消可
   传播，退出可 join；禁止无主 `spawn` 依赖“底层最终会自己结束”。
8. **End-to-end deadline**：解析、排队、建连、握手、转发、drain 使用分层 deadline；每一
   层都不得把超时无限延长成下一层的隐式等待。
9. **Generation fencing and idempotence**：register/unregister、reload、revoke、retry、
   LRU eviction 都必须携带 generation/identity，旧完成事件不能改写新状态。
10. **Failure isolation / bulkhead**：一个连接、session、DNS miss、H2 driver 或 metrics
    sink 的失败不能扩大成整个 registry 或 runtime 的错误；fail-closed 要有精确边界。
11. **Protocol truth and security boundary**：解析器严格拒绝歧义；wire schema 变更要 bump
    version/capability，认证/路由失败要返回稳定公开错误，内部细节只进结构化日志。
12. **Observability without hot-path tax**：指标 label 低基数、计数器无锁，日志字段避免
    `format!` 预先分配；必须能观测队列深度、容量耗尽、generation、任务存活和 drain 结果。
13. **Profile before cleverness**：以固定 cpuset、固定 allocator、固定数据集和同一编译
    参数采集 p50/p95/p99.9、完成率、alloc 次数、RSS、CPU cache miss、队列等待和 syscall；
    没有证据的微优化只能保留在实验分支。

### 2.1 共享状态与异步任务的方案比较

“共享状态”与“异步任务”本身不是反模式；网络服务通常需要它们来表达连接复用、流控、
单飞建连和配置热更新。真正需要避免的是没有明确 owner、提交顺序和容量边界的共享。当前
场景应按以下取舍，而不是统一改成无锁或统一改成 actor：

| 场景 | 方案候选 | 优点 | 代价/风险 | 当前建议 |
| --- | --- | --- | --- | --- |
| 路由/配置读多写少 | `ArcSwap` 不可变快照 + 单写者提交 | 读路径无锁、快照一致、回滚简单 | 更新时复制快照，超大表会有提交成本 | 保留；对大表测量复制成本后再考虑分片快照 |
| 连接索引高并发读写 | actor 串行提交 + 快照读 | 身份和 generation 易于证明，避免锁顺序死锁 | 邮箱延迟和单写者吞吐上限 | 保留模型，补齐 registration fencing 和满载策略 |
| 低频管理面、写冲突高 | 分片 `RwLock`/`DashMap` | 实现简单，更新局部化 | 读侧有锁/哈希，无法自动解决 ABA | 仅在 profile 证明 actor 成为瓶颈时采用，不能作为一致性修复 |
| H2/连接 driver | owner 管理的 `JoinSet`/`TaskTracker` | 取消、join、drain 顺序可验证 | join 需要预算，错误传播路径更复杂 | 关闭和 shutdown 必须纳入 owner；正常长生命周期可继续异步运行 |
| UDP session 建立 | worker 内直接 await；per-session task；session actor | 直接 await 最省调度；独立 task 隔离慢建连；actor 便于单写 socket | 独立 task 增加调度/内存；actor 需要 mailbox 和退出协议 | worker 只做短操作；慢 DNS/bind/connect 使用有界 single-flight，是否 actor 化以 profile 决定 |
| 短生命周期后台动作 | detached `spawn`；owner tracker；共享 worker | detached 调度开销低；tracker 可控；worker 可限并发 | detached 难回收和观测；tracker 可能拖慢 drain；worker 可能 HOL | 以资源归属和退出语义选型；有 cancellation、上限、指标和幂等清理时，detached 不自动构成缺陷 |

因此，本报告中的“异步任务风险”指的是**生命周期和背压未闭环**，不是异步模型本身错误；
“共享状态风险”指的是**身份/代际和提交语义不完整**，不是共享状态应被全部删除。

## 3. 当前实现中值得保留的部分

- `RuntimeGeneration`、`ArcSwap` 路由快照和 listener generation fence 已经接近 RCU/单写者模型。
- `ConnectionState` 使用 owned state 和 retire fencing，避免旧 inflight guard 修改替换连接。
- registry/client pool 的选择路径使用不可变快照，选择时不持久化读锁。
- per-connection pending semaphore、UDP 全局/连接容量和有界 worker queue 体现了明确背压。
- HTTP retry 已限制为安全且空 body 的方法，避免普通 POST/PATCH 的隐式重放。
- `RoutingInfo`/`UdpSessionKey` 使用 `IpAddr` 消除了重复 parse/format；但需要协议版本策略。

## 4. 新发现的问题

### P1（依赖 identity 不变量）— ClientRegistry 的 compare-and-remove 语义未固化

证据：`server/ingress/registry.rs:361-403,405-415,471-481`。

生产入口当前为每条 QUIC connection 生成新的 UUID `ClientId`，所以只要 UUID 不复用，
`unregister(ClientId)` 的 key 本身就是 connection identity，旧 A 不会删除新 B。真正需要
补强的是：`ClientId` 可被任意字符串构造，registry 没有重复注册防护，actor 也没有将“不存在
或已替换的 identity”显式视为幂等 no-op。

```text
old(A) visible → old disconnect starts unregister(A)
new(B) with same ClientId registers and becomes visible
actor processes stale unregister(A) → clients.remove(ClientId) / snapshot removes B
```

在未来出现重复 ID、测试注入或新的注册入口时，仍可能表现为新 tunnel 被摘除、路由瞬时 503
或 inflight slot 泄漏。当前 O(1) map 优化应固化 UUID ClientId 的唯一性，并对重复 register、
失败回滚、purge 和 unregister 使用 compare-and-remove；只有允许同一 ID 重注册时，才需要
额外 generation。

### P1 — 注册替换不是事务，容量耗尽会先摘掉旧连接

actor 在 `Register` 中先 `existing.handle.retire()`，再 `inflight_table.allocate()`。当容量不足
时直接返回错误，旧连接已经不可选，但旧索引和快照仍可能保留到 purge。替换操作必须满足
“新连接 prepare 成功后再 retire 旧连接”，否则短暂容量压力会放大成长期掉线。

### P1 — unregister mailbox 满载时 fail-close 整个 registry

`unregister()` 使用 `try_send`；队列满时调用 `fail_closed()`，关闭全部连接。高并发断线风暴
本来就是最容易触发 mailbox 满的场景，却被扩大为全组不可用。应区分“单条 unregister
丢失”和“actor 已死亡”，用 coalesced reconcile/purge 或异步 backpressure 替代全局 fail-close。

### P1 — H1 预解析失败静默回退到 localhost

`tunnel-lib/src/protocol/driver/h1.rs:30-43` 对非法 scheme 回退 HTTP，对非法 authority 回退
`127.0.0.1`。配置错误或协议边界错误因此变成错误目标连接，属于错误路由和潜在 SSRF 语义。
预解析应返回 `Result<Http1Driver>`，在 prepare 阶段拒绝无效目标；热路径只能使用已验证值。

### P1（生命周期闭环需证据）— H2 driver 是无主 detached task

`tunnel-lib/src/proxy/h2_proxy.rs:134-146` 使用 `tokio::spawn` 启动 driver，仅靠 QUIC stream
最终报错来退出。detached 本身可能是合理的长生命周期模型，但当前代码需要证明它受
connection/session cancellation 约束、退出时能清理 cache/sender/inflight，并且 shutdown/drain
有明确等待预算。若这些保证已由上层 owner 间接提供，则问题可降为可观测性和维护性；若没有，
则会在慢 driver 或异常 transport 下造成任务、sender 和资源计数延迟释放。

### P1 — UDP worker 将慢 session 放大成跨 session HOL

`server/ingress/handlers/udp_datagram.rs:156-185,275-335` 按 session hash 固定到 4 个 worker，
worker 内直接 await DNS、bind、connect 和 socket send。一个慢 upstream 会阻塞同 shard 的
所有 session；`to_vec`、编码和 `Bytes::copy_from_slice` 仍发生在每个数据包。

### P1（证据待补）— 自定义 buffer pool 的复杂度和保留上限未被证明合理

`tunnel-lib/src/engine/copy.rs` 和 `infra/peek_buf.rs` 叠加 TLS free-list、全局 CAS queue、
容量替换和 RAII wrapper。其收益依赖 worker 数、buffer size、mimalloc 配置和连接迁移；当前
还有固定每线程保留数、全局队列上限、undersized 丢弃等策略，可能造成 RSS 放大或更长指令链。
客户端/服务端使用 mimalloc，但 tunnel-service/library 场景并不共享同一 allocator 假设。

### P1 — 热路径仍有可消除的 String/Map 成本

- `client/ingress/handler.rs:17-22` 每条 reverse stream 用 `format!` 组装日志字段；即使
  日志级别关闭，也可能先产生 String。
- `tunnel-lib/src/proxy/http_connector.rs:52-54,175` 每个 HTTP 请求构造
  `scheme://target_host` String 查询 prefer-H1 cache。
- H2C/TLS 每连接的 route/sender cache 使用 `RwLock<HashMap>`，读路径仍有锁计数和 key clone。

这些不是立即 correctness blocker；是否值得重构要由分配/锁 profile 决定。若确实是热点，
再统一到 typed、pre-canonicalized key 和 immutable snapshot；否则保留当前简单实现反而更稳妥。

### P2 — 其他需要明确决策的项

- `RoutingInfo`、`UdpSessionKey` 从 String 改成 `IpAddr` 是 wire schema 变更；若不是严格的
  双端原子升级，必须提升协议版本并在握手阶段拒绝混用。MVP 可接受破坏兼容，但不能静默。
- `current_monotonic_deciseconds()` 启动永久 `time-updater` OS thread，没有 runtime owner 或
  shutdown；应改成可停止的 runtime clock，或使用已验证的 coarse clock 实现。
- `6facf1b` 的 FNV 优化随后被 `b769f74` 恢复为 `DefaultHasher`；当前代码与 commit message/
  性能文档不一致。不要据此宣称已完成 FNV 优化；是否采用快速 hash 必须同时考虑碰撞/DoS。
- HTTP upstream 当前以“收到任意响应”标记 backend healthy；5xx/协议错误与 transport failure
  应分层计数，避免健康状态掩盖应用层故障。
- wildcard host lookup 仍是线性扫描，H2 cache 的 eviction 也不是严格 LRU；只有 profile 证明
  route lookup/cache lock 占比后再引入 trie 或分片 RCU。

## 5. 工业化修复方案

以下方案只对已确认的正确性/生命周期问题直接落地；对纯性能项采用 benchmark gate，
避免把“看起来更先进”的结构强行引入生产路径。

### M0-A：Registry identity/fencing（先做）

1. 将认证后生成的 UUID `ClientId` 明确作为不可复用的 connection identity；registry actor
   拒绝同一 ClientId 的重复 register。若未来需要 ID 重用，再引入内部 sequence。
2. `SelectedConnection`、unregister、purge 和失败回滚都使用事件产生时捕获的 ClientId；
   actor 只对该 key 做 compare-and-remove，旧事件对已不存在的 identity 必须幂等 no-op。
3. Register 采用 prepare/commit：先校验 token/group、分配 slot、构建新 handle 和新 snapshot；
   commit 成功后再 retire/remove predecessor。任何 prepare 失败都保留旧连接；同 shard 替换也
   必须 retire predecessor。
4. unregister mailbox 改成有界 backpressure + 按 ClientId 合并的 pending reconcile；队列满只
   标记具体 identity 并计数，不得直接清空所有 groups。只有 actor join/health 明确失败才
   fail-close。
5. 增加不变量：不同 connection 使用不同 ClientId；重复 register 被拒绝；旧 unregister 永不
   影响其他 ClientId；同一 ClientId 至多释放一次 slot；register 失败不改变旧可用连接；actor
   重启/失败后所有可见快照为空。

### M0-B：任务和 session 所有权（按场景选型）

1. 先画出 connection/session/runtime 的 owner 图和退出时序；只有无法证明取消、join、
   幂等清理时，才把 `H2Sender` driver 纳入 connection-level `TaskTracker`/`JoinSet`。长生命
   周期 driver 可以继续 detached，但必须接收 root cancellation、设置资源上限并暴露 age/active
   指标。
2. UDP session creation 采用 per-key single-flight reservation（`DashMap<Entry>` + owned
   permit/notify），resolver/bind/connect 在专用 task 中执行，不阻塞 datagram worker；若
   profile 显示 task 调度成本高，再评估每 session actor，而不是默认 task-per-packet。
3. worker 只负责有限预算的 enqueue/dequeue；队列满丢包并记录 reason，session actor 或
   task 对发送设置 per-session deadline，跨 session 不能互相占满 worker。
4. 所有异步方案都必须给出 cancellation、容量、超时、错误传播和清理证明；没有这些证据的
   detached task 才视为缺陷。

### M0-C：协议和配置边界

1. H1 `try_new` 在 prepare 阶段解析并校验 `Scheme/Authority`，删除 localhost fallback。
2. 为 `RoutingInfo`/`UdpSessionKey` 增加 wire schema version 或 capability gate；严格 MVP 可以
   只支持同版本，但必须 fail-fast 并在 CI 中覆盖旧/新帧拒绝。
3. 配置加载阶段统一 canonicalize host、scheme、port、UDP entry ID；拒绝重复 `proxy_name`
   和超出 buffer/session/connection hard limit 的配置。

### M1：热路径证据化重构

1. 建立 A/B：标准 `BytesMut::with_capacity`、当前 pool、仅 peek pool、不同 allocator；采集
   alloc/free 次数、RSS、p99.9、CPU cycles、cache miss 和大 buffer pin 时间。
2. 若自定义池没有稳定收益，删除 `copy.rs`/`PeekBufPool` 的业务池逻辑；若保留，改为固定
   size-class、每 worker 预算和 shutdown flush，禁止动态替换/无界保留。
3. 将 `PreferH1Key`、route key、sender key 变成预解析的 `Arc<str>`/newtype；读侧用 ArcSwap
   immutable map，写侧单 actor 执行 TTL/eviction。
4. 日志字段改为结构化 primitive；指标 label 只允许静态低基数值。

## 6. 验证闭环

### 正确性/并发

- 并发 register(A)、register(B)、stale unregister(A) 的 property/loom 测试；验证 B 永不被删。
- 注册容量耗尽、actor mailbox 满、actor panic/restart、旧 handle drop 的状态机测试。
- H1 非法 scheme/authority 必须在 prepare 失败，不得建立 localhost 连接。
- H2 driver cancel、QUIC close、reload 和 shutdown 交错下，task/stream/sender/inflight 全部归零。
- UDP 同 key single-flight、不同 key 公平性、DNS/bind/send timeout、队列满丢包和 session eviction。

### 性能/稳定性

- 固定 cpuset/allocator/编译参数，覆盖 H1/H2/TCP/TLS/UDP、短连接/keepalive、单/多 host、
  DNS miss/hit、正常/故障/重连风暴。
- 门槛至少包括：错误率、完成 QPS、p50/p95/p99.9、CPU、RSS、alloc 次数、队列等待、
  active task/stream、drain 时长；不能只看平均 RPS。
- 每个优化必须保留可回滚开关和前后火焰图；没有稳定收益的实验回退，不进入主线。

## 7. 本批次范围与已有 TODO

附件中关于多 Endpoint、immutable route index、UDP 零拷贝、task-local buffer、H2 sender
小池、容量扩展和 profile 基线的多数项目，已在 `docs/todo.md` 有记录，本报告不重复展开。
本次新增且不应被已有 TODO 淹没的是：**registry identity 不变量/注册事务、unregister mailbox
fail-close、H1 localhost fallback、detached H2 driver 生命周期**。这些应优先于更多微优化。

## 8. 最新工作区实施复核（2026-07-28）

本次实现已经落地了部分方案，但尚未形成完整的生产闭环：

### 已确认有效

- Registry 的 register 已改为 actor 内 prepare/commit，旧连接在 commit 后 retire；同 shard
  替换路径不再遗留旧 handle。
- H1 driver 使用已解析的 `Scheme`/`Authority`，HTTP 入口解析失败时返回 502，不再静默回退
  到 localhost。
- `prefer_h1` 增加 TTL 检查和过期替换，读路径保持无锁；buffer pool 也已做 feature gate。
- UDP 使用 per-key `UdpRegId` 做提交和 eviction 的 compare-and-remove，避免旧建连任务删除
  新 session。

### 尚未闭环、应在标记完成前修复

1. **UDP 建连任务缺少并发上限和统一所有权（P1）**：establisher 从有界 channel 取出请求后
   直接 `tokio::spawn`，请求很快被完全 dequeue，实际 DNS/bind/connect 并没有 semaphore；这些
   task 也不在 `TaskTracker` 中，shutdown 只取消 dispatcher，无法等待或 abort 已启动的建连
   task。Connecting entry 同样未占用 session capacity，连接风暴可耗尽 task/socket/DNS 资源。
   需要对“建连中”计数、设置独立并发预算，并把 task 纳入可取消、可 join 的 owner。
2. **UDP 创建队列失败的清理存在竞态（P1）**：`create_tx.try_send` 失败时按 key 无条件
   `sessions.remove(key)`，可能把 eviction 或新一代 session 删除。必须改为按 `UdpRegId`
   compare-and-remove；同时为队列满、建连失败、pending 丢弃增加可观测的 drop reason。
3. **UDP pending drain 没有 deadline/错误传播（P1）**：commit 后逐包 `send().await` 且忽略错误，
   建连 task 可能被单次发送永久卡住；Connected 路径已有限时，但 pending 路径没有同等保证。
   另外 pending drain 与新到达包可能交错，若要求同 key 有序，需要一个 drain gate 或统一
   writer。
4. **Registry pending unregister 仍可能泄漏（P1）**：pending 使用 Vec，满 4096 后直接丢弃
   unregister；可见索引虽已 retire，但 actor 侧 entry 仍可能长期留在 `clients/group_conns`
   和快照中。pending 写入后也没有独立唤醒 actor 的机制，可能一直等下一条 mailbox 消息。
   应按 ClientId 去重合并、提供 Notify/reconcile 触发，并对 overflow 做 fail-close 或可靠重试。
5. **协议版本提升不是双向 fail-fast（P1，取决于是否滚动混部）**：新代码的
   `MIN_SUPPORTED_VERSION=2` 能拒绝旧 client，但旧 server 仍可能接受新 client 并协商到 v1；
   当前 data-plane 没有在 `NegotiatedProtocol.version < 2` 时阻止新 `RoutingInfo`/UDP 帧发送。
   若发布期间允许新旧混部，需要能力门控或在握手后立即拒绝低于 v2 的协商结果。
6. **性能基准覆盖不足（P1 证据缺口）**：新增 benchmark 只测
   `copy_buffered_then_shutdown` + `tokio::io::sink`，未覆盖实际改动最大的
   `copy_buffered_then_finish(SendStream)`，也无法反映 QUIC flow-control、chunk split/copy
   和 buffer pool 的真实收益。该 benchmark 不能作为 zero-copy 优化的放行依据。

### 次要质量项

- Connecting pending 使用 `Vec::remove(0)`，是 O(n)；应使用 `VecDeque`，并记录丢弃计数。
- pending drain 使用的 `Notify` 当前没有等待方，属于无效状态；应删除或接入明确的 single-flight
  等待协议。
- `docs/todo.md` 将 UDP 非阻塞建连/HOL 消除标为完成，和上述生命周期、容量、顺序问题不一致，
  应改为“部分完成/待补齐”。当前 diff 还存在多处 trailing whitespace，应在提交前清理。

因此，本批次可评为“核心方向正确、M0 部分完成”，不建议在补齐以上 P1 前将 UDP 和协议兼容
工作标记为 fully complete；H1 fallback 与 prefer-H1 TTL 则已达到可合并的实现质量。

## 9. 最新修订复核（第二轮）

本轮已修复上一版中的几项问题：建连请求现在受 session permit 约束，建连 task 纳入
`TaskTracker`，pending 使用 `VecDeque`，队列失败按 `UdpRegId` 清理，actor 有 pending
`Notify`，benchmark 也加入了真实 QUIC `SendStream/RecvStream` 路径；协议版本的混部风险则
通过 client 校验 `negotiated_version >= MIN_SUPPORTED_VERSION` 得到 fail-fast。

但仍有一个新的关键竞态：

1. **P0/P1：UDP Connecting→Connected 的转换不是原子的**。`establish_session_inner` 先从
   `DashMap` 删除 Connecting，再 drain pending；在 drain 期间同 key 的 datagram 会看到 vacant，
   创建并插入新一代 Connecting。旧 task 随后无条件 `insert(Connected)`，会覆盖新一代 entry，
   丢失新 pending 包并泄漏/提前释放新一代 permit。必须把状态转换改为原子 replace，或让
   entry 在 drain 期间保持同一代并用 per-session send gate 排序，不能出现“删除后等待再插入”的
   空窗。
2. **P1：shutdown 仍不能及时取消已开始的建连 task**。establish task 虽进入
   `TaskTracker`，但内部只等待多个 3 秒 operation timeout，没有监听 `root_cancel`；shutdown
   超时后只 abort datagram worker，没有 abort 建连 task。关闭期间旧 task 仍可能完成 bind/connect
   并在已清空的 session map 中重新插入 Connected。应在每个阶段 race root cancellation，并保留
   建连 task 的 abort handle 或使用可取消的 `JoinSet`。
3. **P1：Registry pending unregister 改成 Notify 后变成无界 Vec，且仍不去重**。actor 忙时可以
   持续累积重复 ClientId，存在内存压力/DoS 面；应按 ClientId 合并，设置上限并在溢出时计数、
   触发可靠 reconcile 或 fail-close，而不是无限增长。
4. **P2：pending drain 的 timeout/error 仍被完全忽略**。超时或 socket 错误会静默丢包，且没有
   drop reason/指标；至少应区分建连失败、超时、容量耗尽和 pending 溢出。

因此当前状态是“上一轮大部分问题已修复，但 UDP 状态转换竞态仍阻断合并”。协议 fail-fast、
QUIC benchmark、H1 fallback 和 prefer-H1 TTL 本轮可视为方向正确；`docs/todo.md` 仍不应将
UDP 生产级收尾标记为 fully complete。

## 10. 本轮修复结果与 identity 决策

本轮已将 UDP session 改为单一 `SessionEntry + SessionState` 状态机：Connecting、Draining、
Connected、Failed 只改变原 entry 的 phase，不再执行“从 map 删除、异步 drain、再插入新 entry”。
pending 队列和 Connected 发送共享 per-session gate，避免新一代 entry 被旧建连 task 覆盖；所有
解析、bind、connect、send 阶段都监听 manager cancellation，shutdown 会先标记失败、取消 socket
和清空可见 map。Registry pending unregister 改为有界去重集合，溢出时 fail-close，并拒绝重复
ClientId 注册。

关于“一个 client 多条 connection”：MVP 不需要新增第二个 wire 字段。当前 QUIC accept loop
每条 connection 生成新的 UUID `ClientId`，它实际承担 `ConnectionId` 角色；同一逻辑 client 的
多条连接通过相同 `GroupId`/token 归组，并可同时存在于 registry。`unregister`、snapshot、
inflight 和 compare-and-remove 都针对这条 connection 的 UUID。只有未来要表达跨连接的稳定
principal（例如按用户限额、撤销该用户全部连接、审计聚合）时，才新增独立的
`ClientPrincipalId`，形成一对多关系；不能把可复用的逻辑 ClientId 直接拿来做连接清理 key。
