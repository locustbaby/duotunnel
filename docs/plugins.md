# Ingress / Egress 核心 vs 插件化设计

> 状态:**设计稿**(仅文档,未实施)。落地拆分到后续独立 PR。
> 最后更新:2026-04-19

## 1. 动机

目前 `server/handlers/{http,tcp,quic}.rs` 与 `client/app.rs` **直接 import 具体符号**来拼装功能:

```rust
// server/handlers/http.rs 开头 —— 这就是今天的现实
use crate::{metrics, ServerState};
use tunnel_lib::extract_host_from_http;
use tunnel_lib::proxy;
use tunnel_lib::RouteTarget;
```

handler 直接 call `metrics::open_bi_begin()`、直接访问 `ServerState::routing` 的具体字段、直接调 `proxy::forward_h2_request` 这种自由函数。**调用方和被调用方的绑定发生在 `use` 这一行**,编译时就定死。

后果(按影响程度排序):

1. **改动半径大**:改个 TLS 逻辑要碰 `handlers/http.rs`;改个指标名要碰 4 个 handler;改个 vhost 查找要碰 `main.rs` + `handlers/http.rs` + `listener.rs`。"谁影响谁"没法一眼看出。
2. **新增能力要动入口**:加一种协议、一种 LB、一种鉴权 → 必须改主干文件,冲突概率高。
3. **测试要起真家伙**:handler 里的依赖是具体 struct,测试时只能起真 TLS、真 QUIC、真 registry。
4. **依赖升级牵一发动全身**:PR #27 升 tonic 0.14 时,250 行手写胶水全重写,原因就是上下游都用**具体类型**相互握手。

目标:**把调用依赖从"具体符号"换成"trait 方法"**。入口模块只认抽象接口,具体实现通过注册表在运行时注入。所谓"plugin"只是这个思路的自然产物 —— 每个具体实现就是一个 plugin。

> 本文不谈 cargo feature 裁剪、不谈动态加载、不谈跨语言。只谈**代码内部模块之间的调用依赖怎么从硬编码换成 trait 依赖**。

## 2. 能力分类(基于代码事实)

### 2.1 Ingress(server 侧)

| 类别 | 能力 | 位置 |
|---|---|---|
| **CORE** | QUIC accept 循环 + Login 握手 | `server/handlers/quic.rs:10-87` |
| **CORE** | 客户端组注册 / 选择(最少 inflight) | `server/registry.rs:99-177` |
| **CORE** | bi 流打开 + 背压护栏 | `tunnel-lib/src/open_bi.rs:33-67` |
| **CORE** | 字节级 relay(H1 / WS / TCP 直通) | `tunnel-lib/src/proxy/base.rs:8-55`、`engine/bridge.rs:61-93` |
| **CORE** | 协议嗅探 *基础档*(TLS / H2 / H1 三分支) | `tunnel-lib/src/protocol/detect.rs:5-43`、`proxy/core.rs:77-103` |
| OPTIONAL | TLS 终止 + SNI 提取 | `server/handlers/http.rs:104-184`、`protocol/detect.rs:45-98` |
| OPTIONAL | Plaintext H2(h2c)服务 + authority rewrite | `server/handlers/http.rs:185-334` |
| OPTIONAL | Vhost 路由(exact + wildcard) | `tunnel-lib/src/transport/listener.rs:63-150` |
| OPTIONAL | 每端口 HTTP router 组装 | `server/main.rs:453-485` |
| OPTIONAL | 非 vhost 的纯 TCP 接入 | `server/handlers/tcp.rs` |
| OPTIONAL | Prometheus 指标(全内联) | `server/metrics.rs` + 各 handler 内 |
| OPTIONAL | Overload / slow-path | `tunnel-lib/src/overload.rs`、`open_bi.rs` |
| OPTIONAL | Token 热吊销广播 | `server/handlers/quic.rs:144-166` |

### 2.2 Egress(client 侧 + `tunnel-lib/src/proxy/`)

| 类别 | 能力 | 位置 |
|---|---|---|
| **CORE** | 接收 RoutingInfo | `client/proxy.rs:8-28` |
| **CORE** | 协议决策 + UpstreamPeer dispatch | `tunnel-lib/src/proxy/core.rs:42-72`、`client/app.rs:89-210` |
| **CORE** | 打开 upstream 连接(TCP dial) | `tunnel-lib/src/proxy/tcp.rs:132-165` |
| **CORE** | 双向 relay | `tunnel-lib/src/engine/bridge.rs:61-93`、`engine/relay.rs:56-66` |
| OPTIONAL | LB(round-robin;`lb_policy` string 已读但未 dispatch) | `tunnel-lib/src/proxy/upstream.rs:13-24` |
| OPTIONAL | DNS 解析缓存(30s TTL) | `client/app.rs:47-77` |
| OPTIONAL | QUIC 连接池(least-inflight) | `client/conn_pool.rs:9-51` |
| OPTIONAL | H2 sender cache(upstream H2 多路复用) | `tunnel-lib/src/proxy/h2_proxy.rs:23-91` |
| OPTIONAL | TLS 配置池 + system roots | `tunnel-lib/src/proxy/tcp.rs:75-87` |
| OPTIONAL | egress vhost 规则(scheme / SNI) | `client/app.rs:100-210` |
| OPTIONAL | 针对 opaque TLS 的 MITM H2 upstream | `client/app.rs:145-151`(inline impl) |
| OPTIONAL | TCP 参数调优(NODELAY / keepalive / buffer) | `tunnel-lib/src/transport/tcp_params.rs` |
| CROSS | Timeouts(open_stream / login / connect / resolve) | `client/main.rs:137,266,354,383`、`client/connect.rs:55-84` |

### 2.3 已经是插件形态的抽象(直接复用,不另起炉灶)

| Trait | 位置 | 已有实现 |
|---|---|---|
| `BackgroundService` | `server/service.rs:7` | HotReload / ControlClient |
| `ConfigSource`(async) | `server/config.rs:19` | File / Db / Merged |
| `RuleStore`(async) | `tunnel-store/src/rules.rs:71` | SqliteRuleStore |
| `AuthStore`(async) | `tunnel-store/src/traits.rs:25` | SqliteAuthStore / LocalTokenCache |
| `UpstreamResolver` | `tunnel-lib/src/proxy/core.rs:23` | ClientApp / ServerEgressMap / EgressProxy |
| `UpstreamPeer` | `tunnel-lib/src/proxy/peers.rs:29` | Tcp / Http / H2 / Dyn(MitmH2) |
| `ProtocolDriver`(async) | `tunnel-lib/src/protocol/driver/mod.rs:16` | Http1Driver |

## 3. 最小可跑隧道(CORE 判定)

一条 TCP over QUIC 隧道能跑起来**真正必需**的只有:

1. **Tunnel control plane**:`Login` / `LoginResp` / `Ping` —— 不需要 vhost、不需要 TLS、不需要认证插件,就是 token 比对。
2. **QUIC accept + bi-stream open**:`server/handlers/quic.rs` + `tunnel-lib/src/open_bi.rs`。
3. **客户端注册 / 选择**:`server/registry.rs`(按 group_id 选一条活的客户端连接)。
4. **双向字节 relay**:`tunnel-lib/src/engine/bridge.rs`。
5. **RoutingInfo 传递**:`tunnel-lib/src/models/msg.rs`。
6. **Egress 侧 TCP dial + relay**:`tunnel-lib/src/proxy/tcp.rs` 的 `TcpPeer`(不带 TLS / LB / pool)。

其他一切 —— 包括 vhost、TLS、H2C、H2 多路复用、LB、DNS 缓存、Prometheus —— 都是"要不要"的问题,不该进 CORE。

## 4. 调用依赖解耦设计

### 4.0 核心命题:从"符号耦合"到"trait 耦合"

一切设计从这一条出发。

**今天(AS-IS)—— 入口模块对具体符号的依赖**:

```
server/handlers/http.rs
├── use crate::metrics               ← 具体 module(内含自由函数)
├── use crate::ServerState           ← 具体 struct,字段硬访问
├── use tunnel_lib::RouteTarget      ← 具体 struct
├── use tunnel_lib::proxy            ← 具体 module(含 forward_h2_request 等)
└── use tunnel_lib::extract_host_from_http  ← 具体自由函数

  调用 → metrics::open_bi_begin(...)
  调用 → state.routing.load().get(...)
  调用 → proxy::forward_h2_request(...)
```

入口文件里每一行 `use` 都把自己绑死到一个具体实现上。换掉任何一个都要改 `use` 和所有调用点。

**目标(TO-BE)—— 入口模块只依赖 trait**:

```
server/handlers/http.rs (或任何入口)
├── use tunnel_lib::plugin::{IngressProtocolHandler, RouteResolver, MetricsSink};
└── 参数:stack: &IngressStack, ctx: &PluginCtx

  调用 → ctx.metrics.incr("open_bi_total", ...)
  调用 → ctx.route_resolver.resolve(port, &route_ctx).await
  调用 → stack.accept(stream, ctx).await
```

入口文件**不 import 任何具体实现**。`stack` 里装的是什么 handler、`ctx.metrics` 后面是 Prometheus 还是 no-op,入口不知道也不关心。换一个实现 = 换一个传进来的对象,入口代码一行不改。

### 4.1 解耦的 4 条硬规则(review 时可检查)

| # | 规则 | 违反的信号 |
|---|---|---|
| R1 | **入口模块(`handlers/*.rs` / `client/app.rs` / `client/proxy.rs`)禁止 `use` 任何具体 plugin 实现** | 出现 `use crate::plugins::tls::...` |
| R2 | **plugin 之间不互相 `use` 具体符号**,要协作就通过 trait | 出现 `use crate::plugins::vhost::VhostResolver` |
| R3 | **基础设施层(`tunnel-lib`)禁止 `use` server / client 的 plugin** | 出现 `use server::plugins::...` |
| R4 | **新增一个 plugin 不应该改任何入口模块**(入口只知道 trait,不知道 plugin 名) | 新 plugin 的 PR 里出现 handler 文件 diff |

这 4 条是"是否真解耦"的硬检验。一旦通过,新增能力就只是新建一个 module + 在注册表里 `insert()` 一行。

### 4.2 选型:Rust trait + 运行时注册表

**所有 plugin 都编进二进制**,启动时由配置决定"用哪几个、什么顺序"。不走 cargo feature 做编译期裁剪 —— 那是 embedded / 多形态部署才值得引入的复杂度,本仓不需要。

- 抽象形态:`Vec<Arc<dyn IngressProtocolHandler>>` 这类运行时注册表。
- 启动流程:读配置 → 按名字从注册表里取 plugin → 组装 `IngressStack` / `EgressStack`。
- 好处:加新 plugin 只改配置,不重新编译;测试可以替换任何一个 plugin;灵活扩展。
- 拒绝动态加载(dlopen / WASM / 外部 plugin registry),本仓不需要热加载也不需要跨语言。
- 已有 6 个 trait(`BackgroundService` / `ConfigSource` / ...)证明这个模式在本仓可行。

### 4.3 新增的三类扩展点

#### A. Ingress(server 侧)

```rust
#[async_trait]
pub trait IngressProtocolHandler: Send + Sync + 'static {
    /// 仅看首批字节决定这条连接归不归自己管
    fn probe(&self, first_bytes: &[u8]) -> ProbeResult;
    async fn handle(&self, conn: TcpStream, route: Route) -> Result<()>;
}

pub enum ProbeResult {
    Matches,
    NotMine,
    NeedMore(usize), // 告诉上层再多读 N 字节
}

#[async_trait]
pub trait RouteResolver: Send + Sync + 'static {
    async fn resolve(&self, listener_port: u16, ctx: &RouteCtx) -> Option<Route>;
}

pub trait MetricsSink: Send + Sync + 'static {
    fn incr(&self, name: &'static str, labels: &[(&str, &str)]);
    fn observe(&self, name: &'static str, value: f64, labels: &[(&str, &str)]);
}

#[async_trait]
pub trait AdmissionController: Send + Sync + 'static {
    async fn admit(&self, ctx: &AdmissionCtx) -> AdmitDecision;
}
```

组装形式:`IngressStack { handlers: Vec<Arc<dyn IngressProtocolHandler>>, ... }` —— 按 probe 顺序匹配,第一个 `Matches` 拿走连接。

现有代码映射为插件:

| 现有代码 | 插件名 | 默认组装? |
|---|---|---|
| `H1Handler`(字节透传)—— 由 `handlers/http.rs:335-389` 抽出 | `ingress-h1` | 是 |
| `PassthroughTcpHandler`—— `handlers/tcp.rs` 抽出 | `ingress-tcp` | 是 |
| `TlsHandler`(SNI + 解密 + H2/H1 分发)—— `handlers/http.rs:104-184` | `ingress-tls` | 是 |
| `H2cHandler`(明文 H2 升级)—— `handlers/http.rs:185-334` | `ingress-h2c` | 是 |
| `VhostResolver`—— `transport/listener.rs:63-150` | `route-vhost` | 是;配置里去掉时退化为 `StaticPortResolver`(CORE 保底) |
| `PrometheusSink`—— `server/metrics.rs` + 内联调用 | `metrics-prometheus` | 是;配置里去掉时用 no-op sink |
| `TokenAdmission`—— 目前的 AuthStore 套进来 | `admission-token` | 是 |

#### B. Egress(client 侧)

```rust
pub trait LoadBalancer: Send + Sync + 'static {
    fn pick<'a>(&self, targets: &'a [Target], ctx: &PickCtx) -> Option<&'a Target>;
}

#[async_trait]
pub trait UpstreamDialer: Send + Sync + 'static {
    fn matches(&self, target: &Target) -> bool;
    async fn dial(&self, target: &Target, ctx: &DialCtx) -> Result<Connected>;
}

#[async_trait]
pub trait Resolver: Send + Sync + 'static {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>>;
}
```

现有代码映射为插件:

| 现有代码 | 插件名 | 默认组装? |
|---|---|---|
| RR LB(`proxy/upstream.rs:13-24`) | `lb-round-robin` | 是 |
| Least-inflight LB(`conn_pool.rs:45-49` 的算法可以复用) | `lb-least-inflight` | 否(已编入,按配置启用) |
| Weighted LB(将来) | `lb-weighted` | 否(已编入,按配置启用) |
| `TcpDialer`(`proxy/tcp.rs` TcpPeer) | `egress-tcp` | 是 |
| `TlsTcpDialer`(`proxy/tcp.rs` 的 TLS 分支) | `egress-tls` | 是 |
| `H1Dialer`(`proxy/http.rs` HttpPeer) | `egress-h1` | 是 |
| `H2Dialer`(`proxy/h2.rs` H2Peer) | `egress-h2` | 是 |
| `MitmH2Dialer`(`client/app.rs:145-151`) | `egress-mitm-h2` | 否(已编入,按配置启用) |
| H2SenderCache(`proxy/h2_proxy.rs:23-91`) | `egress-h2-mux` | 是 |
| `SystemResolver`(std::net) | CORE | 是 |
| `CachedResolver`(`client/app.rs:47-77` 的 30s TTL 缓存) | `resolver-cached` | 是 |
| `HostsFileResolver` / DoH 等 | `resolver-xxx` | 否(已编入,按配置启用) |

#### C. 控制面已经是 trait 的不动

`ConfigSource` / `RuleStore` / `AuthStore` / `BackgroundService` / `UpstreamResolver` / `UpstreamPeer` / `ProtocolDriver` 继续沿用。新 plugin 挂到它们上面即可。

### 4.4 crate 布局

```
tunnel-lib/
  src/
    plugin/
      mod.rs            ← 新:trait 定义聚合(IngressProtocolHandler 等)
      registry.rs       ← 新:按 name → Arc<dyn _> 的注册表

server/
  plugins/              ← 新目录,每个 plugin 独立一个 module
    tls/                ← ingress-tls
    h2c/                ← ingress-h2c
    vhost/              ← route-vhost
    prometheus/         ← metrics-prometheus

client/
  plugins/
    lb_rr/
    lb_least/
    h2_mux/             ← egress-h2-mux
    dns_cached/
```

所有 plugin 模块都**无条件编进二进制**,不加 cargo feature。启动时由配置决定启用哪些。

### 4.5 配置驱动组装(草案)

```yaml
# server config
ingress_plugins:
  - ingress-h1            # CORE 保底
  - ingress-tcp           # CORE 保底
  - ingress-tls
  - ingress-h2c
  - route-vhost
  - admission-token
  - metrics-prometheus

# client config
egress_plugins:
  - egress-tcp            # CORE 保底
  - egress-tls
  - egress-h1
  - egress-h2
  - egress-h2-mux
  - lb-round-robin
  - resolver-cached
```

配置里引用了注册表中不存在的 plugin 名 → 启动时**报错** `unknown plugin "xxx"`,不静默忽略。

### 4.6 具体怎么抽象 —— 从现在的代码里拆出来

"抽象"不是空话 —— 它落到代码上就是 **3 个具体动作**:定义 trait、搬家实现、把调用点换成注册表查找。下面把每个动作展开。

#### 4.6.1 新 trait 与现有 trait 的分工(避免叠床架屋)

现有 6 个 trait 已经覆盖了**控制面 + 上游 peer 抽象**,新 trait 只补"入口调度"和"横切关注点"这两块空白:

```
┌──────────────────────────────────────────────────────────────┐
│ Server side                                                  │
│                                                              │
│  TcpListener 收到连接                                         │
│    │                                                         │
│    ▼                                                         │
│  ┌─────────────────┐   new ── 入口调度(Ingress)              │
│  │ IngressStack    │   替换 handlers/http.rs:49-95 的大 if/else │
│  │  .handlers[]    │                                         │
│  └────┬────────────┘                                         │
│       │ probe() 命中的 IngressProtocolHandler                 │
│       ▼                                                      │
│  ┌─────────────────┐   new ── 路由查找                        │
│  │ RouteResolver   │   替换 main.rs:453-485 + VhostRouter     │
│  └────┬────────────┘                                         │
│       │ Route { client_group, proxy_name }                   │
│       ▼                                                      │
│  ┌─────────────────┐   existing                              │
│  │ ClientRegistry  │   server/registry.rs:99-177 不动         │
│  └────┬────────────┘                                         │
│       │ quinn::Connection                                    │
│       ▼ open_bi + send_routing_info                          │
│     隧道                                                      │
│                                                              │
│  横切:MetricsSink(new) + AdmissionController(new) 从入口     │
│        注入,贯穿到所有 handler                                │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ Client side                                                  │
│                                                              │
│  work stream 收到 RoutingInfo                                 │
│    │                                                         │
│    ▼                                                         │
│  ┌─────────────────┐   existing                              │
│  │ UpstreamResolver│   ClientApp 不动,但内部实现替换          │
│  └────┬────────────┘                                         │
│       │ Target                                               │
│       ▼                                                      │
│  ┌─────────────────┐   new ── LB                             │
│  │ LoadBalancer    │   替换 proxy/upstream.rs:13-24 硬编码 RR │
│  └────┬────────────┘                                         │
│       │ &Target                                              │
│       ▼                                                      │
│  ┌─────────────────┐   new ── DNS                            │
│  │ Resolver        │   替换 client/app.rs:47-77 内联缓存      │
│  └────┬────────────┘                                         │
│       │ Vec<SocketAddr>                                      │
│       ▼                                                      │
│  ┌─────────────────┐   new ── 拨号                           │
│  │ UpstreamDialer  │   替换 proxy/{tcp,http,h2}.rs 的         │
│  └────┬────────────┘   connect_inner + app.rs 里的协议分派    │
│       │ Connected                                            │
│       ▼                                                      │
│  ┌─────────────────┐   existing                              │
│  │ UpstreamPeer    │   Dialer 返回的东西实现这个,不动 trait   │
│  └─────────────────┘                                         │
└──────────────────────────────────────────────────────────────┘
```

**分工规则**:
- `UpstreamResolver` / `UpstreamPeer` / `ProtocolDriver` 保留,作为 **plugin 产物的统一出口类型**。新 `UpstreamDialer::dial()` 返回的东西最终要满足 `UpstreamPeer`。
- 新 trait 只做 **"哪条路 / 选哪个目标 / 怎么去"** 的分派,不替代 peer 本身。
- 不搞平行世界:一个能力要么用新 trait,要么用老 trait,文档里明确规定,避免两套都写。

#### 4.6.2 注册表的数据结构

```rust
// tunnel-lib/src/plugin/registry.rs

pub struct PluginRegistry {
    ingress_handlers: HashMap<&'static str, IngressBuilder>,
    route_resolvers:  HashMap<&'static str, RouteBuilder>,
    metrics_sinks:    HashMap<&'static str, MetricsBuilder>,
    admissions:       HashMap<&'static str, AdmissionBuilder>,
    lbs:              HashMap<&'static str, LbBuilder>,
    dialers:          HashMap<&'static str, DialerBuilder>,
    resolvers:        HashMap<&'static str, ResolverBuilder>,
}

// 每个 Builder 是 "(plugin 配置 json) -> Arc<dyn Trait>"
type IngressBuilder =
    Box<dyn Fn(&PluginConfig, &PluginCtx) -> Result<Arc<dyn IngressProtocolHandler>> + Send + Sync>;
```

启动流程 (`server/main.rs` 里):

```rust
let mut reg = PluginRegistry::new();
plugins::register_builtin_ingress(&mut reg);  // 调 reg.ingress_handlers.insert("ingress-tls", Box::new(|cfg, ctx| ...));
plugins::register_builtin_egress(&mut reg);

let stack = IngressStack::from_config(&reg, &config.ingress_plugins, &ctx)?;
// config.ingress_plugins 是 Vec<{name, params}>
```

**关键点**:
- plugin 的**配置 schema 由 plugin 自己声明**(plugin 自己的 `Deserialize` struct),注册表只负责把一段不透明的 YAML 子树交给 builder。新增 plugin 不用改全局 config struct。
- plugin 之间共享的东西走 `PluginCtx`(里面是 `Arc<TcpParams>`、`Arc<dyn MetricsSink>` 等),不用 `static`。
- Plugin init 错误(如 TLS 证书文件不存在)→ 启动时直接 fail-fast;plugin 运行时错误(如单次 handle 失败)→ 返回 `Result`,上层记指标,连接关闭。

#### 4.6.3 AS-IS → TO-BE:以 `server/handlers/http.rs` 为例

**现在**(摘要,行号对照 `handlers/http.rs:49-95`):

```rust
pub async fn handle_http_connection(stream: TcpStream, state: Arc<ServerState>, port: u16) {
    let first_byte = peek(&stream).await?;
    if first_byte == 0x16 {
        handle_tls_connection(stream, state, port).await  // -> line 104
    } else if is_h2_preface(&peek_buf) {
        handle_plaintext_h2_connection(stream, state, port).await  // -> line 185
    } else {
        handle_plaintext_h1_connection(stream, state, port).await  // -> line 335
    }
}
```

**抽完之后**:

```rust
// server/handlers/http.rs 变成这么简单
pub async fn handle_http_connection(
    stream: TcpStream,
    stack: &IngressStack,
    ctx: &PluginCtx,
    port: u16,
) -> Result<()> {
    stack.accept(stream, port, ctx).await
}

// tunnel-lib/src/plugin/ingress.rs
impl IngressStack {
    pub async fn accept(&self, mut stream: TcpStream, port: u16, ctx: &PluginCtx) -> Result<()> {
        let mut peek_buf = [0u8; 1024];
        let mut needed = 1;
        loop {
            let n = peek_exact(&mut stream, &mut peek_buf[..needed]).await?;
            for handler in &self.handlers {
                match handler.probe(&peek_buf[..n]) {
                    ProbeResult::Matches => {
                        let route = self.resolver.resolve(port, &RouteCtx { ... }).await
                            .ok_or_else(|| anyhow!("no route"))?;
                        return handler.handle(stream, route, ctx).await;
                    }
                    ProbeResult::NotMine => continue,
                    ProbeResult::NeedMore(want) => { needed = want; break; }
                }
            }
        }
    }
}
```

`handle_tls_connection` / `handle_plaintext_h2_connection` / `handle_plaintext_h1_connection` 被搬到 `server/plugins/{tls,h2c,h1}/` 各自的 module 里,各自实现 `IngressProtocolHandler`。`probe()` 分别认 `0x16` / H2 preface / 兜底 `NotMine`。

#### 4.6.4 迁移策略:怎么保证"行为等价"

PR ② 把 handler 搬家时,**并行保留旧路径 1 个版本**:

1. 在 `ServerState` 加开关 `use_plugin_stack: bool`,默认 `false`。
2. `handle_http_connection` 写成 `if state.use_plugin_stack { stack.accept(...) } else { legacy_dispatch(...) }`。
3. 集成测试跑两遍(`use_plugin_stack=false` 和 `=true`),断言结果一致。
4. 下一个 PR 翻默认值、再下一个删 legacy 分支。

这是标准的影子迁移,防止"重构的时候把协议细节搞丢了"。

#### 4.6.5 `probe()` 的协议细节

`ProbeResult::NeedMore(usize)` 的语义:

- plugin 返回 `NeedMore(N)` = "至少再让我看 N 字节才能决定"。
- `IngressStack::accept()` 负责循环 `peek`,累计到 N 字节再把所有 handler 重跑一遍 probe。
- 设硬上限(默认 256 字节),超过还没有任何 handler `Matches` → 拒绝连接 + 记指标。
- 注意 `peek` 是 MSG_PEEK,不消耗数据;真正 `handle()` 里 plugin 自己读。

这块是 plugin 协议最容易出 bug 的地方,本文档作为唯一真源。

### 4.7 模块分层调用依赖(AS-IS vs TO-BE)

这不是 crate 依赖图,是**代码模块之间谁调谁**。

#### AS-IS(当前事实,2026-04)

```
                      ┌─────────────────────────────────────────┐
                      │ server/main.rs                          │
                      │  - 直接 use: config, metrics,            │
                      │    registry, handlers, listener_mgr,    │
                      │    hot_reload, control_client, service  │
                      └────────┬────────────────────────────────┘
                               │ 直接实例化 + 直接调方法
        ┌──────────────────────┼──────────────────────┐
        ▼                      ▼                      ▼
┌───────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│ handlers/http.rs  │  │ handlers/tcp.rs  │  │ handlers/quic.rs │
│                   │  │                  │  │                  │
│ use crate::       │  │ use crate::      │  │ use crate::      │
│   metrics         │←─┼─ metrics         │←─│  metrics         │  ◀── 指标耦合
│   ServerState     │  │  ServerState     │  │  ServerState     │
│   (.routing,      │  │                  │  │  (.registry,     │  ◀── 状态字段硬访问
│    .tcp_params,   │  │                  │  │   .auth_store)   │
│    .config)       │  │                  │  │                  │
│                   │  │                  │  │                  │
│ use tunnel_lib::  │  │ use tunnel_lib:: │  │ use tunnel_lib:: │
│   proxy           │  │   proxy          │  │   (msg types)    │
│   RouteTarget     │  │   open_bi_guarded│  │                  │
│   extract_host_…  │  │                  │  │                  │
└───────────────────┘  └──────────────────┘  └──────────────────┘
        │                      │                      │
        └──────────────────────┴──────────────────────┘
                               │ 同一个 proxy module 被多方直接 call
                               ▼
                    ┌─────────────────────────┐
                    │ tunnel-lib/src/proxy/   │
                    │  forward_h2_request     │
                    │  forward_with_initial_  │
                    │  open_bi_guarded        │
                    │  ...                    │
                    └─────────────────────────┘
```

**红色信号**:
- 每个 handler 都 `use crate::metrics` —— 4 处调用都是具体自由函数。
- `ServerState` 被当成**上下文背包**,handler 直接伸手取字段(`state.routing`、`state.tcp_params`、`state.config.server.open_stream_timeout_ms`)。任何 `ServerState` 字段的重构都要扫全部 handler。
- `tunnel-lib::proxy` 模块导出的自由函数被 server 直接 call,**server 和 tunnel-lib 之间没有 trait 边界**。

#### TO-BE(抽完后的目标拓扑)

```
入口层(Entry)
 ┌──────────────────────────────────────────────────────────┐
 │ server/handlers/http.rs / tcp.rs / quic.rs               │
 │                                                          │
 │ 只 use 这些抽象:                                          │
 │   tunnel_lib::plugin::{IngressStack, PluginCtx}          │
 │   tunnel_lib::plugin::traits::IngressProtocolHandler     │
 │                                                          │
 │ 传参方式:                                                 │
 │   async fn handle(stack: &IngressStack, ctx: &PluginCtx) │
 │                                                          │
 │ 禁止出现的符号:                                           │
 │   ✗ crate::metrics                                        │
 │   ✗ crate::plugins::tls                                   │
 │   ✗ state.routing / state.tcp_params(直接字段访问)        │
 └───────────────────┬──────────────────────────────────────┘
                     │ 只调 trait 方法
                     ▼
抽象层(Abstractions)—— 定义在 tunnel-lib/src/plugin/
 ┌──────────────────────────────────────────────────────────┐
 │ trait IngressProtocolHandler                             │
 │ trait RouteResolver                                      │
 │ trait MetricsSink                                        │
 │ trait AdmissionController                                │
 │ trait LoadBalancer / UpstreamDialer / Resolver (egress)  │
 │                                                          │
 │ 已有 trait(不动):                                        │
 │   UpstreamResolver / UpstreamPeer / ProtocolDriver       │
 │   ConfigSource / RuleStore / AuthStore                   │
 └───────────────────┬──────────────────────────────────────┘
                     │ 由具体实现"向上"满足
                     ▼
实现层(Plugins)—— server/plugins/* 和 client/plugins/*
 ┌──────────────────────────────────────────────────────────┐
 │ plugins/tls/       impl IngressProtocolHandler          │
 │ plugins/h2c/       impl IngressProtocolHandler          │
 │ plugins/h1/        impl IngressProtocolHandler          │
 │ plugins/vhost/     impl RouteResolver                   │
 │ plugins/prometheus/ impl MetricsSink                    │
 │ plugins/token/     impl AdmissionController             │
 │                                                          │
 │ plugin 互相之间 禁止 use 具体 plugin 符号。              │
 │ 要用对方能力 → 通过 PluginCtx 里的 trait object。        │
 └───────────────────┬──────────────────────────────────────┘
                     │ 只调基础库的纯工具函数
                     ▼
基础层(Primitives)—— tunnel-lib/src/{engine,transport,open_bi,...}
 ┌──────────────────────────────────────────────────────────┐
 │ engine::bridge::relay_with_first_data  ← 纯 I/O 工具      │
 │ open_bi::open_bi_guarded               ← 纯控制工具       │
 │ transport::listener::VhostRouter       ← 数据结构         │
 │ protocol::detect::*                    ← 无副作用函数     │
 │                                                          │
 │ 这一层无状态、不依赖 plugin、不依赖 server/client 身份。 │
 └──────────────────────────────────────────────────────────┘
```

**分层 3 条硬规则**:

1. **调用方向单一**:入口 → 抽象 → 实现 → 基础。不允许反向。
2. **跨层只能通过 trait**:实现层不能被入口层直接 `use`(入口只看 trait)。
3. **同层禁止横向 `use` 具体符号**:同是 plugin,要互通就走 PluginCtx 里的 trait。

这三条和 §4.1 的 R1-R4 是一回事,换角度描述,方便从"拓扑图"和"文件 `use` 列表"两个侧面做 review。

### 4.8 一条连接的完整调用链(AS-IS vs TO-BE)

看一条 HTTPS 入站连接从 accept 到进入隧道的全程,每一步标注"谁"和"调的具体符号 / 调的 trait 方法"。

#### AS-IS

| # | 位置 | 动作 | 对谁的依赖 |
|---|---|---|---|
| 1 | `handlers/http.rs:run_http_accept_loop` | accept TCP | `tunnel_lib::run_accept_worker`(具体函数) |
| 2 | 同上 | `state.tcp_params.apply(&stream)` | `ServerState` 字段(具体 struct) |
| 3 | `handlers/http.rs:handle_http_connection` | peek 首字节判断 TLS / H2 / H1 | 内联 `if first == 0x16 { ... }`(硬编码) |
| 4 | `handle_tls_connection` | 查 vhost | `state.routing.load().get(port)`(字段) + `VhostRouter::match_host`(具体方法) |
| 5 | 同上 | 选 client | `state.registry.select_healthy(group)`(具体方法) |
| 6 | 同上 | 记指标 | `metrics::open_bi_begin(...)`(具体自由函数) |
| 7 | 同上 | 打开 QUIC stream | `open_bi_guarded(...)`(具体自由函数) |
| 8 | 同上 | 发送 RoutingInfo | `tunnel_lib::send_routing_info(...)`(具体自由函数) |
| 9 | 同上 | 双向 relay | `proxy::forward_h2_request(...)`(具体自由函数) |

**9 个步骤、至少 7 个具体符号直接调用**。任何一个要换,都要改入口。

#### TO-BE

| # | 位置 | 动作 | 对谁的依赖 |
|---|---|---|---|
| 1 | `handlers/http.rs` | accept TCP | `tunnel_lib::run_accept_worker`(基础层工具,保留) |
| 2 | 同上 | 应用 TCP 参数 | `ctx.tcp_params.apply(...)`(`PluginCtx` 方法) |
| 3 | `IngressStack::accept` | `probe()` 循环选 handler | `trait IngressProtocolHandler::probe`(trait 方法) |
| 4 | 命中的 TlsHandler | 查路由 | `ctx.route_resolver.resolve(port, &rctx).await`(trait 方法) |
| 5 | 同上 | 选 client | `ctx.client_registry.select(group)`(trait 方法) |
| 6 | 同上 | 记指标 | `ctx.metrics.incr("open_bi_total", &[...])`(trait 方法) |
| 7 | 同上 | 打开 QUIC stream | `ctx.stream_opener.open_bi_guarded(...).await`(trait 方法) |
| 8 | 同上 | 发 RoutingInfo | 仍是 `send_routing_info`(基础层,因为 wire 协议是跨方共同依赖,不需要抽象) |
| 9 | 同上 | 双向 relay | 仍是 `engine::bridge::relay_with_first_data`(基础层) |

**入口模块不 `use` 任何具体 plugin**。换 TLS 实现 / 换指标后端 / 换路由策略 = 配置层换一个名字,入口文件一行不改。

> 注意 #8 #9 仍是"直接 call 自由函数"—— 这**没问题**,因为它们是基础层、无状态、在任何部署下语义都一样,不需要抽象。**过度抽象本身就是反模式**。本设计只抽"有多种可能实现"的东西。

### 4.9 数据依赖与生命周期

#### PluginCtx 的内容

`PluginCtx` 是入口层和 plugin 层之间**唯一的共享数据管道**,它取代现在的 `Arc<ServerState>` 广撒网。

```rust
pub struct PluginCtx {
    // 抽象能力(trait objects)
    pub metrics:         Arc<dyn MetricsSink>,
    pub route_resolver:  Arc<dyn RouteResolver>,
    pub admission:       Arc<dyn AdmissionController>,
    pub stream_opener:   Arc<dyn StreamOpener>,
    pub client_registry: Arc<dyn ClientRegistry>,  // server 专用
    pub lb:              Arc<dyn LoadBalancer>,    // client 专用
    pub dialer:          Arc<dyn UpstreamDialer>,  // client 专用
    pub resolver:        Arc<dyn Resolver>,        // client 专用

    // 不值得抽象的共享只读配置(直接放)
    pub tcp_params: Arc<TcpParams>,
    pub overload:   OverloadLimits,
    pub timeouts:   Timeouts,
}
```

规则:
- **有多种实现可能**的 → `Arc<dyn Trait>`。
- **纯配置值、跨所有实现不变**的 → 直接放 struct / Arc<T>,不抽象。过度抽象有害。
- `PluginCtx` 构造一次(`main.rs` 组装 registry 时),然后 `Arc` 到处传。**不存在**"第二个 PluginCtx"。

#### 生命周期

```
main.rs 启动
  1. 读 config
  2. PluginRegistry::new() + register_builtin_*
  3. 按 config.ingress_plugins / egress_plugins 调 builder → 实例化具体 plugin
  4. 组装 PluginCtx(把具体 plugin Arc<dyn> 填进来)
  5. 把 PluginCtx 传给 handler 入口 / 客户端 worker
  6. 跑直到 shutdown
  7. 关闭:按反序让 plugin 释放资源
```

热重载(PR ⑥ 再做,不在首批 PR):
- `PluginCtx` 里的 `Arc<dyn _>` 用 `ArcSwap<dyn _>` 包一层 → 配置变更时整体换一份 PluginCtx,handler 下次调用就看到新的。
- 现存连接不受影响(已经持有旧 Arc)。

#### plugin 间协作

两条 plugin A / B 的协作,**不准**通过 `use plugins::b::B;` 直接抓 B 的具体类型。走**两种**合规路径:

1. B 提供的是已有 trait(如 `MetricsSink`)→ A 通过 `PluginCtx` 拿 `Arc<dyn MetricsSink>`。
2. B 是 A 特有的协作对象 → 在 PluginCtx 里新增一个 trait(或者 A 把 B 的 `Arc<dyn XxxTrait>` 作为 A 自己的**构造参数**传进去,由 registry 负责组装时把 B 注入给 A)。

路径 2 的例子:TLS handler 可能需要调用 admission controller 做额外检查,那 TlsHandler 的 builder 签名里就要求 `admission: Arc<dyn AdmissionController>`,registry 组装时从 PluginCtx 里取出传入。**TlsHandler 永远不知道** admission 背后是 token 实现还是 JWT 实现。

---

## 5. 抽象完之后的收益

"抽象"不是为了好看,是为了**让具体角色在具体场景里干得更快**。下面按**角色 × 场景**列,每条都可以验证:

### 5.1 给未来加新协议的人

**场景**:要加一个 `proxy-protocol v2` 入口(HAProxy PROXY protocol header),现在和将来的对比。

| 步骤 | 现在(AS-IS) | 抽完之后(TO-BE) |
|---|---|---|
| 1. 新建 module | 改 `handlers/http.rs`,在 `handle_http_connection` 大 if/else 里加一支 | 新建 `server/plugins/proxy_protocol/mod.rs`,impl `IngressProtocolHandler` |
| 2. 协议嗅探 | 改 `protocol/detect.rs` 加 magic bytes | plugin 自己 `probe()` 返回 `Matches` if `data.starts_with(b"PROXY ")` |
| 3. 路由联动 | 顺手改 `main.rs:453-485` 确保新协议能走 vhost | 不用动,plugin 从 `route` 参数拿路由 |
| 4. 加指标 | 在 4 个 handler 里插 `metrics::counter!(...)` | plugin 内部用 `ctx.metrics`,统一收口 |
| 5. 加 config 字段 | 改 `server/config.rs` 全局 struct | plugin 自己定 `ProxyProtoConfig: Deserialize`,注册表路由 |
| 6. 测试 | 起真 QUIC + TLS 拿通路 | 单测里 `handler.handle(fake_stream, fake_route, &ctx)` |

**量化**:改动文件从 5 个(`handlers/http.rs` + `detect.rs` + `main.rs` + `config.rs` + `metrics.rs`)降到 1 个独立 module。

### 5.2 给升级 `tonic` / `hyper` / `rustls` 的人

**场景**:就是刚合完的 PR #27 —— tonic 0.12→0.14 让 `tonic::body::BoxBody` 变 private,250 行手写 gRPC 胶水全部要重写。

| 现在 | 抽完之后 |
|---|---|
| tonic 改动影响 `ci_test_client.rs` + `grpc_echo_server.rs` 两个文件的业务逻辑;TLS 改动影响 `handlers/http.rs` 的 TLS 分支;hyper 改动影响 H2 dispatcher;**重构范围不可预测** | TLS 改动只在 `server/plugins/tls/`;gRPC 改动只在 `ci-helpers/plugins/grpc/`(如果抽出);hyper H2 ingress 改动只在 `server/plugins/h2c/`。**单 plugin 闭环** |

**量化**:PR #27 改了 4 个文件;同等改动在抽完之后预计 1-2 个文件。

### 5.3 给 profile / 压测的人

**场景**:压测想看纯转发性能,不要 Prometheus 指标的干扰。

- **现在**:`server/metrics.rs` + 4 个 handler 里几十处 `metrics::counter!()` / `gauge!()` / `histogram!()` 内联。想关掉 = 满仓库改代码,或者整个 crate 去掉 `metrics` feature(牵连太广)。
- **抽完之后**:配置里把 `metrics-prometheus` 换成 `metrics-noop`。`Arc<dyn MetricsSink>` 变成空实现,**所有调用点**走一次间接分派、直接 return。一行 config 修改。

**量化**:Criterion benchmark 对比有意义,指标开关成了一等公民。

### 5.4 给测试的人

**场景**:要测"vhost 命中 H2 upstream 时的 authority rewrite"。

- **现在**:`tests/` 里起真 TLS + 真 QUIC server + 真 client + H2 upstream 模拟,网络栈全开,测试慢且易 flaky。
- **抽完之后**:Mock `RouteResolver` 返回固定 `Route`,Mock `UpstreamDialer` 返回 `MemoryStream`,直接测 `plugins/h2c` handler 的 authority rewrite 逻辑。

**量化**:从 integration test(秒级、起进程)降到 unit test(毫秒、纯内存)。

### 5.5 给部署 / 运维的人

**场景**:同一份二进制要跑 3 种部署 —— dev(本地、无 TLS、无 metrics)、canary(灰度、加 shadow LB、带 admission dry-run)、prod(全量)。

- **现在**:要么三套 build feature 组合,要么代码里写 `if env == "dev"` 散落各处。
- **抽完之后**:三份 YAML 配置,差别就是 `ingress_plugins` / `egress_plugins` 列表不同。二进制同一份。

**量化**:CI 里不再需要 matrix build,节约构建时间。

### 5.6 给 review PR 的人

**场景**:有人交一个"加 WebSocket Ping/Pong 心跳"的 PR。

- **现在**:这个改动可能碰 detect、handler、proxy/tcp、config 多处,review 时要在大脑里把"这改动会不会搞坏 TLS / H2 / vhost"全跑一遍。
- **抽完之后**:改动大概率就在 `server/plugins/ws/`(或单独新建 plugin),review 认知负担从"隧道整体"降到"这个 plugin 自己"。

### 5.7 给"我不想跑某个功能"的人

**场景**:边缘节点不想背 Prometheus / vhost / admission 的代码路径。

- **现在**:代码还是会跑,只是数据走空逻辑。占 CPU 占分支预测缓存。
- **抽完之后**:不在 config 里列出来,就不实例化,也不在热路径上出现。

### 收益对照 = 验证标准

表里每一条"抽完之后",都对应第 5 章验证标准里的某条具体测试:

| 收益 | 对应验证 |
|---|---|
| 5.1 加新协议只改 1 module | "每个 plugin 有独立 integration test"(5 章第 4 条) |
| 5.3 压测能剥离 metrics | "最小 plugin 集"跑通 E2E(5 章第 1 条) |
| 5.5 同一二进制多部署 | 配置里去掉 egress-h2-mux 仍工作(5 章第 2 条) |
| 5.7 不配就不跑 | 没 admission 要 fail-fast(6 章第 3 条) |

---

## 6. 验证标准

1. 配置里只列 `[ingress-h1, ingress-tcp]` + `[egress-tcp]` 这组最小 plugin 时,server 能启动、接受 1 个纯 TCP listener、把流量正确代理到 client 的 upstream(端到端 E2E)。
2. 配置里去掉 `egress-h2-mux` 后,client 依然能跑 H2 upstream(退化成每次新开 H2 连接)。
3. 配置里没有任何 admission plugin 时,启动**明确报错** "no admission plugin installed" —— 不允许裸奔。
4. 每个 plugin 有独立 integration test;CORE 有 smoke test。
5. Criterion benchmark 对比"最小 plugin 集"vs"全量 plugin"的吞吐,证明注册表抽象(Arc + dyn dispatch)的额外开销 <1% 或有明确解释。

## 7. 实施节奏(建议后续 PR)

推荐分成 4-6 个独立 PR,每个都可以独立 review / revert:

1. **PR ①**:定义 `IngressProtocolHandler` / `RouteResolver` / `MetricsSink` / `AdmissionController` 四个 trait 和 `IngressStack` 骨架,**不改现有代码**,仅添加 + 一个最小 smoke test。
2. **PR ②**:把 `H1Handler` / `PassthroughTcpHandler` / `TlsHandler` / `H2cHandler` 从 `server/handlers/http.rs` 抽出到 `server/plugins/*`,实现 trait;`IngressStack` 替换旧的 if-else dispatcher。行为等价。
3. **PR ③**:`VhostResolver` vs `StaticPortResolver`,把 vhost 相关从 `main.rs:453-485` / `listener.rs:63-150` 封装成 plugin。
4. **PR ④**:`MetricsSink` trait 化 `server/metrics.rs` 所有调用点;`PrometheusSink` 作为 default plugin,no-op sink 作为 CORE 保底。
5. **PR ⑤**:egress 侧的 `LoadBalancer` / `UpstreamDialer` / `Resolver` trait,把 `proxy/{tcp,http,h2}.rs`、`proxy/upstream.rs`、`client/app.rs:47-210` 的决策分支抽出。
6. **PR ⑥**:配置层接 `ingress_plugins` / `egress_plugins` 两个列表,支持动态组装。

每个 PR 走完 `cargo check / test / clippy -D warnings` 再进下一步。

## 8. 非目标

- 不做 WASM / dlopen / shared library 式动态加载。
- 不走 cargo feature 做编译期裁剪 —— 所有 plugin 都编进二进制,运行时组装就够用。
- 不引入 libp2p 风格的 plugin registry / handshake。
- 不重构已经是 trait 的部分(`ConfigSource` 等)。
- 不改 wire protocol。
- 不在本设计中承诺"未来所有协议都能加"—— 只证明现有能力可以被这套 seam 覆盖。

## 9. 相关文件清单(仅引用,不在本设计中修改)

| 文件 | 对应的现有能力 |
|---|---|
| `server/handlers/http.rs:49-389` | HTTP ingress dispatcher 分层(→ PR ②) |
| `server/handlers/tcp.rs` | TCP ingress(→ PR ②) |
| `server/handlers/quic.rs:10-172` | QUIC 握手 + 客户端注册(CORE) |
| `server/registry.rs:99-177` | 客户端组 + least-inflight(CORE) |
| `server/egress.rs` | 服务端 egress map |
| `server/main.rs:453-485` | build_routing_snapshot(→ PR ③) |
| `server/metrics.rs` | Prometheus 调用点(→ PR ④) |
| `client/main.rs`、`client/app.rs:47-210` | Egress 决策逻辑(→ PR ⑤) |
| `client/proxy.rs:8-28` | work stream(CORE) |
| `client/conn_pool.rs:9-51` | least-inflight pool |
| `tunnel-lib/src/proxy/core.rs:23-72` | UpstreamResolver / ProxyEngine(CORE) |
| `tunnel-lib/src/proxy/peers.rs:7-36` | PeerKind / UpstreamPeer(CORE) |
| `tunnel-lib/src/proxy/tcp.rs:13-165` | TcpPeer + UpstreamScheme |
| `tunnel-lib/src/proxy/h2_proxy.rs:23-91` | H2SenderCache |
| `tunnel-lib/src/proxy/upstream.rs:13-24` | 硬编码的 RR LB(→ PR ⑤) |
| `tunnel-lib/src/protocol/detect.rs:5-98` | 协议嗅探 |
| `tunnel-lib/src/overload.rs`、`open_bi.rs` | 背压 |
| `tunnel-lib/src/engine/bridge.rs:61-93` | relay(CORE) |
| `tunnel-lib/src/models/msg.rs` | wire 消息(CORE) |
| `tunnel-lib/src/transport/listener.rs:63-150` | VhostRouter(→ PR ③) |
