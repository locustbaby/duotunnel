# DuoTunnel 架构改造计划

## 基于代码的现状分析

### 问题 A — Runtime 无层次

**现状（`server/main.rs`）：**
- `run_with_tokio` / `run_with_dial9` 各自内部创建单一 `new_multi_thread` runtime
- `spawn_task` 是一个全局函数，透明地把任务散落到同一 runtime
- metrics server（`handlers/metrics.rs`）、QUIC accept loop、HTTP/TCP listener、hot_reload/control_client 全部共享这一个 runtime 的 worker pool
- `handlers/metrics.rs` 是 200 行手写 TCP HTTP 解析器，有 512 字节截断 bug（`buf = [0u8; 512]`）且无 keep-alive；PrometheusBuilder `install_recorder()` 在 metrics server 协程内部调用，晚于其他 metrics 使用点

**影响：** metrics server 与 QUIC/TCP 热路径竞争 worker 线程；recorder 安装顺序不确定

---

### 问题 B — Proxy 抽象不完整

**现状（`client/proxy.rs:20-52`）：**
```rust
if routing_info.protocol == Protocol::H2 {
    // 直接调 serve_h2_forward，完全绕过 ProxyEngine
    serve_h2_forward(stream, proxy_map.https_client, ...)
} else {
    ProxyEngine::run_stream(...)
}
```

**现状（`server/handlers/http.rs`）：**
- TLS 路径（`handle_tls_connection`）：直接调 `forward_h2_request`，完全不走 ProxyEngine
- Plaintext H2 路径（`handle_plaintext_h2_connection`）：同上，使用 `Arc<Mutex<HashMap>>` 维护 per-request sender cache，有 std Mutex + 每请求 lock 的开销
- Plaintext H1 路径（`handle_plaintext_h1_connection`）：走字节级 relay，不走 ProxyEngine

**现状（`server/egress.rs` + `server/tunnel_handler.rs`）：**
- Server egress（client 发起的反向代理流）走 `ProxyEngine::run_stream` → `ServerEgressMap::upstream_peer()` → 返回 `PeerKind`
- Client ingress（client 处理 server 推来的 work stream）：H2 绕过，非 H2 走 ProxyEngine

**根本原因：** `H2Peer::connect_inner` 已经实现了完整的 H2 proxy，但 client 侧分支从未用到它；`ProxyEngine::run_stream` 只接收 `Option<RoutingInfo>`，没有暴露 protocol 信息到 peer 选择之前，只能在外部判断后另走分支。

---

## 预期效果

### 改造后

1. **三 Runtime 分层：** metrics server 跑在独立 `current_thread` runtime，不与热路径争线程；background（hot_reload / control_client）跑在另一个 `current_thread` runtime
2. **PrometheusBuilder 提前安装：** 在所有 runtime 启动前完成，消除 recorder 未安装时的 metrics 静默丢失
3. **metrics server 重写：** hyper http1，~40 行，无截断 bug，keep-alive 正常
4. **BackgroundService trait：** hot_reload 和 control_client 统一接口，shutdown 信号可控
5. **N-accept-loop：** 每个 TCP/HTTP ingress listener 启动 `accept_workers`（默认 4）个并发 accept task，共享同一 `Arc<TcpListener>`；QUIC 不变（quinn 内部单 fd）
6. **client H2 路径统一：** `client/proxy.rs` 删除 `if protocol == H2` 特殊分支，H2 通过 `ClientApp::upstream_peer()` 返回 `PeerKind::H2`，走正常 `ProxyEngine::run_stream → peer.connect()` 路径

---

## 改造步骤（按 commit 拆分）

### Commit 1 — `refactor(runtime): extract runtime builders to tunnel-lib`

**文件：** `tunnel-lib/src/infra/runtime.rs`

在现有 `apply_worker_threads` 之后新增两个 builder helper：

```rust
pub fn build_proxy_runtime() -> tokio::runtime::Runtime {
    let mut b = tokio::runtime::Builder::new_multi_thread();
    apply_worker_threads(&mut b);
    b.enable_all().thread_name("proxy-worker").build().expect("proxy runtime")
}

pub fn build_single_thread_runtime(name: &str) -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name(name)
        .build()
        .expect("single thread runtime")
}
```

同时 `tunnel-lib/src/lib.rs` 补充 re-export：`pub use infra::runtime::{build_proxy_runtime, build_single_thread_runtime};`

**验证：** 编译通过，无行为变化

---

### Commit 2 — `refactor(server): rewrite metrics server with hyper`

**文件：** `server/handlers/metrics.rs`（全量替换）

```rust
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

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
                        .keep_alive(true)
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

async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    ready: &Arc<AtomicBool>,
) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, std::convert::Infallible> {
    use http_body_util::Full;
    use std::sync::atomic::Ordering;
    let (status, body) = if req.uri().path() == "/healthz" {
        if ready.load(Ordering::Acquire) {
            (200u16, "ok\n")
        } else {
            (503u16, "not ready\n")
        }
    } else {
        (200u16, "") // body from encode() below
    };
    if req.uri().path() == "/healthz" {
        Ok(hyper::Response::builder()
            .status(status)
            .body(Full::new(bytes::Bytes::from(body)))
            .unwrap())
    } else {
        let body = crate::metrics::encode();
        Ok(hyper::Response::builder()
            .status(200)
            .header("content-type", "text/plain; charset=utf-8")
            .body(Full::new(bytes::Bytes::from(body)))
            .unwrap())
    }
}
```

签名变化：增加 `shutdown: CancellationToken` 参数（Commit 3 的前置）

**验证：** 编译通过；/metrics 和 /healthz 行为不变

---

### Commit 3 — `refactor(server): split into three runtimes`

**文件：** `server/main.rs`

核心改动：把 `run_server()` 里的异步逻辑拆成三段同步 `std::thread::spawn` + `runtime.block_on`：

```
main() [no runtime]
  ├── PrometheusBuilder::new().install_recorder()  ← 移到这里，最顶部
  ├── let proxy_rt = build_proxy_runtime()
  ├── let proxy_handle = proxy_rt.handle().clone()
  ├── let shutdown = CancellationToken::new()
  ├── let ready = Arc::new(AtomicBool::new(false))
  │
  ├── std::thread::spawn: metrics_rt.block_on(run_metrics_server(port, ready.clone(), shutdown.clone()))
  ├── std::thread::spawn: background_rt.block_on(background_main(state, shutdown.clone(), proxy_handle.clone()))
  └── proxy_rt.block_on(proxy_main(state, shutdown, proxy_handle, ready))  ← main thread blocks
      then join background, then join metrics
```

具体改动点：

1. `run_server` 改为同步函数（或内部手动拆，不使用 `async fn`），在最顶部调用 `PrometheusBuilder::new().install_recorder()`，结果存入 `crate::metrics::set_handle(handle)` 的 OnceLock（参照现有 `metrics.rs` 的 `set_handle` 设计）

2. 新增 `async fn proxy_main(state, shutdown, proxy_handle, ready)` — 包含当前 `run_server` 中的 QUIC server + sync_listeners 逻辑，`ready.store(true)` 在 QUIC bind 成功后设置

3. 新增 `async fn background_main(state, shutdown, proxy_handle)` — 根据 ctld_addr 调 `spawn_control_client` 或 `spawn_config_watcher`，但这两个函数内部改为直接 `tokio::task::spawn` 而非通过 `proxy_handle`（background_rt 自身是 current_thread runtime，足够运行 watch loop）；`sync_listeners` 调用改为通过 `proxy_handle.spawn(...)` 提交到 proxy_rt

4. 删除 `spawn_task` 函数（`dial9-telemetry` feature 的 handle spawn 也在 proxy_rt 范围内，可直接用 `tokio::task::spawn`）；如需保留 dial9 feature，`spawn_task` 改为 proxy_rt 内部的局部 helper

5. `sync_listeners` 签名增加 `proxy_handle: &tokio::runtime::Handle` 参数，内部 spawn 改为 `proxy_handle.spawn(...)`

6. shutdown 信号处理（SIGTERM/SIGINT）在 `proxy_main` 内部，触发 `shutdown.cancel()`

7. `run_with_tokio` 变为调用 `build_proxy_runtime().block_on(async_main())`（等效，保持 dial9 路径不变）

**验证：** 编译通过；`cargo test` 通过；metrics server 运行在独立线程；QUIC/TCP 不受影响

---

### Commit 4 — `refactor(server): add BackgroundService trait, convert hot_reload and control_client`

**文件：** `server/service.rs`（新文件）、`server/hot_reload.rs`、`server/control_client.rs`、`server/main.rs`

**`server/service.rs`：**

```rust
use crate::ServerState;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub trait BackgroundService: Send + 'static {
    fn name(&self) -> &'static str;
    async fn run(
        self: Box<Self>,
        state: Arc<ServerState>,
        shutdown: CancellationToken,
        proxy_handle: tokio::runtime::Handle,
    ) -> anyhow::Result<()>;
}
```

**`server/hot_reload.rs`：**
- `spawn_config_watcher` → `pub struct HotReloadService { config_path: String }` 实现 `BackgroundService`
- `watch_loop` 内部 `loop { rx.recv() ... }` 加 `select! { _ = shutdown.cancelled() => break, ... }`
- `reload_routing` 中的 `crate::sync_listeners(state, &listeners)` 改为通过 `proxy_handle.spawn(...)` 提交（或保持同步调用，因为 `sync_listeners` 自身不阻塞）

**`server/control_client.rs`：**
- `spawn_control_client` → `pub struct ControlClientService { ctld_addr: SocketAddr }` 实现 `BackgroundService`
- `connect_and_watch` 主 loop 加 `select! { _ = shutdown.cancelled() => return Ok(0), ... }`
- `apply_snapshot` 签名增加 `proxy_handle` 参数（用于 `sync_listeners` 提交到 proxy_rt）

**`server/main.rs`：**
- `background_main` 使用 `BackgroundService::run(Box::new(svc), state, shutdown, proxy_handle).await`

**验证：** 编译通过；热重载行为不变；ctld 模式行为不变

---

### Commit 5 — `refactor(server): N-accept-loop for TCP/HTTP ingress listeners`

**文件：** `server/listener_mgr.rs`、`server/handlers/http.rs`、`server/handlers/tcp.rs`、`server/config.rs`（或 `tunnel-store` 对应 config）、`server/main.rs`

**`server/config.rs` / `ServerConfigFile`：**
增加字段 `accept_workers: Option<usize>`（`None` 默认取 4）

**`server/listener_mgr.rs`：**
`sync_listeners` 签名增加 `proxy_handle: &tokio::runtime::Handle, accept_workers: usize`

```rust
let listener = Arc::new(tunnel_lib::build_reuseport_listener(addr.parse()?)?);
for _ in 0..accept_workers {
    let listener = listener.clone();
    let s = state.clone();
    let cancel = cancel.clone();
    proxy_handle.spawn(async move {
        if let Err(e) = run_http_accept_loop(Arc::clone(&listener), s, port, cancel).await {
            error!(port, error = %e, "HTTP accept loop failed");
        }
    });
}
```

**`server/handlers/http.rs`：**
`run_http_listener(state, port, cancel)` 改为 `run_http_accept_loop(listener: Arc<TcpListener>, state, port, cancel)`；内部 `listener.accept()` 直接用传入的 `Arc<TcpListener>`；`crate::spawn_task` → `tokio::task::spawn`

**`server/handlers/tcp.rs`：**
同上，`run_tcp_listener` → `run_tcp_accept_loop(listener: Arc<TcpListener>, ...)`；`crate::spawn_task` → `tokio::task::spawn`

**验证：** 编译通过；每个 listener 端口有 4 个并发 accept 协程；`netstat` 可见只有一个 fd per port（SO_REUSEPORT）

---

### Commit 6 — `refactor(client): unify H2 path through ProxyEngine`

**文件：** `client/proxy.rs`、`client/app.rs`

**问题根因：** `client/proxy.rs:20` 的 `if routing_info.protocol == Protocol::H2 { ... }` 完全绕过了 ProxyEngine，但 `H2Peer::connect_inner` 已经实现了相同逻辑（`serve_h2_forward`）。只需让 `ClientApp::upstream_peer()` 在 protocol 为 H2 时返回 `PeerKind::H2`。

**`client/app.rs`：**

`ClientApp::upstream_peer(ctx: &mut Context)` 中，在拿到 `proxy_name` 后：

```rust
// 现有逻辑
let upstream_addr = self.proxy_map.get_local_address(&proxy_name)...;

// 新增：H2 路径返回 H2Peer
if ctx.protocol == Protocol::H2 {
    let (scheme, authority) = self.proxy_map
        .get_h2_addr_for_server(&upstream_addr)
        .ok_or_else(|| anyhow::anyhow!(...))?;
    return Ok(PeerKind::H2(Box::new(H2Peer {
        target_host: authority,
        scheme,
        https_client: self.proxy_map.https_client.clone(),
        h2c_client: self.proxy_map.h2c_client.clone(),
    })));
}
// 原有 Tcp/Http peer 逻辑不变
```

**`client/proxy.rs`：**

删除整个 `if routing_info.protocol == Protocol::H2 { ... }` 分支，改为统一路径：

```rust
pub async fn handle_work_stream(...) -> Result<()> {
    let routing_info = recv_routing_info(&mut recv).await?;
    let app = ClientApp::new(proxy_map, tcp_params);
    let engine = ProxyEngine::new(app);
    let client_addr = ...;
    engine.run_stream(send, recv, client_addr, Some(routing_info)).await
}
```

**验证：**
- H2 ingress（server 推 H2 work stream → client）：`H2Peer::connect_inner` 被调用，`serve_h2_forward` 行为与原分支一致
- 非 H2：行为不变
- 删除后文件减少 ~20 行无重复逻辑

---

## 不改动的文件

以下文件本次改造不涉及，保持原样：

- `server/metrics.rs`（`encode()` / `set_handle()` / 计数函数）
- `server/registry.rs`
- `server/egress.rs`
- `server/tunnel_handler.rs`
- `server/local_auth.rs`
- `server/config.rs`（除新增 `accept_workers` 字段）
- `tunnel-lib/src/proxy/h2.rs`
- `tunnel-lib/src/proxy/h2_proxy.rs`
- `tunnel-lib/src/proxy/http.rs`
- `tunnel-lib/src/proxy/tcp.rs`
- `tunnel-lib/src/proxy/core.rs`
- `tunnel-lib/src/engine/bridge.rs`
- `client/connect.rs`
- `client/entry.rs`
- `client/pool.rs`

---

## 注意事项

### PrometheusBuilder 安装时机
必须在 Commit 3 中移到 `run_server` 的最顶部（在任何 runtime 构建前），否则 proxy_rt 启动后的 metrics 调用会在 recorder 未安装时静默丢失。当前 `handlers/metrics.rs` 的 `run_metrics_server` 里调用 `install_recorder()` 是 bug，这个移动同时修复它。

### proxy_handle 生命周期
Commit 3 中，`background_rt` 持有 `proxy_handle`（`proxy_rt.handle().clone()`）。必须保证 background thread join 在 `proxy_rt.block_on` 结束之前，否则 background task 通过 `proxy_handle.spawn()` 可能 panic。join 顺序：先 background thread，再 metrics thread，`proxy_rt` 最后 drop。

### N-accept-loop 与 SO_REUSEPORT
`build_reuseport_listener`（`tunnel-lib/src/transport/listener.rs`）已设置 SO_REUSEPORT。N 个 task 共享同一 `Arc<TcpListener>`（同一 fd），内核 accept(2) 每次只唤醒一个 waiter，无 thundering herd。

### Commit 6 前提
`ProxyEngine::run_stream` 已通过 `detect_protocol` 从 `routing_info.protocol` 读取协议类型，并设置到 `ctx.protocol`，再传给 `upstream_peer`。`ClientApp::upstream_peer` 可直接读 `ctx.protocol`，无需额外改动 `core.rs`。

### dial9-telemetry feature
Commit 3 中删除 `spawn_task` 全局函数后，`dial9-telemetry` feature 的 `DIAL9_HANDLE.get().spawn(future)` 需要迁移到 `proxy_rt` 内部。最简方案：在 `proxy_main` 入口处检查 `DIAL9_HANDLE`，将所有 `tokio::task::spawn` 替换为条件分支，或保留 `spawn_task` 但只作为 proxy_rt 内部模块私有函数。

---

## 关键文件变化汇总

| 文件 | Commit | 变化 |
|------|--------|------|
| `tunnel-lib/src/infra/runtime.rs` | 1 | 新增 `build_proxy_runtime`, `build_single_thread_runtime` |
| `tunnel-lib/src/lib.rs` | 1 | re-export 两个 builder |
| `server/handlers/metrics.rs` | 2 | 全量重写，~200→~60 行，hyper http1 |
| `server/main.rs` | 3 | 三 runtime 拆分，PrometheusBuilder 移至顶部，删 `spawn_task` |
| `server/service.rs` | 4 | 新文件，`BackgroundService` trait |
| `server/hot_reload.rs` | 4 | `HotReloadService` impl `BackgroundService` |
| `server/control_client.rs` | 4 | `ControlClientService` impl `BackgroundService` |
| `server/config.rs` | 5 | 新增 `accept_workers: Option<usize>` |
| `server/listener_mgr.rs` | 5 | N-accept-loop，签名增加 `proxy_handle` + `accept_workers` |
| `server/handlers/http.rs` | 5 | 接收 `Arc<TcpListener>`，spawn_task→spawn |
| `server/handlers/tcp.rs` | 5 | 同上 |
| `client/app.rs` | 6 | H2 时返回 `PeerKind::H2` |
| `client/proxy.rs` | 6 | 删除 H2 特殊分支，统一走 ProxyEngine |


实际问题确认
                                                                                                                                                                                          
  问题 A（Runtime）：                                                                                                                                                                     
  - handlers/metrics.rs 里的 PrometheusBuilder::new().install_recorder() 在 metrics 协程内部调用——recorder 安装晚于其他 metrics 使用点，存在静默丢失风险
  - metrics server 用 512 字节固定 buffer 读 HTTP，有截断 bug，无 keep-alive                                                                                                              
  - 所有任务共享单一 multi-thread runtime                                   
                                                                                                                                                                                          
  问题 B（H2 路径）：                                                                                                                                                                     
  - client/proxy.rs:20 的 if protocol == H2 分支完全绕过 ProxyEngine，直接调 serve_h2_forward                                                                                             
  - 但 H2Peer::connect_inner（tunnel-lib/src/proxy/h2.rs:89）已实现完全相同的逻辑——两条路径重复                                                                                           
  - 统一的前提已满足：ProxyEngine::run_stream 通过 ctx.protocol 传递协议信息，ClientApp::upstream_peer 可直接读取
                                                                                                                                                                                          
  6 个 Commit 顺序                                                                                                                                                                        
                                                                                                                                                                                          
  ┌─────┬─────────────────────────────┬───────────────────────────────────────────────────────────────────┐                                                                               
  │  #  │           Commit            │                             核心改动                              │
  ├─────┼─────────────────────────────┼───────────────────────────────────────────────────────────────────┤                                                                               
  │ 1   │ refactor(runtime)           │ tunnel-lib 增加 build_proxy_runtime / build_single_thread_runtime │
  ├─────┼─────────────────────────────┼───────────────────────────────────────────────────────────────────┤                                                                               
  │ 2   │ refactor(metrics server)    │ hyper http1 重写，~200→~60 行，修复截断 bug                       │                                                                               
  ├─────┼─────────────────────────────┼───────────────────────────────────────────────────────────────────┤                                                                               
  │ 3   │ refactor(three runtimes)    │ main.rs 拆三 runtime，PrometheusBuilder 移至顶部                  │                                                                               
  ├─────┼─────────────────────────────┼───────────────────────────────────────────────────────────────────┤                                                                               
  │ 4   │ refactor(BackgroundService) │ hot_reload / control_client 统一 trait，支持 shutdown 信号        │
  ├─────┼─────────────────────────────┼───────────────────────────────────────────────────────────────────┤                                                                               
  │ 5   │ refactor(N-accept-loop)     │ 每个 listener 4 个并发 accept task，共享 Arc<TcpListener>         │
  ├─────┼─────────────────────────────┼───────────────────────────────────────────────────────────────────┤                                                                               
  │ 6   │ refactor(unify H2 path)     │ client/proxy.rs 删除 H2 特殊分支，统一走 ProxyEngine              │
  └─────┴─────────────────────────────┴───────────────────────────────────────────────────────────────────┘     