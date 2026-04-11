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

## 一、数据流与 relay 层

### 现状

relay 逻辑分散在三个模块，共 6 个函数：

| 函数 | 位置 | IO 类型 | 差异 |
|---|---|---|---|
| `relay` | `engine/bridge.rs` | `AsyncR+W` × 2 | 通用，BufReader |
| `relay_unidirectional` | `engine/bridge.rs` | `AsyncR` → `AsyncW` | 单向 |
| `relay_quic_to_tcp` | `engine/bridge.rs` | `RecvStream` + `SendStream` + `TcpStream` | into_split，BufReader |
| `relay_with_first_data` | `engine/bridge.rs` | 同上 + `Option<&[u8]>` | 加初始数据 |
| `relay_bidirectional` | `engine/relay.rs` | `RecvStream` + `SendStream` + `AsyncR+W` | io::split，无 BufReader |
| `relay_with_initial` | `engine/relay.rs` | 同上 + `&[u8]` | 加初始数据 |

`forward_to_client` / `forward_with_initial_data`（`proxy/base.rs`）是又一套，针对 `(SendStream, RecvStream, TcpStream)` 方向相反的绑定。

**问题**：核心逻辑是同一件事（join 两个方向的 copy_buf + shutdown/finish），差别只是：

1. 是否有初始数据（`Option<&[u8]>`）
2. IO 类型是否需要 `into_split`（已有所有权）还是 `split`（借用，内部 Mutex）
3. 方向命名：`quic_recv → tcp_write` vs `a → b`

这三个维度分别独立，但代码把它们组合成了 6 个函数全量枚举，任何新增（如加 timeout、加字节计数回调）都要改 6 处。

**语义性**：`relay_with_first_data` 和 `relay_with_initial` 名字几乎相同，做的也是同一件事，命名没有区分依据。`forward_to_client` 的方向语义（quic → tcp）比 `relay_quic_to_tcp` 更隐晦——前者说"转发给谁"，后者说"从哪到哪"，两套命名约定在同一功能域共存。

**所有权与生命周期**：`forward_to_client` 内部强制调用 `into_split()`，这剥夺了调用方对 Stream 的继续控制权。如果业务逻辑需要在 Relay 后执行统计或复用，这种硬编码的解构方式缺乏灵活性。

**改进方向**：统一成一个 `relay_bidir<R1,W1,R2,W2>` + `Option<&[u8]>` 初始数据参数，`into_split` vs `split` 在调用点处理。引入 `RelayConfig` 或回调钩子来处理字节计数，而不是全量枚举函数。

---

## 二、协议检测层

### 现状

两套检测路径并存：

- `protocol/detect.rs::detect_protocol_and_host` — 入站 TCP 侧（server/client entry），返回 `(&'static str, Option<String>)`
- `proxy/core.rs::detect_protocol` — QUIC stream 侧（client 收到后），返回 `Protocol` 枚举

前者返回字符串，后者返回强类型枚举，但两者检测的协议集合重叠（h1、h2、websocket、tcp）。

**语义性问题**：`detect_protocol_and_host` 返回 `&'static str` 而不是枚举，调用方必须匹配字符串字面量（`"h2"`、`"websocket"`），拼写错误在编译期无法发现。`RoutingInfo.protocol` 字段也是 `String`，整条链路上协议类型靠字符串传递，`tunnel_handler.rs:16` 再手动把字符串转回枚举——来回两次转换，中间有丢失信息的窗口。

**`detect_protocol_and_host` 做了三件事**：H2 preface 检测、HTTP 解析（host + websocket upgrade）、TLS SNI 提取，三种完全不同的协议层混在一个函数里，且函数名只说了"协议和 host"，没说它还会解 TLS。

**改进方向**：
1. `detect_protocol_and_host` 返回值改为 `(Protocol, Option<String>)`，消除字符串协议名
2. `RoutingInfo.protocol` 改为枚举（或在 send/recv 边界序列化为 u8）
3. TLS SNI 提取拆为独立函数（现在已有 `extract_tls_sni`，但 `detect_protocol_and_host` 内部还有一段内联的 TLS 检测）

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

**语义性（`ProxyApp` 的错觉）**：`ProxyApp` 这个名字具有强烈的误导性，听起来像是一个业务应用的生命周期框架。但从实际数据流转规则来看，它在 Server 端被 `EgressProxy(ServerEgressMap)` 实现，在 Client 端被 `ClientApp` 实现。它的唯一职能是：接收包含有最初几字节元数据的 `Context`，根据 `RoutingInfo` 中的信息（如 `proxy_name` 或剥离后的 `host`），在内部映射表中路由，最终返回一个代表下游对象的 `PeerKind`（如 HTTP 客户端或原生 TCP 连接）。
**它的本质应当是一个 `UpstreamResolver` 或 `RouteMatcher`。** 由于目前错误的接口命名，不仅掩盖了其实际作用（寻址与路由查找），还导致理解数据走向时产生歧义。

**`Dyn` 变体**：`UpstreamPeer` trait 目前只有一个实现路径（`Dyn` 分支），且 trait 方法签名要求手写 `Pin<Box<dyn Future>>`（async_trait 的痛点）。如果 `Dyn` 分支没有真实的多态需求，可以删掉，让 `PeerKind` 保持纯枚举。

**`TcpPeer` 和 `TlsTcpPeer`** 的 `connect_inner` 签名和逻辑高度相似，只在 TLS 握手处分叉。二者可以合并成一个 `TcpPeer { addr, tls: Option<TlsConfig> }` 来消除重复，同时让调用方不需要在构造时区分 Tcp/Tls。

**`UpstreamScheme`** 在 `tcp.rs` 里定义，但 `egress/http.rs::parse_upstream` 在 `transport/addr.rs` 里也做了近乎相同的 URL 解析，且两者的"port 443 → https" 启发式规则各自独立实现。这是同一个域知识的两份代码。

---

## 四、消息帧协议层

### 现状

`models/msg.rs` 设计整洁：

- `MessageType` 用 `#[repr(u8)]` 编码，`from_u8` 做边界检查
- `send_message` / `recv_message` 泛型化，`Serialize/Deserialize` 约束
- `send_routing_info` / `recv_routing_info` 是语义化的薄包装

**问题**：`recv_routing_info` 内部先读 type byte，再断言是 `RoutingInfo`，如果是其他类型就报错。但 `recv_message_type` + `recv_message` 的分步调用在其他地方也存在（`tunnel_handler`、`handlers/quic.rs`），调用方需要自己管理"先读 type 再读 body"的两步协议，容易遗漏类型校验。

**改进方向**：提供 `recv_typed_message<T>(reader, expected_type)` 统一封装，消除调用方的两步手动序列。

---

## 五、路由与注册表层（server 侧）

### 现状

```
RoutingSnapshot { http_routers: HashMap<port, VhostRouter<(group_id, proxy_name)>> }
ClientRegistry { groups: DashMap<group_id, Arc<ClientGroup>> }
```

两步查找：`VhostRouter.get(host)` → `(group_id, proxy_name)` → `ClientRegistry.select_client_for_group(group_id)` → `SelectedConnection`。

**语义性**：`VhostRouter<T>` 泛型设计好，但 `T = (Arc<str>, Arc<str>)` 是匿名元组，调用方必须靠位置（.0/.1）区分 group_id 和 proxy_name，没有类型保护。改为 `RouteTarget { group_id: Arc<str>, proxy_name: Arc<str> }` 能消除这个歧义。

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

**`entry.rs` 的 peek buf 复用逻辑**和 `proxy/core.rs` 里的 `STREAM_PEEK_BUF` 是相同的 thread-local unsafe Vec 模式，代码重复了两次。可以提取到 `infra` 或 `transport` 里的一个共享 `ThreadLocalBuf` 工具。

---

## 八、公开 API 面（`lib.rs`）

### 现状

`tunnel-lib/src/lib.rs` 直接 `pub use` 了 38 个符号，混合了：

- 底层工具（`relay_bidirectional`、`relay_with_initial`、`relay_quic_to_tcp`、`relay_with_first_data`）
- 高层工具（`forward_h2_request`、`send_routing_info`）
- 配置类型（`QuicTransportParams`、`TcpParams`）
- 基础设施（`apply_worker_threads`、`init_cert_cache`）

调用方（server、client）无法从模块结构判断哪些是稳定 API、哪些是内部细节。

**改进方向**：分组导出，内部实现细节用 `pub(crate)` 或 `#[doc(hidden)]` 标注。至少把 relay 的 6 个函数收进 `pub mod relay`，让调用方明确表达"我在用底层 relay"。

---

## 九、指标与观测性系统

### 现状

`server/metrics.rs` 暴露了一组全局静态指标，业务逻辑通过显式调用 `auth_success()`、`open_bi_begin()` 等函数更新状态。

**语义一致性问题**：
- 指标操作具有侵入性：业务逻辑中混杂了大量的 metrics 调用。在高性能路径上，这不仅破坏了代码的可读性，也使得“业务逻辑”与“监控逻辑”高度耦合。
- 桶（Buckets）硬编码：`open_bi_wait_ms` 的直方图桶是静态设定的。在不同的网络拓扑下（如跨公网 vs 跨可用区），固定的桶分布往往会导致分位数观察失真。

**改进方向**：利用 `tracing` 库的订阅者模式。业务层只需抛出事件（如 `info!(event = "auth_fail", group = "...")`），由独立的 `TelemetrySubscriber` 在后台汇总。

---

## 十、配置加载架构

### 现状

`ConfigSource` 采用装饰器模式（`MergedSource`），通过主动 `load()` 全量获取快照。热更新依赖 `spawn_config_watcher` 监视文件变化并触发重载。

**抽象评价**：目前是“拉（Pull）”模型，配置变更的实时性依赖轮询或文件系统事件的触发。

**改进方向**：演进为“流（Stream）”模型。定义 `ConfigStream` 返回 `Stream<Item = RoutingSnapshot>`。这样业务层只需要监听流的变化，而无需关注“配置从哪来”和“何时触发重载”，配置源可以更自然地扩展到 Nacos、Etcd 等动态中心。

---

## 总结

| 维度 | 评分 | 主要问题 |
|---|---|---|
| 命名语义 | ★★★★☆ | `ProxyApp`、relay 方向语义硬编码、指标调用侵入性 |
| 类型安全 | ★★★☆☆ | 路由信息元组、LB 计数器手动维护、协议名字符串传递 |
| 抽象复用 | ★★★☆☆ | Relay 逻辑多处重复、peek buf 模式重复、URL 解析未归一 |
| 模块边界 | ★★★★☆ | `ConfigSource` 宜向流式架构演进，`lib.rs` API 暴露过宽 |
| 组合正交性 | ★★★★☆ | 协议检测与接入点耦合，IO 与 Relay 配置耦合 |

**优先改进**：

1. **强类型路由信息**：`RoutingInfo.protocol` 改为枚举，`VhostRouter` 返回结构化对象。
2. **Relay 逻辑归一化**：支持 `initial_data` 与 `split` 选项的统一 Relay 实现，消除物理 IO 与业务方向的耦合。
3. **解耦观测性逻辑**：尝试使用事件驱动（Tracing）方式记录指标。
4. **URL 解析归一化**：将 `transport/addr.rs` 与 `egress/http.rs` 的解析逻辑合并。
5. **配置流式化**：探索 `ConfigStream` 以支持更复杂的动态路由配置场景。
