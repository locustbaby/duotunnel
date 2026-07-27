# D9 · RuntimeGeneration 与运行时可靠性

> 承接：[14 性能、健壮性与长期稳定性补遗](../14-performance-robustness-stability-addendum.md)。
> 目标：用一套 revision、ownership 与失败语义同时解决 control Patch 丢失、配置撕裂、
> listener orphan、token revoke、slot ABA、readiness 误报和 drain 不完整。本轮仅设计，
> 不修改代码。

## 1. 设计不变量

1. `applied_revision` 只增不减；相同 revision 必须对应相同 canonical hash。
2. 每个业务 work unit 从鉴权/选路到结束只观察一个 generation；传输 admission 与
   L7 stream 可以分别 pin，但边界必须显式。
3. commit 前不得产生外部可见的服务状态变化；prepare 的 socket/task 等副作用必须有
   唯一 owner，且失败、超时或 supersede 时可完整回滚。
4. prepared/active/draining 资源都有唯一 owner、revision、deadline 和 exactly-once close。
5. connection load 在最后一个 handle/guard 释放前不得复用。
6. readiness 由已提交状态派生，不允许多个 worker last-writer-wins。
7. retired generation、prepared socket、队列、task、FD 和 snapshot bytes 全部有上限。
8. 安全 epoch（token revoke/disable）不允许按普通 stale TTL 无限 fail-open。

## 2. RuntimeGeneration

```rust
struct RuntimeGeneration {
    revision: u64,
    content_hash: [u8; 32],
    generated_at: SystemTime,
    auth: Arc<AuthIndex>,
    routes: Arc<RouteIndex>,
    client_configs: Arc<ClientConfigIndex>,
    egress: Arc<EgressPlan>,
    listeners: Arc<ListenerPlan>,
}

struct ServerState {
    current: ArcSwap<RuntimeGeneration>,
    operational: Arc<OperationalState>,
}
```

`RuntimeGeneration` 完全不可变，从一个控制快照构建。schema、token hash、引用完整性、
端口冲突和资源预算任一失败，都拒绝整代并保留旧代。禁止 `filter_map` 吞掉坏项后发布
“部分正确”的配置。

`OperationalState` 保存需要跨 generation 延续的状态：

- backend health/outlier；
- HTTP/TLS connection pools；
- active session registry；
- listener/component 状态；
- drain tracker 与 readiness facts。

这些对象使用稳定 `BackendId/ListenerId/SessionId` 和配置 fingerprint；pool 复用的
白名单至少覆盖地址、协议、TLS/SNI/证书验证、代理参数、DNS policy 与连接限制。配置
删除或 fingerprint 实质变化时，旧 pool 停止 admission 并 retire/drain；只有同一
fingerprint 才迁移状态，不能因 key 名相同跨代复用不兼容连接。

### 2.1 pin 规则

| 协议/路径 | 何时 load | pin 到何时 |
| --- | --- | --- |
| QUIC login | 登录开始 | auth + 下发 client config 完成 |
| TCP/H1、TLS+H1 | accept/connection dispatch | connection 结束 |
| TLS+H2、H2c | transport accept 时 pin admission generation；每个 stream 开始再 load 业务 generation | transport / stream 各自结束 |
| UDP | 新 session 创建 | idle eviction/force drain |

H2 route cache key 必须包含 revision；UDP manager 不能永久持有启动时 egress map。
listener 删除或安全策略变化时，必须 fence 旧 H2 transport 的新 stream admission，
不能只阻止新 TCP accept。长连接可能让旧 generation 存活，因此需要 admission fence、
oldest-age/bytes 指标和 retire deadline；包含已撤销秘密的旧 generation 使用更短安全
deadline。

## 3. 可靠 control delivery

### 3.1 M0a-0：旧线协议完整 Snapshot 止血

`tokio::watch` 只通知“revision changed”，watcher 每次醒来读取当前完整 Snapshot 并发送。
合并中间信号是安全的，因为最新 Snapshot 累计包含全部状态。

第一步只发送**现有 rkyv 固定布局的旧版 Full Snapshot**，不向结构中直接加入
revision/hash/ACK。这样可先升级 ctld，旧 server 仍可解析，立即消除相对 Patch 丢失；
它只保证最终内容收敛，不声称具备端到端 applied 证明。

### 3.2 M0a-1：ControlProtocolV2 双栈

revision/hash/ACK 属于线协议变更，必须先增加明确的
`ControlProtocolV2` capability/version negotiation：server 先支持 v1+v2，ctld 根据
协商结果选择编码；确认所有 server 支持后才默认 v2，最后再退役 v1。不得把新字段塞进
旧 rkyv 固定布局并宣称兼容。

V2 Snapshot 必须：

- 从同一 DB transaction/revision 读取 routing、token、client config；
- revision 使用持久 `{epoch, sequence}` 或等价单调 ID，与配置在同一 DB transaction
  提交；单 leader 分配，多副本切主不得倒退；
- 携带 canonical content hash：以协议版本/domain separator 开头，配置集合按稳定 key
  排序后确定性编码；排除 `generated_at` 和节点本地派生值；
- 有明确最大字节数和构建内存预算。

server 规则：

| 收到状态 | 行为 |
| --- | --- |
| `revision > applied` | prepare 新 generation；允许跳跃 |
| `revision == applied && hash 相同` | 幂等忽略 |
| `revision == applied && hash 不同` | control corruption，拒绝并告警 |
| `revision < applied` | rollback/replay，拒绝 |

ACK 必须表示 `applied revision/hash`，不能仅表示“已接收”。控制链路身份验证与传输保护
另行配置；hash 只能发现损坏/不一致，不能代替认证签名。

### 3.3 可选 delta

只有全量 Snapshot 的尺寸/频率数据证明需要时才引入：

```text
Patch { base_revision, new_revision, content_hash, operations }
Ack   { applied_revision, content_hash }
Nack  { current_revision, reason = Gap|HashMismatch|Invalid }
```

ctld 保存有界 replay log；gap、越序或超出 replay window 必须返回 FullSnapshot。
禁止再让 `watch` 直接承载不可丢的相对 delta。

### 3.4 crash-safe LKG

落盘流程：

1. 写同目录 `snapshot.<rev>.tmp`，权限 0600；
2. flush + `sync_all`；
3. atomic rename；
4. directory fsync；
5. 保留最近两份带 revision/hash/generated_at 的 LKG。

LKG 带独立 format version、control protocol version、revision/hash/generated_at 和
payload length；验证 canonical hash、最大尺寸、未来时间容差以及普通/安全 stale
policy。启动按 revision 从高到低验证。live commit 成功但落盘失败时标
`durability_degraded`，不回滚已经正确服务的 generation。

## 4. ConfigApplier 与 Listener actor

唯一 `ConfigApplier` 串行处理有界更新队列；连续更新可 coalesce 到最高 revision，但每个
请求必须得到 applied/rejected/superseded ACK。

```text
Received
  → Validated
  → RuntimeBuilt
  → ListenersPrepared
  → Committed
  → OldDraining
```

任一步失败进入 `Rejected`，旧 generation 保持服务；若 commit 已对外可见后 worker
失败，必须发布失败事实并 forward-fix/restart，不能静默“回滚”到已不可证明一致的旧代。

Listener 每端口状态：

```text
Absent
Preparing { revision, lease }
Prepared  { revision, sockets, gate }
Active    { revision }
Draining  { revision, deadline }
Failed    { revision, error }
```

- 同一 `(addr, port)` 的长期运行 acceptor 跨 generation 稳定存在；普通配置更新只原子
  切换其 dispatch target，新 accept 的连接携带当时的 `Arc<RuntimeGeneration>`，避免
  `SO_REUSEPORT` 预绑定阶段已参与内核分流；
- 新端口才允许 prepare bind；删除端口在 commit 时 fence 新 accept，再 drain；
- socket option 等必须换 socket 的变更使用平台专用 FD handoff/replace，并声明有界
  disruption，不能承诺零影响；
- 全部可准备资源 ACK 后才能逻辑 commit；多 listener 通过 transaction ID/fencing
  避免配置拼接，但 OS 层不是瞬时全局原子；
- worker completion 只有 revision/transaction 匹配才可 CAS 激活；
- stale completion、abort、coordinator 超时必须释放 prepared socket；
- commit 后 worker 意外退出进入 supervisor/readiness，不得只记日志；
- reload 与 shutdown 共享一个 lifecycle owner，禁止同时 drain 同一资源。

多 listener 的原子性是逻辑原子，不是 OS 原子。已经接受的 work unit 固定观察一个
generation；端口集合切换存在有界过渡窗口，readiness/指标必须暴露该状态。

## 5. ConnectionState ownership

推荐移除共享 dense slot table：

```rust
struct ConnectionState {
    lifecycle: AtomicU8, // Active → Fenced → Draining → Closed
    pending: CachePadded<AtomicUsize>,
    active: CachePadded<AtomicUsize>,
    notify: Notify,
}

struct InflightGuard {
    state: Arc<ConnectionState>,
    phase: InflightPhase,
}
```

registry/LB snapshot 移除连接时先 fence，防止新选择；旧 request/guard 继续持 Arc，
最后一个引用释放后对象自然回收。registry 总连接数使用 Semaphore/配置预算，不再借
slot table 同时承担容量和计数。

如必须保留 slab，最低要求是 `SlotLease{id, epoch}`、最后 lease 回收和每次访问 epoch
校验；裸 `usize slot_id + immediate free` 明确淘汰。

close 必须幂等、单调且 exactly-once；任务不得形成 Arc cycle。unregister 使用可靠有界
command，队列满不能静默 `try_send` 丢失。

## 6. Readiness、staleness 与 supervisor

```text
ServerHealthFacts {
  quic_bound,
  required_listeners: Map<ListenerId, {desired_revision, active_revision, healthy}>,
  optional_listeners_degraded,
  config_revision,
  config_valid,
  control_last_applied,
  control_source,
  critical_components: Map<ComponentId, Health>,
  applying,
}

ClientHealthFacts {
  entry_listeners_active,
  active_tunnels,
  desired_tunnels,
  pool_actor_alive,
}
```

```text
server_ready =
  quic_bound
  && every required listener is healthy at desired revision
  && config_valid
  && every critical component is healthy
  && control_policy_ok

client_ready =
  entry_listeners_active
  && active_tunnels >= min_ready_tunnels
  && pool_actor_alive
```

`EntryConnPool::push/remove` 返回 Result；只有 actor 成功提交后才修改 active count。
required/optional、`min_ready` 与进入/退出 readiness 的 hysteresis 显式配置。暴露
`active/desired`，低于 desired 但仍满足下限时标 degraded，不直接 false。

控制面断开：

- `Fresh`：正常；
- `Degraded`：soft TTL 内继续旧 generation，告警并暴露指标；
- `Stale`：hard TTL 后 ready=false，拒绝新 login/public connection；现有流按策略排空。

TTL 从 `last_successfully_applied` 的单调时钟计算，不是 last received。LKG 重启的 age
使用 wall clock，并处理未来时间/时钟回拨。安全面使用独立单调 `security_epoch` 和更短
`max_security_staleness`：无法证明 freshness 时拒绝新鉴权/新安全敏感 work；已有会话按
安全 deadline fence。普通配置 stale 不得隐式覆盖这条规则。

Supervisor 必须持续监控长期组件。shutdown 前返回 Ok 也属于 `UnexpectedExit`；
panic/Err 进入 restart policy，超过 restart budget 后 ready=false 并使进程失败，让外部
supervisor 重启。

## 7. 重连与协议级 drain

### 7.1 重连

- 登录成功记录 session start；
- session 存活超过 `stable_window` 或成功完成业务后 reset backoff；
- 短 flap 才继续指数退避；
- 多 slot 使用 full jitter + server-level reconnect limiter；
- 认证/协议全局 fatal 才停整个 pool；endpoint-local transient 只重建单 slot。

### 7.2 drain

固定阶段：

```text
FenceNewWork
  → ProtocolDrain
  → WaitInflight(total deadline)
  → ForceCancel
  → CloseResources
  → JoinTasks
```

协议动作：

- H1：不再复用，响应 `Connection: close`；
- H2：发送 GOAWAY，停止新 stream；
- QUIC：停止新 work，deadline 后 CONNECTION_CLOSE/reset；
- UDP：停止新 session，取消 pumps；
- listener：先 fence accept，再 drain 已接收连接。

Typed tracker 分别统计 TCP conn、H1/H2 request、QUIC reverse stream、UDP pump。
每阶段有 budget，总 deadline 到达后必须强制推进并报告 forced 数量。

revocation 的 applied ACK 边界是：新 generation 已 commit，精确匹配的旧 session 已同步
fence 新 stream，并登记关闭任务与 deadline。QUIC CONNECTION_CLOSE 可以异步完成，但
未完成数必须可追踪；仅更新 token cache 不得 ACK applied。

## 8. 有界资源与满载语义

| 资源 | 必须配置/观测 | 满载或超限行为 |
| --- | --- | --- |
| control update queue | items + bytes | latest-wins supersede；每个请求返回 superseded/rejected |
| Snapshot/LKG | default/hard max bytes、build peak | 构建前拒绝；旧代继续；记录 reject reason |
| prepared resources | 最大 transaction/FD/task + lease | 超限拒绝；lease 到期回收 |
| retired generations | count/bytes/oldest age | fence admission；deadline 后强制 drain |
| component restart | attempts/window/backoff | 超预算 ready=false 并退出 |
| accepted connections/tasks | global/per-listener semaphore、handshake/sniff timeout | admission reject/close，不允许无界 spawn |

所有大小与乘法用 checked arithmetic；磁盘满、部分写、future timestamp、超大 LKG 与
未知 format version 都必须产生结构化错误并保留最后已验证代。

## 9. 分阶段落地

| 阶段 | 内容 | 兼容/风险 |
| --- | --- | --- |
| M0a-0 | ctld 在旧 wire 上只发 Full Snapshot，消除 Patch 丢失 | ctld-first，兼容旧 server |
| M0a-1 | 持久 revision/epoch；V2 capability/双栈；hash/applied ACK；原子 LKG；client active count | server 双栈先行，再启用 V2 |
| M0b | 稳定 acceptor/listener actor + server 聚合 readiness；stable-session backoff reset | 同端口不重 bind；新/删端口 fencing |
| M0c | RuntimeGeneration 整代发布；revocation + security epoch/stale policy | 需要明确 work-unit pin 语义 |
| M0d | owned ConnectionState；可靠 unregister；typed drain | 替换计数 owner，需并发回归 |
| M0e | 普通配置 stale TTL、component restart policy、H2/UDP revision-aware cache、health state 外置 | 策略默认值需灰度 |

完成 M0 后再推进 HttpFilter、LB 扩展和 profile-gated 多 Endpoint。

## 10. 上线阻断验收

- [ ] watch consumer 阻塞跨 1000 revision，最终 applied hash 与 ctld 一致；
- [ ] rollback、duplicate、gap、equal-revision-different-hash 均按表处理；
- [ ] ctld 重启与 leader failover 后 revision/epoch 不倒退，server 不会永久拒绝新配置；
- [ ] v1 server + 新 ctld、双栈 server + v1/v2 ctld 均按协商矩阵工作；
- [ ] 任一 login/request 只观察一个 revision；
- [ ] 并发 reload 1000 次和 actor 在 prepare/commit 任一点退出，旧代或最高新代保持完整；
- [ ] A→B→C 收敛后只有 C 接受新 work；若 B 在 commit 前被 supersede 则从未对外生效，
  prepared FD/task 不泄漏；
- [ ] 长连接跨 10 次 reload 时 retired generation 数量/age/bytes 有界；
- [ ] 旧 guard drop 不改变新 connection state；
- [ ] N 条 tunnel 低于显式 `min_ready` 时 readiness 才 false；低于 desired 但仍达下限时
  degraded；
- [ ] worker panic、watcher 退出、control hard stale 均使 readiness/重启策略正确变化；
- [ ] shutdown 与 reload 并发时只有一个 lifecycle owner，所有 typed tracker 最终归零。
- [ ] revocation 的 applied ACK 发生前，匹配会话已 fence 新 stream；安全 freshness 不可
  证明时拒绝新鉴权；
- [ ] 超大/损坏/future-time LKG、磁盘满与 restart budget 耗尽均按 §8 有界失败。
