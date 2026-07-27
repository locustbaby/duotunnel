# D10 · 性能加固与证据门槛

> 承接：[14 性能、健壮性与长期稳定性补遗](../14-performance-robustness-stability-addendum.md)。
> 目标：先消除代码可直接证明的 HOL、内存放大、无效配置和高基数，再用可信 profile
> 决定多 Endpoint、runtime 与 pool 分片。暂不落地代码。

## 1. 原则

1. 正确性/稳定性 M0 先于架构级性能改造；
2. 区分“确定成本”“条件热点”“假设”，不把静态结构直接写成主瓶颈；
3. 每项优化同时记录吞吐、p50/p99、RSS/连接、FD、丢包/错误率；
4. 配置项必须实际进入运行对象，才能作为调优变量；
5. 所有队列、cache、pool 和并发数必须有总预算，而非每 shard/endpoint 各自放大。

## 2. P0：性能与稳定性共同项

### 2.1 owned ConnectionState

采用 [D9 §5](./09-runtime-reliability.md#5-connectionstate-ownership) 的 owned state，
同时获得：

- 消除 slot ABA；
- 删除 free-list Mutex；
- slot 数从 `streams × connections × 2` 降为每 connection 一个；
- P2C 读取直接跟随已选择 ConnectionHandle，减少 table 二次索引；
- Notify 与 counter 生命周期自然绑定。

性能验收应比较连接创建 RSS、steady-state P2C latency 和高 churn 下 allocator/lock。

### 2.2 聚合 readiness 与连接池事务

`push -> Result<Inserted>`、`remove -> Result<Removed>`；actor 内维护 active count。
health endpoint 只读聚合快照，不由 supervisor 写 bool。除稳定性外，这能避免 readiness
抖动导致上层负载均衡反复摘挂实例。

### 2.3 UDP 与 stream accept 解耦

当前 client/server 在同一 QUIC connection loop 内处理：

```text
accept_bi
read_datagram → decode → DNS/session init/socket send
shutdown
```

改为：

```text
QUIC datagram reader
  → hash(session_key) % N
  → N 个独立 bounded shard queue
  → shard worker
  → session table/socket send
```

设计约束：

- 固定 worker 数，不每包 spawn；
- reader 直接选择独立队列；不使用多个 worker 竞争单消费者 Receiver；
- 同 session 保序、不同 session 公平；DNS/connect/send 使用 per-session singleflight，
  但受全局 bounded concurrency 与 timeout 控制，避免一个慢 session 卡住整个 shard；
- queue 满丢包并记录 `udp_queue_dropped_total`；
- session table 有 global/per-connection max、创建速率、idle TTL 和满载 drop/evict 策略；
  singleflight 失败/取消必须清理占位；
- shutdown 先停 reader，再有界 drain queue/pump；
- 队列预算按 bytes 而非只按 item：
  `connections × shards × capacity × (max_datagram + envelope overhead)`，乘法 checked。

验收：人为阻塞 UDP DNS/upstream 时，TCP/H1/H2 accept 和 p99 不显著退化。

### 2.4 指标基数治理

删除原始 Host/IP/query 等不受控 label。egress rejection 只保留静态 reason 和稳定、
全局有 cardinality cap 的 route ID；热更新删除旧 route 后 series 仍可能留在
Prometheus，因此“单代配置有界”不够。原始 Host 进入采样且限速的日志，并限制长度、
转义控制字符、去除 query/credential 等敏感内容。

先解决 time-series 数无界，再评估 MetricsSink 的 Vec/字符串分配。

### 2.5 配置与 admission 预算

- QUIC window/idle timeout 的 fallible conversion、native root 加载全部返回结构化错误，
  禁止 `unwrap` 把配置/主机环境问题升级为 panic；
- connection/slot/buffer/task 容量计算全部 checked，并受进程级 hard max 约束；淘汰
  server 固定 4096 slot 的隐含天花板；
- UDP entry 的 `proxy_name`/稳定 identity 在同一配置代内唯一，重复项整代拒绝，禁止
  `HashMap::insert` 静默覆盖 reply socket；
- TCP accept、TLS/协议 sniff、health/control watch 连接均有 semaphore、读写/握手
  timeout 与 overload metric，不能每连接无界 spawn。

## 3. P1：确定存在的热路径成本

### 3.1 UDP 二进制 wire/session key

- `IpAddr/SocketAddr` 使用二进制编码，避免每包 `to_string/parse`；
- encoder 直接返回 Bytes；
- retry loop 外构造一次 Bytes，重试只 clone；
- payload 不做 `to_vec` 再 `copy_from_slice`；
- 小候选/尝试集合使用栈数组或 SmallVec；
- registry handle 使用无锁/分片读路径，避免每回包 RwLock。

二进制 wire key 是协议变更，必须按 D8 的 capability/version 先双栈 decode，再切换
encode，最后退役旧格式；不能在滚动升级中直接替换字段布局。

### 3.2 ResolvedProxyBuffers

引入启动期校验后的统一对象：

```rust
struct ResolvedProxyBuffers {
    peek: usize,
    h1_header: usize,
    h1_body_chunk: usize,
    relay: usize,
    max_header_bytes: usize,
}
```

由 server/client state 共享，注入 sniffer、dispatcher、H1 driver 和 relay。每个字段
做 checked min/max；`并发上限 × per-connection buffer` 与全局预算也用 checked
arithmetic。预算包含 reload 时 old+new generation 双驻留峰值；0、极端值、乘法溢出或
超过 hard max 在启动/prepare 阶段结构化拒绝。删除固定
4096/8192 与“配置定义存在但运行时不使用”的分叉。启动日志打印最终解析值和估算的
单连接/全局内存预算。

### 3.3 H1 body/head

- 评估 Quinn `read_chunk(configured_body_chunk, true)`，直接把 Bytes 交给 Hyper；
- response-head BytesMut 作为 driver scratch 复用；
- 删除重复 request-side BoxBody；
- chunk prefix 小分配必须 microbench 后再决定：若改成多次 write 增加 poll/syscall，
  可能得不偿失。

### 3.4 immutable route index

RuntimeGeneration 已不可变，内部 router 不需要 DashMap/RwLock。可改为构建期 immutable
HashMap + wildcard trie/排序表，并提供借用式 ASCII canonicalization。

该项虽然方向明确，但仍需 route lookup profile；H1 keepalive 并非每请求重新选路，收益
与连接 churn/协议分布相关。

## 4. P1/P2：条件优化

### 4.1 H2c fast path

默认 `h2_single_authority=true` 时，首请求 pin authority/route/sender，后续请求不必每次：

- authority `to_string + lowercase`；
- 锁 first_authority；
- 查 route_cache；
- 查 sender_cache；
- 提前 clone HeaderMap 作为可能永远不用的 retry template。

先按 h2c 单独做 allocator/lock profile。关闭 single-authority 时 cache 必须有容量/TTL，
不能为了 fast path 引入无界 map。

### 4.2 DNS singleflight

TTL 到期并发 miss 可重复 `lookup_host`；cache hit 又 clone 全地址 Vec。连接风暴场景可
增加 per-key singleflight 与 cache 内 RR。只有 DNS miss/allocator 占比显著时实施，
并确保失败结果有短负缓存和容量上限。

### 4.3 MetricsSink

通用 MetricsSink 当前可能为每次调用构造 Vec/复制 label。若 allocator profile 证明
显著，再增加预注册 handle 和静态标签专用 API。禁止把 raw host/client IP 带回新 API。

## 5. 多 Endpoint 决策

多 Endpoint 调整为 **P2、profile-gated**，不再是“无前置、性能线第一优先”。

启动实验的必要条件：

1. cpuset 隔离后的 benchmark 可复现；
2. profiler 显示 Quinn endpoint UDP I/O/锁是主要 CPU 或 p99 来源；
3. owned ConnectionState、UDP HOL、确定分配项已完成；
4. 能给出 endpoint/connection/FD/socket buffer/QUIC window 总预算。

方案约束：

- 明确 strict N 或 degraded `active >= min_ready`，禁止静默缩容；
- endpoint 与 connection slot 使用稳定 N:M 映射；
- 单 slot transient 只重启该 slot，全局 auth/protocol fatal 才停池；
- coordinator/RAII 持 endpoint，统一 close/wait_idle；
- socket buffer 按总预算切分并读取 OS 实际值；
- 部分创建失败必须清理已经创建的 endpoint；
- 验收同时看吞吐、p99、RSS/连接、FD、UDP drop，而非只看 QPS。

如果 endpoint 不在 profile 前列，继续优化它属于架构投机，应停止。

## 6. per-core/runtime/pool 分片约束

- Tokio task 会迁移，不能把“当前 worker”直接当稳定 shard identity；
- per-core counter 使用 striped atomic/task-local 稳定 shard，不依赖瞬时线程；
- Hyper pool 分片会放大 idle connection、TLS/DNS state 和 FD，`max_idle_per_host` 必须按
  全局预算切分；除法取整仍不得突破全局 idle 上限；
- pool key 包含 D9 的配置 fingerprint；旧 shard pool 停止 admission 后有界
  retire/drain。启动/热更新预热使用全局并发/速率预算，避免所有 shard 同时 warm-up；
  跨 runtime 只传请求消息，队列满载和 shutdown 语义显式；
- runtime-bound socket/timer/driver 不跨 runtime 移动，只传 owned message/data；
- NoSteal 必须覆盖倾斜流量、阻塞 task、worker panic 和队列满；没有 fallback/监控不得
  作为默认模式。

## 7. 性能验收矩阵

| 维度 | 必须记录 |
| --- | --- |
| 吞吐 | accepted QPS、successful QPS、bytes/s |
| 延迟 | p50/p95/p99/p99.9，按 H1/H2/TCP/UDP 分开 |
| CPU | server/client user+sys、throttle、per-thread、scheduler |
| 内存 | steady RSS、峰值 RSS、每连接、每 active stream、snapshot apply 峰值 |
| 分配 | alloc count/bytes，按协议与连接 churn 分开 |
| 网络 | UDP drop、socket errors、QUIC retransmit、accept/open wait |
| 资源 | FD、task、queue depth、pool idle/active、retired generation bytes |
| 稳定性 | reconnect rate、forced drain、listener restart、config apply reject |

统计口径：

- 隔离 cpuset/NUMA/频率与背景负载，固定数据集、协议占比和连接 churn；
- baseline 与 candidate 至少交错运行 5 次，报告 median、置信区间和环境噪声；
- 只有差值置信区间越过零且超过已测噪声才判回归/改善；无法分辨 1–2% 时不得使用
  1–2% 硬门槛；
- 同时报 absolute error/drop/timeout，不能用失败请求抬高 accepted QPS。

默认回归预算建议：

- M0/D9 热路径回归门槛设在已测噪声之上，并按吞吐、p99、错误率分别批准；
- 确定微优化必须在目标协议上有可重复改善，且其他协议无显著回归；
- 多 Endpoint 只有在吞吐/p99 改善同时不显著恶化 RSS/FD/drop 时才保留。

## 8. 推进顺序

| 阶段 | 内容 | 性质 |
| --- | --- | --- |
| P0 | owned ConnectionState、聚合 readiness、UDP/stream 解耦、指标基数 | 正确性 + 确定性能 |
| P1a | 先留存可信 cpuset baseline；协议分层 allocator/lock/CPU profile | 测量 |
| P1b | ResolvedProxyBuffers、UDP 二进制/Bytes、H1 read_chunk/head scratch | 确定成本，但必须保留 before 数据 |
| P2 | H2c fast path、DNS/Metrics/router 优化 | profile-gated |
| P2-experiment | 多 Endpoint、pool/runtime/per-core 改造 | 严格 profile-gated |

## 9. 验收

- [ ] 连接池内存与 `max_concurrent_streams` 不再线性预分配 slot；
- [ ] UDP 阻塞不会阻塞同一 QUIC connection 的 stream accept；
- [ ] 10 万不同 Host 后 time-series 数和 RSS 有界；
- [ ] 每个 proxy buffer 配置都能改变对应运行对象；
- [ ] H1 body 路径不再固定 scratch + copy；
- [ ] 所有 queue/cache/pool 有容量、满载行为和指标；
- [ ] TCP accept/handshake/sniff 有 global/per-listener task 上限与超时，慢连接不能无界
  占用 socket/task；
- [ ] UDP session/queue 按 bytes 与数量双重有界，慢 session 不阻塞其他 session；
- [ ] 多 Endpoint 只有满足 §5 全部前置和验收门槛时进入实现。
