# D2 · 统一 LB + 健康/Outlier + 重试预算 —— 详细设计

> 承接：08（选择分片缺陷）、09 §4.2/§4.3（健康/重试）、10 §6.2/§6.3（统一抽象）。
> 目标：把 LB 从"三处各写、仅连接级健康、无重试预算"升级为"统一算法 + brownout
> 剔除 + 防重试雪崩"。**08 的修复落在本设计的统一 P2C 里。** 设计模式,暂不落地。

## 1. 背景与问题

**选择逻辑三套、四处**:
1. client 本地 upstream:`LoadBalancer::pick`（`plugin/egress.rs:15`,RoundRobinLb)✅ 走 trait;
2. server egress upstream:`UpstreamGroup::next_healthy`（`proxy/upstream.rs:90`,内联 RR)❌;
3. client 连接选择(1对多):`ClientGroup::select_healthy`→`pick_from_preferred_shards`
   （`registry.rs`+`lb/shard.rs`)❌ —— **08 缺陷所在**(只从 group 哈希的单一 shard 取);
4. client entry pool:`EntryConnPool::next_conn_for_shard_excluding` ❌ 同款分片 P2C。

**健康**:egress 仅 TCP connect 信号 + 10s TTL 自动恢复(`upstream.rs:23-88`),对
brownout(可连但持续 5xx/超时)无感;连接选择仅 `close_reason()`。
**重试**:各 handler 内联 `max_attempts=3`(`h1/mod.rs:54`、`tls/mod.rs:148`),
**无全局预算**→ 后端故障时 3× 放大(雪崩)。

## 2. 设计总览:一套算法核心 + 三处能力

```
┌─ 统一 LB 算法核心(lb/) ─────────────┐   ┌─ OutlierDetector ─┐   ┌─ RetryBudget ─┐
│ RoundRobin / P2CInflight(08修复)   │←──│ 按请求 outcome 剔除 │   │ 全局重试率上限 │
│ Weighted / ConsistentHash/Sticky   │   │ 退避+慢启动         │   │ 防雪崩         │
└────────────┬───────────────────────┘   └─────────┬─────────┘   └──────┬────────┘
   4 处选择统一走它            health/weight 反馈到 Candidate      handler 重试查预算
```

**关键取舍(见 §5)**:选择输入用轻量 `Candidate` 视图,非侵入具体类型;分片连接选择
为避免每请求 flatten 分配,P2C **核心函数**支持"跨 shard 采样"零分配路径,与 trait
共享同一算法实现。

## 3. 统一选择设计

### 3.1 Candidate 视图与 LoadBalancer(增强)

```rust
// tunnel-lib/src/lb/mod.rs
pub struct Candidate {
    pub healthy: bool,
    pub inflight: usize,   // 无状态上游填 0
    pub weight: u32,       // 默认 1;异构后端/慢启动用
    pub affinity_key: u64, // 一致性哈希/sticky;0=无
}
pub struct PickCtx {
    pub client_addr: std::net::SocketAddr,
    pub affinity_key: Option<u64>, // 会话亲和(如按 client_ip/cookie)
}
pub trait LoadBalancer: Send + Sync + 'static {
    fn pick(&self, candidates: &[Candidate], ctx: &PickCtx) -> Option<usize>;
}
```

内置策略(一份实现,配置选择):
- `RoundRobin`：无状态上游(egress upstream servers、本地 upstream)。
- `P2CInflight`：连接选择——随机两个健康候选,取 inflight 小者。**08 修复即此**。
- `Weighted`：加权 RR / 加权最少请求(weight 来自配置或慢启动)。
- `ConsistentHash` / `Sticky`：按 `affinity_key` 确定性选择(有状态后端/会话亲和)。

### 3.2 零分配的跨 shard P2C 核心(08 修复的落点)

现状 `pick_from_preferred_shards`(`lb/shard.rs:35`)"返回首个非空 shard"——**这是 08
缺陷**。修复不改分片存储(突变成本收益保留),只改**选择改为跨全部 shard 采样**:

```rust
// lb/shard.rs 新增:在多个 shard 快照的并集上做 P2C,不 flatten、不分配
pub fn pick_p2c_across_shards<T, H, I>(
    shards: &[arc_swap::Guard<Arc<Vec<T>>>], // 各 shard 当前快照
    is_healthy: H, inflight: I,
) -> Option<T>
where T: Clone, H: Fn(&T)->bool, I: Fn(&T)->usize {
    // 1. 统计各 shard len,得 total;total<2 退化取唯一健康者
    // 2. 生成两个 [0,total) 的随机全局下标,映射回 (shard, local) 定位候选
    // 3. 取健康且 inflight 小者;不健康则重试有限次
}
```

`ClientGroup::select_healthy` 改为调 `pick_p2c_across_shards`(读全部 shard 的
`ArcSwap::load`,N=核数,量级小);`EntryConnPool` 同改。**`shard_count=1` 时行为与现状
逐字节一致**(08 §2.7 已论证)。

### 3.3 egress upstream 接入统一 LB

`UpstreamGroup` 增 `weight` 字段与 outlier 联动;`next_healthy` → 构造
`Vec<Candidate>`(servers 数量少,分配可忽略)→ `LoadBalancer::pick`(默认
`Weighted`+outlier 健康)。保留 `next_healthy` 名兼容,内部走统一核心。

## 4. 健康 / OutlierDetector 设计

### 4.1 trait 与类型

```rust
pub enum RequestOutcome { Success, ServerError(u16), Timeout, ConnectError, Cancelled }
pub enum BackendHealth { Healthy { weight: u32 }, Ejected { until: Instant } }

pub trait OutlierDetector: Send + Sync + 'static {
    /// 每请求结果回传(按后端聚合)。
    fn record(&self, backend: &str, outcome: RequestOutcome);
    /// 供选择读取:健康(含慢启动权重)或被剔除。
    fn health(&self, backend: &str) -> BackendHealth;
}
```

内置 `DefaultOutlierDetector`:
- **剔除判据**:连续 N 次 `ServerError`/`Timeout`/`ConnectError`(可配),或滑窗错误率
  超阈值——**不再只看 connect**;
- **退避**:剔除时长随连续失败**指数增长**(base→max),而非固定 10s;
- **恢复**:剔除到期后进入**半开探测**(主动探 or 首个试探请求成功才转健康),
  **不到期即自动恢复**(修 09 §4.2 的 flapping);
- **慢启动**:恢复后 `weight` 在窗口内线性爬升(避免 thundering)。

### 4.2 outcome 回传通道(依赖 D1)

请求结果从两处产生:
- L7:D1 的 `on_response`(拿到 status)+ 终态(上游错误/超时)→ 映射 `RequestOutcome`;
- relay/连接:`open_stream`/relay 的 Err 分类(已有 `ProxyError.kind`)→ `ConnectError`
  等。
统一经一个 `record(backend, outcome)` 汇入 OutlierDetector。**backend 粒度按上游地址,
非按 QUIC 连接**(H2 一连多请求要按后端归因)。

### 4.3 与连接选择健康的关系

client 连接选择(1对多)的"健康"当前是 `close_reason()`(连接存活)。可选增强:把
"该连接上的请求错误率"也纳入 outlier,使坏 client(能连但持续失败)被降权/剔除——但
需谨慎(避免误伤瞬时抖动);建议 v2,v1 先保连接存活 + egress 后端 outlier。

## 5. 重试预算 / RetryBudget 设计

### 5.1 trait 与类型

```rust
pub trait RetryPolicy: Send + Sync + 'static {
    fn is_retryable(&self, outcome: &RequestOutcome, idempotent: bool) -> bool;
    fn per_try_timeout(&self) -> Duration;
}
pub struct RetryBudget { /* 滑窗: retries / requests, per-core 聚合 */ }
impl RetryBudget {
    pub fn on_request(&self);            // 每请求计数
    pub fn try_acquire_retry(&self) -> bool; // 重试前查:retries/requests ≤ ratio?
}
```

### 5.2 语义与集成

- handler 重试循环(`h1/mod.rs`、`tls/mod.rs`、`proxy/http.rs` HttpPeer)改为:
  首次请求照发;失败后**先 `budget.try_acquire_retry()`**,超预算(默认 20%)则**停止
  重试、快速失败**(而非固定 3 次)。
- `is_retryable`:仅 connect/timeout/幂等的 5xx 可重试;带 body 的非幂等请求不重试
  (现有 `tls/mod.rs:173-180` 已有此约束,纳入 RetryPolicy)。
- `RetryBudget` 计数是热路径原子 → **per-core 近似聚合**(同 02 K1),scrape 求和。

### 5.3 与熔断的关系

重试预算(全局、简单)先行;每后端熔断(circuit breaker,状态更多)可后置——OutlierDetector
的"剔除"已提供后端级短路的雏形,熔断是其强化(半开/滑窗),按需再设计。

## 6. 场景覆盖 & Corner Cases

| 场景 | 处理 |
| --- | --- |
| `shard_count=1`(CI/单核) | 跨 shard P2C 退化为单快照 P2C,**与现状逐字节一致** |
| 全部候选不健康 | fail-open:选最不坏者(或按现有 `next()` 兜底),记指标,不返回 None 断服 |
| 单后端 group | outlier 不把唯一后端剔到零(保留 fail-open) |
| H2 一连多请求的 outcome 归因 | 按**上游后端地址**记录,非按 QUIC 连接 |
| 健康状态跨路由热重载丢失 | `ServerEgressMap` 重建 `UpstreamGroup` 会清空 unhealthy(09 §4.2 corner);设计:重建时迁移健康态,或接受一次探测窗口 |
| 慢启动与突发 | 恢复后 weight 线性爬升;爬升期遇新故障立即重新剔除 |
| 重试与 unregister 叠加 | h1 失败即 unregister 换连接(`h1/mod.rs:97`),重试换连接正确;预算限制"换后继续重试"的总量 |
| 非幂等请求 | RetryPolicy 拒绝重试(保留现有 body 约束) |
| 预算计数竞争 | per-core 聚合,避免热路径共享缓存行 |
| affinity_key 冲突/倾斜 | 一致性哈希用有界虚节点;倾斜时回退 P2C |

## 7. 论证 / 备选

- **为何统一算法而非各处加**:三处内联(RR/P2C)已导致"加一个加权策略要改三处 +
  三套测试"的漂移;统一后策略一份、可测、可换(10 §6.2)。
- **为何 08 修复用"跨 shard P2C"而非"取消分片"**:分片保留突变(注册/注销)的快照
  重建成本收益;仅选择读全 shard(08 §2.6 备选表已论证 A 优于取消分片/轮询 preferred)。
- **为何 Candidate 视图 + 零分配核心并存**:纯 `&[Candidate]` trait 对无状态上游最简;
  分片连接选择为避免每请求 flatten 分配,用共享 P2C 核心的"跨 shard 采样"变体——
  两者算法同源,只是入口形状不同。
- **为何 outlier 看 HTTP 错误而非仅 connect**:生产故障多为 brownout(可连但坏响应),
  connect 健康对此零覆盖(Envoy outlier detection 的存在理由,09 §4.2)。
- **为何重试预算而非固定次数**:固定次数稳态无害、故障态是放大器;预算把"系统级
  重试量"作为受控资源,是唯一防雪崩机制。

## 8. 取舍 / 改动量 / 影响

- **取舍**:引入 outlier/retry 状态组件 + outcome 回传接线;换取 brownout 防护 +
  防雪崩 + 一致可换的 LB 策略。08 修复替换一条现有测试语义(preferred-first → 跨 shard
  均衡)。
- **改动量**:统一算法核心 + 08 修复(~1-2 天,含跨 shard P2C 与测试);OutlierDetector
  + outcome 回传(~2-3 天,依赖 D1 的 on_response 通道);RetryBudget(~1 天)。合计
  **~1-1.5 周**。
- **影响面**:所有选择路径与 egress 上游健康、所有 L7 重试循环。`shard_count=1`
  零行为变化;需补跨 shard 均衡、brownout 剔除+退避+慢启动、重试预算封顶等测试。

## 9. 分阶段实施(设计蓝图,暂不落地)

| 阶段 | 内容 | 依赖 | 测试 |
| --- | --- | --- | --- |
| P1 | 统一算法核心 + Candidate/LoadBalancer;local/egress upstream 走它(配置=现状 RR) | — | 与现状等价回归 |
| P2 | **08 修复**:跨 shard P2C;registry/pool 接入 | — | 跨 shard 均衡(变异系数<20%)、shard_count=1 逐字节回归 |
| P3 | OutlierDetector + outcome 回传(经 D1) | **D1** | brownout 剔除、指数退避、半开恢复、慢启动、单后端 fail-open |
| P4 | RetryBudget 接入重试循环 | — | 预算封顶重试放大、非幂等不重试 |
| P5(可选) | 加权/一致性哈希/sticky + 连接级 outlier | P1-P4 | 权重分配、亲和命中、倾斜回退 |

## 10. 验收

- [ ] `shard_count>1` 下一 group 的多 client 流量近似均衡(08 修复,变异系数<20%);
- [ ] `shard_count=1` 回归逐字节一致;
- [ ] 后端 brownout(持续 5xx/超时)被 outlier 剔除、指数退避、探测确认恢复、慢启动;
- [ ] 后端集体 brownout 时对后端重试放大 ≤ 预算(默认 20%,当前 3×);
- [ ] 选择策略可配(RR/P2C/加权/一致性哈希),四处走同一套算法;
- [ ] outcome 按后端归因(非按 QUIC 连接)。
```
