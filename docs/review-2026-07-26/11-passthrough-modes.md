# 透传模式进度评估：TCP(L4) / L7 / UDP（2026-07-26）

## 背景

用户预期 DuoTunnel 支持三类透传：**TCP 透传（L4 裸字节）**、**L7 透传（HTTP 感知
转发）**、**UDP 代理透传（over QUIC datagram）**。本文核对三者的**实现进度**、
**缺口**、**如何补齐**。方法：代码为准，file:line。

三种模式共享同一套下游选择（`ClientRegistry` P2C）+ relay 引擎，因此前述文档的
若干跨切面缺陷（08 选择分片、01§3.1 relay UB、09§4.1 客户端 IP 透传）**在三种模式
上都成立**，本文不重复展开，只在每模式标注"受哪条影响"。

## 问题陈述

对 TCP/L7/UDP 三种透传：(1) 现在**能不能用**、覆盖到什么程度？(2) 距离"生产可用
的通用透传"还差哪些？(3) 补齐的具体改法、改动量与依赖？

## 结论速览

| 模式 | 进度 | 能用？ | 主要缺口 | 补齐量级 |
| --- | --- | --- | --- | --- |
| **TCP(L4) 透传** | ~85% | ✅ 可用 | 无 PROXY protocol（后端看不到客户端 IP）；路由仅端口→group（裸 TCP 无 SNI 路由）；共享 relay UB / 08 选择缺陷 | 小 |
| **L7 透传** | ~75% | ✅ 可用（多协议） | chunked 正确性(01§3.2)、100-continue(09§4.5)、XFF/改写无 seam(09§4.1/10§6.1)、H2 防滥用(09§4.4)；upstream 级 L4/L7 开关未实现且**已搁置**(01§4.2) | 中 |
| **UDP 代理** | ~60% | 🟡 MVP（小包/DNS 可用） | **仅 egress 方向，无 UDP ingress 监听器**(§6.1)；session 表无上限(07§4.3)、1200 硬编码不按 MTU、每包分配+rkyv+wall-clock、无 UDP 限流 | 中 |

**一句话**：三种模式**都已跑通**（不是缺失，而是完成度参差）。TCP 透传最完整、
只差 PROXY protocol；L7 透传覆盖最广但带 01/09 的正确性尾巴与缺 L7 filter 层；
UDP 是**最小可用原型**且**只有 egress 一个方向**（server 侧没有 UDP ingress 监听器，
§6.1），高 PPS/大包/容量治理尚未做，是三者中离生产最远的。

---

## 4. TCP(L4) 透传 —— ~85%，最完整

### 4.1 现状与证据

**两条入口**：
1. **专用 `mode: tcp` 监听器**：`run_tcp_accept_loop`（`handlers/tcp.rs:12`）带
   **静态** `group_id`+`proxy_name`（来自监听器配置，`listener_mgr.rs:89-90`、
   `config.rs:76-77`）；仍做 sniff（`tcp.rs:56-73`，用于识别 protocol + 取 SNI/host
   记日志），但**路由是端口→group 固定**，不按 SNI 路由。
2. **HTTP 监听器回退**：`TcpPassHandler`（`plugins/tcp_pass/mod.rs`，
   `ProtocolKind::Tcp`）——当 HTTP 监听器上 sniff 出不可识别字节 / 不透明 TLS 时走它，
   此路**可按 SNI 经 vhost 路由**（route 来自 `RouteResolver`）。

**转发**：两条入口都走 `forward_prefixed_to_client`（`proxy/base.rs:51`）——先把
sniff 预读的 preface 回放进 QUIC，再双向字节 relay（`copy_quic_to_shutdown` +
`copy_buffered_then_finish`）。opaque TLS 透传（不解密）由此实现。

### 4.1.1 TCP 监听器的转发规则与 sniff 的真实作用 `[新发现]`

**转发规则（已核实，无二级路由）**：
`TcpListenerDef { client_group: GroupId, proxy_name: ProxyName }`
（`crates/duotunnel-store/src/config/mod.rs:228-232`）——一个 `mode: tcp` 端口**静态绑定唯一一组
`(group, proxy_name)`**。所以"哪个端口进来的请求全部转发到对端（的同一个 proxy）"这个理解
**完全正确**：端口即路由键，没有任何二级路由（无 SNI、无 host、无路径）。

**但 sniff 仍然会做**（`handlers/tcp.rs:56-73`），且 sniff 结果被用于**两件事**：

1. **填充 `RoutingInfo.protocol`** —— 这个字段**决定了 client 侧走 L7 还是 L4**
   （client 收到 `Protocol::H1` 就跳过二次 sniff、直接进 `Http1Driver` 完整解析链，
   见 01§1.1）；
2. 提取 host/SNI **仅用于日志**。

**由此得出的反直觉结论**：**从 `mode: tcp` 监听器进来的流量，只要 sniff 判定它是 HTTP，
client 侧依然会做完整的 L7 解析/重建/重编码**——用户配置了一个"四层监听器"，得到的却是
七层代理的成本与语义（含 01§3.2 的 chunked 改写）。这**大概率不是配置 `mode: tcp` 时的预期**。

**关联**：消除该行为的手段正是 **01§4.2 的 upstream 级 `mode: l4` 开关**（`l4` 时
`upstream_peer` 忽略 sniff 出的 protocol、直接返回 `PeerSpec::Tcp`）——该项**当前已搁置
（仅记录需求，暂不实施）**，故本反直觉行为在近期仍然存在，属**已知且被接受**的现状。

### 4.2 缺口

- **无 PROXY protocol → 后端看不到真实客户端 IP**：`RoutingInfo.src_addr` 已带
  客户端地址，但 TCP 透传只把**裸字节**交给后端，src_addr 仅用于 client 侧
  `client_addr`（日志/选路），**从不注入**。裸 TCP 无 HTTP 头可用，须用
  **PROXY protocol v2**（HAProxy 事实标准）在字节流前置一段头传地址。这是 L4 LB
  的对应能力（09§4.1 的 L4 版本）。
- **裸 TCP 路由仅端口→group**：非 TLS 的裸 TCP 在 `mode: tcp` 监听器上无法按目标
  区分（一个端口一个后端组）；要多路复用一个端口需要 TLS-SNI 或应用层信息。
- **共享缺陷**：relay UB（01§3.1）、下游选择 08 分片缺陷、`max_attempts=3` 无预算
  （09§4.3）。

### 4.3 补法 / 论证 / 改动量

- **PROXY protocol v2**：在 client 侧 `connect_peer` 的 TCP 分支、向后端建连后**先写
  一段 PROXY v2 头**（携带 `RoutingInfo.src_addr:src_port` 与本地目的），再进 relay。
  配置开关 `proxy_protocol: true`（后端需支持，故默认关）。~0.5-1 天。
  - *论证*：PROXY v2 是 L4 传客户端地址的唯一通用手段（Envoy/HAProxy/nginx-stream
    均用）；不能像 HTTP 那样加 header。二进制 v2 优于 v1（定长、含 TLV）。
  - *Corner*：后端不支持 PROXY 时会把头当数据 → 必须按后端显式开启；opaque TLS
    透传下 PROXY 头在 TLS 记录之前，后端需先解 PROXY 再 TLS。
- 其余缺陷随对应文档（01/08/09）修复自动受益。

**进度判定 ~85%**：功能完整、opaque TLS/裸 TCP 都能透传；主要只差 PROXY protocol
这一项 L4-LB 标配，加上共享的正确性/选择修复。

---

## 5. L7 透传 —— ~75%，覆盖最广

### 5.1 现状与证据

覆盖协议最全，且**已有两种 L7 "透传"语义**：
- **透明字节级回放**（更接近"L7-aware 透传"）：`H1Handler`（`plugins/h1/mod.rs`）
  识别 H1/WS 后**回放 preface + 字节 relay**，不重编码——server ingress 侧即此。
- **完整解析/重编码**（终止侧）：`Http1Driver`（`driver/h1.rs`）keep-alive 循环，
  在做 L7 的一侧（ingress=crates/duotunnel-client/egress=server）完整解析请求、构造、重编码响应。
- **H2c**（`plugins/h2c`，每请求 vhost 解析）、**TLS 终止→H2**（`plugins/tls`，
  MITM 动态签发→serve H2→按请求转发）、**WebSocket**（sniff→`Protocol::WebSocket`
  →TCP relay）。

### 5.2 缺口（均已在 01/09/10 详列，此处汇总定位）

- **正确性**：响应无条件 chunked + 204/304/HEAD 违规（01§3.2）；`Expect:100-continue`
  下游挂起（09§4.5）；请求走私 CL/TE 未防（07§4.7）。
- **能力/抽象**：客户端 IP 透传缺失、header/路径改写/镜像/重定向**无 per-request
  filter seam**（09§4.1 + 10§6.1）；面向公网 H2 未设显式防滥用上界（09§4.4）。
- **性能**：三重 BoxBody 装箱、body 双拷贝（01§4.1/§3.4）。
- **未实现的优化**：**upstream 级 L4/L7 开关**（01§4.2，原称"passthrough-vhost 快路径"）——
  对不需要改写/按请求路由的 upstream（`mode: l4`）退化为字节转发以省 L7 重建；
  **⏸️ 已搁置（仅记录需求，暂不实施）**。它同时也是 §4.1.1 那条"TCP 监听器仍走 L7"反直觉行为的解法。

### 5.3 补法

L7 透传的"补齐"本质是 **10§6.1 的 HttpFilter 层 + 01 的正确性修复**：
- 先修 01§3.2/09§4.5（正确性，P0）；
- 引入 `HttpFilter` 链（10§6.1）承接 XFF/改写/镜像/重定向 + 每请求可观测；
- upstream 级 L4/L7 开关（01§4.2）可提升纯转发容量——**当前搁置，不在补齐路径上**。
改动量中（依赖 10§6.1 的 filter 抽象）。**进度 ~75%**：多协议已通，差正确性尾巴 +
L7 filter 承接层。

---

## 6. UDP 代理（over QUIC datagram）—— ~60%，最小可用原型

### 6.1 现状与证据

**首要事实：UDP 只有 egress 一个方向** `[新发现]`

现有 UDP 链路**只覆盖 client→server 的 egress 方向**：client 侧 `udp_entries` 绑本地 UDP 监听
→ QUIC datagram → server → 外部 upstream。**server 侧不存在 UDP ingress 监听器**：
`IngressModeDef` 只有 `Http` 与 `Tcp` 两个变体（`crates/duotunnel-store/src/config/mod.rs:217-220`），
配置层根本无法声明一个 UDP 入口。

| | **ingress**（外部 → server → 隧道 → client → 内网服务） | **egress**（本地 → client → 隧道 → server → 外部） |
| --- | --- | --- |
| **TCP** | ✅ `mode: tcp` 监听器（`TcpListenerDef`，见 §4） | ✅ client 侧 TCP 监听（§4/§5 共用 relay） |
| **UDP** | ❌ **不存在**（`IngressModeDef` 无 Udp 变体） | ✅ `udp_entries` + QUIC datagram（§6.1 链路） |

**因此："对齐 TCP listener"（在 server 侧加一个 UDP ingress 监听器：外部 UDP → server → 隧道 →
client → 内网 UDP 服务）是一个新功能，不是补缺口**——它需要新增 `IngressModeDef::Udp` 配置变体、
server 侧 UDP 监听与 session 表、以及 client 侧的 UDP upstream 连接方向（现有 session 管理只在
server 侧、只面向外部 upstream）。本文其余缺口（§6.2）全部只针对既有的 egress 方向。

**链路**（TODO-26 已完成基础，**egress 方向**）：
- **client**：`udp_entries` 绑本地 UDP（`egress/udp_listener.rs:53`）→ 每包封成
  `UdpDatagramEnvelope`（rkyv，`msg.rs`）→ **QUIC datagram** 发送；按
  `(proxy_name, client_ip, client_port)` 哈希选 shard、P2C 选连接（**每包重选**，
  `udp_listener.rs:88-116`）。
- **server**：`UdpSessionManager`（`handlers/udp_datagram.rs`）每 QUIC 连接一个；
  按 `UdpSessionKey=(proxy_name,client_addr,client_port)` 建 session、每 session 一个
  上游 UDP socket + reply pump（`pump_udp_replies:141`）；**30s 空闲淘汰**（10s tick
  扫描，`:47-72`）。
- **上游选择**：`resolve_udp_target`→`next_healthy` 轮询（`crates/duotunnel-server/egress/mod.rs:43`）。

### 6.2 缺口

- **session 表无上限**（`udp_datagram.rs:33` `DashMap` 无 cap）：攻击者变化源端口可
  撑大 session 数 + 每 session 一个 socket+task+reply pump → **FD/内存耗尽**
  （07§4.3 / TODO-144）。
- **`MAX_DATAGRAM_BYTES=1200` 硬编码**（`msg.rs:15`）：大于此的包被**静默丢弃**
  （`udp_listener.rs:83-86`、`udp_datagram.rs:161-166`）；未按协商的 QUIC
  datagram / path MTU 处理，也无分片（TODO-144）。
- **每包开销**：`buf[..n].to_vec()`（`udp_datagram.rs:159`）每包分配 + rkyv
  encode/decode 每包一次 + `current_time_secs()`（`:18`，`SystemTime::now` wall-clock
  syscall）每包调用——高 PPS 下 allocator/syscall 受限（TODO-144 + TODO-88）。
- **每包重选连接**：一个 UDP flow 的包可能跨不同 QUIC 连接发送；server 端因 session
  key 稳定仍能正确归组，但 reply 走收到包的那条连接，flow 分散、次优。
- **无 UDP 限流 / 无 PPS 过载保护**；datagram 不可靠（符合 UDP 语义，但无上层重传选项）。

### 6.3 补法 / 论证

1. **session 容量治理**（并入 TODO-144）：session 数上限 + LRU/每客户端配额 +
   exhaustion 指标；~1 天。*论证*：无上限的 per-flow socket/task 是明确 DoS 面。
2. **按协商 MTU 处理大包**（TODO-144）：读 QUIC `max_datagram_size`，超限时**分片
   或明确回退**（而非静默丢），至少暴露 drop 指标。~1 天。
3. **降每包开销**：`Bytes`/借用式 decode 避免 `to_vec`；粗粒度单调时钟（TODO-88）
   替 `SystemTime::now`；紧凑定长头替 rkyv envelope（可选）。benchmark-gated，
   先按 TODO-140 的 PPS 基线证明热点再做。
4. **flow 亲和**（可选）：同一 UDP flow 固定一条 QUIC 连接（sticky by session key），
   减少 reply 分散。

**范围裁剪（用户定位决定，本次记录）**：DuoTunnel 的 UDP 定位是**小包 / DNS 场景**，因此
**近期只需要第 1 项 session 表容量上限（~1 天）**；TODO-144 全量范围里的**高 PPS 零拷贝、
MTU 分片处理**（第 2、3 项）**当前不需要**，仅保留为登记项，待出现真实大包/高 PPS 需求再评估。
UDP ingress 方向（见 §6.1 对比表）同样属未排期的新功能。

**进度 ~60%**：DNS/小包/低 PPS 场景可用（**即当前定位的目标场景**）；容量治理是近期唯一必做项，
大包/高 PPS 按定位暂不投入。

---

## 7. 实施顺序与依赖

```mermaid
flowchart TD
    subgraph 正确性/安全（P0，先行）
        C1[L7: chunked/100-continue 01§3.2/09§4.5]
        C2[UDP: session 容量上限 07§4.3/TODO-144]
    end
    subgraph 能力补齐
        P1[TCP: PROXY protocol v2]
        HF[L7: HttpFilter 层 10§6.1 → XFF/改写/镜像]
        U1["UDP: MTU 大包处理 + drop 指标<br/>⏸️ 按定位暂不需要"]
    end
    subgraph 优化（benchmark-gated）
        PV["L7: upstream 级 L4/L7 开关 01§4.2<br/>⏸️ 搁置（仅记录需求）"]
        U2["UDP: 每包零拷贝 + 粗时钟 TODO-144/88<br/>⏸️ 按定位暂不需要"]
    end
    SHARED[跨模式: 08 选择修复 + 01§3.1 relay UB + 09§4.3 重试预算]
    C1 --> HF
    SHARED -.三模式共享受益.-> P1
    SHARED -.-> HF
    SHARED -.-> U1
    T140[TODO-140 基线] --> U2
    T140 --> PV
```

**顺序理由**：
- **跨模式的共享修复（08 选择、01§3.1 UB、09 重试预算）优先**——三种透传都受益，
  且是正确性/公平性底线。
- **L7 正确性（chunked/100-continue）与 UDP session 容量**是各自模式的 P0。
- **能力补齐**：TCP 的 PROXY protocol 独立小改；L7 的 XFF/改写依赖 10§6.1 的
  HttpFilter 层；UDP 大包处理独立，但**按当前小包/DNS 定位暂不需要**（§6.3）。
- **优化项**（01§4.2 的 upstream 级 L4/L7 开关、UDP 每包零拷贝）原本 **benchmark-gated**，
  现均**已搁置**：前者由用户决定仅记录需求，后者按 UDP 定位不投入。UDP 侧近期唯一必做项是
  **session 容量上限**。

## 8. 验收

- [ ] TCP：可选开启 PROXY v2，后端能读到真实客户端地址；
- [ ] L7：chunked/100-continue 合规；XFF 经 HttpFilter 注入；
- [ ] UDP：session 数达上限时快速失败且有指标（**近期唯一必做项**）；
      大于协商 MTU 的包不再静默丢弃（**按小包/DNS 定位暂不需要，仅登记**）；
- [ ] 三模式共享的选择均衡（08 修复）、relay 安全（01§3.1）、重试预算（09§4.3）到位。
