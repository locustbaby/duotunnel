# 性能、健壮性与长期稳定性补遗（2026-07-27）

> 本文是 01–13 的第三轮静态复核补遗。范围收窄为：**吞吐/p99/内存、并发与资源
> 生命周期、配置一致性、故障恢复、长期运行稳定性**。本轮由性能、架构、健壮性三个
> agent 独立核查并交叉反审，**未运行测试、压测或 benchmark**；因此文中严格区分
> “代码可直接证明的问题”与“必须 profile 才能决定的优化”。
>
> **后续实施记录（2026-07-27）**：本文识别的 M0a-0～M0e 已在
> `fix/m0-runtime-consistency` 完成代码实现，并由三个 Agent 分别复核 control/LKG、
> listener/UDP/readiness、QUIC/drain/upstream 后交叉收口。本地 workspace test、
> check、all-targets Clippy 已通过；未执行性能压测，因此本文 P1/P2 的收益判断保持不变。

## 1. 结论

原报告的数据面、LB、扩展性和产品能力覆盖较广，但对以下四类问题覆盖不足：

1. **控制面投递与运行时提交不是一个事务**；
2. **连接、listener、inflight slot 和后台任务的 owner/retire 条件不完整**；
3. **readiness、重连退避和 drain 不是聚合状态机**；
4. **性能路线把多 Endpoint 提得过早，确定存在的 HOL、分配、内存放大和无效配置反而
   没有进入第一批**。

因此，主 README 中“P0 已全部闭合 / M1 已完成”的表述只适用于 PR #58 当时列出的
旧清单，**不能代表当前 HEAD 已不存在上线阻断项**。在继续 D1–D7 或多 Endpoint 前，
应新增 **M0：运行时一致性与生命周期加固**。

## 2. 新增发现速览

| 级别 | 问题 | 影响 | 原报告覆盖 |
| --- | --- | --- | --- |
| P0 | `tokio::watch` 承载相对 Patch，慢消费者可跳过中间版本 | 路由/token cache 永久静默分叉 | 未覆盖 |
| P0 | token revoke broadcast 无发送端 | 已认证 QUIC 会话撤销后继续工作 | 未覆盖 |
| P0/P1 | 空 body 非幂等 HTTP 请求会被自动重放 | POST/PATCH/DELETE 可能重复副作用 | 部分覆盖，但结论错误 |
| P1 | inflight slot 在旧 guard/handle 退出前复用 | 新连接计数被旧流修改，过载/drain 失真 | 未覆盖 |
| P1 | listener reconcile 无 prepare/commit/fencing | bind 失败假成功、端口空窗、孤儿 worker | 未覆盖 |
| P1 | 多连接 client 共写一个 readiness bool | 仍有可用 tunnel 时错误 503，或失败 push 后假 ready | 未覆盖 |
| P1 | 健康 session 断线不会正确 reset backoff | 长期运行后正常断线也可能等待最大退避 | 未覆盖 |
| P1 | 后台组件错误/worker panic 被吞 | 进程存活但控制面、listener 或 metrics 已死亡 | 未覆盖 |
| P1 | snapshot 直接截断写、无 TTL/hash | 崩溃后 fallback 损坏；旧撤销无限 fail-open | 未覆盖 |
| P1 | upstream unhealthy 过期 entry 不清理 | 后续失败无法重新标 unhealthy | 部分覆盖，误判 `contains_key` 为正确去重 |
| P1 | QUIC stream accept 与 UDP 慢路径同一 select 内 await | DNS/socket/send 阻塞同连接 TCP/H2 stream accept | 未覆盖 |
| P1 | health/metrics/watch 接口无连接任务预算和读超时 | 慢连接堆积 task/FD/内存 | 部分覆盖端口暴露，未覆盖稳定性 |
| P1 | client inflight table 按 stream 数量预分配 | 配置放大时不必要的 cacheline/Notify 内存 | 未覆盖 |
| P1 | 多个 proxy buffer 参数未进入真实运行路径 | 配置调优无效，阻断可信内存/吞吐实验 | 未覆盖 |
| P1 | QUIC window/idle timeout 与 native root 初始化使用 `unwrap` | 极端配置或主机证书环境可直接 panic | 未覆盖 |
| P1 | 连接/slot 容量乘法未 checked，server slot 固定 4096 | 溢出、过量分配或隐含容量天花板 | 部分覆盖 |
| P1 | UDP listener 以 `proxy_name` 直接覆盖 registry socket | 重名配置导致回包错投/旧 socket 失联 | 未覆盖 |
| P2 | 多 Endpoint 被静态推断为主瓶颈 | 复杂度先于证据，可能优化错目标 | 结论过强 |

## 3. 控制面与运行时一致性

### 3.1 增量 Patch 可以被静默跳过 `[P0]`

`ControlService` 使用 `tokio::sync::watch` 保存单个最新 `WatchEvent`
（`tunnel-service/src/control/service.rs:17-24`），每次发布的是 previous→next 的相对
Patch（`:98-109`）。`watch` 只保证消费者看到“有变化”和最新值，**不保证交付每个
中间值**；但 `watch.rs:96-126` 的注释和实现按“每个 Patch 都会到达”使用。

server 在 `server/control/control_client.rs:195-220,300-331` 直接应用收到的 Patch，
没有验证 `base_revision == applied_revision`，也没有 gap/重复/回滚处理。慢消费者若从
v1 跳过 v2、直接收到 v2→v3 Patch，会把 v1 当成 v2 应用，最终版本号写成 v3，但内容
永久不完整。

**推荐分两步止血**：先让 `watch` 只作“changed”信号，每次仍按现有 rkyv 旧线协议发送
完整 Snapshot，允许 ctld-first 升级且立即获得最终收敛。revision/hash/applied ACK 需要
明确 ControlProtocolV2 capability 和双栈 rollout，不能直接扩充旧固定布局。V2 的
revision 还必须跨 ctld 重启持久化，否则 server 拒绝回退后会永久拒绝从 1 重新计数的
合法配置。若未来确需 delta，再增加 base/new revision、ACK/NACK 与 replay log。

### 3.2 token 撤销没有闭合到现有会话 `[P0]`

`server/bootstrap/mod.rs:319` 创建 `revocation_tx`，QUIC handler 在
`server/ingress/handlers/quic.rs:346,398-421` 订阅，但全库没有对应 `send`。
ctld revoke/rotate 仅发布配置（`tunnel-service/src/control/service.rs:130-141`），
server 仅替换本地 token cache（`server/control/control_client.rs:236-264`）。

结果是新登录会失败，但已经认证的连接会持续工作到自然断开。修复不能只按 group 粗暴
关闭：registry 应保存稳定 token identity/hash 与认证 revision，在新运行代提交后 diff
撤销项，fence 新 stream 并关闭精确匹配的会话。

### 3.3 snapshot 构建、应用和落盘均非事务 `[P1]`

- ctld 分别读取 routing 与 token provider，未证明来自同一数据库 revision
  （`tunnel-service/src/control/service.rs:77-93`）；
- server 依次更新 token、routing、listener，期间存在混合状态窗口
  （`server/control/control_client.rs:228-234,279-298`）；
- `local_snapshot.json` 使用直接覆盖写（`:72-84`），进程或磁盘故障可留下半文件；
- fallback 没有 hash、生成时间或最大陈旧时间，旧 token 可无限期 fail-open。

最佳方向不是继续给多个 `ArcSwap` 补顺序，而是构建单一不可变
`RuntimeGeneration`，完成 schema/语义/引用校验和 listener prepare 后一次 commit。
具体设计见 [D9](./design/09-runtime-reliability.md)。

## 4. 生命周期、readiness 与 drain

### 4.1 inflight slot 存在 ABA 生命周期错误 `[P1]`

registry/client pool 在连接从当前 map 移除时立即 `free_slot`
（`server/ingress/registry.rs:199-215,226-247`；
`client/tunnel/conn_pool.rs:138-149`）。但旧 `SelectedConnection`、`ConnectionHandle`、
`OpenedStream` 和 `InflightGuard` 都可能仍被旧快照或正在运行的请求持有。
`InflightGuard::drop` 只保存裸 `slot_id` 并修改当前槽位
（`tunnel-lib/src/lb/inflight.rs:60-63,75-122`）。

槽位分配给新连接后，旧 guard 的 promote/drop 会修改新连接 pending/active，造成
P2C、过载、通知和 drain 计数失真。跨 shard P2C 会延长被选对象的生命周期，未修复
ownership 前不应先扩展选择逻辑。

**推荐方案**：删除共享裸 slot 表。`ConnectionState` 内嵌 cache-padded 计数和 Notify，
`ConnectionHandle`/guard 持同一个 `Arc<ConnectionState>`；从 LB 快照移除只代表不再
选择，最后一个引用释放时才回收。registry 总容量另用 Semaphore/显式预算控制。

### 4.2 listener 热更新可能产生孤儿 worker `[P0/P1]`

`sync_listeners_inner` 发现端口不在 map 时只异步 spawn，未同步放入 `Starting` 占位
（`server/ingress/listener_mgr.rs:250-261`）；bind 完成后
`activate()` 才直接 `HashMap::insert`（`:55-70`）。

快速 A→B→C 更新可能同时 bind 同一 SO_REUSEPORT 端口，后完成的任务覆盖 map 中句柄，
被覆盖的 accept worker 仍运行但已无法 cancel。replacement 还可能在等待旧 listener
drain 时被更新 C 超过，最后由延迟到达的旧 B 再覆盖 C。当前 generation 只用于日志，
不是 fencing token。

修复必须由单一 `ConfigApplier/ListenerManager` actor 串行：
`Received → Validated → Prepared → Committed → OldDraining`。同 `(addr, port)` 应保留
稳定 acceptor，只切换 dispatch generation；`SO_REUSEPORT` 下“先 bind、暂不 accept”
也可能参与内核分流，不能视为无副作用 prepare。新端口才预绑定，所有 worker 携带
revision，stale completion 必须自行关闭，不能直接覆盖新 generation。

### 4.3 client readiness 是 last-writer-wins `[P1]`

多个 supervisor 共享一个 `AtomicBool`。每条连接成功后写 true
（`client/tunnel/client.rs:58-60`），任一连接退出后写 false（`:107-110`）。
所以 N>1 时仍有健康 tunnel 也可能返回 503。`EntryConnPool::push()` 又吞掉 channel/
actor 失败并返回 `()`（`client/tunnel/conn_pool.rs:181-194`），连接未真正入池也会
随后写 true。

readiness 应由 actor 成功提交后的 `active_tunnels` 派生：
`client_ready = entry_listeners_active && active_tunnels > 0`，并暴露
`active/desired` 表示 degraded。`push/remove` 必须返回结构化 Result，禁止 supervisor
直接写共享布尔。

### 4.4 重连退避不会在健康 session 后正确复位 `[P1]`

普通 `conn.closed()/accept_bi/read_datagram` 都返回 transient Err
（`client/tunnel/client.rs:66-101`），而 `backoff.reset()` 仅在 `run_client` 返回 Ok
时执行（`client/tunnel/supervisor.rs:94-104`）；正常 shutdown 又在进入 match 前返回。
因此多次相隔数小时的正常断线仍会把 backoff 累积到最大值。

应在登录成功后记录 `session_started`，session 存活超过 `stable_window` 或成功处理一次
业务后 reset；短连接 flap 才继续指数退避。多个 slot 还应共享 server-level reconnect
预算，避免同时重连。

### 4.5 drain 统计不等于协议级优雅停机 `[P1]`

当前全局 drain 主要统计 accepted TCP connection/pending open，没有完整覆盖 H2 request、
reverse QUIC stream、UDP pump。既有 Hyper keepalive 未收到 quiesce/GOAWAY，空闲连接也
可能吃满 drain timeout。

需要显式三阶段：

1. `Quiesce`：停止 accept；H2 GOAWAY；H1 禁止复用；拒绝新 QUIC stream/UDP session；
2. `Drain`：typed tracker 等待 TCP、H1/H2、QUIC、UDP 各类资源；
3. `ForceClose`：总 deadline 到达后 reset/close/cancel，并 join 所有 tracked task。

只有 shutdown token 导致的返回可以视为正常；长期 worker 在 shutdown 前 `Ok(())`
退出同样应判 `UnexpectedExit`。

## 5. HTTP/LB 稳定性

### 5.1 空 body 不等于可安全重试 `[P0/P1]`

TLS/H2c 仅用 `req.body().is_end_stream()` 生成重试模板
（`server/ingress/plugins/tls/mod.rs:130-226`；
`server/ingress/plugins/h2c/mod.rs:270-361`）。`forward_h2_request` 的错误可能发生在
`send_request(...).await` 等待响应期间（`tunnel-lib/src/proxy/h2_proxy.rs:107-132`），
此时上游可能已经执行请求。

RetryPolicy 必须同时判断：

- method 是否默认安全/幂等；
- 失败发生在 pre-dispatch 还是 post-dispatch/ambiguous；
- 是否有显式 Idempotency-Key/业务 opt-in；
- per-try 与 overall deadline；
- 是否成功取得同一 scope 的 retry budget。

默认只自动重试安全方法的可确认可重放失败；空 body POST/PATCH/DELETE 不应自动重试。
原 [D2](./design/02-lb-quality.md) 中“保留现有 body 约束”的描述已不成立。

### 5.2 upstream health 状态机会失效 `[P1]`

`is_healthy` 在 TTL 到期后返回 true，但不删除 entry
（`tunnel-lib/src/proxy/upstream.rs:81-88`）；后续 `mark_unhealthy` 看到
`contains_key` 就直接 return（`:23-28`）。如果主动探测一直失败，过期 entry 永久留在
map，后续失败无法刷新剔除时间。

健康状态应是显式 `Healthy/Ejected/HalfOpen` 状态机，以稳定
`BackendId(upstream, resolved SocketAddr, generation)` 为 identity；half-open probe
需要 CAS/single-flight，旧 probe 不得清除更新一代的故障。状态跨配置代迁移，但删除或
目标实质变化时必须清理。

## 6. 确定存在的性能问题

### 6.1 client inflight table 过度预分配

`EntryConnPool::new` 的容量为
`max_concurrent_streams × connections × 2`，最少 1024
（`client/tunnel/conn_pool.rs:80-87`），但 slot 实际按 connection 分配。每个 slot 又是
`CachePadded<InflightSlot>` 并单独持 `Arc<Notify>`
（`tunnel-lib/src/lb/inflight.rs:12-52`）。

这不是预留 stream credit，而是实存对象；配置放大时会造成数量级不必要的内存。采用
owned `ConnectionState` 可同时解决 ABA 与预分配。

### 6.2 UDP 慢路径阻塞 QUIC 主循环

server/client 都在承载 stream accept 的 `select!` 分支内直接 await UDP 转发。
server 新 session 还会执行 DNS、bind、connect
（`server/ingress/handlers/quic.rs:364-390`；
`server/ingress/handlers/udp_datagram.rs:114-131`）。一个慢 DNS/socket send 可阻塞同一
QUIC connection 的 reverse stream accept，形成跨协议 HOL。

推荐专用 datagram reader + **有界队列** + 固定数量、按 session hash 分片的 worker。
队列满按 UDP 语义丢包并计数，禁止每包 spawn。

### 6.3 UDP 每包分配与字符串地址

client 发送路径每包做 IP `to_string`、payload `to_vec`、动态尝试集合和重试复制
（`client/egress/udp_listener.rs:71-102`）；回包路径还有字符串 IP parse 与 registry
读锁（`:27-42`）。server reply 同样 `to_vec` 后再次 `Bytes::copy_from_slice`
（`server/ingress/handlers/udp_datagram.rs:190-201`）。

wire/session key 应改成二进制 `IpAddr/SocketAddr`；编码直接产出 `Bytes`，重试只 clone
引用；小尝试集合使用栈数组/SmallVec。

### 6.4 高基数指标先于指标微优化

外部可控 Host 被直接复制进 Prometheus label
（`client/egress/listener.rs:134-137` →
`client/metrics.rs:28-34`）。不同 Host 会持续创建 time series，既有每次 String 分配，
又有长期内存放大。

应删除原始 Host label，只保留有限 reason/配置定义 route ID。MetricsSink 的 Vec/label
分配可以 profile 后再优化，不能本末倒置。

### 6.5 H1 copy/scratch 与无效配置

- H1 body 仍使用固定 8 KiB scratch，再复制成 Bytes；
- chunked response 每 frame 分配 chunk prefix；
- response head 每响应新建 BytesMut；
- `http_header_buf_size/http_body_chunk_size` 仅在配置转换存在，driver 仍硬编码；
- server dispatcher 另建固定 4096 pool，没有使用 bootstrap 已构造的共享 pool。

推荐先引入已校验的 `ResolvedProxyBuffers`，真实注入 sniffer/H1/relay；请求 body 评估
直接使用 Quinn `read_chunk(configured_size, true)` 返回的 Bytes；response head scratch
复用。chunk prefix 是否改成多次 write 必须 microbench，避免减少小分配却增加 syscall/
poll 次数。

### 6.6 未校验配置、隐含容量与重名资源 `[P1]`

代码还能直接证明三组健壮性问题：

- QUIC transport 对 receive window 和 idle timeout 使用
  `try_into().unwrap()`（`tunnel-lib/src/transport/quic.rs:49-56`），极端但可解析的配置
  会 panic；HTTPS client 的 `.with_native_roots().unwrap()`
  （`tunnel-lib/src/egress/http.rs:59-60`）也把主机证书加载失败升级为进程 panic。
- client inflight capacity 直接计算
  `max_concurrent_streams × connections × 2`
  （`client/tunnel/conn_pool.rs:86`），缺少 checked multiplication、hard max 和全局内存
  预算；server 又固定 `new_inflight_table(4096)`
  （`server/ingress/registry.rs:100`），形成未配置化的连接天花板。
- UDP listener registry 用 `proxy_name` 直接 `insert`
  （`client/egress/udp_listener.rs:19-21,54-57`）；两个 entry 重名时后者静默覆盖回包
  socket，旧 listener 仍收包但 reply 会发向错误端口。

方案统一为：启动/prepare 阶段做 checked conversion/multiplication 和 hard max 校验；
初始化失败返回结构化错误而非 panic；连接、task、slot、buffer 使用单一资源预算；
`proxy_name` 在配置代内强制唯一，或 registry identity 扩展为稳定 entry ID，重复项整代
拒绝。详见 D9 §8 与 D10 §3.2。

## 7. 必须 profile 后再决定

| 候选 | 静态结论 | 决策门槛 |
| --- | --- | --- |
| 多 Endpoint | 结构上可能并行 UDP 收包，但没有证据证明是当前主瓶颈 | Quinn endpoint UDP I/O/锁在 CPU 或 p99 中占主导 |
| H2c 多把 Mutex/字符串 clone/HeaderMap 预克隆 | 成本确定存在；默认 single-authority 下作用范围有限 | 按协议 allocator/lock profile |
| DNS cache singleflight/借用 key | 连接风暴或 TTL 同时到期时收益大 | DNS miss/allocator profile |
| MetricsSink 预注册 handles | 主要按连接而非 keepalive request 触发 | allocator flamegraph 显示占比 |
| immutable router/trie | 当前 lookup 有 lowercase/复制 | route lookup 成为显著热点 |
| per-core/no-steal runtime | 可能降低共享计数/调度成本，也可能在倾斜负载下饥饿 | cpuset 后的 scheduler/queue profile |

多 Endpoint 从“性能线第一优先、无前置”降为 **P2、严格 profile-gated**。在它之前应完成
可信基线、owned ConnectionState、UDP HOL、确定分配和资源预算治理。

## 8. 推荐实施顺序

```text
M0a-0 旧 wire 完整快照止血
 └─ M0a-1 持久 revision + V2 双栈/hash/ACK + crash-safe LKG + client active count
     └─ M0b 稳定 acceptor/listener actor + server readiness + backoff reset
         └─ M0c RuntimeGeneration + revocation + security stale policy
             └─ M0d owned ConnectionState + reliable unregister + typed drain
                 └─ M0e 普通 stale TTL + component restart
                     └─ P1 可信基线 + 确定热点优化
                         └─ P2 profile-gated 多 Endpoint / runtime / pool 分片
```

截至 2026-07-27，M0a-0～M0e 的代码闭环均已完成；跨版本/多 leader、大规模并发、
故障注入与长稳仍按验收清单执行。后续主线先收这些 rollout 证据，再进入 P1 可信基线。
M0 的代码闭环不等于 P1 性能收益已被证明，也不放宽多 Endpoint/runtime 改造的
profile 门槛。

详细方案：

- [D9 · RuntimeGeneration 与运行时可靠性](./design/09-runtime-reliability.md)
- [D10 · 性能加固与证据门槛](./design/10-performance-hardening.md)

## 9. 上线阻断验收

- [ ] consumer 阻塞跨 1000 revision 后，server 最终 snapshot hash 与 ctld 一致；
- [x] duplicate/out-of-order/rollback/equal-revision-different-hash 均被确定处理；
- [x] login、public request/connection、QUIC reverse stream 与 UDP session 在各自 work
  unit 内只观察一个 generation；
- [ ] A→B→C 收敛后稳定 acceptor 仅向 C dispatch；新端口 bind failure 不改变已提交旧端口，
  被 supersede 的 prepared 资源均释放；
- [x] 旧 inflight guard 在新连接加入后 drop，不改变新连接计数；
- [x] N 条 tunnel 低于显式 `min_ready` 才 false；低于 desired 但仍达下限时为 degraded；
- [x] 稳定 session 后首次断线从初始 backoff 开始，短 flap 才指数增长；
- [ ] idle keepalive 在 quiesce 后立即退出，所有 typed tracker 最终归零；
- [ ] 10 万个不同 Host 不增加无界 time series；
- [ ] UDP upstream/DNS 阻塞时 TCP/H1/H2 p99 不发生跨协议显著退化；
- [x] 本批触达的异步队列/cache/session 定义容量与满载策略；
- [ ] retired-generation count/age/bytes 与 revoke close unfinished/deadline 指标；
- [x] shutdown 与 reload 由 listener/security apply gate、generation fence 和 owned
  close handle 保证单一 owner。
