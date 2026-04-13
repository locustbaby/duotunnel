⏺ 现在全貌已清楚，做完整对比分析。                                                                                                                                                        
                                                                                                                                                                                          
  ---                                                                                                                                                                                     
  DuoTunnel Proxy 抽象 vs Pingora 对比                                                                                                                                                    
                                                                                                                                                                                          
  整体结构对比                                                                                                                                                                            
                                                                                                                                                                                          
  Pingora：                                                                                                                                                                               
  ProxyHttp trait (用户实现)                                                                                                                                                              
      upstream_peer()          → 返回 Peer (含地址、TLS配置、连接池key)                                                                                                                   
      request_filter()         → 过滤/修改请求 hook                                                                                                                                       
      response_filter()        → 过滤/修改响应 hook                                                                                                                                       
      upstream_peer_filter()   → 决定是否去upstream hook                                                                                                                                  
      ...8个以上 hook                                                                                                                                                                     
                                                                                                                                                                                          
  HttpProxy (框架实现)                                                                                                                                                                    
      process_request()        → 固定流程骨架                                                                                                                                             
          ↓ request_filter hook                                                                                                                                                           
          ↓ proxy_cache hook                                                                                                                                                              
          ↓ proxy_to_upstream()
              get_http_session()  → 连接池取/建连                                                                                                                                         
              send_request        → 发给 upstream                                
              read_response       → 读响应                                                                                                                                                
              release_session()   → 归还连接池                                                                                                                                            
          ↓ response_filter hook                                                                                                                                                          
                                                                                                                                                                                          
  DuoTunnel：                                                                                                                                                                             
  UpstreamResolver trait (用户实现)
      upstream_peer()          → 返回 PeerKind                                                                                                                                            
                                                                                 
  ProxyEngine (框架实现)                                                                                                                                                                  
      run_stream()             → 固定流程骨架
          peek 初始字节                                                                                                                                                                   
          detect_protocol()                                                                                                                                                               
          ↓ upstream_peer()                                                                                                                                                               
          peer.connect()       → 连接+relay全部在此                                                                                                                                       
                                                                                                                                                                                          
  PeerKind (enum dispatch)                                                                                                                                                                
      Tcp  → TcpPeer::connect_inner()   → raw relay
      Http → HttpPeer::connect_inner()  → Http1Driver loop → hyper client                                                                                                                 
      H2   → H2Peer::connect_inner()    → serve_h2_forward                                                                                                                                
      Dyn  → UpstreamPeer::connect_boxed()                                                                                                                                                
                                                                                                                                                                                          
  ---                                                                                                                                                                                     
  具体差距分析                                                                   
                                                                                                                                                                                          
  1. Hook 体系：Pingora 有，DuoTunnel 无                                         
                                                                                                                                                                                          
  Pingora 的 ProxyHttp trait 有 8+ 个 hook，覆盖请求生命周期每个阶段：                                                                                                                    
  early_request_filter → request_filter → upstream_peer_filter                                                                                                                            
  → upstream_request_filter → response_filter → upstream_response_filter                                                                                                                  
  → logging                                                                                                                                                                               
           
  DuoTunnel 的 UpstreamResolver 只有一个 upstream_peer()，连接建立前后完全不可扩展。想加 header 注入、访问日志、重试逻辑，只能改 PeerKind::connect_inner 内部，侵入性强。                 
                                                                                                                                                                                          
  2. 错误重试：Pingora 有，DuoTunnel 无
                                                                                                                                                                                          
  Pingora 在 proxy_to_upstream 里有显式重试循环，upstream_peer() 可以被多次调用（每次失败换一个 backend）。DuoTunnel 选完 peer 就直接 connect，失败即报错，无重试。                       
  
  3. connect_inner 签名：消费 self，不可重用                                                                                                                                              
                                                                                 
  // DuoTunnel — self 被消费                                                                                                                                                              
  pub async fn connect_inner(                                                    
      self,              // ← move，用完即丢
      send: SendStream,                                                                                                                                                                   
      recv: RecvStream,
      initial_data: Option<Bytes>,                                                                                                                                                        
  ) -> Result<()>                                                                

  // Pingora — &self，可重用，连接池持有引用                                                                                                                                              
  pub async fn proxy_to_upstream(&self, session, ctx) -> Result<bool>
                                                                                                                                                                                          
  DuoTunnel 的 TcpPeer/HttpPeer 每次都是新建，持有的 hyper client 虽然是 clone（引用计数），但 peer 结构体本身不能复用，也不能放进池里。                                                  
                                                                                                                                                                                          
  4. H2 路径游离于 PeerKind 之外（已在 CODE_REVIEW 里记录）                                                                                                                               
                                                                                 
  proxy.rs:handle_work_stream 里对 H2 有特殊 if 分支，绕过了 ProxyEngine，导致同一功能两套路径。Pingora 没有这个问题，H1/H2 都走同一个 proxy_to_upstream 骨架，只是内部 session 类型不同。
                                                                                 
  5. ProtocolDriver trait 有但未被充分使用                                                                                                                                                
                                                                                 
  // 定义了但只有 h1 实现，h2 没走这条路                                                                                                                                                  
  pub trait ProtocolDriver {                                                                                                                                                              
      async fn read_request(&mut self) -> Result<Option<ProxyRequest>>;                                                                                                                   
      async fn write_response(&mut self, response: http::Response<Incoming>) -> Result<()>;                                                                                               
  }                                                                                                                                                                                       
                                                                                                                                                                                          
  Pingora 的 HttpSession 是统一抽象，H1/H2 都实现同一套接口，上层 proxy 逻辑完全不感知协议版本。DuoTunnel 的 ProtocolDriver 本来可以做到这一点，但 H2 绕过了它。                          
                                                                                                                                                                                          
  6. UpstreamGroup 过于简单                                                                                                                                                               
                                                                                 
  // DuoTunnel — 只有 round-robin
  pub struct UpstreamGroup {                                                                                                                                                              
      servers: Vec<String>,
      counter: AtomicUsize,                                                                                                                                                               
  }                                                                                                                                                                                       
  
  Pingora 的 LoadBalancer 支持 RoundRobin / Weighted / ConsistentHash，health check 独立运行，后端动态更新。DuoTunnel 的 UpstreamGroup 无 health check，无权重，无一致性哈希。（对于      
  tunnel 场景影响不大，因为 server 侧选的是 client QUIC 连接而非 HTTP backend，ClientGroup::select_healthy 更重要）
                                                                                                                                                                                          
  ---                                                                            
  总结：哪里值得改
                  
  ┌────────────────────────────────┬─────────────────────────────────────────┬───────────────────────────────────────────────────────────────────────────────────────────┐
  │              问题              │                  影响                   │                                           建议                                            │                
  ├────────────────────────────────┼─────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────┤
  │ 无 hook 体系，逻辑不可扩展     │ 中：加功能要改 core                     │ 在 UpstreamResolver 之外加 pre_connect/post_connect hook，或允许 PeerKind 包装 middleware │                
  ├────────────────────────────────┼─────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────┤
  │ H2 绕过 ProxyEngine            │ 中：两套路径，维护负担                  │ 让 serve_h2_forward 也走 PeerKind::H2，proxy.rs 删掉特殊分支                              │                
  ├────────────────────────────────┼─────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────┤                
  │ connect_inner 消费 self        │ 低：peer 重建成本很低                   │ 可以改为 &self，但收益不大                                                                │                
  ├────────────────────────────────┼─────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────┤                
  │ ProtocolDriver trait 未用于 H2 │ 低：当前 H2 路径够用                    │ 若未来统一骨架再补                                                                        │
  ├────────────────────────────────┼─────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────┤                
  │ UpstreamGroup 无 health check  │ 低：tunnel 场景 egress backend 通常固定 │ 按需加，不急                                                                              │
  └────────────────────────────────┴─────────────────────────────────────────┴───────────────────────────────────────────────────────────────────────────────────────────┘                
                                                                                 
  最值得做的是H2 路径统一进 ProxyEngine，消除两套调用路径；其次是在 run_stream 骨架里加 pre/post hook 点，让可观测性（日志、metrics）能在骨架层注入而非散落在各 peer 实现里。 



  ~/.claude/plans/hazy-mapping-milner-ultraplan.md





  ====

  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
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