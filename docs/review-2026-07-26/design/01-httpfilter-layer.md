# D1 · HttpFilter 每请求过滤层 —— 详细设计

> 承接：09 §4.1（XFF/可观测）、10 §6.1（缺失抽象）、12 §6.2（内网定位下的能力排序）。
>
> **定位（2026-07-26 复审，D-2）**：本层的正当性是三条——
> **① XFF 客户端 IP 透传**（内网后端必须看到真实来源 IP）、**② 每请求可观测**
> （按后端/路由归因的计时挂载点）、**③ 未来认证/限流的插件插入点**（默认不启用，
> 只保证"插得进去"）。①② 是现在就要的功能缺口，③ 只是**抽象义务**。
> 它**不再**被表述为"安全暴露四件套的公共地基"——目标场景是企业内网、不暴露公网，
> 最终用户认证已降为可选插件（design/03）。下文全部技术设计不受此影响。
> 设计模式，暂不落地代码。

## 1. 背景与问题

### 1.0 内网定位下这一层解决什么（驱动力排序）

**XFF > 每后端可观测 > 认证（可选插件，默认关闭）**：

1. **XFF 客户端 IP 透传（现在就缺的功能）**——内网后端**看不到真实来源 IP**（只看到
   client 进程的本地连接地址），其访问日志、审计、按 IP 的策略/灰度全部失效。数据其实
   **已经跨隧道传过来了**（`RoutingInfo.src_addr`，`msg.rs:98-104`），只差"写进请求头"
   这一步，而**当前没有任何地方能写**——这是本层最直接的驱动力。
2. **每请求可观测（现在就缺的功能）**——指标停留在进程级，无法回答"哪个后端在变慢/
   报错"；需要每请求的计时与归因挂载点（`upstream`/`route` label），它同时是 D2 的
   outlier/重试预算的**判据前置**。
3. **认证/限流的插件插入点（抽象义务，非本轮交付）**——认证在内网定位下**默认关闭、
   按需 opt-in**（D3 §0）；限流以**容量公平**为由保留（D5）。本层对它们的责任仅是
   **留出位置**：让这类能力将来是"加一个 filter impl + 一段配置"，而不是再动一次
   转发路径骨架。

紧迫性因此与 09 第一批的 F1/F2 同档（由 ①② 驱动），**技术设计与集成方式不变**。

### 1.1 根因：三个 L7 分叉点各自内联，无统一插入点

DuoTunnel 的 L7 转发在**三个分叉点**各自内联构造请求/响应,**没有统一的每请求
插入点**:
- `Http1Driver`（`driver/h1.rs`,经 `HttpPeer::connect_inner` `proxy/http.rs:19`）
  ——ingress(client 侧) 与 egress(server 侧) 的 H1;
- `serve_h2_forward` service_fn（`proxy/h2.rs:23`）——H2 上游转发;
- TLS 终止的 service_fn（`plugins/tls/mod.rs:110`)——server 面向公网客户端的 H2。

`ConnectionModule`（`plugin/module.rs:14`）只有连接级 `pre_admission`/`on_complete`,
**看不到 `Request`/`Response`**。因此 XFF、header 改写、认证、请求级限流**无处插入**。
这也是 Pingora `ProxyHttp` 有、DuoTunnel 缺的那一层(与 TODO-77/68 同源)。

**设计目标**:引入一个每请求 filter 链,在**请求发上游前**与**响应写下游前**被调用,
一次性承接上述能力;短期以共享 helper 接入三个分叉点,长期随 TODO-77 收敛到统一
Session 拥有该链。

## 2. 抽象设计

### 2.1 核心 trait 与类型

```rust
// duotunnel-lib/src/plugin/http_filter.rs (新)
use http::{HeaderMap, Method, Uri, Version, StatusCode};
use std::net::SocketAddr;
use std::time::Instant;

/// 请求侧可见/可改的视图。只暴露 head(不含 body),v1 不做 body 级改写。
pub struct FilterRequestCtx<'a> {
    pub method: &'a Method,
    pub uri: &'a mut Uri,               // 允许 path/query 改写
    pub headers: &'a mut HeaderMap,     // 允许增删改 header(XFF/request-id/auth 透传)
    pub version: Version,
    /// 真实客户端地址——来自 RoutingInfo.src_addr(隧道已携带),这是 XFF 的数据源
    pub client_addr: SocketAddr,
    pub route: &'a Route,               // group_id / proxy_name
    pub upstream: Option<&'a str>,      // 已解析的上游(若可得)
    pub started_at: Instant,            // 计时起点(每请求可观测)
    pub request_id: &'a mut Option<String>, // 生成/透传的 request-id,供响应侧回填
    pub protocol: crate::proxy::core::Protocol, // H1/H2/WS
}

/// 响应侧视图。
pub struct FilterResponseCtx<'a> {
    pub status: StatusCode,
    pub headers: &'a mut HeaderMap,     // 允许加安全头/回填 request-id
    pub client_addr: SocketAddr,
    pub route: &'a Route,
    pub upstream: Option<&'a str>,
    pub started_at: Instant,            // 用于算端到端/上游耗时
    pub request_id: Option<&'a str>,
}

/// 请求侧结果:继续,或短路返回一个本地响应(认证拒绝/限流 429/重定向 3xx)。
pub enum FilterOutcome {
    Continue,
    ShortCircuit(ShortResponse),
}
pub struct ShortResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: bytes::Bytes,
}

#[async_trait::async_trait]
pub trait HttpFilter: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    /// 链内顺序,低者先执行请求侧、后执行响应侧(洋葱模型)。
    fn order(&self) -> i32 { 0 }

    /// 发上游前。默认放行。可改写 head、生成 request-id、短路。
    async fn on_request(&self, _ctx: &mut FilterRequestCtx<'_>) -> anyhow::Result<FilterOutcome> {
        Ok(FilterOutcome::Continue)
    }
    /// 写下游前。默认不改。可加安全头、回填 request-id、记录分位。
    async fn on_response(&self, _ctx: &mut FilterResponseCtx<'_>) {}
}
```

### 2.2 链与执行语义

```rust
pub struct HttpFilterChain {
    filters: Vec<Arc<dyn HttpFilter>>, // 构造时按 order 升序
}
impl HttpFilterChain {
    /// 请求侧:按 order 升序逐个 on_request;任一返回 ShortCircuit 立即停并回该响应。
    pub async fn apply_request(&self, ctx: &mut FilterRequestCtx<'_>) -> anyhow::Result<FilterOutcome>;
    /// 响应侧:按 order 降序逐个 on_response(洋葱:请求侧先进者响应侧后出)。
    pub async fn apply_response(&self, ctx: &mut FilterResponseCtx<'_>);
    pub fn is_empty(&self) -> bool;
}
```

- **洋葱模型**:请求侧 A→B→C,响应侧 C→B→A。符合"外层 filter 包裹内层"的直觉
  (如:认证在最外,改写在内)。
- **空链快路径**:`is_empty()` 时集成点直接跳过,零开销(保证未启用 filter 时无回归)。

### 2.3 注册(复用现有 PluginRegistry)

```rust
// plugin/registry.rs 扩展
pub struct PluginRegistry {
    // ... 既有字段 ...
    pub http_filters: Arc<HttpFilterChain>,   // 新增,默认空链
}
impl PluginRegistry {
    pub fn set_http_filters(&mut self, filters: Vec<Arc<dyn HttpFilter>>) { /* 排序建链 */ }
}
```

filter 实例在 bootstrap 由 config 构建后注入(§4)。`ServerCtx` 已持
`Arc<PluginRegistry>`/`metrics`,把 `http_filters` 一并暴露给 L7 handler。

## 3. 集成点(精确 callsite)

三个 L7 分叉点各调用一次 `apply_request` / `apply_response`。**关键前置改造 = 把
`client_addr`(真实客户端)+ `route` 下沉到这三处**——目前它们拿不到(见每点说明)。

### 3.1 Http1Driver / HttpPeer（H1,ingress+egress 共用）

- 现状:`HttpPeer::connect_inner`（`proxy/http.rs:19-90`)循环
  `driver.read_request()`→构造 hyper `Request`→`connector.request()`→
  `driver.write_response()`。**`HttpPeer` 当前不持有 client_addr/route/filters**。
- 改造:
  1. `HttpPeer` 增字段 `filters: Arc<HttpFilterChain>`、`client_addr: SocketAddr`、
     `route: Route`(由 `IngressClientApp::connect_peer` / `ServerEgressMap::connect_peer`
     构造时传入——它们持有 `Context.client_addr` 与 routing)。
  2. `read_request` 得到 `ProxyRequest{method,uri,headers,body}` 后,若 `!filters.is_empty()`:
     构造 `FilterRequestCtx`(借用 `req.headers`/`&mut req.uri`)→ `apply_request`:
     - `Continue`→ 照常发上游;
     - `ShortCircuit(resp)`→ 用 `driver` 写该本地响应(新增 `driver.write_short(resp)`),
       **不开上游**,continue 循环(keep-alive)或按 `Connection: close` 结束。
  3. `connector.request()` 拿到响应后,`apply_response`(借用响应 headers)→ 再
     `driver.write_response()`。
- 计时:`started_at` 在 `read_request` 返回时取,`on_response` 里可算上游耗时。

### 3.2 serve_h2_forward（H2 上游转发）

- 现状:`proxy/h2.rs:23-79` service_fn,`req.into_parts()`→改 uri/host→
  `connector.request()`。**service_fn 闭包当前不含 client_addr/route/filters**。
- 改造:`serve_h2_forward` 签名增 `filters`、`client_addr`、`route`(由
  `HttpConnector::serve_h2` 调用链传入);service_fn 内 `into_parts` 后
  `apply_request`(借用 `parts.headers`/`&mut parts.uri`),`ShortCircuit`→ 直接返回该
  响应(H2 天然按流应答);上游响应 `apply_response` 后返回。

### 3.3 TLS 终止 service_fn（server 面向下游客户端的 H2,请求最早可见点）

- 现状:`plugins/tls/mod.rs:110-227` service_fn 已有 client_addr(`src_addr`)、
  route_target——**数据齐全,只差调用 filter 链**。
- 改造:`into_parts` 后、`forward_h2_request` 前 `apply_request`;`ShortCircuit`→
  `error_response`-style 直接返回(限流/重定向/可选认证的拒绝落此);响应侧 `apply_response`。
  **这是下游请求最早可见的接入点**——XFF/可观测在此挂载,（可选启用的）认证/限流
  也落在此处。

### 3.4 关于三处重复:与 TODO-77 的关系

三处各调一次是短期务实解;根因是 L7 路径分叉(H1 driver / H2 service / TLS service)。
**长期随 TODO-77 收敛到统一 `Session`,由 Session 拥有 filter 链、只调一处**。D1 的
`FilterRequestCtx`/`FilterResponseCtx` 视图刻意设计成协议中立(head-only),正是为了
未来平滑并入 Session。**建议 D1 与 TODO-77 协同排期**:先落 filter 链 + 三处接入,
Session 统一时把三处收敛为一处。

## 4. 配置 schema

```yaml
# server.yaml (tunnel_management 或 server 级,按作用域)
http_filters:
  - type: forwarded_for            # 客户端 IP 透传(D6)
    mode: append                   # append | replace
    trusted_proxies: ["10.0.0.0/8"] # 先剥离不可信入站 XFF 再追加真实源
    set_forwarded: true            # 同时写 RFC 7239 Forwarded
  - type: request_id               # 生成/透传 x-request-id
    header: x-request-id
  - type: oidc_auth                # D3(占位,详见 D3 设计)
    issuer: ...
  - type: rate_limit               # D5(占位,详见 D5 设计)
    key: client_ip
    rate: 100/s
```

- 顺序 = 配置顺序(或显式 `order`);建议默认序:认证 → 限流 → XFF/request-id → 业务改写。
- bootstrap 按 `type` 构造对应 `HttpFilter` impl,注入 `set_http_filters`。

## 5. 内置 filter(v1 交付)

### 5.1 `ForwardedForFilter`(交付 D6 的 XFF)
- `on_request`:按 `trusted_proxies` 策略——入站若来自不可信源,**先剥离**已有
  `X-Forwarded-For` 再追加 `client_addr.ip()`;来自可信前置代理则**追加**保留链。
  同时(可选)写 `X-Forwarded-Proto`、`Forwarded`(RFC 7239)。
- 数据源:`ctx.client_addr`(= `RoutingInfo.src_addr`,隧道已带真实客户端)。

### 5.2 `RequestIdFilter`
- `on_request`:无 `x-request-id` 则生成(uuid/ulid),写入 `ctx.request_id`+header;
- `on_response`:回填 `x-request-id` 到响应 header(便于端到端追踪)。

### 5.3 认证/限流 filter
- 作为 D3/D5 的 impl 挂在同一链上(本文只留接口位)。

## 6. 场景覆盖 & Corner Cases

| 场景/边界 | 处理 |
| --- | --- |
| **TCP 透传 / WebSocket / 原始 TCP** | 无 HTTP 语义,**filter 不适用**——L7 handler 才调链,passthrough 路径不调;客户端 IP 走 PROXY protocol(11 §4) |
| **H2/gRPC 多路复用** | service_fn 本就每请求(每流)调用,filter 天然按流生效 |
| **ShortCircuit 必须不开上游** | 短路在"发上游前",此时尚未 `open_bi`;直接回本地响应,不建 QUIC 流,不占 inflight |
| **XFF 伪造** | `trusted_proxies` 策略:不可信入站先剥离再追加真实源;可信前置代理才 append |
| **filter 内 await 的成本** | `XFF`/`request_id` 是**同步**逻辑,`async_trait` 会 box 一个立即 ready 的 future,热路径有微开销。设计:为纯同步 filter 提供 `on_request_sync` 默认路径(或链在空/全同步时走无 await 快路径);以 06 microbench 验证,避免每请求装箱 |
| **filter 错误策略** | 每 filter 声明 fail-open / fail-closed:认证 filter **fail-closed**(出错=拒绝),XFF/request-id **fail-open**(出错=放行不改写)。链执行遇错按该 filter 策略处理 |
| **顺序敏感** | 限流应在认证**之前**还是之后可配(之前=省认证开销,之后=按已认证身份限流);默认认证→限流 |
| **响应短路后的 keep-alive** | H1 短路响应后按 `Connection` 头决定复用/关闭,复用 driver 现有 should_close 逻辑 |
| **body 级能力(镜像/body 改写)** | v1 仅 head;镜像/body 改写需 body tee,列 **v2**(设计上 `FilterRequestCtx` 预留,不在 v1 实现) |
| **client_addr 缺失** | 极少数无 `src_addr` 的路径,filter 收到 `Unspecified`,XFF 跳过并告警 |

## 7. 论证 / 备选

- **为何是"filter 链"而非"在 Http1Driver 里加 XFF 参数"**:单加 XFF 只解一个点,
  认证/限流/改写/重定向/镜像仍无处放;filter 链是 Pingora/Envoy 的通用承接层,一次
  到位,后续能力都是"加一个 filter impl"。
- **为何请求+响应两个 hook**:XFF/认证/限流在请求侧,安全头/request-id 回填/分位在
  响应侧,缺一不可。
- **为何 head-only(v1)**:head 级覆盖本层全部目标能力(XFF/request-id/可观测/改写/
  重定向,及可选的认证/限流);body 级(镜像/body 改写)需 body 流 tee,复杂度高、
  收益集中在少数场景,列 v2。
- **为何短期接三处而非等 TODO-77**:**XFF 与每后端可观测**是当前内网定位下的关键路径
  (09 第一批 F1/F2),不宜阻塞在大重构上;三处接入的重复由后续 Session 统一消除,
  视图已为此设计。
- **备选(在 ProxyEngine 统一调用)**:ProxyEngine::run_stream 是更上游的单一点,但它
  此刻只做 sniff+resolve+connect,不解析 HTTP 请求(解析在 driver/service 内),无法在
  此看到 Request——所以 filter 必须落在解析之后的三处,除非先做 TODO-77。

## 8. 取舍 / 改动量 / 影响

- **取舍**:引入一层每请求抽象 + `client_addr`/`route` 下沉到三处 L7 函数的 plumbing;
  换取 **XFF/每请求可观测/改写的统一承接**,以及认证/限流的**插件插入点**。
  空链零开销保证无回归(未启用任何 filter = 今天的行为)。
- **改动量**:trait+chain+ctx(~150 行)+ registry 字段 + **三处 L7 callsite 接入及
  client_addr/route plumbing**(HttpPeer/serve_h2_forward 签名变更,是主要工作)+ 两个
  内置 filter(XFF/request-id)+ 测试。**~1 周**(plumbing 是大头)。
- **影响面**:所有 HTTP 转发路径;passthrough/UDP 不受影响。需回归 header 正确性、
  keep-alive、H2 按流、短路语义。热路径新增以 06 microbench + TODO-140 基线验证。

## 9. 分阶段实施(设计蓝图,暂不落地)

| 阶段 | 内容 | 交付 | 测试 |
| --- | --- | --- | --- |
| P1 | trait+chain+ctx + registry 字段 + client_addr/route 下沉到三处 + 空链接入(no-op) | 骨架,零行为变化 | 空链回归=逐字节一致;client_addr 正确到达三处 |
| P2 | `ForwardedForFilter` + `RequestIdFilter` + config 解析 | **D6 的 XFF 落地** | XFF 追加/剥离/伪造、request-id 生成回填 |
| P3 | 响应侧 + ShortCircuit 全链路(H1 write_short + H2 直接返回) | 重定向/拒绝能力就绪 | 短路不开上游、keep-alive 正确 |
| P4（可选） | 挂 D5(限流)/D3(认证,**默认关闭**)filter | 验证插入点可用;认证按需 opt-in,不在主线交付 | 见 D3/D5 |

## 10. 验收

- [ ] 空链下 L7 转发逐字节无变化(零回归);
- [ ] `client_addr`/`route` 正确到达三处 L7 集成点;
- [ ] `ForwardedForFilter` 按 trusted_proxies 策略正确追加/剥离,后端可见真实客户端 IP;
- [ ] `ShortCircuit` 不开上游 QUIC 流、不占 inflight;
- [ ] 纯同步 filter 无每请求装箱开销(microbench 验证);
- [ ] passthrough/WS/UDP 路径不触发 filter。
```
