# 设计模式优化分析

## 当前架构问题

### 1. 转发模块 (Forwarder)
**当前实现：**
```rust
match protocol_type {
    "http" => http::forward_http_request(...).await,
    "wss" => wss::forward_wss_request(...).await,
    "grpc" => grpc::forward_grpc_request(...).await,
    _ => Err(...)
}
```

**问题：**
- 硬编码协议类型字符串
- 添加新协议需要修改多处代码
- 违反开闭原则（OCP）

**优化：策略模式 (Strategy Pattern)**
```rust
trait ForwardStrategy {
    async fn forward(&self, request: &[u8], target: &str) -> Result<Vec<u8>>;
}

struct HttpForwardStrategy;
struct WssForwardStrategy;
struct GrpcForwardStrategy;

struct Forwarder {
    strategies: HashMap<ProtocolType, Box<dyn ForwardStrategy>>,
}
```

### 2. 配置管理 (ConfigManager)
**当前实现：**
```rust
match payload {
    Payload::ConfigSyncResponse(resp) => handle_config_sync(...),
    Payload::HashResponse(resp) => handle_hash_response(...),
    Payload::IncrementalUpdate(update) => handle_incremental_update(...),
    _ => warn!("unexpected message")
}
```

**问题：**
- 消息处理逻辑耦合
- 难以测试单个消息处理器
- 违反单一职责原则（SRP）

**优化：责任链模式 (Chain of Responsibility)**
```rust
trait MessageHandler {
    fn can_handle(&self, payload: &Payload) -> bool;
    async fn handle(&self, payload: Payload) -> Result<()>;
}

struct ConfigSyncHandler;
struct HashResponseHandler;
struct IncrementalUpdateHandler;

struct MessageHandlerChain {
    handlers: Vec<Box<dyn MessageHandler>>,
}
```

### 3. 请求转换 (Request Transformation)
**当前实现：**
```rust
// HTTP 解析
let mut headers = [httparse::EMPTY_HEADER; 64];
let mut req = Request::new(&mut headers);
req.parse(request_bytes)?;

// 手动构建请求
let mut builder = hyper::Request::builder()
    .method(method)
    .uri(uri);
for header in req.headers.iter() {
    builder = builder.header(name, value);
}
```

**问题：**
- 解析和构建逻辑分散
- 重复代码多
- 难以复用

**优化：建造者模式 (Builder Pattern) + 适配器模式 (Adapter Pattern)**
```rust
struct HttpRequestBuilder {
    method: Option<String>,
    uri: Option<String>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpRequestBuilder {
    fn from_raw_bytes(bytes: &[u8]) -> Result<Self>;
    fn to_hyper_request(self) -> Result<hyper::Request>;
}
```

### 4. 路由匹配 (Rule Matching)
**当前实现：**
```rust
for rule in rules {
    if rule.r#type != rule_type { continue; }
    let rule_host = rule.match_host.split(':').next()...;
    if rule_host.eq_ignore_ascii_case(host) {
        return Ok(Some(rule));
    }
}
```

**问题：**
- 线性查找 O(n)
- 匹配逻辑分散
- 难以支持复杂规则

**优化：规则引擎 (Rule Engine) + 索引优化**
```rust
struct RuleMatcher {
    // 按协议类型索引
    by_protocol: HashMap<String, Vec<Rule>>,
    // 按 host 精确匹配索引
    by_exact_host: HashMap<String, Rule>,
    // 按 host 前缀索引
    by_prefix: Vec<(String, Rule)>,
}

impl RuleMatcher {
    fn match_rule(&self, protocol: &str, host: &str) -> Option<&Rule> {
        // 1. 精确匹配 O(1)
        // 2. 前缀匹配 O(log n)
        // 3. 正则匹配 O(n)
    }
}
```

### 5. 连接池管理 (EgressPool)
**当前实现：**
```rust
pub struct EgressPool {
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
}
```

**问题：**
- 单一客户端，无法针对不同 upstream 优化
- 无法实现熔断、限流
- 缺少健康检查

**优化：对象池模式 (Object Pool) + 装饰器模式 (Decorator)**
```rust
trait ConnectionPool {
    async fn get_connection(&self, target: &str) -> Result<Connection>;
    async fn return_connection(&self, conn: Connection);
}

struct PooledConnection {
    inner: Connection,
    pool: Arc<dyn ConnectionPool>,
}

struct CircuitBreakerDecorator {
    inner: Arc<dyn ConnectionPool>,
    state: Arc<CircuitBreakerState>,
}

struct RateLimitDecorator {
    inner: Arc<dyn ConnectionPool>,
    limiter: Arc<RateLimiter>,
}
```

### 6. 会话管理 (Session Management)
**当前实现：**
```rust
pub sessions: Arc<DashMap<u64, Arc<Mutex<SessionState>>>>,
```

**问题：**
- 无超时清理
- 无内存限制
- 可能内存泄漏

**优化：LRU 缓存 + 过期策略**
```rust
struct SessionManager {
    sessions: Arc<Mutex<LruCache<u64, SessionState>>>,
    max_size: usize,
    ttl: Duration,
}

impl SessionManager {
    async fn get_or_create(&self, id: u64) -> Arc<Mutex<SessionState>>;
    async fn cleanup_expired(&self);
}
```

## 推荐优化优先级

### 高优先级（立即实施）
1. **路由匹配优化** - 性能提升明显
2. **会话管理优化** - 防止内存泄漏

### 中优先级（逐步实施）
3. **转发策略模式** - 提升可扩展性
4. **请求转换建造者** - 减少重复代码

### 低优先级（可选）
5. **配置责任链** - 代码更清晰
6. **连接池装饰器** - 高级特性

## 具体实施方案

### 方案 1: 路由匹配优化（推荐先做）
**收益：**
- 性能提升 10-100x（O(n) → O(1) 或 O(log n)）
- 支持更复杂的匹配规则
- 代码更清晰

**工作量：** 1-2 小时

### 方案 2: 会话管理优化（推荐先做）
**收益：**
- 防止内存泄漏
- 自动清理过期会话
- 限制内存使用

**工作量：** 1 小时

### 方案 3: 转发策略模式
**收益：**
- 添加新协议更容易
- 每个协议独立测试
- 符合 SOLID 原则

**工作量：** 2-3 小时

## 代码示例

### 路由匹配优化
```rust
// 当前：O(n) 线性查找
for rule in rules {
    if matches(rule, request) {
        return Some(rule);
    }
}

// 优化：O(1) 哈希查找
let key = format!("{}:{}", protocol, host);
if let Some(rule) = exact_match.get(&key) {
    return Some(rule);
}
```

### 会话管理优化
```rust
// 当前：无限增长
sessions.insert(id, session);

// 优化：LRU + TTL
session_manager.insert_with_ttl(id, session, Duration::from_secs(300));
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        session_manager.cleanup_expired().await;
    }
});
```

### 转发策略模式
```rust
// 当前：硬编码
match protocol {
    "http" => http_forward(...),
    "wss" => wss_forward(...),
}

// 优化：策略模式
let strategy = forwarder.get_strategy(protocol)?;
strategy.forward(request, target).await?
```

## 总结

**最值得优化的 3 个点：**
1. 路由匹配（性能）
2. 会话管理（稳定性）
3. 转发策略（可扩展性）

**建议实施顺序：**
1. 路由匹配优化（立即）
2. 会话管理优化（立即）
3. 转发策略模式（下一步）
