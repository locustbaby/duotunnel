# D7 · Phase B：client 多 Endpoint —— 详细设计

> 承接：02 §2.1（S1 候选收包串行点）、D10 §5（多 Endpoint 决策门槛）。
> 目标：当 profile 证明 **quinn endpoint driver** 是主导热点时，通过多个 UDP endpoint
> 实验验证收包扩展。机制上与 per-core runtime 可独立实施，但**不代表无条件必做**。
>
> **状态：已记录、待定，未排期；P2 / profile-gated。** 前置为 D9/M0、D10 的可信
> benchmark 和 endpoint 热点证据。未满足门槛时不实施。
> 重启时先复核 §1.1 的代码事实是否漂移（本设计基于 2026-07-26 的 HEAD）。

## 1. 背景与问题

### 1.1 现状拓扑（已核实）

```
crates/duotunnel-client/runtime/app.rs:38   build_quic_endpoint()  ← 全进程唯一一次
        │
        └─ TunnelPoolService { endpoint }        （crates/duotunnel-client/tunnel/mod.rs:17）
              │
              └─ pool.rs:23  endpoint.clone() × N  ← N 个 supervisor 共享同一 Endpoint
                    ├─ supervisor 1 → Connection 1 ┐
                    ├─ supervisor 2 → Connection 2 ├─ 处理已并行
                    ├─ supervisor 3 → Connection 3 │  但收包全挤在
                    └─ supervisor 4 → Connection 4 ┘  一个 driver task
```

关键代码事实：

| 事实 | 证据 | 对设计的意义 |
| --- | --- | --- |
| endpoint 全进程建一次 | `crates/duotunnel-client/runtime/app.rs:38` | 唯一改动源头 |
| N 个 slot 共享同一 clone | `crates/duotunnel-client/tunnel/pool.rs:23` | **改这里即可** |
| `supervisor` 签名已是**拥有** `endpoint: quinn::Endpoint` | `supervisor.rs:50-57` | **签名不用改** |
| 重连时复用同一 endpoint（只重建 Connection） | `supervisor.rs:72-87` 循环内 `run_client(&config, &endpoint, …)` | 语义保持不变 |
| UDP socket 绑 `0.0.0.0:0`（内核分配临时端口） | `crates/duotunnel-client/tunnel/endpoint.rs:24` | 多 endpoint 天然不冲突 |
| `EntryConnPool` **不感知** endpoint，只存 `ConnectionHandle` | `crates/duotunnel-client/tunnel/conn_pool.rs` 全文无 endpoint | **pool 不用改** |
| 无 `endpoint.close()` / `wait_idle()` 调用 | 全 client 目录 grep 无 | 关闭路径需顺带补（§2.7） |

**结论：改动面比预想的小得多**——`supervisor` / `conn_pool` / 重连逻辑都不用动，
只需把"一个 endpoint clone N 份"换成"建 N 个 endpoint 分给 N 个 slot"。

### 1.2 它能解什么，以及尚未证明什么

quinn 的 **Connection driver 已经并行**（每 Connection 独立 task，tokio 可调度到
不同核），endpoint driver 则先接收 UDP 包再按 CID 分发。多 Endpoint 可以并行这一
阶段，但静态结构不能证明它在当前负载中占主导，也不能证明收益大于新增 socket buffer、
FD、调度、连接池和 lifecycle 成本。启动本设计前必须由 profiler 证明 endpoint UDP
I/O/锁进入主要 CPU 或 p99 路径。

---

## 2. 详细设计

### 2.1 核心改动

```rust
// crates/duotunnel-client/tunnel/mod.rs —— TunnelPoolService 改为持有 N 个 endpoint
pub(crate) struct TunnelPoolService {
    pub(crate) endpoints: Vec<quinn::Endpoint>,   // 原: endpoint: quinn::Endpoint
    pub(crate) resolved_connections: u32,
    // ... 其余不变
}

// crates/duotunnel-client/tunnel/pool.rs —— coordinator 保留 owner；每个 connection slot 取稳定映射的 clone
for slot_idx in 0..resolved_connections {
    let endpoint = endpoints[slot_idx % endpoints.len()].clone();
    slots.spawn(run_supervisor(cfg.clone(), endpoint, /* … */));
}

// crates/duotunnel-client/runtime/app.rs —— 建 N 个而非 1 个
let endpoints = build_quic_endpoints(&config, endpoint_count).await?;
```

`run_supervisor` 签名 `endpoint: quinn::Endpoint`（拥有）**保持不变**——现在每个
supervisor 真正拥有自己的 endpoint，而不是一份 clone。

### 2.2 设计决策

#### D-B1：endpoint 何时创建 —— **启动时预建 N 个**

| 方案 | 优点 | 缺点 | 判定 |
| --- | --- | --- | --- |
| **A 启动预建 `Vec<Endpoint>`** | **fail-fast**：FD/端口/内存不足在启动即暴露；生命周期清晰（= 进程） | 启动多 N 次 socket 创建（µs 级，可忽略） | ✅ **选用** |
| B 每 slot 懒建 | 代码更局部 | 失败发生在运行期；重连风暴时可能反复建/销毁 socket | ❌ |

与现状一致：endpoint 生命周期 = 进程生命周期，重连不重建。

#### D-B2：重连时复用还是重建 —— **复用（保持现状语义）**

endpoint 是**传输层资源**（UDP socket），Connection 失败不代表 socket 坏。
`supervisor.rs:72-87` 的重连循环复用 endpoint 只重建 Connection，**这个语义不变**。

> Corner：若 endpoint 自身故障（socket 错误、`Endpoint::accept` 返回 None），
> v1 不做自愈重建——quinn endpoint 极少进入该状态，且重建涉及"该 slot 的所有连接
> 需重新登录"。**列为 v2**，v1 只加日志与指标暴露该状态。

#### D-B3：endpoint 数量 —— **默认保持 1，实验显式启用 N:M**

```yaml
quic:
  connections: 0      # 0 = auto = min(核数, cgroup quota)  —— 现状不变
  endpoints: 1        # 新增：默认保持现状；N>1 = profile 证明后显式实验
```

- **默认 1**：功能合入不改变现有多连接部署的 socket/资源拓扑；
- **允许 N:M**：若 §2.3 的内存核算成为约束，可设 `endpoints < connections`，
  多个 connection 共享一个 endpoint（按 `slot_idx % endpoint_count` 分配）。

#### D-B4：端口分配 —— **维持 `0.0.0.0:0`，内核分配**

N 个 endpoint 各自绑 `0.0.0.0:0`，内核分配 N 个不同临时端口，**无冲突**。
N 很小（=核数），临时端口耗尽不是现实风险。

> **这正是 client 侧比 server 侧容易的根本原因**：不同源端口 ⇒ 四元组不同 ⇒
> 内核天然把包分流到对应 socket，**不需要 SO_REUSEPORT，更不需要 eBPF CID 路由**。

#### D-B5：创建失败处理 —— **显式 strict/degraded 策略**

第 k 个 endpoint 建失败（多为 FD 上限 `RLIMIT_NOFILE`）：

```yaml
quic:
  endpoint_startup_policy: strict   # strict | degraded
  min_ready_endpoints: 1
```

- `strict`（推荐默认）：必须成功创建精确 N 个，否则清理已创建 endpoint 并启动失败；
- `degraded`：成功数达到 `min_ready_endpoints` 才启动，并暴露 configured/active；
- degraded 不得把 connection slots 静默缩成 k。全部 slots 按
  `slot_idx % active_endpoints` 稳定映射到成功 endpoint；
- 成功数为 0 或低于下限时清理已创建资源并失败。

这样把可用性取舍交给显式策略，不让 FD/端口不足悄悄改变配置拓扑。

#### D-B6：shutdown —— **顺带补 `close()` + `wait_idle()`**

现状**没有任何 endpoint 关闭调用**（全 client 目录 grep 无）。coordinator 必须保留
endpoint owners；slot 只持稳定映射后的 clone。shutdown 顺序固定为：

```text
停止本地 accept
  → fence 新 work
  → drain streams
  → close QUIC connections
  → endpoint.close
  → 有界 wait_idle
```

不能把 endpoint `into_iter` 后完全移交 supervisor，否则中央无法统一 close/wait_idle。
具体 drain owner 与 typed tracker 见 D9。

### 2.3 ⚠️ 资源开销核算（本设计最重要的约束）

每个 endpoint 的固定成本：

| 资源 | 每 endpoint | N=4 | N=16 |
| --- | --- | --- | --- |
| UDP socket（FD） | 1 | 4 | 16 |
| UDP socket buffer requested upper bound | `udp_recv_buf_bytes + udp_send_buf_bytes` 默认请求 8 MiB + 8 MiB | requested 64 MiB | requested 256 MiB |
| endpoint driver task | 1 | 4 | 16 |

`SO_RCVBUF/SO_SNDBUF` 是 requested 值/内核 accounting 上界，不等于启动时一次性提交的
常驻 RSS；Linux `getsockopt` 还常显示 bookkeeping 加倍后的 effective 值。N=16 的
256 MiB 只能写成 requested 合计，不能宣称“已常驻 256 MiB”。仍需治理线性资源上限，
并同时记录 requested、OS effective/accounting、进程 RSS/内核 socket memory 实测值。

**必须一并处理**，三选一：

| 方案 | 做法 | 判定 |
| --- | --- | --- |
| **A 总量分摊** | checked rounding 后按 endpoint 分配严格总预算；若低于单 endpoint 最小值则减少 endpoint 或启动失败 | ✅ **推荐**——保持"总内存预算"语义不变 |
| B 保持每 endpoint 全额 | 文档化 N× 放大 | ❌ 静默放大，违反最小惊讶 |
| C 降低默认值 | 把 8 MiB 默认调小 | ⚠️ 会影响单 endpoint 场景的吞吐，不应因多 endpoint 而改单 endpoint 默认 |

**推荐 A 作为实验预算**，并在启动日志打印：
`endpoints=4 udp_recv_requested=2MiB/ep (requested total 8MiB)`。
不能在总量分摊后再无条件给每个 endpoint 设置 1 MiB 下限，否则高 N 时总量会再次突破
预算。所有乘法/取整使用 checked arithmetic，并读取/记录 OS 最终实际 buffer 值。
注意内核可能对 `SO_RCVBUF` 的报告值加倍并受系统上限限制；必须以该平台实际
`getsockopt` 与内核 accounting 为准，不能由配置值反推 RSS。

QUIC connection/stream flow-control window 是逻辑信用上限，通常也不是 upfront
allocation；预算表需另列 connection/crypto/CID 状态和实测 resident，避免重复同一误判。

### 2.4 场景覆盖 & Corner Cases

| 场景/边界 | 行为 |
| --- | --- |
| `connections: 1`（单连接调试） | endpoints=1，**行为与现状完全一致**，零回归 |
| `connections > 1, endpoints: 1` | 默认路径与现有单 Endpoint 多连接语义等价 |
| `endpoints < connections` | 按 `slot_idx % endpoint_count` 分配，多个 connection 共享一个 endpoint（退化到现状的部分共享） |
| 单核 / cgroup 限 1 核 | `connections` auto=1 → endpoints=1 → 与现状一致 |
| FD 上限不足 | strict 失败并清理；degraded 达到显式下限才运行，均有指标 |
| 重连风暴 | endpoint 不参与重连（D-B2），socket 不反复创建/销毁——**优于每 slot 懒建方案** |
| NAT 场景 | N 个源端口 ⇒ NAT 表项从 1 条变 N 条。N=核数量级，对企业 NAT 无压力；但需在文档注明 |
| 服务端看到的连接 | 从"1 个源端口的 N 条连接"变成"N 个源端口各 1 条"。server 侧按 CID 认连接，**无影响**；但 server 的连接级日志/指标会显示不同源端口 |
| QUIC 连接迁移 | client 主动迁移不受影响（各 endpoint 独立）；**注意**：多 endpoint 下"迁移"不会跨 endpoint 发生 |
| 与 UDP 代理（`udp_entries`）共存 | UDP entry 走的是**已建立的 QUIC 连接的 datagram**（`conn.send_datagram`），不新建 endpoint——**不受影响** |
| shutdown | coordinator 按 D-B6 顺序统一 close + wait_idle（带总超时） |
| 指标 | 需新增 `endpoint_count`（配置 vs 实际）；建议每 endpoint 的收包量可观测，用于验证分流是否均匀 |

### 2.5 论证 / 备选

- **为何不用 SO_REUSEPORT（client 侧）**：client 是**发起方**，每个 socket 用不同
  源端口即可让内核按四元组分流；`SO_REUSEPORT` 是给"多个 socket 监听同一端口"的
  服务端用的。client 侧用它是多余且更复杂。
- **为何 server 侧不能照搬**：所有 client 连到**同一目标端口**，多 socket 必须
  `SO_REUSEPORT`；而 QUIC **连接迁移**会改变四元组，内核哈希会把同一连接的后续包
  分到**另一个 socket**，那个 socket 不认识该 CID → 丢包。所以 server 侧需要
  **eBPF 按 CID steering**（Phase D / TODO-24），这是两者难度差异的根本原因。
- **为何不做 endpoint 池化/复用**：endpoint 生命周期 = 进程，无复用需求；池化只会
  增加状态机复杂度。
- **备选：多进程 + SO_REUSEPORT**：也能并行收包，但引入进程间状态同步（EntryConnPool、
  指标、配置热重载都要跨进程），代价远大于多 endpoint。

### 2.6 取舍 / 改动量 / 影响面

| 项 | 内容 |
| --- | --- |
| **取舍** | 付出 N 个 UDP socket（FD + buffer 内存，靠 §2.3 分摊控制）与 N 条 NAT 表项；换取收包并行 |
| **改动量** | `app.rs`（建 N 个）+ `mod.rs`（字段改 `Vec`）+ `pool.rs`（分发而非 clone）+ buffer 分摊 + 配置项 + shutdown close ≈ **~80-120 行**，含测试约 **1 天** |
| **不需要改** | `run_supervisor` 签名、重连逻辑、`EntryConnPool`、`ConnectionHandle`、server 侧全部 |
| **影响面** | 仅 client 启动与 QUIC 传输层；`connections=1` 时零行为变化；回滚 = 还原 `Vec` 为单个 |

### 2.7 分阶段实施

| 阶段 | 内容 | 验收 |
| --- | --- | --- |
| Step 1 | coordinator 持 `Vec<Endpoint>` + slots 稳定 N:M 映射 | `connections=1` 与 `connections>1,endpoints=1` 语义等价 |
| Step 2 | requested/effective/accounting/RSS 观测 + 总预算 | 不把 socket window 误报为常驻 RSS |
| Step 3 | strict/degraded（D-B5）+ endpoint 指标 + D9 shutdown | strict 失败；degraded 仅在达到下限时运行；shutdown 无泄漏 |
| Step 4 | 显式 opt-in 扩展曲线验证（1/2/4 核） | 收包热点改善且 RSS/FD/drop 未显著恶化 |

---

## 3. Phase B 需要自定义 Runtime 吗？——**不需要**

**修订结论：Phase B 与运行时模型在机制上可独立，但二者都必须由 D10 的数据门槛触发；
不能把“可独立”写成“必做”或“无前置”。**

| 理由 | 说明 |
| --- | --- |
| 收包并行**不依赖**线程模型 | N 个 endpoint = N 个 driver task；在 multi_thread 下 tokio 会把它们调度到不同 worker，**已经并行** |
| 改动面完全不重叠 | Phase B 动的是"建几个 socket"；自定义 runtime 动的是"用几个 runtime 跑 task" |
| 风险与可回滚性 | Phase B ~1 天、可独立回滚；自定义 runtime 3-5 天且改变调度语义 |
| 顺序上 B 是 C 的前置 | 若将来做 C，需要"每个 runtime 拥有自己的 endpoint"——**B 先做正好为 C 铺路** |

**所以：先由 D10 的 profile 决定是否做 B；一旦决定实施，它不要求同时引入自定义
runtime，但必须遵守 D9 的 ownership/readiness/shutdown 模型。**

---

## 4. 自定义 Runtime vs multi_thread + 绑核：区别与性能差异

### 4.1 机制层面的区别

| 维度 | **multi_thread（+绑核）** | **N × current_thread（自定义 runtime 池）** |
| --- | --- | --- |
| 任务调度 | work-stealing：本地队列空时偷 victim 一半（原子 CAS） | 纯本地队列，**无窃取** |
| 任务迁移 | 可能跨 worker 迁移 → 冷缓存重放 | **永不迁移**（task 绑定 runtime） |
| 负载均衡 | **自动**（窃取兜底） | 靠**入口分发**（SO_REUSEPORT / endpoint 分配） |
| I/O driver | **共享 epoll**，worker 争抢 driver 所有权 | 每 runtime **独立 epoll** |
| 资源本地化 | 需**手动分片**（hyper 池、计数器） | **天然本地**（每 runtime 一份） |
| 尾部延迟 | 窃取/迁移带来抖动 | 更平稳 |
| **鲁棒性** | 某核任务变重 → **其他核帮忙** | 某核卡住 → **无人帮忙**（需入口层均衡兜底） |
| 线程数动态性 | worker 数**启动固定**（tokio 本来如此）；blocking pool 动态 | 同样固定；blocking pool 可共享 |

### 4.2 性能差异有多大——分解来看

**关键洞察：per-core runtime 的收益要拆成两块，而其中大头不需要换 runtime 就能拿到。**

| 收益来源 | 量级（估计） | **换 runtime 才能拿到？** |
| --- | --- | --- |
| **① 调度本身**（免窃取 CAS、免任务迁移、独立 epoll） | 未测；只在 scheduler profile 显著时评估 | ✅ 是 |
| **② 资源本地化**（pool、计数器、缓冲） | 未测；需分别证明共享状态是热点 | ❌ **否——分片即可**（Phase B′） |

对 DuoTunnel 的具体判断：

- **任务粒度偏大**：每个 task 是"一个请求 / 一条 stream，中间多次 await I/O"，
  不是"百万个纳秒级任务"。work-stealing 成为瓶颈的典型场景是后者，
  **所以 ① 对本项目偏向 5% 一侧而非 15%**。
- S2/K1 的共享结构客观存在，但是否为“大头”尚无 profile 证明；只有热点成立时才分片。

> **可引用的外部依据**：pingora 自陈 NoSteal 的动机是"效率等同单线程 runtime 又能
> 吃满多核"（`pingora-runtime/src/lib.rs:15-24`），但**未给出公开的 A/B 数字**。
> 上表的百分比是**基于机制的估计，不是实测**——真实数字必须用 §4.4 的方法在本项目
> 负载上测出来。**不要把估计当结论使用。**

### 4.3 由此得出的建议顺序

```
Phase B  (多 Endpoint实验)                  ← 仅 profile 指向 S1 时启动
   ↓
Phase A  (绑核,消除迁移抖动)                  ← 1 天,代码可被 C 复用
Phase B′ (hyper 池分片 + per-core 计数,解 S2/K1) ← 拿到收益 ②,不换 runtime
   ↓
压测扩展曲线 (1/2/4 核)
   ↓
仍不线性?  → Phase C 自定义 runtime (仅在 scheduler profile 指向它时)
线性了?    → 不做 C,保住 work-stealing 的鲁棒性
```

**理由**：先用**低风险手段拿走大头收益 ②**，再用数据决定值不值得为剩下的 ①
付出"失去 work-stealing 兜底"的代价。

### 4.4 怎么把估计变成实测（决策前必须做）

三档对照，同一负载、同一 cpuset：

| 档 | 配置 | 用途 |
| --- | --- | --- |
| 基线 | multi_thread，不绑核 | 现状 |
| 中 | multi_thread + 绑核 + 分片（A + B′） | 拿走收益 ② |
| 上限 | N × current_thread（原型即可，不必产品化） | 测出收益 ① 的真实值 |

指标：QPS、p99/p99.9、`QPS(N)/(N×QPS(1))` 扩展比、per-core 利用率
（判断是否还有单点）、上下文切换次数。

**决策规则**：若"中"档已达 `≥0.8` 扩展比，且"上限"档相对"中"档提升 `<10%`，
**不做 Phase C**——不值得为此失去 work-stealing 的鲁棒性兜底。

---

## 5. 验收

**Phase B**：
- [ ] `connections=1` 与 `connections>1,endpoints=1` 时选择/连接语义等价；
- [ ] N>1 时 N 个 endpoint 各自收包，分流大致均匀（每 endpoint 收包量指标可见）；
- [ ] requested/effective/accounting/RSS 分开记录；总预算不随 N 静默突破；
- [ ] FD 不足时 strict 清理并失败；degraded 仅在达到显式下限时运行并告警；
- [ ] shutdown 时 endpoint 正确 close/wait_idle，无 socket 泄漏；
- [ ] 扩展曲线相对改造前提升（收包不再是瓶颈）。

**runtime 决策**：
- [ ] 三档对照数据齐备（§4.4），按决策规则给出"做/不做 Phase C"的结论，
      **而非凭估计拍板**。
