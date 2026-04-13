# Tunnel 代码质量分析

> 分析范围：tunnel-lib / server / client 所有 .rs，忽略 ci-helpers、配置文件、注释。
> 评估维度：语义性（名字是否说清了意图）、抽象组合性（层次是否正交、有无重复/过度抽象）。
> 核心方法论：先梳理业务级流转路线与功能定义，再审视当前代码在模型和语义上是否匹配物理世界的逻辑。

---

## 零、 核心业务流转与领域抽象缺陷

要准确评价代码质量，必须首先建立对业务功能的全局理解。DuoTunnel 是一个高性能的双向代理隧道，其请求流转有两条核心主线：
1. **Ingress (反向代理)**: 外部请求 → Server Ingress 监听器 (根据 Host 或 Port 路由) → 建立/复用 QUIC Stream → 写入 `RoutingInfo` 传递寻址要求 → Client 接收 Stream (反序列化路由信息) → 转发给 Local Backend。
2. **Egress (正向代理)**: 本地服务/应用 → Client Egress 监听器 → 建立 QUIC Stream → Server → 请求公共网络服务。

业务本质是一条连接 L7（HTTP/WS）和 L4（TCP）的纯 L4 QUIC 数据通道。由于涉及多层协议的降级与还原，两端对“元数据（Metadata）”的精确传递与分层隔离尤为关键。

### 缺陷案例剖析：`RoutingInfo` 与 H2 多路复用阻碍 (参照 TODO-56)

由于项目早期在领域语义（Domain Semantics）上缺乏严格的分界，导致代码的抽象极度阻碍了高级网络特性的实现。最核心的反面教材是消息模型中的 `RoutingInfo`：

```rust
pub struct RoutingInfo {
    pub proxy_name: String,   // 连接级：决定去找个哪个后端机器
    pub protocol: String,     // 流级：决定后端将用什么协议解析载荷
    pub src_addr: String,     // 请求级：真实客户端IP
    pub src_port: u16,        // 请求级：真实客户端Port
    pub host: Option<String>, // 请求级：请求携带的真实Host
}
```

- **语义错位**：上述结构体强行把 **流/路由级语义**（Tunnel 应该发给哪个 proxy_name）与 **请求级语义**（每个访客独有的 IP 与 Host）混杂在了一起。且代码在 QUIC Stream 刚建立（`open_bi()` 之后）立刻作为初始化帧一次性发送。
- **架构阻塞**：当我们尝试实施并发优化（例如 TODO-56 提到的 H2 多路复用，通过一条 QUIC Stream 承载成千上万个并发 HTTP 请求）时，这一设计轰然倒塌。因为 H2 复用之后，每个 Request 都有自己的 `src_addr` 和 `host`。如果我们把 `RoutingInfo` 强绑定在 Stream 建连阶段发送，后续的几十万个请求将完全无法携带属于自己的来源语义，要么就只能退变成“一个请求建一条 QUIC Stream”（这完全放弃了 H2 的复用优势）。

**架构重构的唯一出路**：必须把领域对象解耦。`RoutingInfo` 只应保留 `proxy_name` / `protocol` 这种长期不变的“连接标示”。而属于单次请求的 `src_addr` 和 `host` 必须被剥离出隧道控制面协议，转译成 L7 的 `X-Forwarded-For`、`X-Real-IP` 等 Header，交由 H2/H1 proxy 层在具体请求的帧内部传递。

**结论**：这个案例深刻表明：**代码语义如果没能正确反映业务层级（Session vs Stream vs Request），会在引入高阶抽象时造成降维打击。**

### 路由职责越位：Control Plane 与 Data Plane 的过度耦合

在分析 `proxy_name` 的字段流转时，可以发现一个隐藏的架构设计倾向：**Server 承担了过重的“控制面”寻址职责，导致了与 Client 内部网络细节的深度耦合。**

*   **现状困境**：目前的 Ingress (反向代理) 逻辑要求 Server 必须知道 Client 侧定义的 `proxy_name`（如 `"grpc_service"`）。这意味着：
    1.  当一个 Client 想要新增一个本地后端服务时，必须同时在 Server 侧修改路由表映射。
    2.  对于自带寻址语义的 L7 (HTTP) 流量，Server 在 `RoutingInfo` 中同时传递了 `host`（原本已足够寻址）和 `proxy_name`（冗余的指令）。
*   **带来的问题（职责越位）**：Server 作为公网入口（Control Plane），原本只需关心“这波流量发给哪群客户端（Client Group）”。但现在的设计强迫 Server 必须深入到数据面（Data Plane）内部，指名道姓地要求 Client 转发给特定的后端别名。这种“中心化指令式”寻址，在多租户、大规模 Client 场景下，会导致 Server 的路由规则库急剧膨胀且难以维护。
*   **优化思路（去中心化寻址）**：
    *   **对于 L4 (TCP)**：由于流量本身不具语义，`proxy_name` 是必要的。
    *   **对于 L7 (HTTP)**：Server 应仅透传 `host`，由 Client 侧的 Agent 根据自身配置的 `host -> local_backend` 映射规律进行“边缘路由（Edge Routing）”。

带着这一视角，我们往下逐一分析其他模块是否也存在类似问题。


---

## 一、数据流与 relay 层 ✅ **已完成 (CR2)**

> **完成时间**：2026-04 | **Commits**: `65a4375` (relay_inner/forward_inner) + `92d0d28` (copy→copy_buf+BufReader 64KB)

### ~~现状~~ → 完成后状态

每个文件内部两个公开函数共享一个私有 `_inner` 核心，消除了 N 份循环体重复：

| 文件 | 私有核心 | 公开入口 |
|---|---|---|
| `engine/bridge.rs` | `relay_with_first_data` | `relay_quic_to_tcp` (无初始数据委托) |
| `engine/relay.rs` | `relay_inner<S>` | `relay_bidirectional` / `relay_with_initial` |
| `proxy/base.rs` | `forward_inner` | `forward_to_client` / `forward_with_initial_data` |

所有 relay 路径统一使用 `BufReader::with_capacity(64KB) + copy_buf`，替换原来 8KB 的 `tokio::io::copy`。

**注**：`base.rs` 的 `initial_data` 写入 QUIC `send`（把 peek 到的 TCP 字节推给 client），与 `bridge.rs` 写入 TCP 方向语义不同，因此两套不能合并为一个跨文件的 core，当前各文件内部统一即为最优解。

**剩余技术债**：若未来需要加 timeout / 字节计数回调，仍需改 3 处内核而非 1 处。可引入 `RelayConfig` 结构体作为参数统一传递，但当前复杂度不足以驱动该重构。

---

## 二、协议检测层 ✅ **已完成 (CR1)**

> **完成时间**：2026-04 | **Commits**: `3baa0cc` (Protocol enum + RoutingInfo) + `0604599` (Copy trait 性能修复)

### ~~现状~~ → 完成后状态

- `Protocol` 枚举：`#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]`，bincode 编码为 4 字节 variant index，比原 String（10+ 字节）更紧凑。
- `detect_protocol_and_host` 返回 `(Protocol, Option<String>)`，消除所有字符串字面量 match。
- `RoutingInfo.protocol: String` → `Protocol`，整条链路零字符串转换。
- `Copy` trait 确保热路径（entry.rs 每连接、detect_protocol 每 stream）零 clone 开销。

**性能注记**：首次 PR 遗漏 `Copy`，导致 `protocol.clone()` 调用，基准测试下降 ~1700 RPS；加 `Copy` 后完全恢复。

**剩余技术债**：`detect_protocol_and_host` 仍混合了 H2 preface 检测 + HTTP 解析 + TLS SNI 提取三种协议层。TLS SNI 拆分为独立函数的重构价值存在，但优先级低（`extract_tls_sni` 已单独存在，只是内部仍有少量内联 TLS 检测）。

---

## 三、Peer / Upstream 抽象层

### 现状

```
ProxyApp::upstream_peer() → PeerKind
PeerKind::connect(send, recv, initial_data)
  ├─ Tcp(TcpPeer)       → bridge::relay_with_first_data
  ├─ Tls(TlsTcpPeer)    → relay::relay_with_initial
  ├─ Http(HttpPeer)     → Http1Driver loop
  ├─ H2(H2Peer)         → serve_h2_forward
  └─ Dyn(Box<dyn UpstreamPeer>) → connect_boxed
```

**`UpstreamScheme` URL 解析** ✅ **已完成 (CR3)**
> **完成时间**：2026-04 | **Commit**: `ede99cd`
> `UpstreamScheme::from_address` 已统一委托给 `transport/addr.rs::parse_upstream`，消除了 4 个私有辅助函数（`extract_host_part` / `has_explicit_port` / `has_port_443` / `extract_port_number`）。基准测试 +410 RPS（减少冗余字符串扫描）。

---

**待改进**：

**语义性（`ProxyApp` 的错觉）**：`ProxyApp` 这个名字具有强烈的误导性，听起来像是一个业务应用的生命周期框架。它的唯一职能是根据 `RoutingInfo`/`host` 在内部映射表中路由，返回下游 `PeerKind`。**本质应是 `UpstreamResolver` 或 `RouteMatcher`。** — 改名影响范围广（server+client 均有实现），建议作为独立 rename PR。

**`TcpPeer` 和 `TlsTcpPeer` 合并** (`CR-NEW-C`)：两者 `connect_inner` 签名和逻辑高度相似，只在 TLS 握手处分叉。合并为 `TcpPeer { addr, tls: Option<TlsConfig> }` 可消除重复，调用方构造时无需区分 Tcp/Tls。— 中等影响，独立可做。

**`Dyn` 变体**：`UpstreamPeer` trait 若无真实多态需求，可删掉，让 `PeerKind` 保持纯枚举，消除 `Pin<Box<dyn Future>>` 样板。— 低风险，依赖确认 `Dyn` 无外部用途。

---

## 四、消息帧协议层

### 现状

`models/msg.rs` 设计整洁：

- `MessageType` 用 `#[repr(u8)]` 编码，`from_u8` 做边界检查
- `send_message` / `recv_message` 泛型化，`Serialize/Deserialize` 约束
- `send_routing_info` / `recv_routing_info` 是语义化的薄包装

**问题**：`recv_routing_info` 内部先读 type byte，再断言是 `RoutingInfo`，如果是其他类型就报错。但 `recv_message_type` + `recv_message` 的分步调用在其他地方也存在（`tunnel_handler`、`handlers/quic.rs`），调用方需要自己管理"先读 type 再读 body"的两步协议，容易遗漏类型校验。

**改进方向** (`CR-NEW-A`) ✅ **已完成**：`recv_typed_message<T>(reader, expected_type)` 已实现，`recv_routing_info`、`client/main.rs` 登录握手、`control_client.rs` watch 循环均已迁移。`server/handlers/quic.rs` 的 Login 握手保留两步（type 和 body 各有独立 timeout + 错误回包逻辑，合并会丢失分阶段错误细节）。

---

## 五、路由与注册表层（server 侧）

### 现状

```
RoutingSnapshot { http_routers: HashMap<port, VhostRouter<(group_id, proxy_name)>> }
ClientRegistry { groups: DashMap<group_id, Arc<ClientGroup>> }
```

两步查找：`VhostRouter.get(host)` → `(group_id, proxy_name)` → `ClientRegistry.select_client_for_group(group_id)` → `SelectedConnection`。

**语义性** (`CR-NEW-B`)：`VhostRouter<T>` 泛型设计好，但 `T = (Arc<str>, Arc<str>)` 是匿名元组，调用方必须靠位置（.0/.1）区分 group_id 和 proxy_name，没有类型保护。改为 `RouteTarget { group_id: Arc<str>, proxy_name: Arc<str> }` 能消除这个歧义。
- **影响**：低风险，纯类型改名，编译器保证一致性；无运行时开销。
- **依赖**：无（独立可做）。

**负载均衡策略**：`select_healthy` 目前是 O(N) 线性扫描 `least-inflight`。在 Client 规模较大时（如千级以上连接）会产生不必要的 CPU 开销。此外，`inflight` 计数器的增减依赖业务层显式调用 `begin_inflight()`，若在复杂的 `select!` 逻辑中遗漏 `drop`，会导致负载计数永久漂移。

**`ClientGroup`**：RCU 模型（ArcSwap snapshot + Mutex 写）设计正确，注释也说清了。`select_healthy` 现在是 least-inflight，语义准确。`is_empty` 的实现走 snapshot 而非 index，在极端竞争下可能有短暂不一致，不过对这个场景无害。

**`EntryConnPool`（client 侧）**：与 `ClientGroup` 几乎是镜像设计——同样的 `ArcSwap<Vec> + Mutex<Vec>`，但多了一个冗余的 `mu: Mutex<Vec<Connection>>`（snapshot 里已经有全量数据，`mu` 是写侧临时状态，两者内容重复）。TODO 里已有记录，但属于低风险技术债。

---

## 六、H2 代理层

### 现状

`proxy/h2_proxy.rs` 的 `H2SenderCache` 做 double-checked locking：

```rust
try_get_sender()          // fast path: Mutex<Option<SendRequest>>
rebuild_mu.lock().await   // 防止并发重建
try_get_sender()          // second check
open_bi() + handshake     // 实际建连
```

逻辑正确，但 `Mutex<Option<SendRequest>>` 是同步锁 + 异步上下文，在 lock 持有期间不能 await（现在的代码没有问题，因为只做 clone 和赋值），但未来改动容易引入死锁。注释里 TODO-P1 已经规划用 `ArcSwap` 替换。

`send_via` 函数的泛型约束（`B: Body, B::Data: Into<Bytes>, B::Error: Into<Box<dyn Error>>`）在每个调用点都重复出现，可以收进 type alias `type ForwardableBody = ...`。

---

## 七、客户端方向抽象与连接管理层

### 现状

```
pool.rs::run_pool           — N 个 supervisor slot 的 JoinSet
conn_pool.rs::EntryConnPool — 向 entry listener 暴露的 RCU 连接池
entry.rs::start_entry_listener — 监听本地端口 → 发送 QUIC Stream (Egress: Local → Server)
proxy.rs::handle_work_stream  — 接收远端 QUIC Stream → 解析后转发给本地 Backend (Ingress: Server → Local)
```

**双向流转语义的不对称**：DuoTunnel 的一大卖点是“双向通信”。但在代码的物理切割中，接收外部本地流量流向 Server 的模块叫做 `entry.rs`，而接收 Server 取回流向本地 Backend 的模块被叫做 `proxy.rs`。
“Entry” 和 “Proxy” 无法形成天然的方向对照。事实上 `entry.rs` 也是一个 Proxy 的入口。极其容易让后来者混淆 Client 到哪是进、哪是出。
基于隧道的“系统功能模型”，**我们应站在 Tunnel 本身的视角，将两者对称重命名为：`egress_listener.rs`（正向发送入口）和 `ingress_receiver.rs`（反向接收出口）。**

**语义性**：`pool.rs` 叫 `run_pool`，`conn_pool.rs` 叫 `EntryConnPool`，两者名字容易混淆——前者管理 supervisor 任务，后者管理实际连接。可以重命名为 `run_connection_supervisors` / `QuicConnPool`。

**`entry.rs` 的 peek buf 复用逻辑**和 `proxy/core.rs` 里的 `STREAM_PEEK_BUF` 是相同的 thread-local unsafe Vec 模式，代码重复了两次。可以提取到 `infra` 或 `transport` 里的一个共享 `ThreadLocalBuf` 工具。 (`CR-NEW-E`)
- **影响**：低风险，消除 2 处 unsafe 代码重复，集中维护 set_len 安全前提。
- **依赖**：无（独立可做）。

---

## 八、公开 API 面（`lib.rs`）

### 现状

`tunnel-lib/src/lib.rs` 直接 `pub use` 了 38 个符号，混合了：

- 底层工具（`relay_bidirectional`、`relay_with_initial`、`relay_quic_to_tcp`、`relay_with_first_data`）
- 高层工具（`forward_h2_request`、`send_routing_info`）
- 配置类型（`QuicTransportParams`、`TcpParams`）
- 基础设施（`apply_worker_threads`、`init_cert_cache`）

调用方（server、client）无法从模块结构判断哪些是稳定 API、哪些是内部细节。

**改进方向** (`CR-NEW-D`)：分组导出，内部实现细节用 `pub(crate)` 或 `#[doc(hidden)]` 标注。至少把 relay 的 6 个函数收进 `pub mod relay`，让调用方明确表达"我在用底层 relay"。
- **影响**：纯重构，无运行时影响；需要同步更新 server/client 引用路径，属于机械改动。
- **依赖**：无（独立可做，但建议在 CR2 relay 统一后再做，避免两次大范围改引用）。

---

## 九、指标与观测性系统

### 现状

`server/metrics.rs` 暴露了一组全局静态指标，业务逻辑通过显式调用 `auth_success()`、`open_bi_begin()` 等函数更新状态。

**语义一致性问题**：
- 指标操作具有侵入性：业务逻辑中混杂了大量的 metrics 调用。在高性能路径上，这不仅破坏了代码的可读性，也使得“业务逻辑”与“监控逻辑”高度耦合。
- 桶（Buckets）硬编码：`open_bi_wait_ms` 的直方图桶是静态设定的。在不同的网络拓扑下（如跨公网 vs 跨可用区），固定的桶分布往往会导致分位数观察失真。

**改进方向** (`CR4`)：利用 `tracing` 库的订阅者模式。业务层只需抛出事件（如 `info!(event = "auth_fail", group = "...")`），由独立的 `TelemetrySubscriber` 在后台汇总。
- **影响**：低风险，不改业务逻辑；可读性大幅提升，监控路径扩展更容易。
- **依赖**：无（独立可做）。

**实现教训（`cea0261` 已回滚）**：首次实现（`MetricsLayer`）将 Prometheus counter 更新直接放在 `on_event` 里同步执行。`tracing_subscriber` 的 `on_event` 在全局 subscriber 锁内运行，Prometheus counter 更新本身也有锁，两把锁叠加在热路径上（`conn.tcp.open` / `request.done` 在 8k QPS 下每秒数万次触发），导致 tokio worker 线程严重争锁，QUIC keepalive 超时，client 集体断连（约 30s 后全部 unregister）。

**正确实现方式**：`on_event` 内只做一次非阻塞 channel send（`tokio::sync::mpsc::UnboundedSender`），绝不持锁或调用任何同步原语。单独起一个后台 task 消费 channel、更新 Prometheus。这样热路径开销仅为一次原子 push，与锁完全解耦。

---

## 十、配置加载架构

### 现状

`ConfigSource` 采用装饰器模式（`MergedSource`），通过主动 `load()` 全量获取快照。热更新依赖 `spawn_config_watcher` 监视文件变化并触发重载。

**抽象评价**：目前是“拉（Pull）”模型，配置变更的实时性依赖轮询或文件系统事件的触发。

**改进方向** (`CR5`)：演进为”流（Stream）”模型。定义 `ConfigStream` 返回 `Stream<Item = RoutingSnapshot>`。这样业务层只需要监听流的变化，而无需关注”配置从哪来”和”何时触发重载”，配置源可以更自然地扩展到 Nacos、Etcd 等动态中心。
- **影响**：中等风险，涉及 `ServerState` 初始化流程改动；长期收益大（动态路由中心扩展）。
- **依赖**：无直接代码依赖，但建议在 TODO-53 Milestone D（移除旧静态 token map）后进行，两者会同时修改配置加载路径。

---

## 总结

| 维度 | 评分 | 主要问题 |
|---|---|---|
| 命名语义 | ★★★★☆ | `ProxyApp`、relay 方向语义硬编码、指标调用侵入性 |
| 类型安全 | ★★★★☆ | ~~协议名字符串传递~~ ✅ → 路由信息元组待改、LB 计数器手动维护 |
| 抽象复用 | ★★★★☆ | ~~Relay 逻辑多处重复~~ ✅ ~~URL 解析未归一~~ ✅ → peek buf 模式重复待改 |
| 模块边界 | ★★★★☆ | `ConfigSource` 宜向流式架构演进，`lib.rs` API 暴露过宽 |
| 组合正交性 | ★★★★☆ | 协议检测与接入点耦合，IO 与 Relay 配置耦合 |

---

## 待办优先级总结

> 已完成项：CR1（Protocol enum）、CR2（relay 归一化）、CR3（URL 解析归一化）

| 编号 | 描述 | 影响大小 | 风险 | 依赖 | 建议顺序 |
|---|---|---|---|---|---|
| **CR-NEW-B** | `VhostRouter` 返回 `RouteTarget` 结构体替换匿名元组 | 低（类型安全，零运行时） | 低 | 无 | **1** — 改动机械，编译器保证 |
| ~~**CR-NEW-A**~~ | ~~`recv_typed_message<T>` 封装两步消息读取~~ ✅ | — | — | — | 已完成 |
| **CR-NEW-E** | 提取 `ThreadLocalBuf` 共享工具，消除 2 处 unsafe peek buf 重复 | 低（代码整洁，安全集中管理） | 低 | 无 | **3** — 小范围改动 |
| **CR4** | 解耦观测性：业务只抛 tracing 事件，独立 subscriber 汇总 metrics | 中（可读性+扩展性显著提升） | 中（on_event 必须非阻塞，需通过 channel 异步转发到后台 task，不可在锁内直接更新 Prometheus） | 无 | **4** — 独立，但改动面较广 |
| **CR-NEW-C** | 合并 `TcpPeer` / `TlsTcpPeer` 为 `TcpPeer { tls: Option<TlsConfig> }` | 中（消除重复 connect_inner） | 中 | 无 | **5** — 需同步更新 `PeerKind` 构造点 |
| ~~**CR-NEW-D**~~ | ~~`lib.rs` API 分组：relay 函数收入 `pub mod relay`，内部符号 `pub(crate)`~~ ✅ | — | — | — | 已完成：删除 13 个无外部调用方的 re-export，`lib.rs` 从 40 行收缩至 32 行 |
| **CR5** | 配置流式化：`ConfigStream: Stream<Item = RoutingSnapshot>` | 高（动态配置中心扩展基础） | 中 | TODO-53 Milestone D 后进行 | **7** — 架构级改动，收益大但复杂 |
