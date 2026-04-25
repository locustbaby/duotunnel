# Pingora-inspired 重构：Task 清单（2026-04-24 修订）

对照 Cloudflare Pingora (`../pingora`) 梳理可直接移植的设计模式，以 TODO 形式拆分实施步骤。每个 TODO 自足——包含背景、现状痛点、Pingora 参照、Fix 设计、为什么好、依赖与两端影响、Files、Validation。

> **本次修订背景**：上一版文档（2026-01 左右）写于 plugin 系统落地前，很多前提已过期。本版已对照当前代码（`server/plugins/*`、`tunnel-lib/src/plugin/*`、`tunnel-lib/src/proxy/core.rs`）重写，并标注了已经部分落地的 TODO。
>
> **2026-04-21 二次核对修正**：TODO-68/69 的上一版判断把 TLS ingress 误写成“每个 H2 stream 都重新 select”。实际 `server/plugins/tls/mod.rs` 在 TLS 连接建立后只 `select_client_for_group` 一次，H1/TCP 也是 per-connection；只有 h2c 因同一连接可承载多 authority，需要 per-route 并发缓存。因此 TODO-68/69 已收敛为 h2c 并发缓存、failover、错误语义问题，不再建议引入会破坏 multihost 的单 `selected` fast path。
>
> **2026-04-24 三次核对修正**：结合当前代码重新检查后，文档原来的推荐顺序 `70 -> 72 -> 65 -> 66 -> 63` 不再合适。真正卡住后续演进的是 `PeerKind + Dyn` 双轨、`anyhow` 黑盒错误、`HttpPeer/H2Peer` 职责混杂，以及 h2c sticky sender 的失效重选语义。`TODO-70` 仍有价值，但已经从“前置主线任务”降为“对齐性/小优化任务”。
>
> **2026-04-24 进度同步**：`TODO-63` 已完成；`TODO-65` 已完成第一阶段；`TODO-66` Phase 1 已落地并接入 live path；`TODO-69` 第一阶段已有部分实现。当前执行链路已从 `UpstreamResolver -> PeerKind` 切到 `UpstreamResolver -> PeerSpec -> connect_peer`；client 侧 MITM 路径不再依赖 `PeerKind::Dyn`；HTTP upstream/H2 upstream 已统一走 `HttpConnector` Phase 1。编译验证已通过：`cargo check -p tunnel-lib -p server -p client`。
>
> **2026-04-26 代码核对修正**：本次对照实现再次确认后，文档中“`TODO-65` = Done”“`TODO-64` = In Progress”的写法过于乐观。实际状态更接近：`TODO-65` 为 **In Progress（Phase 1 Done）**，因为 `ProxyError` 已接入 `open_bi`/h2c/部分 upstream 路径，但 `UpstreamResolver` 等热路径边界仍大量使用 `anyhow`；`TODO-64` 仍是 **TODO**，仓库内尚未引入 `ids.rs`、`ClientId/GroupId/ReuseHash`。`TODO-66` 的 `prefer_h1` 记忆目前只覆盖 `HttpConnector::request()` 的 cleartext request path，`connect()` 入口仍主要按 `spec.protocol` 分派，故继续记为 Phase 1 进行中；`TODO-69` 已有 stale cache 失效和空 body 一次重试，但 cached value 仍未升级到 `SelectedConnectionHandle/Arc<SelectedConnection>`。

## 代码结构现状（速览）

落地的新基础设施（上一版文档写时尚不存在）：

- **Plugin 系统** `tunnel-lib/src/plugin/`：`dispatcher.rs`（6 相位管线）、`ingress.rs`（`IngressProtocolHandler` trait）、`egress.rs`（`LoadBalancer/UpstreamDialer/Resolver` trait）、`service.rs`（`TunnelService`）、`route.rs`（`RouteResolver`）、`ctx.rs`（`PhaseResult/ServerCtx/RouteCtx/Route/...`）、`metrics.rs`（`MetricsSink`）、`module.rs`（`ConnectionModule`）、`registry.rs`（`PluginRegistry`）
- **Ingress 分派器** `tunnel-lib/src/plugin/dispatcher.rs:88-218`：sniff → pre_admission → admission → route → handle → logging
- **UpstreamResolver trait** `tunnel-lib/src/proxy/core.rs:25-37`：`async fn upstream_peer(&self, ctx: &mut Context) -> Result<PeerSpec>` + `connect_peer(...)`
  - server 实现 `server/egress.rs:65` `impl UpstreamResolver for ServerEgressMap`
  - client 实现 `client/app.rs:108` `impl UpstreamResolver for ClientApp`
- **Ingress 插件** `server/plugins/{h1,h2c,tls,tcp_pass,vhost,prometheus}/mod.rs`
- **统一 accept**：`tunnel-lib/src/accept.rs:20-52` `run_accept_worker<H>()`，HTTP/TCP/client entry 都走它；`transport/listener.rs` 主要保留 `build_reuseport_listener` + `RouteTarget` 工具（`start_tcp_listener` 旧抽象仍存在但当前未被调用，后续可清理）
- **server/handlers/http.rs 瘦身到 70 行**（上一版文档说的 390 行 god-closure 已不复存在），逻辑迁到 `server/plugins/h2c/mod.rs`

仍然需要推进的主线（本文档重点）：

- `HttpConnector`（H1/H2 降级记忆，先复用 hyper pool）
- h2c per-route sticky sender cache 的失效重选、failover 与错误语义
- `ClientId/GroupId/ReuseHash` newtype（放到 connector 稳定后再推）
- P2C pick 算法
- 若干 client 侧小优化

## 目录

| TODO | 标题 | Priority | 状态 |
|---|---|---|---|
| [TODO-63](#todo-63-peer-描述符化--先消灭-peerkind--dyn-双轨) | Peer 描述符化 + 先消灭 `PeerKind + Dyn` 双轨 | High | Done |
| [TODO-64](#todo-64-reusehash--clientidgroupid-newtype收尾类型安全) | ReuseHash + ClientId/GroupId newtype（收尾类型安全） | Medium | TODO |
| [TODO-65](#todo-65-热路径结构化错误--先替换-anyhow-黑盒) | 热路径结构化错误 + 先替换 `anyhow` 黑盒 | High | In Progress（Phase 1 Done） |
| [TODO-66](#todo-66-统一-httpconnector--h1h2-降级记忆替代-httppeerh2peer) | 统一 HttpConnector + H1/H2 降级记忆 | High | In Progress |
| [TODO-67](#todo-67-servicea--serverapp-抽象统一-accept--handle) | ~~Service\<A\> + ServerApp 抽象~~ | — | **部分达成，剩余部分降级为 TODO-67b** |
| [TODO-67b](#todo-67b-keep-alive-loop-下沉到-session-层) | keep-alive loop 下沉到 Session 层 | Medium | TODO |
| [TODO-68](#todo-68-ingress-request-lifecycle-收敛不要扩展-upstreamresolver) | Ingress request lifecycle 收敛（不要扩展 UpstreamResolver） | Medium | TODO |
| [TODO-69](#todo-69-h2c-per-route-sticky-cache-失效重选--failover) | h2c per-route sticky cache 失效重选 + failover | Medium | In Progress |
| [TODO-70](#todo-70-server-端-snapshot-持-arcselectedconnection对齐-client) | Server 端 snapshot 持 Arc\<SelectedConnection\>（对齐 client） | Low | TODO |
| [TODO-71](#todo-71-p2c-pick-算法可选池规模增长后启用) | P2C pick 算法（可选，池规模增长后启用） | Low | TODO |
| [TODO-72](#todo-72-client-端小优化非紧急随手做) | Client 端小优化 | Low | TODO |
| [TODO-73](#todo-73-不要抄-pingora-的部分参考避坑) | 不要抄 Pingora 的部分（参考避坑） | FYI | — |

**推荐落地顺序**：63 → 65 → 66 → 69 → 64 → 70 → 72 → 67b → 68 → 71。理由：先把 resolver 输出收敛成“纯描述符”，再补热路径错误语义，随后用最小改动落地 `HttpConnector` Phase 1，并把 h2c 的 sticky sender 补成可失效重选。`newtype` 和 server snapshot 对齐都值得做，但它们不是当前行为修复和性能演进的前置条件。当前进度：`63` 已完成；`65` 第一阶段已完成但未全量替掉热路径边界的 `anyhow`；`66` Phase 1 进行中；`69` 第一阶段已部分实现。每个 TODO 完成后跑 CI stress phase，观察基准变化。

---

## [TODO-63] Peer 描述符化 + 先消灭 `PeerKind + Dyn` 双轨

**Priority**: High | **Status**: Done
**依赖与影响**:
- 依赖：— （主线起点）
- 被依赖：TODO-65（错误映射边界）、TODO-66（connector 输入形态）、TODO-64（后续 `ReuseHash`）
- 两端影响：server egress 和 client egress 都要改（对称）

**Problem**:
`tunnel-lib/src/proxy/peers.rs:7-12` 的 `PeerKind` enum 混用三种分派：
```rust
pub enum PeerKind {
    Tcp(TcpPeer),                  // 不 box, enum dispatch 零开销
    Http(Box<HttpPeer>),           // 多一次堆分配
    H2(Box<H2Peer>),               // 多一次堆分配
    Dyn(Box<dyn UpstreamPeer>),    // vtable —— 实际是 client 的 MitmH2Peer 后门
}

pub trait UpstreamPeer: Send + Sync {
    fn connect_boxed<'a>(&'a self, send: SendStream, recv: RecvStream,
                         initial_data: Option<Bytes>)
        -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}
```
- **分派策略不一致**：同一 trait（`UpstreamPeer`）只给 `Dyn` 用，`Tcp/Http/H2` 各自裸 `connect_inner(self)` 消耗所有权
- **`Dyn` 是"半活的后门"**：`client/app.rs:163-218` 的 `MitmH2Peer` 就靠它接入（上一版文档以为是死代码，实际不是）；但同一概念走两条路径（裸 enum / `dyn`）本身就是坏味道
- **Peer 不是纯描述符**：`HttpPeer { client: HttpsClient, ... }`（`tunnel-lib/src/proxy/http.rs:62-66`）和 `H2Peer { https_client, h2c_client, ... }`（`tunnel-lib/src/proxy/h2.rs:82-87`）把 `hyper` client 放进 peer。`hyper` client clone 本身仍共享内部 pool；真正的问题是 peer 难以承载 `reuse_hash` / protocol preference / `was_reused` 等连接管理语义。

Pingora 的做法 (`pingora-core/src/upstreams/peer.rs:88-306`)：单一 `Peer` trait + 可选 `PeerOptions` bag-of-settings，具体类型（`BasicPeer`、`HttpPeer`）各自实现。**Peer 只是上游描述符（value object），不持连接**；连接复用在 `HttpConnector` 那层独立做（见 TODO-66）。

这里先不直接照搬成 `Arc<dyn UpstreamPeer>` 全面 trait 化。当前代码更稳妥的第一步，是先把 resolver 输出从“可执行 peer”改成“纯描述符”。

**Fix**:
```rust
pub enum PeerSpec {
    Tcp(BasicPeerSpec),
    Http(HttpPeerSpec),
    MitmH2(MitmPeerSpec),
}

pub struct BasicPeerSpec { addr: SocketAddr, tls: Option<TlsOptions> }
pub struct HttpPeerSpec { addr: SocketAddr, host: String, scheme: Scheme }
```

具体动作：
1. 新增 `PeerSpec` / `BasicPeerSpec` / `HttpPeerSpec` / `MitmPeerSpec`，只承载目标地址、协议、host、TLS 等静态信息。
2. `UpstreamResolver::upstream_peer` 先改为返回 `PeerSpec`，不要再返回带执行逻辑的 `PeerKind`。
3. 保留 `MitmH2Peer` 这类特殊路径，但把它从 `PeerKind::Dyn` 后门收回到 `PeerSpec::MitmH2`。
4. `HttpPeer` / `H2Peer` 的“怎么连、怎么 fallback、怎么复用”移到 TODO-66 的 connector。
5. 如果后续确实需要统一 trait，再在 `PeerSpec` 稳定后做第二阶段 trait 化，而不是一上来把全仓库切成 `Arc<dyn Trait>`。

**为什么好**（对比现状）：

| 维度 | 现状 | 新设计 |
|---|---|---|
| 分派一致性 | `PeerKind` + `Dyn` 后门双轨 | 单一 `PeerSpec` 输入模型 |
| MITM 等特殊路径 | 靠 `Dyn(Box<dyn UpstreamPeer>)` 绕进去 | 显式 `PeerSpec::MitmH2` |
| 连接管理语义 | peer 与 connector 职责混杂 | resolver 只产出描述符，connector 负责执行 |

**Files**:
- `tunnel-lib/src/proxy/peers.rs`（重写成 `PeerSpec`）
- `tunnel-lib/src/proxy/core.rs`（`UpstreamResolver::upstream_peer` 返回 `PeerSpec`）
- `tunnel-lib/src/proxy/http.rs` / `h2.rs` / `tcp.rs`（清掉 peer 自带的执行职责）
- 调用方：`server/egress.rs`、`client/app.rs`（含 MitmH2Peer）

**实际进展（已完成）**:
1. 已引入 `PeerSpec` / `BasicPeerSpec` / `HttpPeerSpec` / `MitmPeerSpec`。
2. `UpstreamResolver` 已改为返回 `PeerSpec`，并新增 `connect_peer(...)` 执行阶段。
3. client 侧 MITM 路径已切到显式 `PeerSpec::MitmH2`，不再通过 `PeerKind::Dyn` 后门注入。
4. `PeerKind::Dyn` / `UpstreamPeer` 活跃路径已移除；`PeerKind` 当前仅作为过渡执行枚举留存给未启用的 dialer seam。

**Validation**: `cargo check -p tunnel-lib -p server -p client` 已通过；静态代码路径已切换完毕。

---

## [TODO-64] ReuseHash + ClientId/GroupId newtype（收尾类型安全）

**Priority**: Medium | **Status**: TODO
**依赖与影响**:
- 依赖：TODO-63（`PeerSpec` 稳定）、TODO-66（connector 需要稳定 key 时一并接入）
- 被依赖：—
- 两端影响：server `registry.rs` + client `conn_pool.rs` 的 `stable_id/client_id` 都要改 newtype

**Problem**:
- `server/registry.rs:10,33` 全链路裸 `String client_id / group_id`（`ClientIndex = HashMap<String, (Connection, InflightCounter)>`、`DashMap<String, Arc<ClientGroup>>`）：typo 编译期不报错
- `tunnel-store/src/rules.rs`、`tunnel-lib/src/models/msg.rs` 同样问题
- h2c plugin 的 `route_cache` key 也是 `String`（`server/plugins/h2c/mod.rs:56`）
- 目前 `RouteTarget` / plugin `Route` 已经摆脱匿名 tuple，但 registry、wire model、store 层还大量裸 `String`
- 对 connector 来说，`ReuseHash` 仍然有价值，但它属于“connector 稳定后的收尾工程”，不是当前修复行为和落地 `HttpConnector` 的前置条件

Pingora 的 `ConnectionPool<S>` (`pingora-pool/src/connection.rs:29-82`) 用 `u64 GroupKey` 作 key，由 `Peer::reuse_hash()` 生成（`pingora-core/src/upstreams/peer.rs:98-101`）：address + scheme + TLS cert serial + ALPN 全部 hash 进去，既保证无冲突又 O(1) 查找。

**Fix**:
新增 `tunnel-lib/src/ids.rs`：
```rust
pub struct ClientId(Arc<str>);     // zero-cost clone
pub struct GroupId(Arc<str>);
pub struct ProxyName(Arc<str>);
pub struct ReuseHash(u64);
```

`ReuseHash` 的计算可以先做成 `PeerSpec` 辅助方法（示意；可先用 `std::collections::hash_map::DefaultHasher`，除非 profile 证明需要引入 `ahash`）：
```rust
impl PeerSpec {
    fn reuse_hash(&self) -> ReuseHash {
        let mut h = DefaultHasher::new();
        match self {
            PeerSpec::Tcp(spec) => {
                spec.addr.hash(&mut h);
                spec.tls.hash(&mut h);
            }
            PeerSpec::Http(spec) => {
                spec.addr.hash(&mut h);
                spec.host.hash(&mut h);
                spec.scheme.hash(&mut h);
            }
            PeerSpec::MitmH2(spec) => {
                spec.addr.hash(&mut h);
                spec.host.hash(&mut h);
            }
        }
        ReuseHash(h.finish())
    }
}
```

分阶段替换：
1. 先在内存热路径引入 `ClientId` / `GroupId` / `ProxyName` / `ReuseHash`，覆盖 `server/registry.rs`、`client/conn_pool.rs`、`server/plugins/h2c/mod.rs`。
2. wire/config 边界暂保留 `String`：`tunnel-lib/src/models/msg.rs` 的 `RoutingInfo`、`LoginResp.client_group` 先不改 wire 形态，避免 rkyv 兼容性和旧 client/server 互通问题；在 encode/decode 边界转换成 newtype。
3. `tunnel-store/src/rules.rs` 等存储层按 schema 兼容性单独迁移，不和热路径重构绑在同一个 PR。

**为什么好**：
- `GroupId ≠ ClientId ≠ ProxyName`，typo 编译期拦截
- 显式整数 key 方便后续自定义 connector / protocol preference，避免不同 ALPN / TLS 设置误共享同一连接管理状态
- `Arc<str>` 零成本共享，比 `String::clone()` 快

**Files**:
- 新增 `tunnel-lib/src/ids.rs`
- `server/registry.rs`、`client/conn_pool.rs`、`tunnel-store/*`
- `tunnel-lib/src/plugin/ctx.rs`
- `tunnel-lib/src/models/msg.rs`（只做边界转换，暂不改 wire schema）
- `server/plugins/h2c/mod.rs`（route_cache/sender_cache key）

**Validation**: 编译通过 + 单元测试；rkyv 兼容测试覆盖旧字段形态；热路径 flamegraph 确认 String clone/hash 成本是否仍值得继续收敛。

**代码核对（2026-04-26）**:
1. 仓库内尚无 `tunnel-lib/src/ids.rs`。
2. `server/registry.rs`、`client/conn_pool.rs`、`server/plugins/h2c/mod.rs` 仍以裸 `String` 和现有结构为主，未见 `ClientId / GroupId / ReuseHash` 接入。
3. 因此本项仍应视为未开始，而不是进行中。

---

## [TODO-65] 热路径结构化错误 + 先替换 `anyhow` 黑盒

**Priority**: High | **Status**: In Progress（Phase 1 Done）
**依赖与影响**:
- 依赖：—（可独立于 63/64 推进，但最好在 63 之后做，边界更清晰）
- 被依赖：TODO-66（Connector 的 `ReusedOnly` 语义）、TODO-72（client retry 可使用结构化 QUIC open 错误，但不直接套 `ReusedOnly`）
- 两端影响：server metrics + client `entry.rs` retry 都受益

**Problem**:
- `anyhow::Error` 全局使用（`tunnel-lib/src/ctld_proto.rs`、`tunnel-store/src/traits.rs`、`tunnel-lib/src/open_bi.rs` 等），丢失语义
- 无法区分「upstream 问题 / 下游问题 / 内部 bug」——告警等级一刀切
- 无法区分「可重试 / 不可重试」
- **关键 bug 类**：H1/H2 长连接服务端 idle 超时 close 是**正常现象**；复用连接首个请求失败应该重试新连接，新建失败才不重试。现状 `HttpPeer::connect_inner` 把错误一股脑上抛，用户看到偶发 502
- `open_bi_guarded` 的错误路径虽然用 `OpenBiOutcome` 枚举回调观测了（`tunnel-lib/src/open_bi.rs`），但返回给调用方的仍是 `anyhow::Result<OpenedStream>`，调用方只能从字符串推测是超时还是握手失败
- `client/entry.rs:106-159` 的重试循环看到错误只能 `warn!` + 继续，无法分辨 QUIC open 的 fatal / transient 类型
- `server/plugins/h2c/mod.rs:205-214` 上游失败时硬编码 502，没有按错误类型分级

Pingora 的 `Error` (`pingora-error/src/lib.rs:31-87`)：三元组 `{ etype, esource, retry }`，`RetryType::ReusedOnly` 变体是长连接代理的关键语义。

这里不建议一口气把全仓库 `anyhow` 替掉。先把真正的热路径和决策边界替掉，避免大范围机械改动。

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

Phase 1 先替换 3 条热路径：
1. `tunnel-lib/src/open_bi.rs`
2. HTTP upstream request（`tunnel-lib/src/proxy/http.rs` / `h2.rs`，后续接入 TODO-66）
3. `server/plugins/h2c/mod.rs` 的 ingress fail response

`ctld_proto.rs`、`tunnel-store/src/traits.rs` 这类外围边界先不强推，等热路径稳定后再扩。

应用 `ReusedOnly` 语义：`HttpConnector` 复用 HTTP upstream 连接失败 → 标记 `ReusedOnly` → 调用方只在 `was_reused=true` 时重试新建。`client/entry.rs` 则使用专门的 `QuicOpenTimeout / QuicConnectionLost / QuicStreamLimit` 分类，不直接套 `ReusedOnly`。

Metrics 增加 error label：`error_total{type="ConnectTimeout", source="upstream"}`，取代现状 `request_completed{status="error"}` 黑盒。

**为什么好**：
- **`ReusedOnly`** 是长连接代理的必备语义——解决一个真实 bug 类（用户看到偶发 502 其实都是 idle close）
- **`source`** 让告警分级：Downstream warn，Upstream error，Internal 直接 sentry
- **`etype`** 让 metrics 打 label，观测性大幅提升

**Files**:
- 新增 `tunnel-lib/src/error.rs`
- `tunnel-lib/src/open_bi.rs`
- `tunnel-lib/src/proxy/http.rs`、`h2.rs`
- `server/plugins/h2c/mod.rs`（上游失败分级）
- `server/handlers/metrics.rs` / `client` 对应位置
- 后续扩展：`tunnel-lib/src/ctld_proto.rs`、`tunnel-store/src/traits.rs`

**实际进展（已完成第一阶段）**:
1. 已新增 `tunnel-lib/src/error.rs`，提供最小 `ProxyError / ErrorKind / ErrorSource / RetryType`。
2. `open_bi_guarded` 已改为返回 `ProxyError`，至少能区分 timeout 和 connection error。
3. H1/H2 upstream 失败日志已改为结构化错误口径。
4. h2c ingress 的 `400 / 404 / 421 / 502 / 503` 已统一走 `ProxyError -> error_response`。
5. 但 `tunnel-lib/src/proxy/core.rs` 的 `UpstreamResolver` 以及不少外围路径仍使用 `anyhow::Result`，还不能算“热路径结构化错误”整体完成。
6. `client/entry.rs` 目前虽然能拿到结构化 `ProxyError`，重试循环仍基本是统一 warn + 换连接，尚未按 `QuicOpenTimeout / QuicOpenConnection / stream-limit` 等更细语义分叉。

**Validation**: `cargo check -p tunnel-lib -p server -p client` 已通过。行为验证仍放在 `TODO-66/69` 后继续补充重试与 failover 测试。

---

## [TODO-66] 统一 HttpConnector + H1/H2 降级记忆（替代 HttpPeer/H2Peer）

**Priority**: High | **Status**: In Progress
**依赖与影响**:
- 依赖：TODO-63（`PeerSpec` 作为统一输入）、TODO-65（结构化错误）
- 被依赖：—
- 吞并：todo.md 的 TODO-62（per-peer 协议自适应直接落地）、TODO-CR-NEW-C（合并 TcpPeer 和 TlsTcpPeer）
- 两端影响：server egress + client egress upstream 都改用 Connector

**Problem**:
- `tunnel-lib/src/proxy/http.rs:62-131` H1 keep-alive loop 和 `h2.rs:82-118` H2 的 `serve_h2_forward` 走不同路径；`HttpPeer { client: HttpsClient }` 和 `H2Peer { https_client, h2c_client }` 字段重复
- `HttpsClient` 已经启用 HTTP/1 + HTTP/2（`create_https_client_with` 里 `enable_http1()` + `enable_http2()`），TLS ALPN 场景可由 hyper/rustls 协商；当前更准确的问题是 duotunnel 没有显式记录 per-peer preferred protocol，也没有把 downstream protocol 与 upstream protocol 解耦成稳定策略。
- 无协议偏好记忆：`server/egress.rs:109-117` 上层按 `context.protocol` 分派到 `HttpPeer`（H1）或 `H2Peer`（H2）；如果 upstream 不支持对应协议，容易直接 502，且不会记住该 peer 的失败结果。
- todo.md TODO-62 明确提出要做运行时探测+记忆，但没有方案

Pingora 的 `Connector<C>` (`pingora-core/src/connectors/http/mod.rs:30-154`) **共享一个 H1 和 H2 pool**，自动 ALPN 协商，并**在全局 `PreferredHttpVersion` map 里记忆 H1 偏好** (`pingora-core/src/connectors/mod.rs:346-373`)——H2 握手失败过的 peer，下次直接走 H1。

**Fix**:
明确分两阶段做，但当前只做 Phase 1。不要因为文档里写了 Phase 2，就提前自研 session pool。

### Phase 1：保留 hyper client，增加 per-peer 协议偏好

新增 `tunnel-lib/src/proxy/http_connector.rs`，包装现有 `HttpsClient` / `H2cClient`：
```rust
pub struct HttpConnector {
    https_client: HttpsClient,      // TLS: hyper + rustls ALPN
    h2c_client: H2cClient,          // cleartext H2 prior-knowledge
    prefer_h1: DashMap<ReuseHash, Instant>,
}

impl HttpConnector {
    pub async fn request(&self, peer: &HttpPeerSpec, req: Request<BoxBody>) -> Result<Response, Error> {
        let key = peer.reuse_hash();

        if peer.is_cleartext_h2c() && !self.prefer_h1.contains_key(&key) {
            match self.h2c_client.request(req).await {
                Ok(resp) => return Ok(resp),
                Err(e) if is_h2c_protocol_mismatch(&e) => {
                    self.prefer_h1.insert(key, Instant::now());
                    // rebuild request if body is replayable; otherwise return structured error
                }
                Err(e) => return Err(Error::from_hyper(e)),
            }
        }

        match self.https_client.request(req).await {
            Ok(resp) => Ok(resp),
            Err(e) if is_reused_idle_close(&e) => Err(Error::reused_only(e)),
            Err(e) => Err(Error::from_hyper(e)),
        }
    }
}
```

关键点：
- TLS `https://` 先复用 hyper/rustls ALPN，不手动拆 pool。
- cleartext `http://` 的 H2c 没有 ALPN，只能基于配置或首次失败记忆；失败 fallback 只有在请求 body 可重放时才能自动重试。
- `prefer_h1` 必须有 TTL / 配置 reload 清理，避免 upstream 升级 H2 后永远停在 H1。
- `RetryType::ReusedOnly` 只用于“复用连接首请求失败”这一类错误，不泛化到所有 upstream 失败。

### Phase 2：只有 profile 证明 hyper pool 不够时，才考虑自研 session pool

如果后续 flamegraph 证明 hyper pool key / reuse 行为成为瓶颈，再引入 `H1Session` / `H2Session`：
```rust
pub struct HttpSessionPool {
    h1_pool: Pool<H1Session>,
    h2_pool: Pool<H2Session>,
    prefer_h1: DashMap<ReuseHash, Instant>,
}
```

Phase 2 必须先有完整验证：TLS ALPN、H2 flow-control、idle close retry、body replayability、gRPC streaming、WS 不回归。

keep-alive loop 收归到 `H1Session::run_loop` 只属于 Phase 2；Phase 1 仍保留 `HttpPeer::connect_inner` 内的 keep-alive loop，但把错误转换成结构化 `Error`。

合并 `TcpPeer` 和其 TLS 变体成 `BasicPeer` 是 TODO-63 的一部分；它现在已经是 `TcpPeer { tls: Option<TlsConfig> }` 形态，后续更多是重命名/PeerSpec 化，不是紧急重写。

**为什么好**：
- **单一策略入口**：`HttpConnector::request` 统一处理 protocol preference、fallback、错误分类；先复用 hyper 的连接池，避免一次性重写过多底层行为
- **命中 TODO-62**：`prefer_h1` map 就是「运行时探测+记忆」
- **ALPN 自适应**：TLS 场景交给 hyper/rustls 协商；H2c 场景用显式策略和失败记忆补齐
- **风险可控**：Phase 1 行为面小，Phase 2 只有证据充分时再做

**Files**:
- 新增 `tunnel-lib/src/proxy/http_connector.rs`
- `tunnel-lib/src/proxy/http.rs` 和 `h2.rs` 先改为调用 connector / 结构化错误，不立即删除全部 session 代码
- `server/egress.rs`、`client/app.rs` 改用 connector
- `ReuseHash` 如有需要可在这一阶段后半段并入，不必阻塞 Phase 1

**实际进展（Phase 1 已落地）**:
1. 已新增 `tunnel-lib/src/proxy/http_connector.rs`，统一封装 `HttpsClient + H2cClient + prefer_h1 TTL cache`。
2. `server/egress.rs` 与 `client/app.rs` 已改为持有共享 `HttpConnector`，`PeerSpec::Http` 不再直接拼 `HttpPeer/H2Peer`。
3. `tunnel-lib/src/proxy/h2.rs` 已改为通过 connector 发 upstream request，H2 路径的协议选择与错误口径已收口。
4. client 侧 MITM H2 也已切到 `HttpConnector::serve_h2(...)`，不再通过 `PeerKind::Dyn` 走特殊执行后门。
5. 当前 `prefer_h1` 记忆只覆盖 `HttpConnector::request()` 的 cleartext request path；`connect()` 入口仍主要按 `HttpPeerSpec.protocol` 分派到 `connect_h1/connect_h2`，因此“统一策略入口”只完成了第一阶段的一部分。
6. H1 keep-alive loop 仍留在 `HttpPeer::connect_inner`，尚未下沉到 session 层；这一尾巴已拆到 TODO-67b。

**Validation**: 编译验证已通过：`cargo check -p tunnel-lib -p server -p client`。后续仍需补 upstream H2-only、H1-only、H2c-only、H1 cleartext 行为验证，以及 `prefer_h1` 命中/miss/TTL 过期测试。

---

## [TODO-67] ~~Service\<A\> + ServerApp 抽象~~（统一 accept → handle）

**Status**: **部分达成** —— 不再作为独立 TODO，剩余部分降级为 TODO-67b。

**历史动机**：消灭「`accept.rs::run_accept_worker` 和 `transport/listener.rs::start_tcp_listener` 两套重叠 accept 抽象」；handler 的「每连接一 task」契约散乱；shutdown 协作散乱。

**现在的状态**：
- ✅ **accept 统一**：`tunnel-lib/src/accept.rs:20-52` `run_accept_worker<H>()` 已经是实际使用的 accept 抽象，HTTP handler（`server/handlers/http.rs`）、TCP handler（`server/handlers/tcp.rs`）、client entry（`client/entry.rs`）都走它。`transport/listener.rs::start_tcp_listener` 旧抽象仍在但当前未被调用，可后续删除。
- ✅ **每连接一 task 契约**：`run_accept_worker` 的回调里 `tokio::task::spawn`，各 handler 不再自己写 accept 循环
- ✅ **「最小实现」抽象**：`tunnel-lib/src/plugin/ingress.rs:86` 的 `IngressProtocolHandler` trait + `tunnel-lib/src/plugin/dispatcher.rs:88` 的 `IngressDispatcher` 已经是 Pingora `ServerApp` 的等价物（一条连接怎么处理、6 相位钩子齐备）
- 🟡 **keep-alive loop 没下沉**：H1 的 `HttpPeer::connect_inner`（`tunnel-lib/src/proxy/http.rs:82-127`）还在 peer 层做 keep-alive；Pingora 是在 `ServerApp` 层做。这块拆到 TODO-67b 独立推进（也是 TODO-66 的副产品）
- ⏸️ **多-endpoint 共享 app**（`add_tcp(:80) + add_tls(:443)`）：duotunnel 当前没这个需求，暂不实现
- ⏸️ **shutdown 用 `tokio::sync::watch` 统一**：现状 `CancellationToken` 各 handler 自己 select，实际工作良好；Pingora 式 watch 是偏好问题而非必需，暂不换

结论：主要目标已经通过 plugin 系统落地，剩下的小尾巴进 TODO-67b。

---

## [TODO-67b] keep-alive loop 下沉到 Session 层

**Priority**: Medium | **Status**: TODO
**依赖与影响**:
- 依赖：TODO-66（`H1Session`/`H2Session` 概念）
- 被依赖：—
- 两端影响：server + client 的 upstream 都受益

**Problem**:
`tunnel-lib/src/proxy/http.rs:82-127` 的 H1 keep-alive loop 写在 `HttpPeer::connect_inner` 里——和 peer（上游描述符）职责耦合。新连接/复用连接的判定（`was_reused`）无处记录，TODO-65 的 `RetryType::ReusedOnly` 无处使用。

**Fix**:
随 TODO-66 落地 `H1Session::run_loop` 时一并完成：
```rust
impl H1Session {
    pub fn was_reused(&self) -> bool { self.reused }
    pub async fn run_loop(&mut self, downstream: &mut BiStream) -> Result<(), Error> {
        loop {
            match self.next_request(downstream).await {
                Ok(()) => continue,                          // keep-alive 继续
                Err(e) if e.retry == RetryType::ReusedOnly && self.reused => {
                    return Err(e);   // 交给调用方换新连接
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

**Files**: `tunnel-lib/src/proxy/h1_session.rs`（随 TODO-66 一起）

**Validation**: idle close 注入测试，确认复用连接失败走新建路径而非 502。

---

## [TODO-68] Ingress request lifecycle 收敛（不要扩展 UpstreamResolver）

**Priority**: Medium | **Status**: TODO
**依赖与影响**:
- 依赖：TODO-65（结构化错误用于统一 fail response）
- 被依赖：TODO-69（h2c sender cache 的错误分类 / retry 复用）
- 两端影响：server ingress；client L4 entry 暂不需要

**Problem**:
上一版提议把 `tunnel-lib/src/proxy/core.rs:23-28` 的 `UpstreamResolver` 扩展成 Pingora 风格 `ProxyHandler`。二次核对后这个方向不合适：

- `UpstreamResolver` 属于 egress / stream proxy 抽象，当前由 `server/egress.rs` 和 `client/app.rs` 实现；server ingress 已经有 `IngressDispatcher`（sniff → admission → route → handle → logging）和 `IngressProtocolHandler`。
- 把 ingress request lifecycle 塞回 `proxy/core.rs` 会重新耦合 ingress plugin 与 egress core。
- h2c 的 request future 是并发执行的，不能用普通 per-connection `&mut Ctx` / `HashMap` “天然无锁”替代当前 `Arc<Mutex<...>>`。同一 h2c 连接上多个 stream 会同时访问 route/sender cache。
- 真实痛点集中在 h2c：`first_authority`、`route_cache`、`sender_cache` 是 handler 内部 ad-hoc 状态；上游失败统一 502；错误分类、retry、failover 与 TLS/H1 路径不一致。

**Fix**:
保留 `UpstreamResolver`，只在 ingress 层增加 request-level helper / state，不新增跨层 `ProxyHandler`。

建议新增或内聚到 `server/plugins/h2c/`：
```rust
struct H2cConnState {
    first_authority: Mutex<Option<String>>,
    route_cache: DashMap<String, Option<RouteTarget>>,
    sender_cache: DashMap<RouteTarget, Arc<CachedSender>>,
}

struct CachedSender {
    selected: SelectedConnectionHandle,  // TODO-70 后直接对齐成 Arc<SelectedConnection>
    sender: tunnel_lib::H2Sender,
}

impl H2cConnState {
    async fn route_request(&self, host: &str, ctx: &RouteCtx) -> Result<Option<RouteTarget>, Error>;
    async fn sender_for_route(&self, route: &RouteTarget, registry: &ClientRegistry)
        -> Result<Arc<CachedSender>, Error>;
    fn invalidate_sender(&self, route: &RouteTarget);
}
```

公共逻辑只提取成小函数 / 小模块：
- authority / Host 提取、`single_authority` 421 校验
- route resolve + negative cache
- route target → `SelectedConnection` → `H2Sender`
- structured error → HTTP response（404 / 421 / 502 / 503）
- per-request logging / metrics

**为什么好**:
- 不碰 `proxy/core.rs`，避免 egress/client 被 ingress 生命周期牵连。
- h2c 并发状态显式用并发容器或短临界区锁表达，避免伪 “无锁 Ctx”。
- H1/TLS/TCP 现有 per-connection selection 保持不变；只把真正复杂的 h2c 多 authority 路径收敛。

**Files**:
- `server/plugins/h2c/mod.rs`（拆出 `H2cConnState` / error response helper）
- 可选新增 `server/plugins/h2c/state.rs`
- `server/plugins/tls/mod.rs` 只复用 error response helper，不改 select 语义
- `tunnel-lib/src/proxy/core.rs` 不改

**Validation**: h2c multihost 同一连接并发请求不串 route；`h2_single_authority=true` 时第二 authority 返回 421；route miss 404；upstream/client 不可用 502/503；stress phase 没有新增锁等待热点。

---

## [TODO-69] h2c per-route sticky cache 失效重选 + failover

**Priority**: Medium | **Status**: In Progress
**依赖与影响**:
- 依赖：TODO-65（结构化错误）；`TODO-70` 可选，不再是前置
- 被依赖：—
- 两端影响：server h2c ingress；H1/TLS/TCP 已是 per-connection selection，不需要套单 `selected` fast path

**Problem**:
`server/plugins/h2c/mod.rs` 已经有 per-route sticky sender cache：
```rust
let mut guard = sender_cache.lock();
if !guard.contains_key(&route_target) {
    if let Some(selected) = registry.select_client_for_group(&conn_group_id) {
        guard.insert(route_target.clone(), (selected.conn, tunnel_lib::new_h2_sender()));
    }
}
```

这说明 h2c 的真实需求不是“每连接一个 selected”，而是：
- 同一 h2c connection 可能承载多个 authority / route，cache key 必须是 `RouteTarget`。
- cached value 当前只保存 `quinn::Connection`，无法携带 `inflight`、`conn_id`、错误分类上下文；第一步至少要升到显式 `SelectedConnectionHandle`，TODO-70 再进一步对齐成 `Arc<SelectedConnection>`。
- cache hit 时没有先检查 `selected.conn.close_reason()`；client 已断开时会等到 `forward_h2_request` 失败才删除。
- 失败路径统一删除 sender 并返回 502；没有“可重试一次并重选 client”的语义，也没有按错误类型区分 502/503。
- `sender_cache: Arc<Mutex<HashMap<...>>>` 在并发 H2 stream 下会形成短临界区；这不是主矛盾。当前真正的问题是 cached sender 的 stale 检测、错误分类和失效重选语义。

### QUIC connection 现状的 provide / update 路径（对照用）

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
                                    clients: DashMap<String, ClientInfo>
                                    groups:  DashMap<String, Arc<ClientGroup>>
                                             └── ClientGroup::set:
                                                   index.lock()
                                                   snapshot.store(Arc::new(build_snapshot(&index)))
                                                                              ▲
                                                          RCU：每次 set/remove 重建 Vec
```

读路径（pick）走 `snapshot.load()`，不锁 `index`；写路径锁 `index` 再重建 snapshot。

### 两端现状

```
                     server       client
  TCP inbound        per-conn     per-conn
  H1 keep-alive      per-conn     per-conn
  TLS/H2 ingress     per-conn     per-conn
  h2c multihost      per-route    per-conn
```

所以 TODO-69 只处理 h2c per-route cache。不要引入单 `ctx.selected` fast path；它会把同一 h2c 连接上的不同 authority 错误复用到第一个 route，破坏 multihost。

**Fix**:
在 h2c 中把 cache value 改成 `Arc<CachedSender>`，并加入失效检查和有限重试：
```rust
struct CachedSender {
    selected: SelectedConnectionHandle,
    sender: tunnel_lib::H2Sender,
}

async fn forward_with_cached_sender(
    state: &H2cConnState,
    route: RouteTarget,
    req: Request<Incoming>,
) -> Result<Response<BoxBody>, Error> {
    for attempt in 0..2 {
        let cached = state.sender_for_route(&route).await?;
        if cached.selected.conn.close_reason().is_some() {
            state.invalidate_sender(&route);
            continue;
        }

        match forward_h2_request(&cached.selected.conn, &cached.sender, routing_info(&route), req).await {
            Ok(resp) => return Ok(resp),
            Err(e) if attempt == 0 && is_retryable_sender_error(&e) => {
                state.invalidate_sender(&route);
                continue;
            }
            Err(e) => return Err(Error::from_forward(e)),
        }
    }
    Err(Error::no_client(route.group_id))
}
```

并发策略：
- route cache 可用 `DashMap<String, Option<RouteTarget>>`；negative route cache 只在连接生命周期内有效。
- sender cache 可用 `DashMap<RouteTarget, Arc<CachedSender>>`；如果发现 double build 成本可观，再加 per-route rebuild mutex。
- 所有 cache key 保留 `RouteTarget`，不要降级成单 connection selected。
- `single_authority=true` 时可以额外 fast path 固定第一 authority，但仍应显式依赖该配置。

**效果**:
- h2c multihost 正确性保留。
- healthy cached sender 走 O(1) lookup，不扫描 registry。
- cached client 死亡或 sender 错误时自动失效，最多重试一次新 selected。
- 结构化错误让 404 / 421 / 502 / 503 可观测。

**Files**:
- `server/plugins/h2c/mod.rs`
- `server/registry.rs`（TODO-70 后可直接返回 `Arc<SelectedConnection>`）
- `server/plugins/h2c/state.rs`（可选）

**实际进展（第一阶段已落地）**:
1. `sender_cache` 已从裸 `(Connection, H2Sender)` 升到显式 `SenderEntry { conn_id, conn, sender }`，开始记录 `quinn::Connection::stable_id()`。
2. cache hit 前已先检查 `close_reason()`；如果 cached connection 已关闭，会先失效再重选，不再等到 `forward_h2_request` 报错后才发现 stale。
3. 失败路径现在按 `conn_id` 条件失效旧条目，避免并发请求把新建的 sender entry 误删。
4. 当前阶段只对“空 body / `is_end_stream()`”请求做一次安全重试：旧 sender 失效后会重建 sender 并重发一次。这能覆盖 benchmark 中的 GET 场景，但不会对不可重放 body 做冒险重试。
5. 非空 body 请求仍然只做条件失效，不做立即重放；这一步必须建立在显式 replayability 语义之上，不能为了表面 failover 牺牲正确性。
6. h2c plugin 已补最小观测口径：`duotunnel_h2c_errors_total{status,type,source}` 与 `duotunnel_h2c_retry_total{result}`，这样 route miss / no client / forward error / retry success 都能单独观察，不必再只看总 502 数。
7. CI 辅助：`ci-helpers/run-bench-case.sh` 的 frp 路径加了端口清理（`pkill` + `reset-failed`）、基于 `is-failed` 的就绪等待、启动失败 diag dump 与 3 次重试，避免 bench 偶发被残留 frpc/frps 卡住。
8. 但 cache value 目前仍是 `SenderEntry { conn_id, conn, sender }`，还不是文档目标里的 `SelectedConnectionHandle / Arc<SelectedConnection>`；本项因此仍处于“第一阶段部分完成”。

**Validation**: `cargo check -p tunnel-lib -p server -p client` 已通过。后续仍需补单 h2c connection 并发 2 个 Host、不串 route；kill cached client 后下一请求重选；以及错误 metrics/type-source 行为验证。

---

## [TODO-70] Server 端 snapshot 持 Arc<SelectedConnection>（对齐 client）

**Priority**: Low | **Status**: TODO
**依赖与影响**:
- 依赖：—（独立）
- 被依赖：—
- 两端影响：**仅 server**——client `conn_pool.rs:13` 已经是 `ArcSwap<Vec<Arc<PooledConnection>>>` 形态，server 对齐即可

**Problem**:
这个任务仍然属实，但优先级已经下降。当前 server 侧并不是“完全落后”，而是已经做到了 `ArcSwap<Vec<SelectedConnection>> + inflight`，只是还没和 client 一样走 `Vec<Arc<_>>`。

`server/registry.rs:39` `snapshot: ArcSwap<Vec<SelectedConnection>>`，pick `.cloned()` 克隆整个 struct → 3 次 Arc++：
- `conn_id: Arc<str>`
- `conn: quinn::Connection`（内部 Arc）
- `inflight: InflightCounter`（= `Arc<CachePadded<AtomicUsize>>`）

**现成参照**：`client/conn_pool.rs:13` 已经是 `ArcSwap<Vec<Arc<PooledConnection>>>`，pick clone 只 1 次 Arc++。

```
                     server                           client
  snapshot 类型      Vec<SelectedConnection>          Vec<Arc<PooledConnection>>  ✓
  pick clone 成本    3 次 Arc++                        1 次 Arc++
```

**server 直接对齐 client 设计即可**。但这是对齐性/小优化任务，不再是后续 connector / h2c 演进的前置条件。

**Fix**:
```rust
type ClientIndex = HashMap<ClientId, Arc<SelectedConnection>>;
snapshot: ArcSwap<Vec<Arc<SelectedConnection>>>
```
`ClientGroup::select_healthy` 返回 `Option<Arc<SelectedConnection>>`，更新 `build_snapshot` / `set` / `remove`。

注意 h2c plugin 的 `sender_cache` 目前 value 里包含 `quinn::Connection`——TODO-69 改成 `CachedSender { selected: Arc<SelectedConnection>, sender }`，保持引用链一致。

**为什么好**：
- pick 返回 `Arc<SelectedConnection>` 而非值，clone 从 3 次原子操作降到 1 次
- TODO-69 实现后 h2c cache miss / failover 仍受益；H1/TLS/TCP 首次选择也受益
- 代码与 client 对称，维护成本降低

**Files**: `server/registry.rs`、`server/plugins/h2c/mod.rs`（sender_cache value）

**Validation**: 单元测试覆盖 register / unregister / reconnect / select；flamegraph 确认 Arc clone 成本下降。

---

## [TODO-71] P2C pick 算法（可选，池规模增长后启用）

**Priority**: Low | **Status**: TODO
**依赖与影响**:
- 依赖：—（独立）
- 被依赖：—
- 两端影响：server `registry.rs` 和 client `conn_pool.rs` 的 pick 函数都可切换；TODO-69 只减少 h2c cache hit 的 pick，不改变 H1/TLS/TCP 首次选择

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
    let healthy_idx: smallvec::SmallVec<[usize; 32]> = items.iter()
        .enumerate()
        .filter_map(|(i, t)| healthy(t).then_some(i))
        .collect();
    match healthy_idx.len() {
        0 => return None,
        1 => return Some(&items[healthy_idx[0]]),
        2 => {
            let a = &items[healthy_idx[0]];
            let b = &items[healthy_idx[1]];
            return Some(if inflight(a) <= inflight(b) { a } else { b });
        }
        _ => {}
    }
    let i = fastrand::usize(..healthy_idx.len());
    let mut j = fastrand::usize(..healthy_idx.len() - 1);
    if j >= i { j += 1; }
    let (a, b) = (&items[healthy_idx[i]], &items[healthy_idx[j]]);
    match (healthy(a), healthy(b)) {
        (true, true) => if inflight(a) <= inflight(b) { Some(a) } else { Some(b) },
        _ => unreachable!("indices were pre-filtered as healthy"),
    }
}
```

注意：如果要做到严格 O(1)，不要先构造 `healthy_idx`，而是在随机探测失败时最多补探几次再 fallback 扫描；上面代码更偏正确性示意。引入前需要决定依赖（`fastrand` / `rand` / thread-local RNG），当前 workspace 还没有这些依赖。

**为什么好**：健康节点较多时固定 2 次 inflight atomic load，不随 N 增长；负载均衡质量接近 least-loaded（Mitzenmacher 1996）。

**触发条件**：单 group client 数 > 10 且扫描成为 flamegraph 热点。TODO-69 实现后，h2c cache hit 不 pick，但其他首次选择路径仍可能受益；没有 profile 证据前不做。

**Files**: `tunnel-lib/src/inflight.rs`、`server/registry.rs`、`client/conn_pool.rs`

---

## [TODO-72] Client 端小优化（非紧急，随手做）

**Priority**: Low | **Status**: TODO
**依赖与影响**:
- 依赖：—（exclude set 可独立做）；如果 TODO-65 已落地，可接入结构化 QUIC open 错误
- 被依赖：—
- 两端影响：**仅 client**（server 侧无对应问题）

**Problem**:
1. **`client/conn_pool.rs:27-34` push 时线性查重**
   ```rust
   if g.iter().any(|c| c.conn.stable_id() == conn.stable_id()) { return; }
   ```
   池大时 O(N)。池规模通常小（<10），现状可接受，不紧急
2. **`client/entry.rs:106-159` 重试循环可能重选同一连接**
   失败后 `inflight` 归还（`InflightGuard::drop`），再 `next_conn()` 可能又命中它——徒劳尝试；而且无论什么错误都重试，没有结构化 QUIC open 错误判断

**Fix**:
1. `push` 加 `HashSet<u64 stable_id>` 辅助 O(1) 去重
2. 重试带 exclude set，避免同一轮反复命中刚失败的 connection：
   ```rust
   let mut tried: SmallVec<[u64; 4]> = SmallVec::new();
   for _ in 0..pool_size.max(1) {
       let conn = pool.next_conn_excluding(&tried)?;
       tried.push(conn.conn.stable_id());
       match open_bi_guarded(&conn.conn, ...).await {
           Ok(o) => return ...,
           Err(e) if is_fatal_quic_open_error(&e) => return Err(e),
           Err(e) => {
               last_err = e;
               continue;
           }
       }
   }
   ```
3. 如果 TODO-65 已落地，client entry 使用专门的 `QuicOpenTimeout / QuicConnectionLost / QuicStreamLimit` 分类；不要直接套用 HTTP upstream 的 `RetryType::ReusedOnly`，那是复用 HTTP upstream 连接首请求失败的语义。

**未来多 server 节点的分组**（超出当前范围）：现在 client 的池是扁平的（所有 QUIC 连接等价）。如果 server 端做多实例 HA，client 需要按 server group 分池——届时直接套 server `ClientGroup` 的 `DashMap<GroupId, ...>` 结构。

**Files**: `client/conn_pool.rs`、`client/entry.rs`

---

## [TODO-73] 不要抄 Pingora 的部分（参考避坑）

**Priority**: FYI | **Status**: — (非实施 task，记录决策避免走弯路)
**依赖与影响**: — （决策记录，不涉及代码改动）

调研 Pingora 时发现以下模式**不应引入**，在此记录决策和理由：

1. **Pingora 的 worker / fork / listenfd 架构**
   - 场景：CDN 需要跨进程 fd 传递做热升级
   - 不抄理由：duotunnel 不做 CDN 规模，`CancellationToken` 协作式 shutdown + 重启已够用
   - 现状：`run_accept_worker` 已经满足需求

2. **`pingora-cache`（HTTP 缓存）**
   - 场景：CDN 的正反向 cache
   - 不抄理由：tunnel 本身不缓存响应体，业务无此需求

3. **`pingora-ketama`（一致性哈希环）**
   - 场景：后端节点多且有 session affinity / 部分故障时最少重分配
   - 不抄理由：duotunnel 单 group 通常几到几十个 client，TODO-71 的 P2C 足够；一致性哈希环维护成本高

4. **`tinyufo`（S3-FIFO + TinyLFU 缓存）**
   - 用途：Pingora 里仅被 `pingora-memory-cache` 包装使用，给业务缓存 DNS 解析、auth 验证结果等小 key 高 QPS 场景
   - 不抄理由：duotunnel 所有 in-memory 缓存（`ClientRegistry`、h2c plugin 的 route_cache 等）都**不需要淘汰语义**——连接在线就在、一次性连接生命周期结束自然回收
   - 将来何时引入：做 upstream DNS 缓存、auth token 缓存、路由 negative cache 或 TLS 证书缓存且 QPS 极高时才值得引入

5. **`ShutdownWatch` 搭配的 `Fds` 无缝升级（SIGHUP 传 fd）**
   - 场景：零 downtime 升级
   - 不抄理由：overkill，duotunnel 允许短暂连接断开

6. **Pingora 的 `Service` 启动顺序 / 依赖图（`ServiceWithDependents` / `ServiceHandle`）**
   - 场景：多 service 有启动依赖、readiness 协调
   - 不抄理由：duotunnel service 数量少，简单 spawn 即可；需要时再按需引入

7. **Pingora `ServerApp` 式的多协议 app-per-service**
   - 场景：一个 app 同时 `add_tcp(:80) + add_tls(:443)`
   - 不抄理由：duotunnel 的协议分派在 `IngressDispatcher` 做（按 sniff 结果选 handler），不需要再引入多 listener-per-app 的复杂度
