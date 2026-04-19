# Ingress / Egress 插件化设计(v2)

> 状态:**进行中** — PR ① 已合入(trait 骨架),PR ②–⑥ 待实施。
> 最后更新:2026-04-19
> v2 变更:参考 Cloudflare Pingora 架构重新设计,改"probe 竞争"为"phase 串行",拆 PluginCtx 为 Server/Egress 两个上下文,补全 phase 短路语义。
>
> **实施进度**
> | PR | 内容 | 状态 |
> |---|---|---|
> | ① | `tunnel-lib/src/plugin/` trait 骨架(9 个文件,7 个 smoke tests) | ✅ 已合入 |
> | ② | Ingress handler 插件化 + `IngressDispatcher`(影子迁移) | 🔜 待实施 |
> | ③ | `RouteResolver` 插件化(VhostResolver + StaticPortResolver) | 🔜 待实施 |
> | ④ | `MetricsSink` 插件化(PrometheusSink + NoopSink)+ 删 legacy 分支 | 🔜 待实施 |
> | ⑤ | Egress 侧插件化(LoadBalancer + UpstreamDialer + Resolver) | 🔜 待实施 |
> | ⑥ | 配置层接入 `ingress_plugins`/`egress_plugins` + 最终清理 | 🔜 待实施 |

---

## 1. 动机

目前 `server/handlers/{http,tcp,quic}.rs` 与 `client/app.rs` **直接 import 具体符号**拼装功能。入口文件每一行 `use` 都把自己绑死到一个具体实现,编译时就定死。

后果:

1. **改动半径大**:改一个 TLS 逻辑要碰 4 个 handler;改指标名要改 4 处;改 vhost 查找要改 `main.rs` + `handlers/http.rs` + `listener.rs`。
2. **新增能力要动入口**:加协议 / LB / 鉴权 → 必须改主干文件,冲突概率高。
3. **测试要起真家伙**:依赖是具体 struct,单测时只能起真 TLS / 真 QUIC / 真 registry。
4. **依赖升级牵一发动全身**:PR #27 升 tonic 时 250 行胶水全重写,根源就是上下游都用具体类型握手。

目标:**把调用依赖从"具体符号"换成"trait 方法"**。入口只认抽象,具体实现通过注册表在运行时注入。

> 本文不谈 cargo feature 裁剪、不谈动态加载、不谈跨语言。只谈**代码内部模块之间的调用依赖怎么从硬编码换成 trait 依赖**。

---

## 2. 从 Pingora 学到的核心设计原则

Pingora (`pingora-proxy`) 是本设计的主要参照。其关键设计决策值得直接采纳:

### 2.1 Phase 串行模型,而非 probe 竞争

Pingora 定义 ~15 个固定 phase(early_request_filter → request_filter → upstream_peer → upstream_response_filter → response_filter → logging …),每个 phase 挂若干 handler **串行执行**,前一个 phase 的输出是后一个的输入。协议识别在最底层(TLS/ALPN/H2 negotiation)完成,上层 handler 收到的已经是已知协议的 session。

**v1 设计的问题**:让多个 `IngressProtocolHandler` 竞争 `probe()`,第一个 `Matches` 拿走连接。这会引入:
- 注册顺序依赖(先注册的先匹配)
- `NeedMore` 循环让所有 handler 重跑,成本随 handler 数量线性增长
- 协议识别逻辑散落在各 plugin 里,无法集中优化

**v2 做法**:协议嗅探提升为独立 phase,输出 `ProtocolHint`;后续 handler 按 hint 精确 dispatch,不再竞争。

### 2.2 双层扩展,职责分离

Pingora 有两层扩展点:
- `ProxyHttp` trait:服务级,一个 proxy 服务一个实现,控制整体路由/鉴权/转发策略
- `HttpModule` trait:连接级,可组合、有优先级,每 connection 独立 ctx

**v2 做法**:对应拆成 `TunnelService` trait(服务级策略,类比 `ProxyHttp`)+ `ConnectionModule` trait(横切关注点,类比 `HttpModule`)。

### 2.3 Phase 短路语义

Pingora 的 `request_filter` 返回 `Result<bool>`,`true` = 短路并直接向 downstream 发响应。每个 phase 都能早退。

**v2 做法**:每个 phase 返回 `PhaseResult` 枚举,支持 `Continue` / `Respond(Response)` / `Reject(status, body)` 三种语义。

### 2.4 Server ctx / Egress ctx 分离

Pingora 的 `ProxyHttp::CTX` 是 per-request 独立 struct,服务端和 egress 完全隔离。

**v1 的问题**:`PluginCtx` 把 `client_registry`(server 专用)和 `lb`/`dialer`(client 专用)全装进同一个 struct,server 侧 plugin 能看到 client 侧字段,没有最小权限。

**v2 做法**:拆成 `ServerCtx` 和 `EgressCtx`,各自只暴露本侧需要的字段。

---

## 3. 能力分类(基于代码事实)

### 3.1 Ingress(server 侧)

| 类别 | 能力 | 位置 |
|---|---|---|
| **CORE** | QUIC accept 循环 + Login 握手 | `server/handlers/quic.rs:10-87` |
| **CORE** | 客户端组注册 / 选择(最少 inflight) | `server/registry.rs:99-177` |
| **CORE** | bi 流打开 + 背压护栏 | `tunnel-lib/src/open_bi.rs:33-67` |
| **CORE** | 字节级 relay(H1 / WS / TCP 直通) | `tunnel-lib/src/proxy/base.rs:8-55`、`engine/bridge.rs:61-93` |
| **CORE** | 协议嗅探 **基础档**(TLS / H2 / H1 三分支) | `tunnel-lib/src/protocol/detect.rs:5-43`、`proxy/core.rs:77-103` |
| OPTIONAL | TLS 终止 + SNI 提取 | `server/handlers/http.rs:104-184`、`protocol/detect.rs:45-98` |
| OPTIONAL | Plaintext H2(h2c)服务 + authority rewrite | `server/handlers/http.rs:185-334` |
| OPTIONAL | Vhost 路由(exact + wildcard) | `tunnel-lib/src/transport/listener.rs:63-150` |
| OPTIONAL | 每端口 HTTP router 组装 | `server/main.rs:453-485` |
| OPTIONAL | 非 vhost 的纯 TCP 接入 | `server/handlers/tcp.rs` |
| OPTIONAL | Prometheus 指标(全内联) | `server/metrics.rs` + 各 handler 内 |
| OPTIONAL | Overload / slow-path | `tunnel-lib/src/overload.rs`、`open_bi.rs` |
| OPTIONAL | Token 热吊销广播 | `server/handlers/quic.rs:144-166` |

### 3.2 Egress(client 侧)

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
| OPTIONAL | 针对 opaque TLS 的 MITM H2 upstream | `client/app.rs:145-151` |
| OPTIONAL | TCP 参数调优(NODELAY / keepalive / buffer) | `tunnel-lib/src/transport/tcp_params.rs` |
| CROSS | Timeouts | `client/main.rs:137,266,354,383`、`client/connect.rs:55-84` |

### 3.3 已经是插件形态的抽象(直接复用,不另起炉灶)

| Trait | 位置 | 已有实现 |
|---|---|---|
| `BackgroundService` | `server/service.rs:7` | HotReload / ControlClient |
| `ConfigSource`(async) | `server/config.rs:19` | File / Db / Merged |
| `RuleStore`(async) | `tunnel-store/src/rules.rs:71` | SqliteRuleStore |
| `AuthStore`(async) | `tunnel-store/src/traits.rs:25` | SqliteAuthStore / LocalTokenCache |
| `UpstreamResolver` | `tunnel-lib/src/proxy/core.rs:23` | ClientApp / ServerEgressMap / EgressProxy |
| `UpstreamPeer` | `tunnel-lib/src/proxy/peers.rs:29` | Tcp / Http / H2 / Dyn(MitmH2) |
| `ProtocolDriver`(async) | `tunnel-lib/src/protocol/driver/mod.rs:16` | Http1Driver |

---

## 4. 调用依赖解耦设计(v2)

### 4.0 核心命题:从"符号耦合"到"phase trait 耦合"

**今天(AS-IS)**:

```
server/handlers/http.rs
├── use crate::metrics               ← 具体 module
├── use crate::ServerState           ← 具体 struct,字段硬访问
├── use tunnel_lib::RouteTarget      ← 具体 struct
└── use tunnel_lib::proxy            ← 具体 module
```

**目标(TO-BE)**:

```
server/handlers/http.rs
├── use tunnel_lib::plugin::{ServerCtx, PhaseResult}
└── 参数: svc: &dyn TunnelService, ctx: &mut ServerCtx

  调用 → svc.sniff_protocol(&peek_buf)   ← phase 1
  调用 → svc.admission(&ctx).await        ← phase 2
  调用 → svc.resolve_route(&ctx).await    ← phase 3
  调用 → svc.open_tunnel(&ctx).await      ← phase 4
  调用 → svc.logging(&ctx)               ← phase 5
```

入口文件**不 import 任何具体实现**。

### 4.1 解耦的 4 条硬规则(review 时可检查)

| # | 规则 | 违反的信号 |
|---|---|---|
| R1 | **入口模块(`handlers/*.rs` / `client/app.rs`)禁止 `use` 任何具体 plugin 实现** | 出现 `use crate::plugins::tls::...` |
| R2 | **plugin 之间不互相 `use` 具体符号**,要协作就通过 trait | 出现 `use crate::plugins::vhost::VhostResolver` |
| R3 | **基础设施层(`tunnel-lib`)禁止 `use` server / client 的 plugin** | 出现 `use server::plugins::...` |
| R4 | **新增一个 plugin 不应该改任何入口模块** | 新 plugin 的 PR 里出现 handler 文件 diff |

### 4.2 选型:Rust trait + 运行时注册表

**所有 plugin 都编进二进制**,启动时由配置决定"用哪几个、什么顺序"。不走 cargo feature 编译期裁剪。

- 服务级策略:`dyn TunnelService`(类比 Pingora `ProxyHttp`)
- 横切关注点:`Vec<Box<dyn ConnectionModule>>`(类比 Pingora `HttpModules`)
- 注册表:`PluginRegistry` 按 name 映射 builder

### 4.3 Phase 模型(v2 核心)

#### Server 侧 — 5 个 phase

```
TCP 连接
  │
  ▼  Phase 1: ProtocolSniff
  │  输入:peek_buf(最多 256 字节)
  │  输出:ProtocolHint { kind, sni, h2_preface, ... }
  │  实现:tunnel-lib/protocol/detect.rs(CORE,不抽象)
  │
  ▼  Phase 2: Admission
  │  输入:AdmissionCtx { peer_addr, hint, token }
  │  输出:PhaseResult::Continue | Reject(status, body)
  │  plugin:admission-token(默认)/ admission-jwt / admission-noop
  │
  ▼  Phase 3: RouteResolve
  │  输入:RouteCtx { port, hint, peer_addr }
  │  输出:PhaseResult::Continue(Route) | Reject
  │  plugin:route-vhost(默认) / route-static(CORE 保底)
  │
  ▼  Phase 4: TunnelOpen
  │  输入:Route + ProtocolHint + TcpStream
  │  输出:PhaseResult::Continue | Respond(early_resp) | Reject
  │  plugin:ingress-tls / ingress-h2c / ingress-h1 / ingress-tcp
  │  (按 hint.kind 精确 dispatch,无竞争)
  │
  ▼  Phase 5: Logging
     输入:ServerCtx(含 phase 1-4 的结果和 timing)
     输出:void
     plugin:metrics-prometheus / metrics-noop
```

#### Egress 侧 — 4 个 phase

```
work stream 收到 RoutingInfo
  │
  ▼  Phase 1: TargetResolve
  │  输入:RoutingInfo { host, port, scheme }
  │  输出:Vec<Target>
  │  plugin:resolver-cached(默认) / resolver-plain(CORE)
  │
  ▼  Phase 2: LoadBalance
  │  输入:Vec<Target> + PickCtx
  │  输出:&Target
  │  plugin:lb-round-robin(默认) / lb-least-inflight
  │
  ▼  Phase 3: Dial
  │  输入:&Target + DialCtx
  │  输出:Connected(impl UpstreamPeer)
  │  plugin:egress-tcp / egress-tls / egress-h1 / egress-h2 / egress-mitm-h2
  │  (按 Target.scheme 精确 dispatch)
  │
  ▼  Phase 4: Logging
     输入:EgressCtx(含 timing + 错误)
     输出:void
     plugin:metrics-prometheus / metrics-noop
```

### 4.4 Trait 定义

#### 4.4.1 服务级 trait(类比 Pingora `ProxyHttp`)

```rust
// tunnel-lib/src/plugin/service.rs

/// 每个 TunnelService 实现对应一类服务部署(如 server 侧 / client 侧)。
/// 框架按固定 phase 顺序调用,实现可以选择性 override。
#[async_trait]
pub trait TunnelService: Send + Sync + 'static {
    type Ctx: Send + 'static;

    fn new_ctx(&self) -> Self::Ctx;

    // ── Server 侧 phases ─────────────────────────────────────────

    /// Phase 2: 鉴权/准入。返回 Reject 则直接关闭连接,不再进入后续 phase。
    async fn admission(
        &self,
        ctx: &mut Self::Ctx,
        req: &AdmissionReq,
    ) -> Result<PhaseResult>;

    /// Phase 3: 路由查找。返回 Reject 则发 RST 或 TCP close。
    async fn resolve_route(
        &self,
        ctx: &mut Self::Ctx,
        route_ctx: &RouteCtx,
    ) -> Result<PhaseResult<Route>>;

    /// Phase 5: 访问日志 / 指标,不允许失败传播。
    fn logging(&self, ctx: &Self::Ctx, result: &PhaseOutcome);
}
```

#### 4.4.2 连接级横切 trait(类比 Pingora `HttpModule`)

```rust
// tunnel-lib/src/plugin/module.rs

/// ConnectionModule 挂在 phase 边界上,做横切关注点。
/// 每条连接独立一个 ctx,框架按 order() 串行执行。
#[async_trait]
pub trait ConnectionModule: Send + Sync + 'static {
    type Ctx: Send + 'static;

    fn order(&self) -> i32 { 0 }  // 数字越小越先执行

    fn new_ctx(&self) -> Self::Ctx;

    /// Phase 2 admission 之前执行(如 rate-limit、IP 黑名单)
    async fn pre_admission(
        &self,
        ctx: &mut Self::Ctx,
        req: &AdmissionReq,
    ) -> Result<PhaseResult> {
        Ok(PhaseResult::Continue)
    }

    /// Phase 5 logging 阶段执行(如写 access log、上报 metrics)
    async fn on_complete(
        &self,
        ctx: &mut Self::Ctx,
        outcome: &PhaseOutcome,
    ) {}
}
```

#### 4.4.3 Ingress Protocol Handler(v2 —— 无 probe 竞争)

```rust
// tunnel-lib/src/plugin/ingress.rs

/// Phase 1 协议嗅探完成后,由框架按 ProtocolHint::kind 精确 dispatch 到对应 handler。
/// handler 不再需要 probe(),因为框架已经确定协议。
#[async_trait]
pub trait IngressProtocolHandler: Send + Sync + 'static {
    /// 声明自己处理哪种协议,框架用这个做 dispatch key。
    fn protocol_kind(&self) -> ProtocolKind;

    /// 处理连接。route 已由 Phase 3 确定,stream 是原始 TcpStream(尚未消耗)。
    async fn handle(
        &self,
        stream: TcpStream,
        route: Route,
        ctx: &ServerCtx,
    ) -> Result<()>;
}

/// 协议嗅探结果(Phase 1 输出),由 tunnel-lib/protocol/detect.rs 产出,不走 plugin。
#[derive(Debug, Clone)]
pub struct ProtocolHint {
    pub kind: ProtocolKind,
    pub sni: Option<String>,         // TLS ClientHello 里的 SNI
    pub authority: Option<String>,   // HTTP/1.1 Host 或 H2 :authority
    pub raw_preface: Bytes,          // peek 缓冲区原始字节,传给 handler 重放
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolKind {
    Tls,
    H2c,       // HTTP/2 cleartext(PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n)
    Http1,
    Tcp,       // 无法识别 → 透传
}
```

**为什么去掉 `probe()`**:

协议识别是 `tunnel-lib/src/protocol/detect.rs` 里的一次性 peek(`peek_exact` + `detect_protocol_and_host`),不需要多轮迭代。Pingora 的做法是 ALPN 在 TLS 握手时由 rustls/openssl 给出,HTTP preface 在第一次 read 时给出。我们对应地让 `detect_protocol_and_host` 一次 peek 最多 256 字节输出 `ProtocolHint`,之后按 `ProtocolKind` 查注册表取 handler,O(1) dispatch,消除竞争。

#### 4.4.4 Egress Traits

```rust
// tunnel-lib/src/plugin/egress.rs

pub trait LoadBalancer: Send + Sync + 'static {
    fn pick<'a>(&self, targets: &'a [Target], ctx: &PickCtx) -> Option<&'a Target>;
}

#[async_trait]
pub trait UpstreamDialer: Send + Sync + 'static {
    /// 声明自己处理哪些 scheme,框架按 Target::scheme 精确 dispatch。
    fn matches_scheme(&self, scheme: &UpstreamScheme) -> bool;

    async fn dial(&self, target: &Target, ctx: &DialCtx) -> Result<Connected>;
}

#[async_trait]
pub trait Resolver: Send + Sync + 'static {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>>;
}
```

#### 4.4.5 MetricsSink(横切,不绑 phase)

```rust
// tunnel-lib/src/plugin/metrics.rs

pub trait MetricsSink: Send + Sync + 'static {
    fn incr(&self, name: &'static str, labels: &[(&str, &str)]);
    fn observe(&self, name: &'static str, value: f64, labels: &[(&str, &str)]);
}
```

`MetricsSink` 注入进 `ServerCtx` / `EgressCtx`,由各 phase 调用。不再是 `ConnectionModule`(Pingora 把 metrics 放在 `logging` phase 回调里,我们也一样 —— `TunnelService::logging()` 里调 `ctx.metrics`。)

### 4.5 ServerCtx 与 EgressCtx(拆分 v1 的 PluginCtx)

#### ServerCtx

```rust
pub struct ServerCtx {
    // ── 抽象能力(trait objects,由注册表注入)──
    pub metrics: Arc<dyn MetricsSink>,

    // ── Phase 输出(随 phase 推进逐步填充)──
    pub hint:    Option<ProtocolHint>,  // 填充于 Phase 1
    pub route:   Option<Route>,         // 填充于 Phase 3
    pub admitted: bool,                 // Phase 2 通过后置 true

    // ── 不值得抽象的共享只读配置 ──
    pub tcp_params: Arc<TcpParams>,
    pub overload:   OverloadLimits,
    pub timeouts:   Timeouts,
    pub peer_addr:  SocketAddr,

    // ── Timing(供 logging phase 使用)──
    pub timing: PhaseTiming,
}
```

#### EgressCtx

```rust
pub struct EgressCtx {
    // ── 抽象能力 ──
    pub metrics: Arc<dyn MetricsSink>,

    // ── Phase 输出 ──
    pub resolved: Vec<SocketAddr>,      // Phase 1
    pub selected: Option<Target>,       // Phase 2
    pub connected: Option<ConnectInfo>, // Phase 3

    // ── 只读配置 ──
    pub tcp_params: Arc<TcpParams>,
    pub timeouts:   Timeouts,

    // ── Timing ──
    pub timing: PhaseTiming,
}
```

**分工规则**:
- **有多种实现可能**的 → `Arc<dyn Trait>`(如 `MetricsSink`)。
- **只有一种实现,或纯配置值** → 直接放 struct 字段(如 `TcpParams`、`Timeouts`)。
- **server 专用**(如 `client_registry`) → 只在 `ServerCtx` 里,不进 `EgressCtx`。
- **egress 专用**(如 `lb`、`dialer`) → 只在 `EgressCtx` 里,不进 `ServerCtx`。

`ServerCtx` 构造在每条连接进来时;`EgressCtx` 构造在 work stream 收到 `RoutingInfo` 时。两者都不是全局单例。

### 4.6 注册表数据结构

```rust
// tunnel-lib/src/plugin/registry.rs

pub struct PluginRegistry {
    ingress_handlers: HashMap<ProtocolKind, Arc<dyn IngressProtocolHandler>>,
    admission_modules: Vec<Arc<dyn ConnectionModule>>,       // 按 order() 排序
    route_resolvers:   HashMap<&'static str, RouteBuilder>,
    metrics_sinks:     HashMap<&'static str, MetricsBuilder>,
    lbs:               HashMap<&'static str, LbBuilder>,
    dialers:           Vec<Arc<dyn UpstreamDialer>>,         // 按 scheme 匹配
    resolvers:         HashMap<&'static str, ResolverBuilder>,
}

type RouteBuilder =
    Box<dyn Fn(&PluginConfig) -> Result<Arc<dyn RouteResolver>> + Send + Sync>;
```

注意 `ingress_handlers` 的 key 是 `ProtocolKind` 而不是字符串:dispatch 是 O(1) HashMap 查找,消除 v1 的 probe 循环。

启动流程 (`server/main.rs`):

```rust
let mut reg = PluginRegistry::new();
server_plugins::register_builtin(&mut reg);
egress_plugins::register_builtin(&mut reg);

let svc = Arc::new(DefaultTunnelService::from_registry(&reg, &config)?);
// svc 里持有 Arc<dyn MetricsSink>、Arc<dyn RouteResolver>、
// HashMap<ProtocolKind, Arc<dyn IngressProtocolHandler>> 等
```

### 4.7 配置驱动组装

```yaml
# server config
ingress_plugins:
  - kind: ingress-h1
  - kind: ingress-tcp
  - kind: ingress-tls
  - kind: ingress-h2c
route_plugin: route-vhost
admission_plugin: admission-token
metrics_plugin: metrics-prometheus

# client config
egress_dialers:
  - egress-tcp
  - egress-tls
  - egress-h1
  - egress-h2
  - egress-h2-mux
lb_plugin: lb-round-robin
resolver_plugin: resolver-cached
```

配置里引用注册表中不存在的 plugin 名 → 启动时 **fail-fast** `unknown plugin "xxx"`,不静默忽略。

### 4.8 一条连接的完整调用链(AS-IS vs TO-BE)

#### AS-IS(HTTPS 入站连接)

| # | 位置 | 动作 | 依赖形式 |
|---|---|---|---|
| 1 | `handlers/http.rs` | accept TCP | `run_accept_worker`(具体函数) |
| 2 | 同上 | `state.tcp_params.apply(&stream)` | `ServerState` 字段(具体 struct) |
| 3 | `handle_http_connection` | peek 首字节判断 TLS/H2/H1 | 内联 `if first == 0x16` (硬编码) |
| 4 | `handle_tls_connection` | 查 vhost | `state.routing.load().get(port)` + `VhostRouter::match_host`(具体) |
| 5 | 同上 | 选 client | `state.registry.select_healthy(group)`(具体) |
| 6 | 同上 | 记指标 | `metrics::open_bi_begin(...)`(具体自由函数) |
| 7 | 同上 | 打开 QUIC stream | `open_bi_guarded(...)`(具体自由函数) |
| 8 | 同上 | 发送 RoutingInfo | `send_routing_info(...)`(具体自由函数) |
| 9 | 同上 | 双向 relay | `proxy::forward_h2_request(...)`(具体自由函数) |

9 步,至少 7 个具体符号。

#### TO-BE(同一条连接)

| # | 位置 | 动作 | 依赖形式 |
|---|---|---|---|
| 1 | `handlers/http.rs` | accept TCP | `run_accept_worker`(基础层工具,保留) |
| 2 | 同上 | 应用 TCP 参数 | `ctx.tcp_params.apply(...)` (`ServerCtx` 字段) |
| 3 | `IngressDispatcher::dispatch` | peek 256B → `detect_protocol_and_host` | CORE 函数,不抽象,一次调用 |
| 4 | 同上 | 按 `hint.kind` 查注册表 → `TlsHandler` | `HashMap<ProtocolKind, Arc<dyn _>>` O(1) |
| 5 | `svc.admission(&ctx, &req).await` | 鉴权 | `trait TunnelService::admission`(trait 方法) |
| 6 | `svc.resolve_route(&ctx, &rctx).await` | 路由 | `trait TunnelService::resolve_route`(trait 方法) |
| 7 | `TlsHandler::handle(stream, route, &ctx)` | TLS 终止 + 选 client + open bi + relay | plugin 内部,入口不感知 |
| 8 | `svc.logging(&ctx, &outcome)` | 记指标 | `trait TunnelService::logging`(trait 方法) |

步骤 7 以后的逻辑(选 client、open_bi_guarded、send_routing_info、relay)仍是调具体基础层函数 —— **这没问题**,它们是无状态工具,不需要抽象。

> 注意:#8 `send_routing_info`、`engine::bridge::relay_with_first_data` 仍直接 call 自由函数 —— 这是**故意的**。过度抽象无益:基础层函数无状态、在任何部署下语义相同,不需要换掉。只抽"有多种可能实现"的东西。

### 4.9 crate 布局

```
tunnel-lib/
  src/
    plugin/
      mod.rs          ← trait 定义聚合
      registry.rs     ← PluginRegistry
      service.rs      ← TunnelService trait
      module.rs       ← ConnectionModule trait
      ingress.rs      ← IngressProtocolHandler + ProtocolHint + ProtocolKind
      egress.rs       ← LoadBalancer + UpstreamDialer + Resolver
      metrics.rs      ← MetricsSink
      ctx.rs          ← ServerCtx + EgressCtx

server/
  plugins/
    tls/              ← ingress-tls  (impl IngressProtocolHandler, ProtocolKind::Tls)
    h2c/              ← ingress-h2c  (impl IngressProtocolHandler, ProtocolKind::H2c)
    h1/               ← ingress-h1   (impl IngressProtocolHandler, ProtocolKind::Http1)
    tcp/              ← ingress-tcp  (impl IngressProtocolHandler, ProtocolKind::Tcp)
    vhost/            ← route-vhost  (impl RouteResolver)
    prometheus/       ← metrics-prometheus (impl MetricsSink)
    token/            ← admission-token (impl ConnectionModule::pre_admission)

client/
  plugins/
    lb_rr/            ← lb-round-robin   (impl LoadBalancer)
    lb_least/         ← lb-least-inflight(impl LoadBalancer)
    h2_mux/           ← egress-h2-mux    (impl UpstreamDialer, scheme H2)
    dns_cached/       ← resolver-cached  (impl Resolver)
    tls_dialer/       ← egress-tls       (impl UpstreamDialer, scheme Https)
```

所有 plugin 模块无条件编进二进制,不加 cargo feature。

### 4.10 模块分层调用依赖(TO-BE)

```
入口层(Entry)
 ┌──────────────────────────────────────────────────────────┐
 │ server/handlers/http.rs / tcp.rs / quic.rs               │
 │                                                          │
 │ 只 use:                                                  │
 │   tunnel_lib::plugin::{ServerCtx, IngressDispatcher}     │
 │   tunnel_lib::plugin::service::TunnelService             │
 │                                                          │
 │ 禁止出现:                                                 │
 │   ✗ crate::metrics                                        │
 │   ✗ crate::plugins::tls                                   │
 │   ✗ state.routing / state.tcp_params(直接字段访问)        │
 └───────────────────┬──────────────────────────────────────┘
                     │ 只调 trait 方法 + IngressDispatcher
                     ▼
抽象层(Abstractions)— tunnel-lib/src/plugin/
 ┌──────────────────────────────────────────────────────────┐
 │ TunnelService / ConnectionModule                         │
 │ IngressProtocolHandler / ProtocolHint / ProtocolKind     │
 │ LoadBalancer / UpstreamDialer / Resolver                 │
 │ MetricsSink                                              │
 │ ServerCtx / EgressCtx                                    │
 │                                                          │
 │ 已有 trait(不动):                                        │
 │   UpstreamResolver / UpstreamPeer / ProtocolDriver       │
 │   ConfigSource / RuleStore / AuthStore                   │
 └───────────────────┬──────────────────────────────────────┘
                     ▼
实现层(Plugins)— server/plugins/* / client/plugins/*
 ┌──────────────────────────────────────────────────────────┐
 │ plugins/tls/    impl IngressProtocolHandler              │
 │ plugins/h2c/    impl IngressProtocolHandler              │
 │ plugins/vhost/  impl RouteResolver                       │
 │ plugins/token/  impl ConnectionModule                    │
 │ plugins/prometheus/ impl MetricsSink                     │
 │                                                          │
 │ plugin 间禁止 use 对方具体符号。                          │
 │ 要协作 → 通过 ServerCtx / EgressCtx 里的 trait object。  │
 └───────────────────┬──────────────────────────────────────┘
                     ▼
基础层(Primitives)— tunnel-lib/src/{engine,transport,open_bi,...}
 ┌──────────────────────────────────────────────────────────┐
 │ engine::bridge::relay_with_first_data  ← 纯 I/O 工具      │
 │ open_bi::open_bi_guarded               ← 纯控制工具       │
 │ protocol::detect::detect_protocol_and_host ← 无副作用     │
 │ transport::listener::VhostRouter       ← 数据结构         │
 │                                                          │
 │ 这一层无状态、不依赖 plugin、不依赖 server/client 身份。 │
 └──────────────────────────────────────────────────────────┘
```

**3 条硬规则**:

1. **调用方向单一**:入口 → 抽象 → 实现 → 基础。不允许反向。
2. **跨层只能通过 trait**:实现层不能被入口层直接 `use`。
3. **同层禁止横向 `use` 具体符号**:同是 plugin,互通走 ctx 里的 trait object。

### 4.11 迁移策略:行为等价保证

PR ② 搬家时**并行保留旧路径 1 个版本**:

1. `ServerState` 加开关 `use_plugin_stack: bool`,默认 `false`。
2. `handle_http_connection` 写成:
   ```rust
   if state.use_plugin_stack {
       dispatcher.dispatch(stream, &svc, &mut ctx).await
   } else {
       legacy_dispatch(stream, &state, port).await
   }
   ```
3. 集成测试跑两遍(`use_plugin_stack=false` 和 `=true`),断言结果一致。
4. 下一个 PR 翻默认值,再下一个删 legacy 分支。

### 4.12 新旧 trait 分工(避免叠床架屋)

| 能力 | 用哪个 trait | 备注 |
|---|---|---|
| 协议嗅探 | **不抽象**(CORE 函数) | `detect_protocol_and_host` 只有一种实现 |
| 鉴权 / 准入 | `ConnectionModule::pre_admission` | 可叠加多个(IP黑名单 + token) |
| 路由查找 | `TunnelService::resolve_route` | 服务级,一个 svc 一个实现 |
| Ingress 协议处理 | `IngressProtocolHandler` | 按 `ProtocolKind` dispatch |
| DNS 解析 | `Resolver` | 可替换为缓存 / DoH |
| LB | `LoadBalancer` | 可替换算法 |
| Upstream 拨号 | `UpstreamDialer` | 按 scheme dispatch |
| Upstream Peer | `UpstreamPeer`(已有) | Dialer 产出此类型,不动 |
| 指标 | `MetricsSink` | 注入进 ctx,logging phase 调用 |
| 配置源 | `ConfigSource`(已有) | 不动 |
| 规则存储 | `RuleStore`(已有) | 不动 |
| 认证存储 | `AuthStore`(已有) | 不动 |
| 后台服务 | `BackgroundService`(已有) | 不动 |

---

## 5. 验证标准

1. 配置里只列 `[ingress-h1, ingress-tcp]` + `[egress-tcp]` 这组最小 plugin 时,server 能启动、接受 1 个纯 TCP listener、把流量正确代理到 client 的 upstream(端到端 E2E)。
2. 配置里去掉 `egress-h2-mux` 后,client 依然能跑 H2 upstream(退化成每次新开 H2 连接)。
3. 配置里没有任何 admission plugin 时,启动**明确报错** "no admission plugin installed"(不允许裸奔)。
4. 每个 plugin 有独立 integration test;CORE 有 smoke test。
5. Criterion benchmark 对比"最小 plugin 集"vs"全量 plugin"的吞吐,证明注册表抽象(Arc + dyn dispatch)额外开销 <1% 或有明确解释。
6. **v2 新增**:协议嗅探 phase 与 ingress handler dispatch 之间没有 `probe()` 循环,单条连接的 phase 1→4 全程最多一次 `peek_exact` 调用。

---

## 6. 收益

### 6.1 给未来加新协议的人

要加 `proxy-protocol v2`(HAProxy PROXY header):

| 步骤 | AS-IS | TO-BE |
|---|---|---|
| 1. 协议识别 | 改 `detect.rs` 加 magic bytes | 改 `detect.rs` 加 `ProtocolKind::ProxyProtocol`(CORE,合理) |
| 2. 新建 handler | 改 `handlers/http.rs` 大 if/else | 新建 `server/plugins/proxy_proto/mod.rs`,impl `IngressProtocolHandler` |
| 3. 注册表 | 无 | 在 `register_builtin()` 里 `insert(ProtocolKind::ProxyProtocol, Arc::new(ProxyProtoHandler))` |
| 4. 指标 | 4 处内联 `metrics::counter!()` | plugin 内部用 `ctx.metrics` |
| 5. 配置 | 改全局 `config.rs` struct | plugin 自己定 `Deserialize` struct |

改动文件:5 个 → 1 个(`detect.rs` + 1 个新 plugin module)。

### 6.2 给升级 tonic / hyper / rustls 的人

TLS 改动只在 `server/plugins/tls/`。PR #27 那类 250 行胶水全重写的情况,下次只改 1 个 plugin module。

### 6.3 给压测 / profile 的人

`metrics_plugin: metrics-noop` 一行配置。`Arc<dyn MetricsSink>` 变空实现,所有调用点一次间接分派直接 return。

### 6.4 给测试的人

Mock `ServerCtx`(替换 `metrics` 为 `NoopSink`),Mock `RouteResolver` 返回固定 `Route`,直接测 `TlsHandler::handle(fake_stream, route, &ctx)`,从 integration test(秒级)降到 unit test(毫秒)。

### 6.5 给部署 / 运维的人

dev / canary / prod 三份 YAML,差别只是 plugin 列表。二进制同一份,CI 不需要 matrix build。

---

## 7. 实施节奏(建议后续 PR)

推荐分成 5-6 个独立 PR,每个都可独立 review / revert:

1. **PR ①**:在 `tunnel-lib/src/plugin/` 定义所有 trait(`TunnelService`、`ConnectionModule`、`IngressProtocolHandler`、`ProtocolHint`、`ProtocolKind`、`MetricsSink`、`LoadBalancer`、`UpstreamDialer`、`Resolver`)和 `ServerCtx` / `EgressCtx` 骨架,**不改现有代码**,加一个最小 smoke test(确认 trait object 能构造)。
2. **PR ②**:把 `H1Handler` / `PassthroughTcpHandler` / `TlsHandler` / `H2cHandler` 从 `handlers/http.rs` 抽出到 `server/plugins/*`,实现 `IngressProtocolHandler`;`IngressDispatcher` 替换旧的 if-else;影子迁移开关默认 `false`。行为等价验证。
3. **PR ③**:`VhostResolver` 插件化;把 `main.rs:453-485` / `listener.rs:63-150` 封装成 `route-vhost` plugin;`StaticPortResolver` 作为 CORE 保底。
4. **PR ④**:`MetricsSink` trait 化所有调用点;`PrometheusSink` 作为 default,no-op sink 作为 CORE 保底;翻影子开关默认值。
5. **PR ⑤**:egress 侧 `LoadBalancer` / `UpstreamDialer` / `Resolver` trait 化;`proxy/{tcp,http,h2}.rs`、`proxy/upstream.rs`、`client/app.rs:47-210` 的决策分支抽成 plugin。
6. **PR ⑥**:配置层接 `ingress_plugins` / `egress_plugins` 列表,支持动态组装;删 legacy 分支。

每个 PR 走完 `cargo check / test / clippy -D warnings` 再进下一步。

---

## 8. 非目标

- 不做 WASM / dlopen / shared library 式动态加载。
- 不走 cargo feature 做编译期裁剪 —— 所有 plugin 都编进二进制,运行时组装就够用。
- 不引入 libp2p 风格的 plugin registry / handshake。
- 不重构已经是 trait 的部分(`ConfigSource` 等)。
- 不改 wire protocol。
- 不在本设计中承诺"未来所有协议都能加"—— 只证明现有能力可以被这套 seam 覆盖。

---

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
| `tunnel-lib/src/protocol/detect.rs:5-98` | 协议嗅探(CORE,不抽象) |
| `tunnel-lib/src/overload.rs`、`open_bi.rs` | 背压 |
| `tunnel-lib/src/engine/bridge.rs:61-93` | relay(CORE) |
| `tunnel-lib/src/models/msg.rs` | wire 消息(CORE) |
| `tunnel-lib/src/transport/listener.rs:63-150` | VhostRouter(→ PR ③) |
