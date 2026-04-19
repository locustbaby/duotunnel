# Ingress / Egress 核心 vs 插件化设计

> 状态:**设计稿**(仅文档,未实施)。落地拆分到后续独立 PR。
> 最后更新:2026-04-19

## 1. 动机

目前 `server/handlers/{http,tcp,quic}.rs` 与 `client/app.rs` 把"跑通一条 TCP over QUIC 隧道"所需的基础能力,和"某些部署才需要的 feature"(TLS SNI、H2C 升级、vhost 路由、LB 策略、DNS 缓存、H2 sender 多路复用、Prometheus 指标、token 准入等)混在同一组热路径里。

后果:

- 新增一种协议/一种 LB 策略/一种认证后端 → 改核心路径。
- 每个部署都扛上全部功能;不用也编进二进制。
- 测试/压测时无法关单个 feature。
- 依赖升级(hyper、rustls、notify)一动就牵大片。

目标:**圈出必需的 CORE**,把其他一切设计为可选插件。不搞动态加载;用 Rust trait + cargo feature 在编译期组装。

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

## 4. 插件化架构

### 4.1 选型:Rust trait + cargo feature

拒绝动态加载(dlopen、WASM runtime、external plugin registry)。理由:

- 本仓编译期就知道要哪些 plugin。
- 不需要热加载、不需要跨语言。
- feature gate 天然帮我们做"不用就不编"。
- 已有 6 个 trait(`BackgroundService` / `ConfigSource` / ...)证明这个模式在本仓可行。

### 4.2 新增的三类扩展点

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

| 现有代码 | 插件名 | 默认启用? |
|---|---|---|
| `H1Handler`(字节透传)—— 由 `handlers/http.rs:335-389` 抽出 | `ingress-h1` | 是(CORE 保底之一) |
| `PassthroughTcpHandler`—— `handlers/tcp.rs` 抽出 | `ingress-tcp` | 是(CORE 保底之一) |
| `TlsHandler`(SNI + 解密 + H2/H1 分发)—— `handlers/http.rs:104-184` | `ingress-tls` | 是 |
| `H2cHandler`(明文 H2 升级)—— `handlers/http.rs:185-334` | `ingress-h2c` | 是 |
| `VhostResolver`—— `transport/listener.rs:63-150` | `route-vhost` | 是;关掉退化为 `StaticPortResolver`(CORE 保底) |
| `PrometheusSink`—— `server/metrics.rs` + 内联调用 | `metrics-prometheus` | 是;关掉用 no-op sink |
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

| 现有代码 | 插件名 | 默认启用? |
|---|---|---|
| RR LB(`proxy/upstream.rs:13-24`) | `lb-round-robin` | 是(CORE 保底) |
| Least-inflight LB(`conn_pool.rs:45-49` 的算法可以复用) | `lb-least-inflight` | 否 |
| Weighted LB(将来) | `lb-weighted` | 否 |
| `TcpDialer`(`proxy/tcp.rs` TcpPeer) | `egress-tcp` | 是(CORE 保底) |
| `TlsTcpDialer`(`proxy/tcp.rs` 的 TLS 分支) | `egress-tls` | 是 |
| `H1Dialer`(`proxy/http.rs` HttpPeer) | `egress-h1` | 是 |
| `H2Dialer`(`proxy/h2.rs` H2Peer) | `egress-h2` | 是 |
| `MitmH2Dialer`(`client/app.rs:145-151`) | `egress-mitm-h2` | 否 |
| H2SenderCache(`proxy/h2_proxy.rs:23-91`) | `egress-h2-mux` | 是 |
| `SystemResolver`(std::net) | CORE | 是 |
| `CachedResolver`(`client/app.rs:47-77` 的 30s TTL 缓存) | `resolver-cached` | 是 |
| `HostsFileResolver` / DoH 等 | `resolver-xxx` | 否 |

#### C. 控制面已经是 trait 的不动

`ConfigSource` / `RuleStore` / `AuthStore` / `BackgroundService` / `UpstreamResolver` / `UpstreamPeer` / `ProtocolDriver` 继续沿用。新 plugin 挂到它们上面即可。

### 4.3 crate 布局

```
tunnel-lib/
  src/
    plugin/
      mod.rs            ← 新:trait 定义聚合
      registry.rs       ← 新:组装 & (可选)热重载
    ...

server/
  plugins/              ← 新目录
    tls/                ← 可选 ingress-tls
    h2c/                ← 可选 ingress-h2c
    vhost/              ← 可选 route-vhost
    prometheus/         ← 可选 metrics-prometheus

client/
  plugins/
    lb_rr/              ← CORE(保底)
    lb_least/           ← 可选
    h2_mux/             ← 可选 egress-h2-mux
    dns_cached/         ← 可选
```

每个可选 plugin 一个 cargo feature flag。默认启用"常用组合"(生产部署的集合),`--no-default-features` 跑最小 CORE。

### 4.4 配置驱动组装(草案)

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

未在 feature 中编进来的 plugin,在配置里引用即报 `unknown plugin "xxx"` 启动错误;不静默忽略。

## 5. 验证标准

1. `cargo build --no-default-features -p server -p client` 编译通过。
2. 关掉所有 OPTIONAL 后,server 能启动、接受 1 个纯 TCP listener、把流量正确代理到 client 的 upstream(端到端 E2E)。
3. 关掉 `egress-h2-mux` 后,client 依然能跑 H2 upstream(退化成每次新开 H2 连接)。
4. 关掉 `admission-token` 后,启动时**明确报错** "no admission plugin installed" —— 不允许裸奔。
5. 每个 plugin 有独立 integration test;CORE 有 smoke test。
6. Criterion benchmark 对比 CORE-only vs full-stack 的吞吐,证明 plugin 抽象的额外开销 <1% 或有明确解释。

## 6. 实施节奏(建议后续 PR)

推荐分成 4-6 个独立 PR,每个都可以独立 review / revert:

1. **PR ①**:定义 `IngressProtocolHandler` / `RouteResolver` / `MetricsSink` / `AdmissionController` 四个 trait 和 `IngressStack` 骨架,**不改现有代码**,仅添加 + 一个最小 smoke test。
2. **PR ②**:把 `H1Handler` / `PassthroughTcpHandler` / `TlsHandler` / `H2cHandler` 从 `server/handlers/http.rs` 抽出到 `server/plugins/*`,实现 trait;`IngressStack` 替换旧的 if-else dispatcher。行为等价。
3. **PR ③**:`VhostResolver` vs `StaticPortResolver`,把 vhost 相关从 `main.rs:453-485` / `listener.rs:63-150` 封装成 plugin。
4. **PR ④**:`MetricsSink` trait 化 `server/metrics.rs` 所有调用点;`PrometheusSink` 作为 default plugin,no-op sink 作为 CORE 保底。
5. **PR ⑤**:egress 侧的 `LoadBalancer` / `UpstreamDialer` / `Resolver` trait,把 `proxy/{tcp,http,h2}.rs`、`proxy/upstream.rs`、`client/app.rs:47-210` 的决策分支抽出。
6. **PR ⑥**:配置层接 `ingress_plugins` / `egress_plugins` 两个列表,支持动态组装。

每个 PR 走完 `cargo check / test / clippy -D warnings` 再进下一步。

## 7. 非目标

- 不做 WASM / dlopen / shared library 式动态加载。
- 不引入 libp2p 风格的 plugin registry / handshake。
- 不重构已经是 trait 的部分(`ConfigSource` 等)。
- 不改 wire protocol。
- 不在本设计中承诺"未来所有协议都能加"—— 只证明现有能力可以被这套 seam 覆盖。

## 8. 相关文件清单(仅引用,不在本设计中修改)

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
