# io_uring 可行性评估（含 pingora 实证对照，2026-07-26）

> **2026-07-27 三轮复核修正**：本文保留“不切 io_uring”的工程结论，但下文所有
> 5–15%、接近线性和 0.5–5 天收益/工期均为当时机制估计，**没有 DuoTunnel 实测依据，
> 不得用于排序或验收**。当前唯一权威性能顺序与统计门槛见
> [14](./14-performance-robustness-stability-addendum.md) 和
> [D10](./design/10-performance-hardening.md)：先可信基线，再按 profile 触发。

## 背景

io_uring 是 Linux 5.1+ 引入的异步 I/O 接口：应用与内核共享一对环形队列
（提交队列 SQ / 完成队列 CQ），把多次操作批量提交、批量收割，从而摊薄
per-syscall 的陷入开销，并让 I/O 走 **completion 模型**（内核持有缓冲区所有权
直到操作完成）。它被提出的唯一动机就是**追求更低的 syscall/调度开销**——在
连接数极高、每连接 I/O 极碎的场景里，syscall 进入次数本身会成为瓶颈。

评估这项技术必须先看 DuoTunnel 的 I/O 构成：

- **主 I/O 是 QUIC/UDP**（quinn），承载绝大部分数据面流量；
- **次 I/O 是 TCP relay**，走大块缓冲区搬运。

结论能否成立，取决于 io_uring 的收益基数在这两条路径上还剩多少——这正是下文
「生态约束分析」与「场景覆盖」要逐条量化的。

## 问题陈述

本文档回答三个具体问题：

1. Rust 下 io_uring 是否因为依赖库都绑定 tokio 而**「没法完全替换」**？
2. DuoTunnel **有没有必要**切换到 io_uring？
3. 业界生产级代理 **pingora 用了 io_uring 吗**？

## 结论速览

**不切换 io_uring。**quinn/tokio 生态的 readiness 契约与 io_uring 的
completion 模型不兼容，桥接层会抵消大部分收益；而 DuoTunnel 的主路径（UDP）
已被 GSO/GRO 批量化、次路径（TCP relay）syscall 已是大块，可挤的空间本就很小；
叠加容器/CI 环境普遍屏蔽 io_uring，理论收益连兑现环境都不稳定。

**决策**：建议把 todo.md 的 **TODO-25** 从 **Deferred** 升级为
**❌ Rejected**——除非 quinn/tokio 官方提供 io_uring 后端再重估。

## pingora 实证（`../pingora` @ 本机 checkout）

**pingora 没有使用 io_uring，任何形式都没有。**

- 全 workspace `*.toml` 无 `io-uring` / `tokio-uring` / `monoio` / `glommio`
  依赖；`*.rs` 无 `io_uring` 符号；CHANGELOG 无相关条目（此前有搜索误报是
  匹配到了英文单词 "d**uring**"；本次以 `\bio_uring\b`、`io[-_]uring` 分别
  复核 `*.rs` / `*.toml` / CHANGELOG，结果均为空）。
- pingora 的多核性能手段是 **`pingora-runtime` 的 `NoSteal` flavor**：N 个
  tokio current-thread runtime（每线程一个），消除 work-stealing 调度开销
  （`pingora-runtime/src/lib.rs:15-24` 明确说明动机——「第三种 flavor：无
  work stealing 的多线程运行时，效率等同单线程运行时又能吃满多核」），配置面
  只有 `threads` + `work_stealing` 两个开关
  （`pingora-core/src/server/configuration/mod.rs:74-82`）。
- pingora **也没有做绑核**（全仓库无 `sched_setaffinity`/`core_affinity`）。

结论：Cloudflare 生产级代理在 epoll(tokio) 上达成其性能目标，靠的是
shared-nothing 运行时形态而非内核 I/O 接口替换。这与 02 号文档的路线互相印证。

## 生态约束分析

「依赖库大量绑定 tokio ⇒ io_uring 无法完全替换」——正确，且比一般理解的更硬。
以下 5 点既是「无法完全替换」的证据，也逐条说明**这具体如何抵消 io_uring 的
理论收益**：

1. **trait 形态不兼容**：tokio/hyper/quinn/tokio-rustls 的 I/O 契约是
   readiness 模型（`AsyncRead/AsyncWrite`，借用调用方缓冲区）；io_uring 是
   completion 模型（内核持有缓冲区所有权直到完成）。桥接必须引入拷贝或
   owned-buffer API（`tokio-uring`/`monoio` 的 `read(buf) -> (res, buf)`）。
   *抵消点*：io_uring 省下的 syscall 开销，被兼容层新增的一次内存拷贝／
   缓冲区所有权来回移交换了回去，**经过兼容层后收益大部分被抵消**。
2. **运行时排他**：`tokio-uring` 要求 current-thread runtime 且维护低活跃度；
   `monoio`/`glommio` 是独立运行时——hyper、quinn、sqlx、metrics 栈都要换或
   自研，等价于重写项目 I/O 层。*抵消点*：理论收益必须先扣掉「整层 I/O 重写＋
   全链路回归」的成本与风险，净收益期望在动手前就已是负。
3. **DuoTunnel 的主 I/O 是 UDP(QUIC)**：quinn 的收发已经通过 `quinn-udp` 用
   `sendmmsg/recvmmsg + GSO/GRO` 把 syscall 摊薄到每批次一次（README 声称的
   该优化在依赖里真实存在）。io_uring 对 UDP 的增量（相对已批量化的路径）
   通常在 5-15% 区间，而且 quinn 上游没有 io_uring 后端可用——自研收益/成本
   完全不成比例。*抵消点*：收益基数是「已批量化后的残余 syscall」，io_uring
   能省的那部分已被 GSO/GRO 提前吃掉大半，可挤空间很薄。
4. **TCP 侧 relay 的 syscall 已经是大块**：64KiB relay buffer
   （`buffer_params.rs:1`）+ `read_chunk` 零拷贝路径下，syscall 次数不是
   主要成本（01 文档 §2 的预算表里 crypto+协议处理 > syscall）。*抵消点*：
   单次 syscall 已搬运 64KiB，per-syscall 固定开销在总成本里占比极低，io_uring
   减少 syscall 次数对这条路径几乎无感。
5. **部署面风险**：Docker 默认 seccomp 自 2023 起屏蔽 `io_uring_*`；GitHub
   Actions runner 的容器环境同样受限；多家大厂（如 Google 生产环境）因漏洞
   历史禁用 io_uring。就算做了，CI 也测不了，生产也未必开得了。*抵消点*：
   收益只在「能开启 io_uring 的环境」里兑现，而目标部署面（容器/CI）大多开不了，
   期望收益要再乘一个远小于 1 的可用性系数。

## 方案论证 / 备选对比

把「减少内核交互/调度开销」这个目标下的各候选手段放在同一张表里对比。列为
收益、成本、部署风险、判定：

| 候选手段 | 收益 | 成本 | 部署风险 | 判定 |
| --- | --- | --- | --- | --- |
| **io_uring 全量**（自研 completion 后端替换 quinn/tokio I/O） | 理论 5-15% UDP 尾部；须绕开整套 readiness 契约 | 重写 I/O 层，数周-数月 | 高：seccomp/CI 普遍屏蔽 | ❌ |
| **tokio-uring 混合**（仅热路径局部改用） | 局部收益，兼容层拷贝/所有权移交抵消大半 | current-thread runtime 排他 + 桥接复杂度 | 高：同上 | ❌ |
| **monoio/glommio 重写**（thread-per-core + io_uring 原生） | 理论上限最高 | hyper/quinn/sqlx/metrics 全换或自研 | 高：同上 | ❌ |
| **维持 epoll + per-core 化**（02 路线，= pingora NoSteal） | 接近线性扩展 + p99 抖动↓ | 1-5 天，纯增量、复用现有 I/O 层 | 低：不新增 syscall 面 | ✅ **最优** |
| **AF_XDP/eBPF 旁路** | 仅极限 UDP PPS 场景才有意义 | 高，独立数据面 | 中-高 | ⏸ 仅 TODO-144 触发时评估 |

**为何「维持 epoll + per-core 化」最优**：它的收益/成本比在全表最高（1-5 天
换接近线性的多核扩展），且**零部署风险**——不引入任何被 seccomp 屏蔽的
syscall，CI 可测、生产可开；路线本身已被 pingora 实证背书（NoSteal ==
per-core current-thread runtime）；更关键的是它**完全复用现有 I/O 层**，不触碰
quinn/tokio 契约，因而没有前三行那种「重写＋回归」的沉没成本。前三行无论收益
排序如何，都被同一个部署面风险和同一份 I/O 层重写成本压到不可取；AF_XDP 则是
更高量级的手段，只在换运行时也救不了的极限 PPS 场景才谈得上，且不需要动运行时。

## 场景覆盖 & Corner Cases

逐一核对可能推翻「不切」结论的场景，确认它们要么不成立、要么反而加固结论：

- **容器化 seccomp 屏蔽 io_uring**：Docker 默认 seccomp profile 自 2023 起
  屏蔽全部 `io_uring_*` 系统调用。DuoTunnel 的目标部署形态包含容器，切了也大
  概率跑在退化路径上 → 收益归零。
- **CI 环境测不了**：GitHub Actions runner 的容器环境同样受限，io_uring 后端
  在 CI 里无法获得覆盖与回归保护 → 一条测不到的关键路径，工程上不可接受。
- **GSO/GRO 已批量化 UDP**：主路径的 UDP 收发已由 `quinn-udp` 的
  `sendmmsg/recvmmsg + GSO/GRO` 摊薄到每批一次 syscall，io_uring 在此之上只剩
  残余增量（5-15% 理论区间）→ 收益基数已被提前吃掉。
- **TCP relay 已大块 syscall**：64KiB relay buffer + `read_chunk` 零拷贝下，
  单次 syscall 搬运量已很大，per-syscall 固定开销占比极低 → io_uring 无感。
- **未来极限 UDP PPS 场景**（例如 UDP 代理 TODO-144）：若真的撞到 UDP PPS
  天花板，正确的第一步是 **AF_XDP/eBPF 旁路评估** 或 **多 endpoint 分片**，
  而不是整体换运行时——换运行时既解决不了 PPS 上限，又要付 I/O 层重写代价。

## 取舍与预期收益

同一份「减少内核交互/调度开销」的预算，应按性价比投给下表（越靠上越优先）：

| 优先级 | 手段 | 预期收益 | 成本 |
| --- | --- | --- | --- |
| 1 | 绑核 + per-core 计数（02 §5） | p99 抖动↓，5-15% QPS | 1 天 |
| 2 | client per-connection endpoint（02 Phase B） | client QUIC I/O 随核扩展 | 0.5 天 |
| 3 | per-core 运行模式（02 Phase C，= pingora NoSteal） | 接近线性的扩展 | 3-5 天 |
| 4 | UDP 批量参数调优（quinn GSO segment 数、socket buf） | 尾部小幅 | 0.5 天 |
| 5 | io_uring | 5-15%（理论），需重写 I/O 层 | 数周-数月 + 部署风险 |

**明确取舍**：本决策**主动放弃 io_uring 那 5-15% 的理论尾部收益**，换取
**不重写 I/O 层**（保住 quinn/tokio/hyper 契约与全部现成生态）以及
**可测试、可部署**（不引入被 seccomp/CI 屏蔽的 syscall 面）。用一份数周-数月
且大概率兑现不了的收益，交换四项 0.5-5 天、CI 可验证、部署无风险的确定性收益
——性价比不成立。

## 结论与后续

**结论：不切换 io_uring。**维持 todo.md 的 TODO-25（Deferred）判断，本文档
补充了 pingora 实证与生态论据，可把该条目升级为
**「❌ Rejected——除非 quinn/tokio 官方提供 io_uring 后端再重估」**。

**触发重估的条件**（满足其一即应重新评估）：

- **quinn / tokio 官方提供 io_uring 后端**——届时无需重写 I/O 层即可获得收益，
  成本项被彻底改写，是唯一能推翻本结论的主要事件；
- 目标部署面的 seccomp/容器限制解除，io_uring 在生产与 CI 均可稳定开启；
- 撞到极限 UDP PPS 天花板（如 TODO-144）——但此时优先评估的是 AF_XDP/eBPF
  旁路或多 endpoint 分片，而非整体换运行时。
