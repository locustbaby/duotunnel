# 控制面与性能任务工业级实施设计

> 本文是 `15-task-breakdown.md` 的实现基线。所有代码修改先满足这里的不变量，再补充具体优化。性能任务使用现有 dial9 作为第一阶段观测手段，不把 hotpath-rs 作为前置依赖。

## 1. 配置 source 与 EffectiveConfig

### 1.1 责任边界

```text
YAML source ─┐
             ├─ ConfigCoordinator ── validate ── persist ── publish
SQLite source┘                                      │
                                                    ▼
                                             EffectiveConfig
                                                    │
                                      Snapshot / Delta Watch
```

- YAML 只产生低优先级 `ConfigLayer`，负责文件读取、轮询、debounce 和错误保留。
- SQLite 只保存 override/tombstone、source revision 和 EffectiveConfig 物化结果。
- `ConfigCoordinator` 是唯一 merge、校验、effective revision 和发布入口。
- source manager 必须校验未知 source、重复 SQLite source、priority 冲突，并保留 source degraded/recovered 状态。
- server 不读取 YAML/SQLite；admin socket 只连接常驻 ctld。
- 未来接入 Etcd 等 source 只能实现 source 接口，不能绕过 coordinator 直接写 routing 表。

### 1.2 Merge 不变量

- 合并粒度是顶层资源 key，不做字段级 deep merge。
- SQLite upsert 覆盖同 key YAML；tombstone 屏蔽同 key YAML；clear 删除 override 后恢复 YAML。
- token 不进入 YAML merge，只通过 SQLite auth transaction 管理。
- YAML source 失效时保留最后有效 layer；SQLite 启动不可用时启动失败，运行中故障不立即抹掉当前 generation。
- EffectiveConfig 必须先完整校验，再写入物化表和 revision 状态；不能发布部分结果。
- merge 后必须做 key normalization、唯一性校验、tombstone invariant 和引用校验；旧物化 routing 表只用于一次性迁移，不作为恢复 authority。

### 1.3 revision/hash 事务

一次 source mutation 的逻辑事务为：

```text
读取最新 YAML layer + SQLite override
  → merge
  → schema/reference/capacity validation
  → canonical effective hash
  → hash 相同：只更新 source revision/degraded 状态，不增加 effective revision
  → hash 变化：同一 SQLite transaction 写 override/effective/config_state/revision
  → transaction commit 后才通知 watch
```

`generated_at_unix_ms`、transport revision 和 epoch 不进入 content hash。所有资源集合按稳定 key 排序，Delta apply 后重新计算的 hash 必须等于 target hash。

YAML source 必须使用同一份已读取 bytes 完成 source revision 和解析，避免 hash/parse TOCTOU。数据库 schema 使用显式 migration version，migration 完成前不启动对外服务。

## 2. Admin socket

admin 不是公网 Web 服务，而是 ctld 内的 Unix domain socket control API：

- 默认路径 `duotunnel-ctld.admin.sock`；
- socket 权限 `0600`，只允许本机同一用户/权限域访问；
- 请求进入 ctld service，由 service 持有 mutation lock 并执行 SQLite transaction；
- CLI 只是 socket client，不直接打开 SQLite；
- healthz/metrics 可以是独立 TCP 只读端口，不承担配置写入；
- admin 写成功只表示 SQLite mutation 已提交，之后由 coordinator 发布新的 EffectiveConfig 和 watch event。
- 启动时不能无条件删除已有 socket；应检测 owner/活跃实例并 fail closed，避免两个 ctld 同时写同一数据库。
- mutation 只使用 POST 语义，返回结构化状态和 400/404/409/410/428/503 区分。所有 mutation
  必须携带 `Idempotency-Key`；CLI 会自动生成并发送该 header，外部 admin caller 缺失时
  直接返回 428。

任何 admin mutation 都必须具备：请求大小上限、解析错误边界、超时和结构化错误码。
普通响应的幂等记录、mutation 和 SQLite commit 在同一持久化 transaction 内完成。create/
rotate 的 bearer token 不写入 SQLite 明文：进程内有界缓存只用于同进程重放，SQLite 只保留
脱敏提交标记；ctld 重启后相同 key 返回 410，避免重复 create/rotate。token rotate 本身
必须在单个 SQLite transaction 内完成，不能让旧 token 和新 token 之间出现不可控空窗。

## 3. Control Watch 与 Delta

### 3.1 服务端状态

ctld 全局只保存一个共享的 immutable current snapshot；每条 watch task 只保存共享 `Arc<ConfigSnapshot>`、last ACK revision/hash 和对应 snapshot 引用，不复制 N 份相同数据。

每条连接严格串行：

```text
send event → flush → wait Applied/Duplicate/ResyncRequired → update connection ACK state
```

ACK 未完成期间不发送第二个 event。中间多个变更由最新 snapshot 与 last ACK snapshot 重新 diff，不使用不可重放的相对 change log。

业务 payload 外层使用固定 control envelope：`magic + wire_version + message_kind + payload_length + payload`。先校验 envelope 和 message limit，再 decode rkyv payload；Rust 业务类型不使用 `V1`/`V2` 命名。

### 3.2 Delta apply 不变量

server 收到 Delta 后必须按一个批次处理：

1. 校验 base epoch/sequence/hash 与当前 applied state；
2. clone 当前 snapshot；
3. 应用全部 operations；
4. 校验唯一性、引用、资源预算和 token identity；
5. 计算 target hash 并与消息 target hash 比较；
6. 构建 RuntimeGeneration；
7. listener prepare/commit、token fence、generation publish；
8. 成功后写 LKG，再返回 Applied。

任意一步失败都不更新 applied revision/hash，不逐条发布 operation。base 不匹配返回 `ResyncRequired`；同 revision 同 hash 返回 `Duplicate`；同 revision 不同 hash 或 target hash 不匹配返回 `Rejected`。不同 epoch 不得走普通 Apply，只能由显式 authority reset 的完整 Snapshot 接受；Delta 不得跨 epoch。

### 3.3 Snapshot fallback

- 首次连接和重连发送 Snapshot；
- Delta 序列化尺寸不小于 Snapshot 时发送 Snapshot；
- Delta/Snapshot 尺寸使用实际 wire encoder 计算，而不是 JSON 近似；
- ResyncRequired 在同一连接中发送 Snapshot，不依赖重连；
- LKG 只作为恢复备份，不能覆盖 ctld 的更高 authority；
- LKG primary/previous 都使用校验、原子替换和有限陈旧窗口。

## 4. RuntimeGeneration 与失败语义

RuntimeGeneration 是唯一对业务路径可见的不可变配置代。构建失败、引用缺失、证书/端口错误时旧代继续服务。

apply 的外部可见顺序固定为：

```text
validate/build candidate and encode LKG
  → acquire security/config fence
  → prepare all listener/resource
  → fence revoked sessions and reject new work
  → commit listener barrier
  → publish RuntimeGeneration
  → persist LKG / update durability state
  → release fence and ACK
```

listener 的 prepare 失败不能提前 revoke token；如果 revoke 已完成而后续 listener commit 失败，必须保持 fail-closed fence，直到下一次完整 apply 成功，不能自动恢复已撤销安全状态。

当前 MVP 不实现 server-side supersede queue。ctld 只在发送前通过 latest snapshot coalesce；一旦某个 event 已发送，必须完成 Applied/Rejected/Resync 流程后再发送下一个。

每个 listener worker、connection、session、queue 和 generation 都要有 owner、revision、deadline 和 exactly-once close 语义。快速 A→B→C 时，过期 completion 不能覆盖最新 generation。

## 5. 性能任务与 dial9

现有 dial9 构建已启用 CPU profiling、Tokio task tracking 和 scheduler/kernel events，并在 8K trace case 采集 server/client artifact。因此：

- TODO-145/hotpath-rs 不作为第一阶段依赖；
- 先用 dial9 判断 runtime/task/scheduler/CPU 热点；
- 只有需要稳定函数边界、allocator 或锁级别归因时才引入专门 profiler；
- 所有性能实现仍需保留 before/after 的 p50/p99、吞吐、CPU、RSS、FD 和错误率；
- 不因 dial9 已存在而跳过有效配置日志和 benchmark case 口径修正。

dial9 当前主要覆盖 server/client 的 H1 8K 数据面，不覆盖 ctld、SQLite/YAML merge、admin、control watch，也不直接提供 allocator、锁、阶段延迟、socket queue 或连接/路由级 p99。Criterion 和 Linux perf 只在 dial9 不足时补充。

优先实现没有 profile 前置的确定性问题：

1. buffer 配置贯通实际 relay/HTTP/sniff 运行对象；
2. TCP 默认 buffer autotuning 的明确配置语义；
3. 已确认的 body copy、容量溢出和跨 shard correctness 问题。

H1 scratch、H2 sender pool、UDP wire/session、DNS singleflight、多 Endpoint 和 custom runtime 都必须由 dial9/benchmark 证据触发。

## 6. 验收分层

| 层级 | 内容 | 工具 |
| --- | --- | --- |
| Deterministic | merge、hash、revision、Delta batch、容量边界、CLI/admin 解析 | Rust unit/integration tests |
| Fault injection | listener apply、LKG、磁盘/网络/worker failure、reload/shutdown | fake dependency + process integration |
| Soak | 1000 revision、长连接、重复 reload、重连和 revoke | 独立 CI job |
| Performance | H1/H2/L4/UDP、p99、资源、cpuset、优化前后 | k6 + resource collector + dial9 |

任何层级的结果不能替代更高层级的验收；尤其不能用 cargo test 结果声称长稳或性能完成。
