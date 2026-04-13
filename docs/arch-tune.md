 DuoTunnel 架构改造：Runtime 分层 + Proxy 抽象对齐      

 Context

 当前 duotunnel server 两个核心问题：

 问题 A — Runtime 无层次：所有任务（QUIC accept、TCP/HTTP ingress listeners、手写 TCP HTTP metrics server、config watcher/control client）共享一个 tokio runtime，通过 spawn_task 散落式
 spawn。metrics server 与 QUIC 热路径竞争 worker 线程；ingress listener 每端口只有 1 个 accept task；handlers/metrics.rs 手写 200 行 TCP HTTP parser，有已知 512 字节截断 + 无 keep-alive
  问题。

 问题 B — Proxy 抽象游离：H2 路径完全绕过 ProxyEngine，在 proxy.rs 有独立 if 分支直接调 serve_h2_forward，在 handlers/http.rs TLS/H2 path 直接调 forward_h2_request。PeerKind::H2
 变体仅在 client 侧 ingress 使用，server 侧 ingress H2 走另一套路径，造成同一功能两套调用路径。ProxyEngine::run_stream 是 Template Method，但没有 pre/post hook，无法做
 metrics、retry、header 注入。

 ---
 Part 1：三 Runtime 分层架构

 目标架构

 main() (std thread，无 runtime)
 │
 ├─ proxy_rt: new_multi_thread, apply_worker_threads()
 │  └─ block_on(proxy_main(state, shutdown, proxy_handle, ready))
 │      ├─ QUIC accept loop (单 task，per-connection spawn)  [不改为 N 并发]
 │      ├─ N accept tasks per HTTP ingress listener (Arc<TcpListener>)
 │      └─ N accept tasks per TCP ingress listener  (Arc<TcpListener>)
 │
 ├─ metrics_rt: new_current_thread (1 线程)
 │  └─ block_on(run_metrics_server(port, ready, shutdown))
 │      └─ hyper http1 accept loop → /metrics + /healthz
 │
 └─ background_rt: new_current_thread (1 线程)
    └─ block_on(background_main(state, shutdown, proxy_handle))
        └─ HotReloadService 或 ControlClientService (BackgroundService trait)

 跨 runtime 共享：
 - Arc<ServerState> — 已是 Arc，无变化
 - CancellationToken（直接 clone，无需 Arc）— shutdown 总线
 - Arc<AtomicBool> (ready) — proxy_rt 写，metrics_rt 读，原子安全
 - tokio::runtime::Handle (proxy_handle) — background_rt 调 sync_listeners 时 spawn 到 proxy_rt

 QUIC accept 不改为 N 并发：quinn::Endpoint 内部单 UDP fd + Arc<Mutex<State>>，N 个并发 endpoint.accept() 只会在同一 Mutex 上竞争，收益为负。并发来自 per-stream spawn，已经足够。

 N-accept-loop for TCP/HTTP ingress

 每个 listener 端口启动 accept_workers（默认 4，加入 ServerConfig）个 task，共享同一个 Arc<TcpListener>：

 Arc<TcpListener>
   ├─ task 0: loop { select! { listener.accept() | shutdown.cancelled() } }
   ├─ task 1: 同上
   ├─ task 2: 同上
   └─ task 3: 同上

 内核 accept(2) 每次只唤醒一个等待者（无 thundering herd）。SO_REUSEPORT 已由 build_reuseport_listener 设置，N task 共享同一 fd，不需额外 socket。

 ---
 Part 2：Proxy 抽象对齐（对标 Pingora）

 DuoTunnel vs Pingora 抽象对比

 ┌────────────────┬─────────────────────────────────────────────────────────────────────────┬─────────────────────────────────────────────┬───────────────────────────────┐
 │      维度      │                                 Pingora                                 │               DuoTunnel 现状                │             差距              │
 ├────────────────┼─────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────┼───────────────────────────────┤
 │ 服务层级       │ ServiceWithDependents：proxy service + background service 分层          │ 全部在单 runtime spawn_task                 │ Runtime 无层次（Part 1 解决） │
 ├────────────────┼─────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────┼───────────────────────────────┤
 │ 请求处理       │ 8+ hook（request_filter、upstream_peer、upstream_connect_established…） │ 单一 upstream_peer()                        │ 无 pre/post hook              │
 ├────────────────┼─────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────┼───────────────────────────────┤
 │ 连接建立后     │ connected_to_upstream hook 可注入 header、改 conn                       │ connect_inner(self,...) 消费 self，无法复用 │ 无法做 retry / reconnect      │
 ├────────────────┼─────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────┼───────────────────────────────┤
 │ H2 路径        │ 统一走 proxy pipeline，H2 是协议                                        │ H2 绕过 ProxyEngine，独立 if 分支           │ 优先修复                      │
 ├────────────────┼─────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────┼───────────────────────────────┤
 │ ProtocolDriver │ 无此 trait（hyper 处理）                                                │ 定义了 trait 但 H2 从未实现                 │ 死代码                        │
 ├────────────────┼─────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────┼───────────────────────────────┤
 │ 负载均衡       │ LoadBalancer trait + 多种策略                                           │ UpstreamGroup 纯轮询，无健康检查            │ 低优先级                      │
 └────────────────┴─────────────────────────────────────────────────────────────────────────┴─────────────────────────────────────────────┴───────────────────────────────┘

 H2 路径统一（优先级最高）

 现状：proxy.rs::handle_work_stream 对 H2 有独立 if 分支：
 if routing.protocol == Protocol::H2 {
     serve_h2_forward(...)   // 绕过 ProxyEngine
 } else {
     engine.run_stream(...)  // 走 ProxyEngine
 }

 目标：H2Peer::connect_inner 已实现 serve_h2_forward，让 PeerKind::H2 走正常的 ProxyEngine::run_stream → peer.connect() 路径，删掉 proxy.rs 里的特殊 if 分支。

 前置条件：ProxyEngine::run_stream 传入 protocol 信息，让 UpstreamResolver::upstream_peer 可以基于 protocol 返回 PeerKind::H2，而不是在 proxy.rs 外部判断。当前 Context struct 已有
 protocol 字段，只需确认 server 侧 ingress 的 resolver 读取它。

 ProxyEngine Hook 点（中期目标）

 在 ProxyEngine::run_stream 的骨架里加两个 hook（trait method，默认 no-op）：

 trait ProxyHooks {
     async fn on_request(&self, ctx: &mut Context) -> Result<()> { Ok(()) }
     async fn on_connected(&self, ctx: &Context, peer: &PeerKind) {}
 }

 用途：
 - on_request：注入 X-Forwarded-For（把 src_addr 从 RoutingInfo 剥离，转为 HTTP header）
 - on_connected：metrics 计数（替代 begin_inflight() 手动调用）

 ---
 实现步骤（顺序执行）

 Step 1 — Runtime 基础设施

 tunnel-lib/src/infra/runtime.rs：
 - 新增 pub fn build_proxy_runtime() -> tokio::runtime::Runtime（Builder::new_multi_thread + apply_worker_threads）
 - 新增 pub fn build_single_thread_runtime(name: &str) -> tokio::runtime::Runtime（Builder::new_current_thread + enable_all + thread_name(name)）

 server/main.rs — run_server() 重构（顺序严格）：
 1. PrometheusBuilder::new().install_recorder() → metrics::set_handle(handle)  ← 必须在所有 runtime 前
 2. let shutdown = CancellationToken::new()
 3. let ready = Arc::new(AtomicBool::new(false))
 4. let proxy_rt = build_proxy_runtime()
 5. let proxy_handle = proxy_rt.handle().clone()
 6. std::thread::spawn → metrics_rt.block_on(run_metrics_server(port, ready.clone(), shutdown.clone()))
 7. std::thread::spawn → background_rt.block_on(background_main(state.clone(), shutdown.clone(), proxy_handle.clone()))
 8. proxy_rt.block_on(proxy_main(state, shutdown, proxy_handle, ready))   ← main thread blocks here
 9. join background thread, join metrics thread
 - 删除 spawn_task fn，proxy_rt 内部统一 tokio::task::spawn()
 - dial9 路径：run_with_dial9 继续包装 proxy_rt 的 block_on，不影响其他两个 runtime

 shutdown 顺序：shutdown.cancel() 由 proxy_rt 信号处理触发 → background_rt select! 退出 → proxy_rt block_on 自然退出。join 顺序：先 background thread，再 metrics
 thread，proxy_rt.block_on 在最后。防止 proxy_handle 在 background 还在使用时失效。

 Step 2 — N-accept-loop

 server/listener_mgr.rs — sync_listeners 签名：
 pub fn sync_listeners(
     state: &Arc<ServerState>,
     desired: &[IngressListener],
     proxy_handle: &tokio::runtime::Handle,
     accept_workers: usize,
 )

 内部 spawn 改为：
 let listener = Arc::new(build_reuseport_listener(addr)?);
 for _ in 0..accept_workers {
     let listener = listener.clone();
     let state = state.clone();
     let cancel = cancel.clone();
     proxy_handle.spawn(async move {
         run_http_accept_loop(Arc::clone(&listener), state, port, cancel).await
     });
 }

 server/handlers/http.rs 和 server/handlers/tcp.rs：
 - run_http_listener / run_tcp_listener 接收 Arc<TcpListener>（不再内部 build）
 - crate::spawn_task → tokio::task::spawn

 Step 3 — Metrics server 重写

 server/handlers/metrics.rs 全量替换（~200 行 → ~50 行）：

 use hyper::server::conn::http1;
 use hyper::service::service_fn;
 use hyper_util::rt::TokioIo;

 pub async fn run_metrics_server(
     port: u16,
     ready: Arc<AtomicBool>,
     shutdown: CancellationToken,
 ) -> Result<()> {
     let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
     info!(port, "metrics server started");
     loop {
         tokio::select! {
             _ = shutdown.cancelled() => break,
             Ok((stream, _)) = listener.accept() => {
                 let ready = ready.clone();
                 tokio::task::spawn(async move {
                     let io = TokioIo::new(stream);
                     let _ = http1::Builder::new()
                         .serve_connection(io, service_fn(move |req| {
                             let ready = ready.clone();
                             async move { handle_request(req, &ready).await }
                         }))
                         .await;
                 });
             }
         }
     }
     Ok(())
 }

 删除 set_handle() + PrometheusBuilder 调用（移到 main.rs Step 1）。

 Step 4 — BackgroundService trait

 server/service.rs（新文件）：
 pub trait BackgroundService: Send + 'static {
     fn name(&self) -> &str;
     async fn run(
         self: Box<Self>,
         state: Arc<ServerState>,
         shutdown: CancellationToken,
         proxy_handle: tokio::runtime::Handle,
     ) -> anyhow::Result<()>;
 }

 server/hot_reload.rs：spawn_config_watcher → HotReloadService { config_path } 实现 BackgroundService，watch loop 加 select! { shutdown.cancelled() } 退出，sync_listeners 调用传入
 &proxy_handle, accept_workers，删除内部 spawn_task。

 server/control_client.rs：spawn_control_client → ControlClientService { ctld_addr } 实现 BackgroundService，watch loop 加 shutdown，apply_snapshot 接收 proxy_handle，删除内部
 spawn_task。

 Step 5 — H2 路径统一（Proxy 抽象）

 server/proxy.rs 或等效文件中的 handle_work_stream**：
 - 删除 if routing.protocol == Protocol::H2 { serve_h2_forward(...) } 特殊分支
 - 确认 server 侧 ingress UpstreamResolver 实现在 protocol == H2 时返回 PeerKind::H2
 - ProxyEngine::run_stream 统一处理所有 protocol 变体

 验证点：删除特殊分支后，H2 ingress 的 H2Peer::connect_inner 被正常调用，serve_h2_forward 通过 H2Peer 而非直接调用。

 Step 6 — ProxyEngine Hook 点（可选，中期）

 如需在 ProxyEngine::run_stream 加 metrics / header 注入，在 tunnel-lib/src/proxy/core.rs 的 Context 或 run_stream 签名里加 optional hook trait，默认 no-op。这一步可在 Step 5
 完成后独立进行，不阻塞前面任何步骤。

 ---
 关键文件

 ┌─────────────────────────────────┬──────────┬───────────────────────────────────────────────────────┐
 │              文件               │   步骤   │                       变化程度                        │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/main.rs                  │ Step 1   │ 大改：三 runtime 启动，删 spawn_task                  │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ tunnel-lib/src/infra/runtime.rs │ Step 1   │ 小改：新增两个 runtime builder helper                 │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/listener_mgr.rs          │ Step 2   │ 中改：加 proxy_handle + accept_workers，N-accept-loop │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/handlers/http.rs         │ Step 2   │ 小改：接收 Arc，spawn_task→spawn                      │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/handlers/tcp.rs          │ Step 2   │ 小改：同上                                            │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/handlers/quic.rs         │ Step 1/2 │ 小改：加 CancellationToken，spawn_task→spawn          │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/handlers/metrics.rs      │ Step 3   │ 全量重写（200行→50行）                                │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/service.rs               │ Step 4   │ 新文件：trait 定义                                    │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/hot_reload.rs            │ Step 4   │ 中改：BackgroundService impl                          │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/control_client.rs        │ Step 4   │ 中改：BackgroundService impl                          │
 ├─────────────────────────────────┼──────────┼───────────────────────────────────────────────────────┤
 │ server/proxy.rs (或对应文件)    │ Step 5   │ 删除 H2 特殊分支                                      │
 └─────────────────────────────────┴──────────┴───────────────────────────────────────────────────────┘

 不变文件：server/metrics.rs, server/registry.rs, server/egress.rs, server/tunnel_handler.rs, server/local_auth.rs, server/config.rs, tunnel-lib/src/proxy/h2.rs,
 tunnel-lib/src/proxy/h2_proxy.rs

 ---
 注意事项

 1. Prometheus recorder 必须在所有 runtime 启动前 install：在 run_server() 的最顶部调用，存入 OnceLock，proxy_rt 里的 metrics 调用才能正常工作（recorder 是全局单例，不依赖 runtime）。
 2. proxy_handle 生命周期：background_rt 的 future 必须在 proxy_rt 的 block_on 结束前完成，否则 proxy_handle 成为悬空 Handle。通过 shutdown 顺序保证：signal → cancel → background 先
 select! 退出 → proxy 退出 → join 顺序：先 background thread，再 metrics thread。
 3. accept_workers 配置：加到 ServerConfig 里，字段名 accept_workers: Option<usize>，None 时默认 4。
 4. H2 统一的前提：ProxyEngine::run_stream 需要能在 detect_protocol 阶段之前或之后获知 Protocol::H2，确认 Context.protocol 已在 handle_work_stream 调用前被正确设置（来自 RoutingInfo
 解析）。



 ===

 
  # DuoTunnel 双向代理重设计方案（校正后的版本）

  ## Summary

  上一版方案方向对，但表达上不够准确：DuoTunnel 不是单向 HTTP proxy，而是两条对称但职责不同的链路同时存在。

  实际必须按两条业务流设计：

  - Ingress：外部 -> server -> select client -> client -> target
  - Egress：本地应用 -> client -> server -> target

  因此不能简单照搬 Pingora 的“单边 HTTP pipeline”思维，也不能只拆成两个大块就结束。正确方案应是：

  1. 共享基础层：TransportDemux、StreamFlow、HttpFlow、RuleEngine、EndpointSelector、UpstreamConnector
  2. 角色适配层：
      - ServerIngressGateway
      - ClientIngressExecutor
      - ClientEgressGateway
      - ServerEgressExecutor

  这样既能双端对齐，又不会把 ingress 和 egress 的责任错误混在一起。

  ## Corrected Architecture

  ### 1. 共享基础层

  这些层是双端共用的，不带“server/client”业务语义。

  - TransportDemux
      - 识别 TLS / H1 / H2 / WS / TCP / opaque TLS
      - 提供 ConnectionContext
      - 决定进入 HttpFlow 或 StreamFlow
      - 不做路由，不做 upstream 选择
  - StreamFlow
      - 用于 TCP / WS / opaque relay
      - 输入是连接或 QUIC stream
      - 进行一次选路、一次建连、全程 relay
      - 只提供 stream 级 hooks
  - HttpFlow
      - 用于 H1 / H2
      - 输入是 request 或 H2 substream
      - 负责 request filter、route resolve、upstream session 获取/释放、response filter、retry
      - 提供 request 级 hooks
  - RuleEngine
      - 纯数据驱动，不直接承担 IO
      - 根据规则集和上下文产出 route decision
      - 规则不是 hook，本质上是 hook 中调用的数据引擎
  - EndpointSelector
      - 把 route decision 映射成实际 endpoint
      - Ingress 场景：选 client group / client connection
      - Egress 场景：选 upstream group / upstream server
  - UpstreamConnector
      - StreamConnector：建立 raw stream / tcp / ws relay
      - HttpConnector：获取/释放 H1/H2 upstream session

  ### 2. 四个角色适配层

  不要只定义“client/server”，要定义“gateway/executor”。

  - ServerIngressGateway
      - 面向外部连接
      - 负责 listener accept、TLS terminate、HTTP/stream demux
      - 根据 ingress rules 决定发给哪个 client group
      - 对 HTTP：
          - request 级路由在这里完成
          - 向 client 发出 request 或 stream
      - 对 TCP/WS：
          - 走 StreamFlow
  - ClientIngressExecutor
      - 接收 server 发来的 ingress work stream / request
      - 根据 client local upstream config 连接本地 target
      - HTTP 走 HttpFlow
      - TCP/WS 走 StreamFlow
  - ClientEgressGateway
      - 面向本地应用 entry listener
      - 做本地协议识别与最小 request/stream 上下文提取
      - 发往 server
      - HTTP 走 HttpFlow
      - TCP/WS 走 StreamFlow
  - ServerEgressExecutor
      - 接收 client 发来的 egress request / stream
      - 根据 egress rules 选择外部 upstream
      - HTTP 走 HttpFlow
      - TCP/WS 走 StreamFlow

  ## Rules and Hook Design

  ### 1. Rules 不应直接建模为 hook type

  这点需要明确。

  hook 是执行时机。
  rule 是数据和匹配逻辑。

  正确关系应为：

  - hook 决定“何时介入”
  - RuleEngine 决定“根据什么规则得出什么路由决策”

  所以规则最适合作为：

  - request_filter / select_route / select_endpoint 阶段使用的数据输入
  - 而不是把每条 rule 直接变成一个 hook 实例

  ### 2. 建议的规则分层

  当前已有规则天然分成两类：

  - Ingress rules
      - listener + host -> client_group + proxy_name
      - 本质是“边缘入口路由规则”
  - Egress rules
      - host -> upstream_group
      - 本质是“外部目标路由规则”

  应重构成统一的中间表达：

  - RouteRule
      - match 条件：port / host / protocol / authority / listener mode
      - action：logical route key
  - EndpointPolicy
      - logical route key -> endpoint group / backend group / client group
  - SelectionPolicy
      - round robin / weighted / least inflight / healthy only

  默认映射：

  - server ingress vhost/tcp rules -> RouteRule + EndpointPolicy
  - server egress vhost rules -> RouteRule + EndpointPolicy
  - client local upstream config -> EndpointPolicy

  ### 3. Hook 最小集合

  为了避免过早变成 Pingora 全量 clone，这次只定义必要 hooks。

  #### Stream hooks

  - on_stream_start
  - on_route_selected
  - on_endpoint_connected
  - on_stream_end
  - on_stream_error

  #### HTTP hooks

  - early_request_filter
  - request_filter
  - select_route
  - select_endpoint
  - connected_to_upstream
  - upstream_request_filter
  - response_filter
  - fail_to_connect

  规则引擎的调用点：

  - ingress HTTP：select_route
  - ingress TCP/WS：on_route_selected
  - egress HTTP：select_route
  - egress TCP/WS：on_route_selected

  ## Metadata / Wire Design

  ### 1. RoutingInfo 不能继续混合 request-level 字段

  上一版判断是对的，这里确认保留。

  新原则：

  - RoutingInfo 仅保留 stream-level control metadata
  - request-level metadata 不通过初始化控制帧传输

  ### 2. 上下文拆分

  引入三层上下文：

  - ConnectionContext
      - remote addr
      - listener port
      - tls info
      - alpn / protocol hint
  - StreamContext
      - route key
      - direction: ingress / egress
      - tunnel metadata
      - selected endpoint metadata
  - RequestContext
      - host / authority
      - method
      - forwarded source info
      - retry state
      - request-scoped annotations

  ### 3. wire shape 默认策略

  - L4/TCP/WS：
      - 继续需要最小 RoutingInfo
      - 用于跨端定位 logical route / direction
  - HTTP：
      - 不再依赖 RoutingInfo.host
      - host/authority 从真实 request 读取
      - src_addr/src_port 转成本地 Forwarded / X-Forwarded-For 注入语义
      - 若 QUIC 对端确需来源信息，使用专门的 request metadata frame，而不是复用 stream init frame

  ## Implementation Plan

  ### Phase 1: 先立正确骨架

  - 引入共享层类型：
      - TransportDemux
      - StreamFlow
      - HttpFlow
      - RuleEngine
      - EndpointSelector
  - 不先清空旧实现，先做 adapter 包住现有逻辑
  - 目标是让 ingress/egress 两端都能映射到同一套共享层，而不是继续复制 handler 逻辑

  ### Phase 2: 先做双向 HTTP 正路

  - ServerIngressGateway -> ClientIngressExecutor
  - ClientEgressGateway -> ServerEgressExecutor
  - 这两条 HTTP 路径统一收敛到 HttpFlow
  - 把 H1/H2 的 request lifecycle、header 注入、response filter、retry 放到一个位置

  完成标准：

  - H2 不再是旁路
  - HTTP 行为不再散在 handler / peer / helper 中

  ### Phase 3: 再统一 TCP / WS / opaque stream

  - ingress TCP/WS 和 egress TCP/WS 统一收敛到 StreamFlow
  - 保证 route select、endpoint connect、relay 生命周期统一

  ### Phase 4: 规则引擎与协议语义收缩

  - 将当前 ingress/egress rules 编译成统一 RouteRule 模型
  - 精简 RoutingInfo


  - ProxyEngine 退化为 StreamFlow compatibility layer 或删除
  - PeerKind 退化为内部 adaptor 或删除
  - 旧 ad-hoc H2 path 删除
  - 旧 handler 中的业务路由决策移除，只保留 accept/demux

  必须按双向场景测，不允许只测 ingress。

  - Ingress HTTP
      - 外部 H1 -> server -> client -> local backend
      - 外部 H2 -> server -> client -> local backend
      - host route、authority 改写、header 注入
  - Ingress TCP/WS
      - server 正确选 client
      - client 正确连接 local target
  - Egress TCP/WS
      - client 正确发送 route metadata
      - server 正确连到 target
  - Metadata correctness
      - request-level host 不依赖 RoutingInfo

  ## Assumptions

  - 本次以“先方案，后实施”为目标，不在当前大改基础上继续增量堆逻辑。
  - 双向流量模型是第一约束，任何抽象都必须同时解释 ingress 和 egress。