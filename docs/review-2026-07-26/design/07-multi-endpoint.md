# D7 · Phase B：client 多 Endpoint —— 详细设计

> 承接：02 §2.1（S1 收包单点）、02 §4 Phase B（修正后的第一优先）。
> 目标：消除 **quinn endpoint driver 的收包单点**，让 client 的 QUIC 收包随核扩展。
> **与运行时模型完全正交**——无论将来是否上 per-core runtime，这一步都要做。
>
> **状态：已记录、待定，未排期。** 技术上无前置、可最先启动，但当前不实施。
> 重启时先复核 §1.1 的代码事实是否漂移（本设计基于 2026-07-26 的 HEAD）。

## 1. 背景与问题

### 1.1 现状拓扑（已核实）

```
client/runtime/app.rs:38   build_quic_endpoint()  ← 全进程唯一一次
        │
        └─ TunnelPoolService { endpoint }        （client/tunnel/mod.rs:17）
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
| endpoint 全进程建一次 | `client/runtime/app.rs:38` | 唯一改动源头 |
| N 个 slot 共享同一 clone | `client/tunnel/pool.rs:23` | **改这里即可** |
| `supervisor` 签名已是**拥有** `endpoint: quinn::Endpoint` | `supervisor.rs:50-57` | **签名不用改** |
| 重连时复用同一 endpoint（只重建 Connection） | `supervisor.rs:72-87` 循环内 `run_client(&config, &endpoint, …)` | 语义保持不变 |
| UDP socket 绑 `0.0.0.0:0`（内核分配临时端口） | `client/tunnel/endpoint.rs:24` | 多 endpoint 天然不冲突 |
| `EntryConnPool` **不感知** endpoint，只存 `ConnectionHandle` | `client/tunnel/conn_pool.rs` 全文无 endpoint | **pool 不用改** |
| 无 `endpoint.close()` / `wait_idle()` 调用 | 全 client 目录 grep 无 | 关闭路径需顺带补（§2.7） |

**结论：改动面比预想的小得多**——`supervisor` / `conn_pool` / 重连逻辑都不用动，
只需把"一个 endpoint clone N 份"换成"建 N 个 endpoint 分给 N 个 slot"。

### 1.2 为什么这解的是主串行点

见 02 §2.1：quinn 的 **Connection driver 已经并行**（每 Connection 独立 task，
tokio 调度到不同核），**只有 endpoint driver 的收包是单点**——所有连接的所有 UDP 包
都要先过那一个 task 才能按 CID 分发。多 Endpoint 的唯一目的就是让这一步并行。

---

## 2. 详细设计

### 2.1 核心改动

```rust
// client/tunnel/mod.rs —— TunnelPoolService 改为持有 N 个 endpoint
pub(crate) struct TunnelPoolService {
    pub(crate) endpoints: Vec<quinn::Endpoint>,   // 原: endpoint: quinn::Endpoint
    pub(crate) resolved_connections: u32,
    // ... 其余不变
}

// client/tunnel/pool.rs —— 循环里各取一个，不再 clone 同一个
for (i, endpoint) in endpoints.into_iter().enumerate() {
    slots.spawn(run_supervisor(cfg.clone(), endpoint, /* … */));  // 签名不变，移动进去
}

// client/runtime/app.rs —— 建 N 个而非 1 个
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

#### D-B3：endpoint 数量 —— **默认 1:1 跟随 `connections`，可配置**

```yaml
quic:
  connections: 0      # 0 = auto = min(核数, cgroup quota)  —— 现状不变
  endpoints: 0        # 新增：0 = 跟随 connections（1:1）；N = 显式指定
```

- **1:1 是默认**：`connections` 本就 = 核数（量级小），1:1 让收包完全并行，最简单；
- **允许 N:M**：若 §2.3 的内存核算成为约束，可设 `endpoints < connections`，
  多个 connection 共享一个 endpoint（按 `slot_idx % endpoint_count` 分配）。

#### D-B4：端口分配 —— **维持 `0.0.0.0:0`，内核分配**

N 个 endpoint 各自绑 `0.0.0.0:0`，内核分配 N 个不同临时端口，**无冲突**。
N 很小（=核数），临时端口耗尽不是现实风险。

> **这正是 client 侧比 server 侧容易的根本原因**：不同源端口 ⇒ 四元组不同 ⇒
> 内核天然把包分流到对应 socket，**不需要 SO_REUSEPORT，更不需要 eBPF CID 路由**。

#### D-B5：创建失败处理 —— **降级 + 下限，不整体失败**

第 k 个 endpoint 建失败（多为 FD 上限 `RLIMIT_NOFILE`）：

```
成功 ≥1 个 → warn（记录期望 N / 实际 k）+ 按实际 k 个运行，connections 相应收敛到 k
成功 = 0 个 → 启动失败（无法工作）
```

理由：FD 限制是运维环境问题，**不应让服务完全起不来**；但必须在启动日志和
`/metrics` 里明确暴露"降级运行"，避免静默性能损失（对齐 TODO-140 的有效配置遥测）。

#### D-B6：shutdown —— **顺带补 `close()` + `wait_idle()`**

现状**没有任何 endpoint 关闭调用**（全 client 目录 grep 无）。1 个 endpoint 泄漏
不明显，**N 个就明显了**。建议 Phase B 顺带：

```rust
// shutdown 时对每个 endpoint
endpoint.close(0u32.into(), b"client shutting down");
// 带超时等待在途包发完
tokio::time::timeout(drain_timeout, endpoint.wait_idle()).await;
```

这同时改善 **CR-AUDIT-21** 的 drain 覆盖（当前 drain 只看 TCP + pending open_bi，
不等 QUIC）。属顺带收益，不扩大 Phase B 的范围。

### 2.3 ⚠️ 资源开销核算（本设计最重要的约束）

每个 endpoint 的固定成本：

| 资源 | 每 endpoint | N=4 | N=16 |
| --- | --- | --- | --- |
| UDP socket（FD） | 1 | 4 | 16 |
| **UDP socket buffer** | **`udp_recv_buf_bytes` + `udp_send_buf_bytes` = 8 MiB + 8 MiB**（`transport/quic.rs:30-31` 默认） | **64 MiB** | **256 MiB** |
| endpoint driver task | 1 | 4 | 16 |

**🔴 关键发现：UDP socket buffer 默认 8 MiB 收 + 8 MiB 发，多 endpoint 会线性放大。**
16 核机器 1:1 就是 **256 MiB 仅 socket buffer**——这是不可接受的静默内存放大。

**必须一并处理**，三选一：

| 方案 | 做法 | 判定 |
| --- | --- | --- |
| **A 总量分摊** | `per_endpoint_buf = total_buf / endpoints`，设下限（如 1 MiB） | ✅ **推荐**——保持"总内存预算"语义不变，用户配的是总量 |
| B 保持每 endpoint 全额 | 文档化 N× 放大 | ❌ 静默放大，违反最小惊讶 |
| C 降低默认值 | 把 8 MiB 默认调小 | ⚠️ 会影响单 endpoint 场景的吞吐，不应因多 endpoint 而改单 endpoint 默认 |

**推荐 A**，并在启动日志打印：`endpoints=4 udp_recv_buf=2MiB/ep (total 8MiB)`。
注意内核会对 `SO_RCVBUF` 加倍并受 `net.core.rmem_max` 限制（`tune-os.sh` 已设 16 MiB），
分摊后更容易落在内核允许范围内，**反而减少"设了但没生效"的静默失败**。

### 2.4 场景覆盖 & Corner Cases

| 场景/边界 | 行为 |
| --- | --- |
| `connections: 1`（单连接调试） | endpoints=1，**行为与现状完全一致**，零回归 |
| `endpoints < connections` | 按 `slot_idx % endpoint_count` 分配，多个 connection 共享一个 endpoint（退化到现状的部分共享） |
| 单核 / cgroup 限 1 核 | `connections` auto=1 → endpoints=1 → 与现状一致 |
| FD 上限不足 | D-B5 降级 + warn + 指标；不整体失败 |
| 重连风暴 | endpoint 不参与重连（D-B2），socket 不反复创建/销毁——**优于每 slot 懒建方案** |
| NAT 场景 | N 个源端口 ⇒ NAT 表项从 1 条变 N 条。N=核数量级，对企业 NAT 无压力；但需在文档注明 |
| 服务端看到的连接 | 从"1 个源端口的 N 条连接"变成"N 个源端口各 1 条"。server 侧按 CID 认连接，**无影响**；但 server 的连接级日志/指标会显示不同源端口 |
| QUIC 连接迁移 | client 主动迁移不受影响（各 endpoint 独立）；**注意**：多 endpoint 下"迁移"不会跨 endpoint 发生 |
| 与 UDP 代理（`udp_entries`）共存 | UDP entry 走的是**已建立的 QUIC 连接的 datagram**（`conn.send_datagram`），不新建 endpoint——**不受影响** |
| shutdown | D-B6 逐个 close + wait_idle（带超时） |
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
| P1 | `Vec<Endpoint>` + 分发（buffer 暂不分摊，仅 `connections=1` 与小 N 验证） | `connections=1` 逐字节等价；N>1 能正常建连转发 |
| P2 | **UDP buffer 总量分摊**（§2.3 A）+ 启动日志打印生效值 | 内存不随 N 线性放大；日志可见 per-endpoint 值 |
| P3 | 失败降级（D-B5）+ endpoint 指标 + shutdown close/wait_idle（D-B6） | FD 不足时降级运行且可观测；shutdown 无 socket 泄漏 |
| P4 | 扩展曲线验证（1/2/4 核） | 收包不再是瓶颈；`QPS(N)/(N×QPS(1))` 相对改造前提升 |

---

## 3. Phase B 需要自定义 Runtime 吗？——**不需要**

**明确结论：Phase B 与运行时模型完全正交，不需要也不应该捆绑自定义 runtime。**

| 理由 | 说明 |
| --- | --- |
| 收包并行**不依赖**线程模型 | N 个 endpoint = N 个 driver task；在 multi_thread 下 tokio 会把它们调度到不同 worker，**已经并行** |
| 改动面完全不重叠 | Phase B 动的是"建几个 socket"；自定义 runtime 动的是"用几个 runtime 跑 task" |
| 风险与可回滚性 | Phase B ~1 天、可独立回滚；自定义 runtime 3-5 天且改变调度语义 |
| 顺序上 B 是 C 的前置 | 若将来做 C，需要"每个 runtime 拥有自己的 endpoint"——**B 先做正好为 C 铺路** |

**所以：先做 B。它在两种运行时模型下都需要，且做完就能拿到收益。**

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
| **① 调度本身**（免窃取 CAS、免任务迁移、独立 epoll） | **约 5–15%**，任务越短越碎差异越大 | ✅ 是 |
| **② 资源本地化**（per-core hyper 池、计数器、缓冲） | **可能大于 ①**（消除 S2 锁竞争 + K1 缓存行颠簸） | ❌ **否——分片即可**（Phase B′） |

对 DuoTunnel 的具体判断：

- **任务粒度偏大**：每个 task 是"一个请求 / 一条 stream，中间多次 await I/O"，
  不是"百万个纳秒级任务"。work-stealing 成为瓶颈的典型场景是后者，
  **所以 ① 对本项目偏向 5% 一侧而非 15%**。
- **② 才是本项目的大头**：S2（进程级共享 hyper 池的 per-host idle 锁）和
  K1（每连接/每 stream 的全局原子计数）在 8k QPS + 多核下是实打实的竞争，
  **而这两个用分片就能解，不必换 runtime**。

> **可引用的外部依据**：pingora 自陈 NoSteal 的动机是"效率等同单线程 runtime 又能
> 吃满多核"（`pingora-runtime/src/lib.rs:15-24`），但**未给出公开的 A/B 数字**。
> 上表的百分比是**基于机制的估计，不是实测**——真实数字必须用 §4.4 的方法在本项目
> 负载上测出来。**不要把估计当结论使用。**

### 4.3 由此得出的建议顺序

```
Phase B  (多 Endpoint,解 S1 收包单点)        ← 主串行点,与 runtime 正交
   ↓
Phase A  (绑核,消除迁移抖动)                  ← 1 天,代码可被 C 复用
Phase B′ (hyper 池分片 + per-core 计数,解 S2/K1) ← 拿到收益 ②,不换 runtime
   ↓
压测扩展曲线 (1/2/4 核)
   ↓
仍不线性?  → Phase C 自定义 runtime (仅剩收益 ①,约 5-15%)
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
- [ ] `connections=1` 时行为与改造前逐字节一致；
- [ ] N>1 时 N 个 endpoint 各自收包，分流大致均匀（每 endpoint 收包量指标可见）；
- [ ] **UDP buffer 总量不随 N 线性放大**（§2.3 分摊生效，启动日志可见 per-endpoint 值）；
- [ ] FD 不足时降级运行 + warn + 指标，而非启动失败；
- [ ] shutdown 时 endpoint 正确 close/wait_idle，无 socket 泄漏；
- [ ] 扩展曲线相对改造前提升（收包不再是瓶颈）。

**runtime 决策**：
- [ ] 三档对照数据齐备（§4.4），按决策规则给出"做/不做 Phase C"的结论，
      **而非凭估计拍板**。
