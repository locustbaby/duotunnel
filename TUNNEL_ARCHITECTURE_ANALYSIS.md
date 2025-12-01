# Tunnel 架构分析文档

## 目录
1. [Client 启动逻辑](#client-启动逻辑)
2. [Server 启动逻辑](#server-启动逻辑)
3. [隧道建立流程](#隧道建立流程)
4. [核心数据结构](#核心数据结构)
5. [请求路由与响应机制](#请求路由与响应机制)

---

## Client 启动逻辑

### 1. 初始化阶段 (`client/main.rs`)

```42:95:client/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ClientConfig::load("../config/client.toml")?;
    // ... 日志初始化 ...
    
    let server_addr = format!("http://{}:{}", config.server_addr, config.server_port);
    let client_group_id = Arc::new(config.client_group_id.clone());
    
    // HTTP入口监听只启动一次，使用动态 tunnel_tx、pending_requests
    let tunnel_tx_holder = StdArc::new(RwLock::new(None));
    let pending_requests_holder = StdArc::new(RwLock::new(None));
    let client_id_holder = StdArc::new(RwLock::new(None));
    let token_holder = StdArc::new(RwLock::new(None::<CancellationToken>));
    let stream_id_holder = StdArc::new(RwLock::new(None));
    
    // 优雅退出信号监听
    let shutdown_token = CancellationToken::new();
    // ... 信号监听任务 ...
    
    // 启动 HTTP 入口监听，只启动一次
    // ... HTTP server 启动 ...
}
```

**关键点：**
- HTTP 入口监听器只启动一次，使用 `RwLock` 包装的动态 channel
- 使用 `JoinSet` 管理多个后台任务
- 支持优雅退出（Ctrl+C / SIGTERM）

### 2. 主循环 - 连接建立与重连 (`client/main.rs:144-259`)

```144:259:client/main.rs
    // main loop，每次重建所有资源
    let mut backoff = ExponentialBackoffBuilder::default()
        .with_max_interval(std::time::Duration::from_secs(10))
        .with_max_elapsed_time(None)
        .build();
    let mut first_attempt = true;
    loop {
        // 1. 新建 channel
        let (tx, rx) = mpsc::channel(128);
        // 2. 新建 TunnelClient
        let client_id = Arc::new(format!("client-{}", Uuid::new_v4()));
        let group_id = Arc::clone(&client_group_id);
        let tunnel_client = Arc::new(TunnelClient::new(
            Arc::clone(&client_id),
            Arc::clone(&group_id),
            server_addr.clone(),
            trace_enabled,
            tx.clone(),
        ));
        // 3. 更新 HTTP入口监听用的 tunnel_tx、pending_requests、client_id、token
        {
            let mut t = tunnel_tx_holder.write().await;
            *t = Some(tx.clone());
            let mut p = pending_requests_holder.write().await;
            *p = Some(tunnel_client.pending_requests.clone());
            // ... 更新 client_id, token ...
        }
        // 4. 建立 gRPC client
        match TunnelServiceClient::connect(server_addr.clone()).await {
            Ok(grpc_client) => {
                let token = shutdown_token.child_token();
                let stream_id = Uuid::new_v4().to_string();
                let tunnel_fut = tunnel_client.connect_with_retry_with_token(grpc_client, &mut rx, token.clone());
                tokio::select! {
                    res = tunnel_fut => {
                        // 处理连接结果
                    }
                    _ = shutdown_token.cancelled() => {
                        break;
                    }
                }
            }
            Err(e) => {
                // 重连逻辑
            }
        }
        // 清理资源并退避重试
    }
}
```

**流程：**
1. 创建新的 `mpsc::channel` (容量128)
2. 创建新的 `TunnelClient` 实例（包含 `client_id`, `group_id`）
3. 更新 HTTP 入口监听器使用的动态 channel
4. 连接 gRPC server
5. 调用 `connect_with_retry_with_token` 建立双向流
6. 连接断开后清理资源，使用指数退避重试

### 3. TunnelClient 连接逻辑 (`client/tunnel_client.rs:74-177`)

```74:177:client/tunnel_client.rs
    pub async fn connect_with_retry_with_token(&self, mut grpc_client: TunnelServiceClient<tonic::transport::Channel>, rx: &mut mpsc::Receiver<TunnelMessage>, token: CancellationToken) -> anyhow::Result<()> {
        loop {
            let child_token = token.child_token();
            let stream_id = Uuid::new_v4().to_string();
            let heartbeat_handle = tokio::spawn(Self::heartbeat_task(...));
            let config_sync_handle = tokio::spawn(Self::config_sync_task(...));
            let stream_type = StreamType::Http;
            let connect_msg = TunnelMessage {
                client_id: (*self.client_id).clone(),
                request_id: Uuid::new_v4().to_string(),
                direction: Direction::ClientToServer as i32,
                payload: Some(tunnel_lib::tunnel::tunnel_message::Payload::StreamOpen(
                    StreamOpenRequest {
                        client_id: (*self.client_id).clone(),
                        group: (*self.group_id).clone(),
                        version: "v1.0.0".to_string(),
                        stream_id: stream_id.clone(),
                        stream_type: stream_type as i32,
                        timestamp: chrono::Utc::now().timestamp(),
                    }
                )),
                trace_id: String::new(),
            };
            if let Err(e) = self.tx.send(connect_msg).await {
                return Err(anyhow::anyhow!("Failed to send StreamOpenRequest: {}", e));
            }
            let outbound = tokio_stream::wrappers::ReceiverStream::new(std::mem::replace(rx, mpsc::channel(10000).1));
            let response = grpc_client.proxy(Request::new(outbound)).await;
            match response {
                Ok(resp) => {
                    let mut inbound = resp.into_inner();
                    // 创建信号量限制并发（最大1000）
                    let semaphore = Arc::new(Semaphore::new(1000));
                    
                    while let Some(message) = inbound.next().await {
                        match message {
                            Ok(msg) => {
                                // 获取信号量许可
                                let permit = match semaphore.clone().try_acquire_owned() {
                                    Ok(p) => p,
                                    Err(_) => {
                                        semaphore.clone().acquire_owned().await?
                                    }
                                };
                                
                                // 并发处理消息
                                tokio::spawn(async move {
                                    client.handle_tunnel_message(msg, &tx).await;
                                    drop(permit);
                                });
                            }
                            Err(e) => {
                                // 错误处理
                            }
                        }
                    }
                }
                Err(e) => {
                    // 重连逻辑
                }
            }
            // 清理并重连
        }
    }
```

**关键点：**
- 发送 `StreamOpenRequest` 注册 stream
- 启动心跳任务（30秒间隔）
- 启动配置同步任务（30秒间隔）
- 使用信号量限制并发处理（最大1000）
- 使用 `tokio::spawn` 并发处理接收到的消息

---

## Server 启动逻辑

### 1. 初始化阶段 (`server/main.rs:29-60`)

```29:60:server/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::load("../config/server.toml")?;
    config.validate_rules()?;
    // ... 日志初始化 ...
    
    let rules_engine = Arc::new(RulesEngine::new(config.clone()));
    let https_client = Arc::new(Client::builder().build::<_, hyper::Body>(https));
    let pending_requests = Arc::new(DashMap::new());
    let tunnel_server = TunnelServer::new_with_config(&config, https_client.clone(), pending_requests.clone());
    let rules_engine = tunnel_server.rules_engine.clone();
    let client_registry = tunnel_server.client_registry.clone();
    let https_client = tunnel_server.https_client.clone();
}
```

**关键组件：**
- `RulesEngine`: 规则匹配引擎
- `ManagedClientRegistry`: 客户端注册表
- `pending_requests`: 等待响应的请求映射
- `https_client`: HTTP/HTTPS 客户端

### 2. 启动三个服务 (`server/main.rs:61-151`)

#### 2.1 HTTP 入口监听 (`server/main.rs:86-108`)

```86:108:server/main.rs
    // HTTP 入口监听
    join_set.spawn(async move {
        let target = target.clone();
        let ctx = ctx.clone();
        let make_svc = make_service_fn(move |_| {
            let ctx = ctx.clone();
            let target = target.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let ctx = ctx.clone();
                    let target = target.clone();
                    async move {
                        http_entry_handler::<ServerHttpEntryTarget>(req, &ctx, &*target).await
                    }
                }))
            }
        });
        let server = HyperServer::bind(&http_addr).serve(make_svc);
        info!("HTTP entry listening on http://{} (tunnel-lib handler)", http_addr);
        if let Err(e) = server.await {
            error!("HTTP entry server error: {}", e);
        }
    });
```

#### 2.2 gRPC 入口监听（存根）(`server/main.rs:110-140`)

#### 2.3 Tunnel 控制端口监听 (`server/main.rs:142-151`)

```142:151:server/main.rs
    // tunnel 控制端口监听
    let tunnel_port = config.server.tunnel_port;
    let tunnel_addr: SocketAddr = format!("0.0.0.0:{}", tunnel_port).parse()?;
    let svc = tunnel_lib::tunnel::tunnel_service_server::TunnelServiceServer::new(tunnel_server);
    info!("gRPC tunnel server listening on {} (tunnel control)", tunnel_addr);
    join_set.spawn(async move {
        if let Err(e) = tonic::transport::Server::builder().add_service(svc).serve(tunnel_addr).await {
            error!("Tunnel server error: {}", e);
        }
    });
```

### 3. 健康检查清理任务 (`server/main.rs:79-84`)

```79:84:server/main.rs
    // 启动健康检查清理任务
    let cleanup_registry = client_registry.clone();
    let cleanup_token = shutdown_token.clone();
    join_set.spawn(async move {
        cleanup_registry.cleanup_task(30, 60, cleanup_token).await;
    });
```

**功能：**
- 每30秒扫描一次
- 清理60秒内无心跳的 stream
- 检查 `CancellationToken` 是否已取消

---

## 隧道建立流程

### 1. Client 发起连接

```
Client                          Server
  |                                |
  |-- StreamOpenRequest ---------->|
  |   (client_id, group, stream_id)|
  |                                |
  |<-- StreamOpenResponse ---------|
  |   (success: true)              |
  |                                |
  |-- Heartbeat (每30秒) ---------->|
  |-- ConfigSync (每30秒) --------->|
  |                                |
```

### 2. Server 处理 StreamOpen (`server/tunnel_server.rs:98-266`)

```98:266:server/tunnel_server.rs
    async fn proxy(
        &self,
        request: Request<tonic::Streaming<TunnelMessage>>,
    ) -> Result<Response<Self::ProxyStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel::<Result<TunnelMessage, Status>>(10000);
        let pending_requests = self.pending_requests.clone();
        let rules_engine = self.rules_engine.clone();
        let client_registry = self.client_registry.clone();
        let https_client = self.https_client.clone();
        let token = CancellationToken::new();
        tokio::spawn(async move {
            let mut client_id = String::new();
            let mut group = String::new();
            let mut stream_id = String::new();
            let mut stream_type = StreamType::Unspecified;
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        client_id = msg.client_id.clone();
                        match msg.payload {
                            Some(tunnel_message::Payload::StreamOpen(ref req)) => {
                                stream_id = req.stream_id.clone();
                                group = req.group.clone();
                                stream_type = StreamType::from_i32(req.stream_type).unwrap_or(StreamType::Unspecified);
                                let (ctx, mut crx) = mpsc::channel::<TunnelMessage>(128);
                                client_registry.sync_stream(
                                    &req.client_id,
                                    &req.group,
                                    &req.stream_id,
                                    stream_type,
                                    ctx.clone(),
                                    token.clone(),
                                );
                                // 回复 StreamOpenResponse
                                let response = TunnelMessage {
                                    client_id: req.client_id.clone(),
                                    request_id: msg.request_id.clone(),
                                    direction: Direction::ServerToClient as i32,
                                    payload: Some(tunnel_message::Payload::StreamOpenResponse(StreamOpenResponse {
                                        success: true,
                                        message: "stream registered/heartbeat ok".to_string(),
                                        timestamp: chrono::Utc::now().timestamp(),
                                    })),
                                    trace_id: msg.trace_id.clone(),
                                };
                                let _ = ctx.send(response).await;
                            }
                            // ... 处理其他消息类型 ...
                        }
                    }
                    Err(_) => {
                        // 清理资源
                        break;
                    }
                }
            }
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
```

**关键步骤：**
1. Server 接收 gRPC 双向流
2. 处理 `StreamOpen` 消息，调用 `client_registry.sync_stream()` 注册
3. 创建内部 channel `(ctx, crx)` 用于向该 stream 发送消息
4. 回复 `StreamOpenResponse`

---

## 核心数据结构

### 1. ManagedClientRegistry (`server/registry.rs`)

```14:19:server/registry.rs
// group -> stream_type -> (client_id, stream_id) -> (tx, CancellationToken, LastHeartbeat)
pub type ConnectedStreams = DashMap<String, DashMap<StreamType, DashMap<StreamKey, (mpsc::Sender<TunnelMessage>, CancellationToken, LastHeartbeat)>>>;

pub struct ManagedClientRegistry {
    pub connected_streams: ConnectedStreams,
}
```

**数据结构层次：**
```
ConnectedStreams (DashMap)
└── group (String)
    └── DashMap<StreamType, ...>
        └── StreamType (Http/Grpc/...)
            └── DashMap<StreamKey, ...>
                └── StreamKey: (client_id, stream_id)
                    └── (tx: mpsc::Sender<TunnelMessage>, token: CancellationToken, last_heartbeat: u64)
```

**关键方法：**

#### `sync_stream()` - 注册/更新 stream

```28:34:server/registry.rs
    pub fn sync_stream(&self, client_id: &str, group: &str, stream_id: &str, stream_type: StreamType, tx: mpsc::Sender<TunnelMessage>, token: CancellationToken) {
        let now = chrono::Utc::now().timestamp() as u64;
        let stream_map = self.connected_streams.entry(group.to_string()).or_insert_with(DashMap::new);
        let client_map = stream_map.entry(stream_type).or_insert_with(DashMap::new);
        client_map.insert((client_id.to_string(), stream_id.to_string()), (tx, token, now));
    }
```

#### `get_healthy_streams_in_group()` - 获取健康的 stream

```36:58:server/registry.rs
    pub fn get_healthy_streams_in_group(&self, group: &str, stream_type: Option<StreamType>, timeout_secs: u64) -> Vec<(String, StreamType, String)> {
        let now = chrono::Utc::now().timestamp() as u64;
        let mut result = Vec::new();
        if let Some(stream_map) = self.connected_streams.get(group) {
            let stream_types: Vec<StreamType> = if let Some(st) = stream_type {
                vec![st]
            } else {
                stream_map.iter().map(|entry| *entry.key()).collect()
            };
            for st in stream_types {
                if let Some(client_map) = stream_map.get(&st) {
                    for entry in client_map.iter() {
                        let ((client_id, stream_id), (tx, token, last_heartbeat)) = entry.pair();
                        if now - *last_heartbeat < timeout_secs && !token.is_cancelled() {
                            result.push((client_id.clone(), st, stream_id.clone()));
                        }
                    }
                }
            }
        }
        result
    }
```

#### `get_stream_info()` - 获取指定 stream 的信息

```60:67:server/registry.rs
    pub fn get_stream_info(&self, group: &str, stream_type: StreamType, client_id: &str, stream_id: &str) -> Option<(mpsc::Sender<TunnelMessage>, CancellationToken, LastHeartbeat)> {
        let stream_map = self.connected_streams.get(group)?;
        let client_map = stream_map.get(&stream_type)?;
        let entry = client_map.get(&(client_id.to_string(), stream_id.to_string()))?;
        let (tx, token, last_heartbeat) = entry.value();
        Some((tx.clone(), token.clone(), *last_heartbeat))
    }
```

### 2. TunnelClient (`client/tunnel_client.rs`)

```25:35:client/tunnel_client.rs
#[derive(Clone)]
pub struct TunnelClient {
    pub client_id: Arc<String>,
    pub group_id: Arc<String>,
    pub server_addr: String,
    pub tx: mpsc::Sender<TunnelMessage>,
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    pub rules_engine: Arc<Mutex<ClientRulesEngine>>,
    pub trace_enabled: bool,
    pub https_client: Arc<Client<HttpsConnector<HttpConnector>>>,
}
```

**关键字段：**
- `tx`: 发送 `TunnelMessage` 到 server 的 channel
- `pending_requests`: 等待响应的请求映射 `request_id -> oneshot::Sender<HttpResponse>`

### 3. TunnelServer (`server/tunnel_server.rs`)

```25:31:server/tunnel_server.rs
#[derive(Clone)]
pub struct TunnelServer {
    pub rules_engine: Arc<RulesEngine>,
    pub client_registry: Arc<ManagedClientRegistry>,
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    pub https_client: Arc<Client<HttpsConnector<HttpConnector>>>,
}
```

---

## 请求路由与响应机制

### 1. 外部访问 Server 时的路由流程

#### 1.1 HTTP 入口处理 (`server/proxy.rs:22-96`)

```22:96:server/proxy.rs
    async fn handle(
        &self,
        req: HyperRequest<Body>,
        _ctx: &HttpTunnelContext,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("");
        let path = req.uri().path();
        if let Some(rule) = self.rules_engine.match_reverse_proxy_rule(host, path, None) {
            if let Some(group) = &rule.action_client_group {
                let stream_type = tunnel_lib::tunnel::StreamType::Http;
                let healthy_streams = self.client_registry.get_healthy_streams_in_group(
                    group,
                    Some(stream_type),
                    60,
                );
                if healthy_streams.is_empty() {
                    return Ok(/* 502 error */);
                }
                for (client_id, _stream_type, stream_id) in healthy_streams {
                    if let Some((tx, token, _last_heartbeat)) = self.client_registry.get_stream_info(group, StreamType::Http, &client_id, &stream_id) {
                        if !token.is_cancelled() {
                            let trace_id = /* ... */;
                            let request_id = Uuid::new_v4().to_string();
                            return forward_http_via_tunnel(
                                req,
                                &client_id,
                                &tx,
                                self.pending_requests.clone(),
                                request_id,
                                tunnel_lib::tunnel::Direction::ServerToClient,
                                stream_id.clone(),
                            ).await;
                        }
                    }
                }
            }
        }
        // 404 error
    }
```

**流程：**
1. 匹配反向代理规则（根据 host/path）
2. 获取规则指定的 `client_group`
3. 从 `client_registry` 获取该 group 下所有健康的 stream
4. 遍历健康的 stream，选择第一个可用的
5. 调用 `get_stream_info()` 获取该 stream 的 `tx`
6. 调用 `forward_http_via_tunnel()` 发送请求

#### 1.2 通过 Tunnel 发送请求 (`tunnel-lib/src/http_forward.rs:19-87`)

```19:87:tunnel-lib/src/http_forward.rs
pub async fn forward_http_via_tunnel(
    http_req: HyperRequest<Body>,
    client_id: &str,
    tunnel_sender: &mpsc::Sender<TunnelMessage>,
    pending_map: Arc<DashMap<String, oneshot::Sender<HttpResponse>>>,
    request_id: String,
    direction: Direction,
    stream_id: String,
) -> Result<hyper::Response<Body>, hyper::Error> {
    // 1. 序列化 HTTP 请求
    let (parts, body) = http_req.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.unwrap_or_default();
    let http_request = HttpRequest {
        stream_id: stream_id.clone(),
        method: parts.method.to_string(),
        url: parts.uri.to_string(),
        // ... 其他字段 ...
    };
    
    // 2. 构建 TunnelMessage
    let tunnel_msg = build_http_tunnel_message(
        client_id,
        &request_id,
        direction,
        http_request,
        &trace_id,
    );
    
    // 3. 创建 oneshot channel 等待响应
    let (tx, rx) = oneshot::channel();
    pending_map.insert(request_id.clone(), tx);
    
    // 4. 发送到 tunnel
    if let Err(e) = tunnel_sender.send(tunnel_msg).await {
        pending_map.remove(&request_id);
        return Ok(/* 502 error */);
    }
    
    // 5. 等待响应（30秒超时）
    match timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(resp)) => {
            // 转换为 HyperResponse
            let mut builder = hyper::Response::builder().status(resp.status_code as u16);
            for (k, v) in resp.headers {
                builder = builder.header(k, v);
            }
            Ok(builder.body(Body::from(resp.body)).unwrap())
        }
        Ok(Err(_)) => {
            Ok(/* 502 error */)
        }
        Err(_) => {
            pending_map.remove(&request_id);
            Ok(/* timeout error */)
        }
    }
}
```

**关键机制：**
- 使用 `oneshot::channel` 实现请求-响应配对
- `request_id` 作为唯一标识
- `pending_map` 存储 `request_id -> oneshot::Sender<HttpResponse>`
- 30秒超时保护

### 2. Client 接收请求并响应

#### 2.1 Client 接收 Tunnel 消息 (`client/tunnel_client.rs:246-344`)

```246:344:client/tunnel_client.rs
    pub async fn handle_tunnel_message(&self, msg: TunnelMessage, tx: &mpsc::Sender<TunnelMessage>) {
        match msg.payload {
            Some(tunnel_lib::tunnel::tunnel_message::Payload::HttpRequest(req)) => {
                // 处理来自 server 的 HTTP 请求（反向代理）
                if msg.direction == Direction::ServerToClient as i32 {
                    let response = self.handle_tunnel_http_request(req, request_id.clone(), trace_id.clone()).await;
                    let response_msg = TunnelMessage {
                        client_id: (*self.client_id).clone(),
                        request_id: request_id.clone(),
                        direction: Direction::ClientToServer as i32,
                        payload: Some(tunnel_lib::tunnel::tunnel_message::Payload::HttpResponse(response)),
                        trace_id: trace_id.clone(),
                    };
                    if let Err(e) = tx.send(response_msg).await {
                        // 错误处理
                    }
                }
            }
            Some(tunnel_lib::tunnel::tunnel_message::Payload::HttpResponse(resp)) => {
                // 收到响应，通过 oneshot channel 发送
                if let Some((_, sender)) = self.pending_requests.remove(&msg.request_id) {
                    let _ = sender.send(resp);
                }
            }
            // ... 其他消息类型 ...
        }
    }
```

**流程：**
1. Client 从 gRPC stream 接收 `HttpRequest` 消息
2. 调用 `handle_tunnel_http_request()` 处理请求（匹配规则、转发到后端）
3. 构建 `HttpResponse` 消息，使用相同的 `request_id`
4. 通过 `tx` 发送回 server

#### 2.2 Server 接收响应 (`server/tunnel_server.rs:163-168`)

```163:168:server/tunnel_server.rs
                            Some(tunnel_message::Payload::HttpResponse(resp)) => {
                                if msg.direction == Direction::ClientToServer as i32 {
                                    if let Some((_, sender)) = pending_requests.remove(&msg.request_id) {
                                        let _ = sender.send(resp);
                                    }
                                }
                            }
```

**流程：**
1. Server 从 gRPC stream 接收 `HttpResponse` 消息
2. 根据 `request_id` 从 `pending_requests` 中找到对应的 `oneshot::Sender`
3. 通过 `oneshot::Sender` 发送响应
4. `forward_http_via_tunnel()` 中的 `rx.await` 被唤醒，返回响应

### 3. 外部访问 Client 时的路由流程

#### 3.1 Client HTTP 入口处理 (`client/proxy.rs:20-75`)

```20:75:client/proxy.rs
    async fn handle(
        &self,
        req: HyperRequest<Body>,
        _ctx: &HttpTunnelContext,
    ) -> Result<HyperResponse<Body>, hyper::Error> {
        // 动态获取当前活跃 channel
        let tunnel_tx = self.tunnel_tx.read().await.clone();
        let pending_requests_opt = self.pending_requests.read().await.clone();
        let client_id = self.client_id.read().await.clone().unwrap_or_else(|| "unknown".to_string());
        let stream_id = self.stream_id.read().await.clone().unwrap_or_else(|| "default-stream".to_string());
        if let Some(tunnel_tx) = tunnel_tx {
            if let Some(pending_requests) = pending_requests_opt {
                let request_id = Uuid::new_v4().to_string();
                return forward_http_via_tunnel(
                    req,
                    &client_id,
                    &tunnel_tx,
                    pending_requests,
                    request_id,
                    tunnel_lib::tunnel::Direction::ClientToServer,
                    stream_id,
                ).await;
            }
        }
        // 502 error
    }
```

**流程：**
1. 从 `RwLock` 中读取当前活跃的 `tunnel_tx` 和 `pending_requests`
2. 调用 `forward_http_via_tunnel()` 发送请求到 server
3. Server 端处理请求（匹配 forward 规则，转发到 upstream）
4. Server 通过 tunnel 返回响应
5. Client 接收响应并通过 `oneshot` channel 返回给 HTTP 客户端

### 4. Server 处理 Forward 规则 (`server/tunnel_server.rs:212-247`)

```212:247:server/tunnel_server.rs
                            Some(tunnel_message::Payload::HttpRequest(ref req)) => {
                                let host = req.host.as_str();
                                let path = req.url.split('?').next().unwrap_or("/");
                                let mut response = error_response(/* ... */);
                                if let Some(rule) = rules_engine.match_forward_rule(host, path, None) {
                                    if let Some(ref upstream_name) = rule.action_upstream {
                                        if let Some(upstream) = rules_engine.get_upstream(upstream_name) {
                                            if let Some(backend) = pick_backend(upstream) {
                                                let set_host = rule.action_set_host.as_deref().unwrap_or("");
                                                response = forward_http_to_backend(
                                                    req,
                                                    &backend,
                                                    https_client.clone(),
                                                    set_host
                                                ).await;
                                            }
                                        }
                                    }
                                }
                                let response_msg = TunnelMessage {
                                    client_id: msg.client_id.clone(),
                                    request_id: msg.request_id.clone(),
                                    direction: Direction::ServerToClient as i32,
                                    payload: Some(tunnel_message::Payload::HttpResponse(response)),
                                    trace_id: msg.trace_id.clone(),
                                };
                                if let Some((tx, _, _)) = client_registry.get_stream_info(&group, stream_type, &client_id, &stream_id) {
                                    let _ = tx.send(response_msg).await;
                                }
                            }
```

**流程：**
1. 匹配 forward 规则
2. 从 upstream 中选择 backend
3. 调用 `forward_http_to_backend()` 转发到后端
4. 构建 `HttpResponse` 消息，使用相同的 `request_id`
5. 通过 `get_stream_info()` 获取 client 的 `tx`
6. 发送响应回 client

---

## 总结

### 数据流图

```
外部请求 -> HTTP入口 -> 规则匹配 -> 选择client/stream -> 获取tx
    |
    v
forward_http_via_tunnel
    |
    v
创建 oneshot channel -> 发送 TunnelMessage -> 等待响应
    |
    v
TunnelMessage (gRPC stream) -> Client/Server 处理 -> 返回 HttpResponse
    |
    v
通过 request_id 找到 oneshot::Sender -> 发送响应 -> HTTP 响应返回
```

### 关键数据结构关系

```
Server:
  ManagedClientRegistry
    └── connected_streams: group -> stream_type -> (client_id, stream_id) -> (tx, token, heartbeat)
  pending_requests: request_id -> oneshot::Sender<HttpResponse>

Client:
  TunnelClient
    ├── tx: mpsc::Sender<TunnelMessage> (发送到 server)
    └── pending_requests: request_id -> oneshot::Sender<HttpResponse>
```

### 请求-响应配对机制

1. **请求发送方**：
   - 生成 `request_id` (UUID)
   - 创建 `oneshot::channel`
   - 将 `(request_id, oneshot::Sender)` 存入 `pending_requests`
   - 发送 `TunnelMessage`（包含 `request_id`）
   - 等待 `oneshot::Receiver` 接收响应

2. **响应接收方**：
   - 接收 `TunnelMessage`（包含 `request_id`）
   - 处理请求
   - 构建 `HttpResponse`，使用相同的 `request_id`
   - 从 `pending_requests` 中找到对应的 `oneshot::Sender`
   - 发送响应

3. **超时保护**：
   - 使用 `tokio::time::timeout(30秒)` 保护
   - 超时后清理 `pending_requests` 中的条目

