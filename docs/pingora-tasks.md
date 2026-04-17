# Pingora-inspired 重构：Task 清单

对照 Cloudflare Pingora (`../pingora`) 梳理可直接移植的设计模式，以 TODO 形式拆分实施步骤。每个 TODO 自足——包含背景、现状痛点、Pingora 参照、Fix 设计、为什么好、依赖与两端影响、Files、Validation。

## 目录

| TODO | 标题 | Priority |
|---|---|---|
| [TODO-63](#todo-63-upstreampeer-trait-统一--消灭-peerkind-分派不一致--主线第一步) | UpstreamPeer trait 统一 + 消灭 PeerKind 分派不一致 | High |
| [TODO-64](#todo-64-reusehash--clientidgroupid-newtype--主线第二步) | ReuseHash + ClientId/GroupId newtype | High |
| [TODO-65](#todo-65-tunnel-lib-error-类型--retrytypereusedonly--主线第三步) | tunnel-lib Error 类型 + RetryType::ReusedOnly | High |
| [TODO-66](#todo-66-统一-httpconnector--h1h2-降级记忆替代-httppeerh2peer) | 统一 HttpConnector + H1/H2 降级记忆 | High |
| [TODO-67](#todo-67-servicea--serverapp-抽象合并-accept--listener) | Service\<A\> + ServerApp 抽象（合并 accept/listener） | Medium |
| [TODO-68](#todo-68-proxyhandler-trait--连接级-ctx消灭-god-closure) | ProxyHandler trait + 连接级 Ctx（消灭 god-closure） | Medium |
| [TODO-69](#todo-69-sticky-selection连接级缓存-upstream-pick--热路径优化server) | Sticky selection：连接级缓存 upstream pick | High |
| [TODO-70](#todo-70-server-端-snapshot-持-arcselectedconnection对齐-client) | Server 端 snapshot 持 Arc\<SelectedConnection\>（对齐 client） | Medium |
| [TODO-71](#todo-71-p2c-pick-算法可选池规模增长后启用) | P2C pick 算法（可选，池规模增长后启用） | Low |
| [TODO-72](#todo-72-client-端小优化非紧急随手做) | Client 端小优化 | Low |
| [TODO-73](#todo-73-不要抄-pingora-的部分参考避坑) | 不要抄 Pingora 的部分（参考避坑） | FYI |

**推荐落地顺序**：63 → 64 → 65 → 66 → 67 → 68 → 69 → 70 → 72 → 71。每个 TODO 完成后跑 CI stress phase，观察基准变化。

依赖关系、吞并关系、server/client 两端影响写在各自 task 的「**依赖与影响**」字段里。

---

## [TODO-63] UpstreamPeer trait 统一 + 消灭 PeerKind 分派不一致 ★ 主线第一步

**Priority**: High | **Status**: TODO
**依赖与影响**:
- 依赖：— （主线起点）
- 被依赖：TODO-64 (reuse_hash 接口)、TODO-66 (PeerSpec)
- 关联：todo.md 的 TODO-36 (Static Dispatch Refactor)
- 两端影响：server egress 和 client egress 都要改（对称）

**Problem**:
`tunnel-lib/src/proxy/peers.rs:7-28` 的 `PeerKind` enum 混用三种分派：
```rust
enum PeerKind {
    Tcp(TcpPeer),               // 不 box, enum dispatch 零开销
    Http(Box<HttpPeer>),        // 多一次堆分配
    H2(Box<H2Peer>),            // 多一次堆分配
    Dyn(Box<dyn UpstreamPeer>), // vtable，但全仓库 grep 无构造点（死代码）
}
```
- **分派策略不一致**：同一 trait 下混用 enum dispatch / box / vtable 三种机制，理解成本高
- **`Dyn` 变体是坏味道**：兜底后门，实际没人用——证明 enum 从设计之初就不够扩展
- **`connect(self)` 消耗所有权**：`HttpPeer` 持 `HttpsClient`，每个请求都要重建，连接复用逻辑被迫外置到 `server/egress.rs`

Pingora 的做法 (`pingora-core/src/upstreams/peer.rs:88-306`)：单一 `Peer` trait + 可选 `PeerOptions` bag-of-settings，具体类型（`BasicPeer`、`HttpPeer`）各自实现。**Peer 只是上游描述符（value object），不持连接**；连接复用在 `HttpConnector` 那层独立做（见 TODO-66）。

**Fix**:
```rust
pub trait UpstreamPeer: Send + Sync {
    fn address(&self) -> &SocketAddr;
    fn scheme(&self) -> Scheme;
    fn reuse_hash(&self) -> u64;
    fn options(&self) -> Option<&PeerOptions>;
}

pub struct PeerOptions {
    pub alpn: ALPN,                  // H1Only | H2Only | H2H1
    pub keep_alive: Option<Duration>,
    pub connect_timeout: Duration,
    pub tls: Option<TlsOptions>,
}

pub struct BasicPeer { addr: SocketAddr, opts: PeerOptions }            // TCP/raw
pub struct HttpPeerSpec { addr: SocketAddr, host: String, scheme: Scheme, opts: PeerOptions }
```

具体动作：
1. 建 trait + 两种实现（BasicPeer / HttpPeerSpec）
2. 拆掉 `HttpPeer` / `H2Peer` 的连接逻辑，下沉到 TODO-66 `HttpConnector`
3. 删除 `PeerKind::Dyn` 死代码
4. 容器统一为 `Arc<dyn UpstreamPeer>` 或具体 enum（按场景）

**为什么好**（对比现状）：

| 维度 | 现状 | 新设计 |
|---|---|---|
| 分派一致性 | 三种混用 | 单一 trait |
| 扩展新协议 | 改 enum + 改所有 match | 新增 `impl UpstreamPeer` 即可 |
| 连接复用 | `self`-consume 无法复用 | Peer 不变，Connector 按 reuse_hash 复用 |
| 死代码 | `Dyn` 无人用 | — |

**Files**:
- `tunnel-lib/src/proxy/peers.rs`（重写）
- `tunnel-lib/src/proxy/http.rs` / `h2.rs` / `tcp.rs`（拆连接逻辑）
- `tunnel-lib/src/proxy/core.rs`（`UpstreamResolver::upstream_peer` 返回 `Arc<dyn UpstreamPeer>`）
- 调用方：`server/egress.rs`、`client/app.rs`

**Validation**: CI 全量（含 stress phase）无回归；metrics `request_completed` 各协议正常。

---

## [TODO-64] ReuseHash + ClientId/GroupId newtype ★ 主线第二步

**Priority**: High | **Status**: TODO
**依赖与影响**:
- 依赖：TODO-63（需要 `UpstreamPeer::reuse_hash` 接口）
- 被依赖：TODO-66（池 key）
- 两端影响：server `registry.rs` + client `conn_pool.rs` 的 `stable_id` 都要改 newtype

**Problem**:
- `server/registry.rs:10-11` 全链路裸 `String client_id / group_id`：typo 编译期不报错
- `tunnel-store/src/rules.rs`、`tunnel-lib/src/models/msg.rs` 同样问题
- 热路径（`select_client_for_group`、pool lookup）用 String 哈希
- duotunnel 的连接池（`hyper_util` 内部）按 `(Scheme, Authority)` 字符串做 key，**维度不全**——同一 host 不同 ALPN/TLS 配置会错误复用

Pingora 的 `ConnectionPool<S>` (`pingora-pool/src/connection.rs:29-82`) 用 `u64 GroupKey` 作 key，由 `Peer::reuse_hash()` 生成（`pingora-core/src/upstreams/peer.rs:98-101`）：address + scheme + TLS cert serial + ALPN 全部 hash 进去，既保证无冲突又 O(1) 查找。

**Fix**:
新增 `tunnel-lib/src/ids.rs`：
```rust
pub struct ClientId(Arc<str>);     // zero-cost clone
pub struct GroupId(Arc<str>);
pub struct ProxyName(Arc<str>);
pub struct ReuseHash(u64);
```

`UpstreamPeer::reuse_hash()` 默认实现：
```rust
fn reuse_hash(&self) -> ReuseHash {
    let mut h = AHasher::default();
    self.address().hash(&mut h);
    self.scheme().hash(&mut h);
    if let Some(o) = self.options() {
        o.alpn.hash(&mut h);
        o.tls.as_ref().map(|t| t.cert_serial.hash(&mut h));
    }
    ReuseHash(h.finish())
}
```

全链路替换：server/registry.rs、server/control_client.rs、tunnel-store/rules.rs、client/conn_pool.rs（`stable_id` 字段）、`models/msg.rs` 的 `RoutingInfo`。

**为什么好**：
- `GroupId ≠ ClientId`，typo 编译期拦截
- O(1) 整数哈希替代字符串哈希，热路径（`select_client_for_group`）更快
- 连接池维度完整：不同 ALPN / TLS 配置的同一 host 不会错误复用
- `Arc<str>` 零成本共享，比 `String::clone()` 快

**Files**:
- 新增 `tunnel-lib/src/ids.rs`
- `server/registry.rs`、`client/conn_pool.rs`、`tunnel-store/*`
- `tunnel-lib/src/models/msg.rs`

**Validation**: 编译通过 + 单元测试；热路径 flamegraph 确认 String hash 成本消失。

---

## [TODO-65] tunnel-lib Error 类型 + RetryType::ReusedOnly ★ 主线第三步

**Priority**: High | **Status**: TODO
**依赖与影响**:
- 依赖：—（可独立于 63/64 推进）
- 被依赖：TODO-66（Connector 的 ReusedOnly 语义）、TODO-72（client retry 逻辑）
- 两端影响：server metrics + client `entry.rs` retry 都受益

**Problem**:
- `anyhow::Error` 全局使用（`tunnel-lib/src/ctld_proto.rs`、`tunnel-store/src/traits.rs`、`tunnel-lib/src/open_bi.rs:39` 等），丢失语义
- 无法区分「upstream 问题 / 下游问题 / 内部 bug」——告警等级一刀切
- 无法区分「可重试 / 不可重试」
- **关键 bug 类**：H1/H2 长连接服务端 idle 超时 close 是**正常现象**；复用连接首个请求失败应该重试新连接，新建失败才不重试。现状 `HttpPeer::connect_inner` 把错误一股脑上抛，用户看到偶发 502
- `open_bi_guarded` 返回 `Err(anyhow!("open_bi timed out after {:?}", stream_timeout))`，调用方只能从字符串推测是超时还是握手失败
- `server/handlers/http.rs` 里硬编码 502 处处出现

Pingora 的 `Error` (`pingora-error/src/lib.rs:31-87`)：三元组 `{ etype, esource, retry }`，`RetryType::ReusedOnly` 变体是长连接代理的关键语义。

**Fix**:
新增 `tunnel-lib/src/error.rs`：
```rust
pub struct Error {
    pub etype: ErrorType,
    pub source: ErrorSource,        // Upstream / Downstream / Internal
    pub retry: RetryType,
    pub context: Option<String>,
    pub cause: Option<BoxError>,
}

pub enum ErrorType {
    ConnectTimeout, ConnectRefused, TLSHandshake,
    H1Error(H1Kind), H2Error(H2Kind),
    QuicStreamReset, QuicOpenTimeout, QuicConnectionLost,
    InvalidConfig, RouteNotFound,
}

pub enum ErrorSource { Upstream, Downstream, Internal }

pub enum RetryType {
    Decided(bool),
    /// 仅在连接被复用时可重试（idle close 典型场景）
    ReusedOnly,
}

impl Error {
    pub fn should_retry(&self, was_reused: bool) -> bool {
        match self.retry {
            RetryType::Decided(b) => b,
            RetryType::ReusedOnly => was_reused,
        }
    }
}
```

替换关键路径：`ctld_proto.rs`、`open_bi.rs`、`tunnel-store/traits.rs`、`HttpPeer/H2Peer`（和 TODO-66 一起整）。

应用 `ReusedOnly` 语义：`HttpConnector::get_session` 复用路径失败 → 标记 ReusedOnly → 调用方重试新建。

Metrics 增加 error label：`error_total{type="ConnectTimeout", source="upstream"}`，取代现状 `request_completed{status="error"}` 黑盒。

**为什么好**：
- **`ReusedOnly`** 是长连接代理的必备语义——解决一个真实 bug 类（用户看到偶发 502 其实都是 idle close）
- **`source`** 让告警分级：Downstream warn，Upstream error，Internal 直接 sentry
- **`etype`** 让 metrics 打 label，观测性大幅提升

**Files**:
- 新增 `tunnel-lib/src/error.rs`
- `tunnel-lib/src/open_bi.rs`
- `tunnel-lib/src/ctld_proto.rs`、`tunnel-store/src/traits.rs`
- `tunnel-lib/src/proxy/http.rs`、`h2.rs`
- `server/metrics.rs` / `client/metrics`

**Validation**: 注入 idle close 场景，确认不再 502，自动重试新连接。保留 `anyhow` 在 binary main 的兜底，不强行全局替换。

---

## [TODO-66] 统一 HttpConnector + H1/H2 降级记忆（替代 HttpPeer/H2Peer）

**Priority**: High | **Status**: TODO
**依赖与影响**:
- 依赖：TODO-63（PeerSpec）、TODO-64（ReuseHash 作池 key）、TODO-65（Error::ReusedOnly）
- 被依赖：—
- 吞并：todo.md 的 TODO-62（per-peer 协议自适应直接落地）、TODO-CR-NEW-C（合并 TcpPeer 和 TlsTcpPeer）
- 两端影响：server egress + client egress upstream 都改用 Connector

**Problem**:
- `tunnel-lib/src/proxy/http.rs:62-131` + `h2.rs:82-118` 两份并列的 keep-alive loop，字段（`target_host: String, scheme: String`）重复、loop 结构重复
- `HttpsClient` 内部 pool 按字符串 host 缓存，维度不全（丢失 ALPN/TLS 证书信息）
- 无协议探测/记忆：每次请求都可能重试 H2 再降级 H1
- 上层必须预先声明 `protocol: H2 | H1`，不符合「按 upstream 实际能力走」
- todo.md TODO-62 明确提出要做运行时探测+记忆，但没有方案

Pingora 的 `Connector<C>` (`pingora-core/src/connectors/http/mod.rs:30-154`) **共享一个 H1 和 H2 pool**，自动 ALPN 协商，并**在全局 `PreferredHttpVersion` map 里记忆 H1 偏好** (`pingora-core/src/connectors/mod.rs:346-373`)——H2 握手失败过的 peer，下次直接走 H1。

**Fix**:
新增 `tunnel-lib/src/proxy/http_connector.rs`：
```rust
pub struct HttpConnector {
    h1_pool: Pool<H1Session>,
    h2_pool: Pool<H2Session>,
    prefer_h1: DashMap<ReuseHash, Instant>,   // ttl-based 降级记忆
}

impl HttpConnector {
    pub async fn get_session(&self, peer: &HttpPeerSpec) -> Result<Session, Error> {
        let key = peer.reuse_hash();

        // 1. 记忆：上次 H2 失败过，直接 H1
        if self.prefer_h1.contains_key(&key) {
            return self.h1_get_or_new(peer, key).await;
        }
        // 2. H2 池复用
        if let Some(s) = self.h2_pool.reuse(key) { return Ok(Session::H2(s)); }
        // 3. H1 池复用（优先于新建 H2，因为新建开销大）
        if let Some(s) = self.h1_pool.reuse(key) { return Ok(Session::H1(s)); }
        // 4. 按 ALPN 尝试新建
        match self.new_h2(peer).await {
            Ok(s) => Ok(Session::H2(s)),
            Err(e) if e.is_alpn_mismatch() => {
                self.prefer_h1.insert(key, Instant::now());
                Ok(Session::H1(self.new_h1(peer).await?))
            }
            Err(e) => Err(e),
        }
    }
}
```

keep-alive loop 收归到 `H1Session::run_loop`，**单份**。删除 `HttpPeer`、`H2Peer` 的 `connect_inner`，只留 PeerSpec（TODO-63 已做 PeerSpec 部分）。

合并 `TcpPeer` / `TlsTcpPeer` 成 `BasicPeer`，按 `opts.tls: Option<TlsOptions>` 分支（兑现 TODO-CR-NEW-C）。

**为什么好**：
- **单一入口**：`Connector::get_session` 替代两套 `connect_inner`，keep-alive 循环只留一份
- **命中 TODO-62**：`prefer_h1` map 就是「运行时探测+记忆」
- **连接真正复用**：`reuse_hash` 作 key 的池，比 hyper 默认字符串 host 池命中率高（加入了 TLS/ALPN 维度）
- **ALPN 自适应**：上游是 H1 only 也不报错，自动降级并记住

**Files**:
- 新增 `tunnel-lib/src/proxy/http_connector.rs`、`h1_session.rs`、`h2_session.rs`（整合现有逻辑）
- 删除 `tunnel-lib/src/proxy/http.rs` 和 `h2.rs` 的 connect_inner 部分
- `server/egress.rs`、`client/app.rs` 改用 Connector

**Validation**: upstream H2-only 和 H1-only 场景都能走通；`prefer_h1` 命中/miss 分别验证；gRPC/WS 基准无回归。

---

## [TODO-67] Service<A> + ServerApp 抽象（合并 accept / listener）

**Priority**: Medium | **Status**: TODO
**依赖与影响**:
- 依赖：—（与 63-66 主线正交，可并行）
- 被依赖：TODO-68（需要 ServerApp 抽象承载 ProxyHandler）
- 两端影响：server ingress 全部 handler + client entry 都改用 Service

**Problem**:
- `tunnel-lib/src/accept.rs::run_accept_worker` 和 `tunnel-lib/src/transport/listener.rs::start_tcp_listener` 两套重叠 accept 抽象，后者包前者，只是多要求 `Handler: Clone`——取舍标准不清
- 各 handler 自己 `tokio::task::spawn`（`server/handlers/http.rs:29`、`tcp.rs`、`quic.rs`），无统一「每连接一 task」契约
- shutdown 协作散乱：`CancellationToken` 每个 handler 自己 select

Pingora 的三层抽象（`pingora-core/src/services/listening.rs:44-113` + `pingora-core/src/apps/mod.rs:164-175`）：
```
Service<A>                         ← 通用 accept runtime（与协议无关）
  持 Listeners + shutdown watch
  accept → spawn → A::process_new(conn, shutdown)

    A: ServerApp                   ← 「最小实现」：一条连接怎么处理
      trait HttpServerApp {
          async fn process_new_http(&self, session: HttpSession, shutdown: &ShutdownWatch)
              -> Option<Stream>;   // 返回 Some 则复用连接进下一请求
      }

        SV: ProxyHttp              ← 业务钩子（见 TODO-68）
```

三层分工：`Service<A>` 协议无关，管 accept / shutdown / 并发限制；`ServerApp` 拿到连接自己做事，keep-alive 循环也在这层；`ProxyHttp` 只写业务语义。duotunnel 现状把这三层揉在 `server/handlers/http.rs` 的 390 行里。

**Fix**:
```rust
pub struct Service<A: ServerApp> {
    name: String,
    listeners: Vec<ListenerSpec>,     // TCP / TLS / UDS 多 endpoint
    app: Arc<A>,
    shutdown: watch::Receiver<()>,
}

pub trait ServerApp: Send + Sync + 'static {
    type Conn: AsyncRead + AsyncWrite + Send + Unpin;
    fn handle(self: Arc<Self>, conn: Self::Conn, peer: SocketAddr)
        -> impl Future<Output = ()> + Send;
}

impl<A: ServerApp> Service<A> {
    pub fn add_tcp(&mut self, addr: SocketAddr);
    pub fn add_tls(&mut self, addr: SocketAddr, tls: TlsConfig);
    pub async fn run(self);  // 统一 accept + spawn + shutdown
}
```

合并 `accept.rs` + `listener.rs` 到 `Service::run_accept_loop`；`server/handlers/{http,tcp,quic}.rs` 和 `client/entry.rs` 都改用 Service。

Shutdown 走 `tokio::sync::watch`，替代散落在 handler 的 `CancellationToken` select。

**为什么好**：
- 单一抽象替代两层（accept_worker / start_tcp_listener）
- 多 endpoint 共享 app：一个 `HttpTunnelApp` 同时 `add_tcp(:80) + add_tls(:443)`
- Shutdown 统一：`watch::Receiver` 广播，accept loop 停接收、in-flight 请求排干，比 `CancellationToken` 散落各处清晰
- 每 endpoint connection filter 一致应用

**Files**:
- 新增 `tunnel-lib/src/service/listening.rs`
- 删除 `tunnel-lib/src/accept.rs` + `tunnel-lib/src/transport/listener.rs`（或留 deprecated wrapper 一版）
- `server/handlers/{http,tcp,quic}.rs`、`client/entry.rs`

**Validation**: CI 通过；graceful shutdown 行为对齐（SIGTERM 后新连接拒绝、老连接排干）。

---

## [TODO-68] ProxyHandler trait + 连接级 Ctx（消灭 god-closure）

**Priority**: Medium | **Status**: TODO
**依赖与影响**:
- 依赖：TODO-67（需要 ServerApp 宿主）
- 被依赖：TODO-69（Ctx 是 sticky selection 的载体）
- 吞并：todo.md 的 TODO-CR4（观测性解耦，随 `logging` 钩子一并落地）
- 两端影响：server 主要受益；client L4 entry 暂不需（是裸 TCP 转发，没有钩子语义），未来 client 做 L7 入口时再接入

**Problem**:
`server/handlers/http.rs:104-334` 两个 handler（TLS + plaintext H2）**总计 200+ 行 god-closure**，职责混杂：
- 路由查找（`lookup_route`）
- TLS 证书生成 / 接受
- `authority` 改写
- 构造 `RoutingInfo`
- 选 QUIC 连接（`select_client_for_group`）
- `forward_h2_request`
- 502 硬编码错误响应
- metrics 散落四处（`http.rs:34,38,40,42`）

加新功能（rate limit、auth 预检）必须改 200 行闭包。三个 `Mutex<HashMap>` ad-hoc 缓存（`first_authority`、`route_cache`、`sender_cache`）散落在 handler 内部——H1 路径又没有，逻辑分叉。

Pingora 的 `ProxyHttp` trait (`pingora-proxy/src/proxy_trait.rs:32-150`) 在请求生命周期的每个阶段都开了钩子：`early_request_filter` / `upstream_peer` / `request_filter` / `response_filter` / `fail_to_connect` / `logging`，带 `CTX` 关联类型贯穿请求。

**Fix**:
```rust
pub trait ProxyHandler: Send + Sync + 'static {
    type Ctx: Send + Default;

    /// 在读请求头之前，给 overload/rate-limit 提前短路的机会
    async fn early_request_filter(&self, ctx: &mut Self::Ctx) -> Result<Option<Response>>;
                                                                    // Some = 短路（overload 503）

    /// 路由：请求头 → upstream peer
    async fn upstream_peer(&self, req: &RequestHead, ctx: &mut Self::Ctx)
        -> Result<Arc<dyn UpstreamPeer>>;

    /// 改写 upstream request（Host 覆盖、header 注入等）
    async fn upstream_request_filter(&self, req: &mut RequestHead, ctx: &mut Self::Ctx)
        -> Result<()>;

    /// 改写返回给 client 的 response
    async fn response_filter(&self, resp: &mut ResponseHead, ctx: &mut Self::Ctx)
        -> Result<()>;

    async fn fail_to_connect(&self, err: &Error, ctx: &mut Self::Ctx)
        -> Option<ResponseHead>;   // None = 框架返回 502

    async fn logging(&self, ctx: &mut Self::Ctx, err: Option<&Error>);
}
```

框架（`HttpTunnelApp: ServerApp`）负责：读请求头 → 钩子串联 → Connector → 转发 → 钩子 → 响应。

duotunnel 映射：
- `overload.rs` backoff → `early_request_filter` 提前返回 503
- 路由规则（`tunnel-store` rules）+ `build_routing_snapshot` → `upstream_peer`
- `protocol/rewrite.rs` 改写、`h2_single_authority` 校验（`http.rs:234-250`）→ `upstream_request_filter`
- Metrics / 访问日志 → `logging`（顺带落地 TODO-CR4）

`server/handlers/http.rs` 390 行 → 拆成 6 个钩子方法，每个 20-30 行。

**为什么好**：
- 删除 `first_authority/route_cache/sender_cache` 三个 Mutex，逻辑收归 `Ctx`
- 职责分离，可单元测试（路由、改写、转发分开测）
- 加新功能有明确切入点，不用再改 200 行闭包
- Metrics 统一走 `logging`，解决 TODO-CR4

**改造后一条 plaintext H2 请求的 tracing**:
```
accept → HttpTunnelApp::handle → HttpInboundSession
  → read_request
  → early_request_filter (overload?)
  → upstream_peer (routing.lookup + registry.select → QuicPeer)
  → upstream_request_filter (authority rewrite)
  → QuicHttpConnector::get_session(peer)    [命中 h2 pool]
  → session.send_request
  → response_filter
  → session.write_response
  → logging(ctx, err=None)
  → [keep-alive] 回到 read_request
```
对比现状 200+ 行闭包，行数从 200+ 降到每钩子 20-30；缓存逻辑收归框架。

**Files**:
- 新增 `tunnel-lib/src/proxy/handler.rs`
- 重写 `server/handlers/http.rs`（预计 390 → ~150 行）
- `tunnel-lib/src/overload.rs` 接 early_request_filter
- `client/proxy.rs` 同步调整（client 端 egress 也有 ProxyHandler 需求）

**Validation**: CI 全绿；stress phase 无回归。

---

## [TODO-69] Sticky selection：连接级缓存 upstream pick ★ 热路径优化（server）

**Priority**: High | **Status**: TODO
**依赖与影响**:
- 依赖：TODO-68（需要 Ctx 承载 `selected` 缓存）
- 被依赖：TODO-70（`select` 返回类型改 `Arc<SelectedConnection>` 配合 fast path）
- 吞并：todo.md 的 TODO-52（ArcSwap 路由快照连接级缓存，本 TODO 完整覆盖）
- 两端影响：**server 主要受益**（per-request → per-conn pick）；client 端是 L4，一个下游 TCP 连接天然只 pick 一次，sticky 已成立，**无需改动**

**Problem**:
`server/registry.rs:88-96` `select_healthy` 每请求调用一次：
```rust
pub fn select_healthy(&self) -> Option<SelectedConnection> {
    let conns = self.snapshot.load();              // (1) ArcSwap load — 一次 atomic
    pick_least_inflight(
        conns.as_slice(),
        |c| c.conn.close_reason().is_none(),       // (2) N 次 atomic load（quinn State 内部）
        |c| c.inflight.load(Ordering::Relaxed),    // (3) N 次 atomic load
    )
    .cloned()                                       // (4) clone SelectedConnection
                                                    //      = Arc<str> + Connection(Arc) + InflightCounter(Arc) = 3 次 fetch_add
}
```

**H2 多路复用恶化**：一条下游 H2 连接 100 个并发 stream → 每 stream 调一遍 `upstream_peer → select_client_for_group`，就是 100 次扫描。

`handle_plaintext_h2_connection` 已经意识到这点，自己维护了三个 `Mutex<HashMap>`（`first_authority`、`route_cache`、`sender_cache`，见 `server/handlers/http.rs:205-296`）做 per-connection 缓存——但这是 handler 内部的 ad-hoc 补丁，H1 和 TLS 路径各有自己的代码，逻辑分叉。

### QUIC connection 现状的 provide / update 路径

先把现状画清楚作为改造前提：

```
[client 启动] ──QUIC handshake──► [server QUIC accept]
                                        │
                                        ▼
                                  Login 消息校验（token）
                                        │   得到 client_id + group_id
                                        ▼
                              registry.register(client_id, group_id, Connection)
                                        │   server/registry.rs:143
                                        ▼
                                  replace_or_register:
                                    clients: DashMap<client_id, ClientInfo>
                                    groups:  DashMap<group_id, Arc<ClientGroup>>
                                             └── ClientGroup::set:
                                                   index.lock()
                                                   snapshot.store(Arc::new(build_snapshot(&index)))
                                                                              ▲
                                                                              │
                                                          RCU：每次 set/remove 重建 Vec
```

更新场景：

| 事件 | 动作 | InflightCounter |
|---|---|---|
| client 首次连 | `register` → 新 entry + 重建 snapshot | 新建 |
| 同 client_id 重连 | `replace_or_register` 覆盖 | 复用 (`registry.rs:63-66`) |
| client 断开 | `unregister` → 删除 + 重建 snapshot | 随 entry 一起删 |
| group 空了 | `remove_if` 清 group | — |

读路径（pick）走 `snapshot.load()`，不锁 `index`；写路径锁 `index` 再重建 snapshot。

### Pingora 怎么做

Pingora 的 `upstream_peer()` (`pingora-proxy/src/proxy_trait.rs:43-47`) 也**每请求调用一次**（不是每连接），但它快是因为：
1. 选择结构无锁：`Backends` 是 `ArcSwap<BTreeSet<Backend>>` + `ArcSwap<HashMap<u64, Health>>`
2. `LoadBalancer` 用 Ketama/P2C **O(1) 或 O(log N)** 选择，不扫全表
3. pick 返回 value object `Box<HttpPeer>`，连接复用在 `HttpConnector` 那层独立做

Pingora 的立场是「选择是业务逻辑，框架不缓存；要缓存业务自己用 `ctx` 带」（`HttpSession::set_user_context`）。duotunnel 正好用得上这个模式。

### 两端对称性分析

Client (`client/conn_pool.rs`) 和 server (`server/registry.rs`) 结构同构，但 pick 调用频率不同：

```
                     server       client
  TCP inbound        per-conn     per-conn     ← 对称
  H1 keep-alive      per-req ⚠️    per-conn
  H2 多路复用         per-req ⚠️    per-conn     ← server 严重恶化
  QUIC 接受流         per-req      —
```

Client 是 L4 entry（裸 TCP 转发），一个下游 TCP 连接只 pick 一次，**sticky 已经天然成立**。所以 TODO-69 **主要给 server**，client 不需要。

**Fix**:
在 `ProxyHandler::upstream_peer` fast path 里用 `Ctx` 缓存选择：
```rust
struct ConnCtx {
    route: Option<RouteTarget>,
    selected: Option<Arc<SelectedConnection>>,
}

async fn upstream_peer(&self, req: &RequestHead, ctx: &mut Self::Ctx)
    -> Result<Arc<dyn UpstreamPeer>>
{
    // fast path：连接级缓存
    if let Some(sel) = &ctx.selected {
        if sel.conn.close_reason().is_none() {
            return Ok(sel.clone());              // 1 次 Arc++，无扫描
        }
        ctx.selected = None;                     // 失效重选
    }
    // slow path：首请求 / selected 失效
    let route = self.routing.lookup(req.host)?;
    let sel = self.registry.select(&route.group_id)?;
    ctx.selected = Some(sel.clone());
    ctx.route = Some(route);
    Ok(sel)
}
```

**效果**：
- H2 一条连接 100 req → 1 次 pick + 99 次 fast path（只 1 次 Arc clone）
- 现状 `handlers/http.rs` 里三个 `Mutex<HashMap>` 可以全删（逻辑收归 ConnCtx）
- H1 keep-alive 自动享受 sticky
- H2 路径每请求的锁次数从 3 降到 0；单次 pick 从 O(N) 降到 O(1)

**权衡 / Failover**:
- selected.conn 死了 `close_reason` 触发重选
- 配合 TODO-65 `RetryType::ReusedOnly`：open_bi 失败按可重试语义换新连接
- **负载均衡精度**：sticky 后新连接进来时选一次、之后整条连接固定——用入口 QPS 分散代替每请求平衡；补偿：select 时仍然挑当前最空的

**Files**:
- `server/handlers/http.rs`（ProxyHandler 实现里）
- `server/registry.rs`（`select` 返回 `Arc<SelectedConnection>`，配合 TODO-70）

**Validation**: 单 H2 连接发 1000 req，确认 `select_healthy` 只被调 1 次（tracing / counter）；kill 选中的 client，后续请求自动 failover。

---

## [TODO-70] Server 端 snapshot 持 Arc<SelectedConnection>（对齐 client）

**Priority**: Medium | **Status**: TODO
**依赖与影响**:
- 依赖：—（独立，但建议和 TODO-69 一起落）
- 被依赖：—
- 两端影响：**仅 server**——client `conn_pool.rs:13` 已经是 `ArcSwap<Vec<Arc<PooledConnection>>>` 形态，server 对齐即可

**Problem**:
`server/registry.rs:39` `snapshot: ArcSwap<Vec<SelectedConnection>>`，pick `.cloned()` 克隆整个 struct → 3 次 Arc++（`Arc<str>` + `quinn::Connection` 是 Arc + `InflightCounter` 是 Arc）。

**现成参照**：`client/conn_pool.rs:13` 已经是 `ArcSwap<Vec<Arc<PooledConnection>>>`，pick clone 只 1 次 Arc++。

```
                     server                           client
  snapshot 类型      Vec<SelectedConnection>          Vec<Arc<PooledConnection>>  ✓
  pick clone 成本    3 次 Arc++                        1 次 Arc++
```

**server 直接对齐 client 设计即可**。这也解释了为什么只有 server 感觉 pick 慢——client 结构本来就好。

**Fix**:
```rust
type ClientIndex = HashMap<ClientId, Arc<SelectedConnection>>;
snapshot: ArcSwap<Vec<Arc<SelectedConnection>>>
```
`ClientGroup::select_healthy` 返回 `Option<Arc<SelectedConnection>>`，更新 `build_snapshot` / `set` / `remove`。

**为什么好**：
- pick 返回 `Arc<SelectedConnection>` 而非值，clone 从 3 次原子操作降到 1 次
- TODO-69 实现后 fast path 已经不走 pick，但 slow path（首次选择 / failover）仍受益
- 代码与 client 对称，维护成本降低

**Files**: `server/registry.rs`

**Validation**: 单元测试覆盖 register / unregister / reconnect / select；flamegraph 确认 Arc clone 成本下降。

---

## [TODO-71] P2C pick 算法（可选，池规模增长后启用）

**Priority**: Low | **Status**: TODO
**依赖与影响**:
- 依赖：—（独立）
- 被依赖：—
- 两端影响：server `registry.rs` 和 client `conn_pool.rs` 的 pick 函数都可切换，但 TODO-69 落地后 server 大部分请求走 fast path 不 pick，优先级进一步降低

**Problem**:
`pick_least_inflight` 是 O(N) 线性扫（`tunnel-lib/src/inflight.rs:28-37`）：
```rust
items.iter().filter(|t| is_healthy(t)).min_by_key(|t| inflight(t))
```
N 小时无感，N>10 开始可观测。

Pingora 的 LoadBalancer 用 Ketama（一致性哈希）或 P2C——选择质量接近 least-loaded，但开销固定。

**Fix**:
新增 `pick_p2c`（Power-of-Two-Choices）：
```rust
pub fn pick_p2c<T>(items: &[T], inflight: impl Fn(&T) -> usize, healthy: impl Fn(&T) -> bool)
    -> Option<&T>
{
    let n = items.len();
    if n <= 2 { return items.iter().find(|t| healthy(t)); }
    let (i, j) = (rand::random::<usize>() % n, rand::random::<usize>() % n);
    let (a, b) = (&items[i], &items[j]);
    match (healthy(a), healthy(b)) {
        (true, true) => if inflight(a) <= inflight(b) { Some(a) } else { Some(b) },
        (true, false) => Some(a),
        (false, true) => Some(b),
        (false, false) => items.iter().find(|t| healthy(t)),  // fallback 扫
    }
}
```

**为什么好**：固定 2 次 atomic load，不随 N 增长；负载均衡质量接近 least-loaded（Mitzenmacher 1996）。

**触发条件**：单 group client 数 > 10 且扫描成为 flamegraph 热点。TODO-69 实现后，大部分请求走 fast path 不 pick，本 TODO 优先级进一步降低。

**Files**: `tunnel-lib/src/inflight.rs`、`server/registry.rs`、`client/conn_pool.rs`

---

## [TODO-72] Client 端小优化（非紧急，随手做）

**Priority**: Low | **Status**: TODO
**依赖与影响**:
- 依赖：TODO-65（retry 逻辑用到 `RetryType::ReusedOnly`）
- 被依赖：—
- 两端影响：**仅 client**（server 侧无对应问题）

**Problem**:
1. **`client/conn_pool.rs:27-34` push 时线性查重**
   ```rust
   if g.iter().any(|c| c.conn.stable_id() == conn.stable_id()) { return; }
   ```
   池大时 O(N)。池规模通常小（<10），现状可接受，不紧急
2. **`client/entry.rs:104-149` 重试循环可能重选同一连接**
   失败后 `inflight` 归还（`InflightGuard::drop`），再 `next_conn()` 可能又命中它——徒劳尝试

**Fix**:
1. `push` 加 `HashSet<u64 stable_id>` 辅助 O(1) 去重
2. 重试带 exclude set：
   ```rust
   let mut tried: SmallVec<[u64; 4]> = SmallVec::new();
   for _ in 0..pool_size.max(1) {
       let conn = pool.next_conn_excluding(&tried)?;
       tried.push(conn.conn.stable_id());
       match open_bi_guarded(&conn.conn, ...).await {
           Ok(o) => return ...,
           Err(e) if !e.should_retry(/* was_reused */ true) => return Err(e),
           Err(_) => continue,
       }
   }
   ```
3. 配合 TODO-65 `RetryType::ReusedOnly`：idle close 换新连接重试，新建失败不重试

**未来多 server 节点的分组**（超出当前范围）：现在 client 的池是扁平的（所有 QUIC 连接等价）。如果 server 端做多实例 HA，client 需要按 server group 分池——届时直接套 server `ClientGroup` 的 `DashMap<GroupId, ...>` 结构。

**Files**: `client/conn_pool.rs`、`client/entry.rs`

---

## [TODO-73] 不要抄 Pingora 的部分（参考避坑）

**Priority**: FYI | **Status**: — (非实施 task，记录决策避免走弯路)
**依赖与影响**: — （决策记录，不涉及代码改动）

调研 Pingora 时发现以下模式**不应引入**，在此记录决策和理由：

1. **Pingora 的 worker / fork / listenfd 架构**
   - 场景：CDN 需要跨进程 fd 传递做热升级
   - 不抄理由：duotunnel 不做 CDN 规模，`tokio::sync::watch` 协作式 shutdown + 重启已够用
   - 替代：TODO-67 的 `ShutdownWatch`

2. **`pingora-cache`（HTTP 缓存）**
   - 场景：CDN 的正反向 cache
   - 不抄理由：tunnel 本身不缓存响应体，业务无此需求

3. **`pingora-ketama`（一致性哈希环）**
   - 场景：后端节点多且有 session affinity / 部分故障时最少重分配
   - 不抄理由：duotunnel 单 group 通常几到几十个 client，TODO-71 的 P2C 足够；一致性哈希环维护成本高

4. **`tinyufo`（S3-FIFO + TinyLFU 缓存）**
   - 用途：Pingora 里仅被 `pingora-memory-cache` 包装使用，给业务缓存 DNS 解析、auth 验证结果等小 key 高 QPS 场景
   - 不抄理由：duotunnel 所有 in-memory 缓存（`ClientRegistry`、handler 内的 Mutex<HashMap> 等）都**不需要淘汰语义**——连接在线就在、一次性连接生命周期结束自然回收
   - 将来何时引入：做 upstream DNS 缓存、auth token 缓存、路由 negative cache 或 TLS 证书缓存且 QPS 极高时才值得引入

5. **`ShutdownWatch` 搭配的 `Fds` 无缝升级（SIGHUP 传 fd）**
   - 场景：零 downtime 升级
   - 不抄理由：overkill，duotunnel 允许短暂连接断开

6. **Pingora 的 `Service` 启动顺序 / 依赖图（`ServiceWithDependents` / `ServiceHandle`）**
   - 场景：多 service 有启动依赖、readiness 协调
   - 不抄理由：duotunnel service 数量少，简单 spawn 即可；需要时再按需引入
