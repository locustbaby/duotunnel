# Envoy 反向隧道 vs 当前实现对比分析

## 执行摘要

通过深入研究 Envoy 的反向隧道实现，发现了多个关键设计优势，可以显著改进当前的实现。

### 🎯 核心差异总结

| 维度 | Envoy 实现 | 当前实现 | 优化建议 |
|------|-----------|---------|---------|
| **连接模型** | HTTP/2 CONNECT + 连接池 | QUIC 双向流 | ✅ 保持QUIC，但借鉴连接池设计 |
| **节点标识** | 三层ID体系 (node/cluster/tenant) | 单层client_id | ⚠️ 需要改进为多层标识 |
| **路由机制** | host_id动态提取 + 格式化器 | 静态规则匹配 | ⚠️ 需要动态路由能力 |
| **连接复用** | 连接池 + 负载均衡 | 单连接 | ⚠️ 需要连接池 |
| **协议支持** | 仅HTTP/2 | HTTP/1.1, gRPC, WebSocket | ✅ 更灵活，但需优化 |
| **安全模型** | mTLS + 握手验证 | 自签名证书 | ⚠️ 需要加强认证 |
| **配置方式** | 声明式YAML | 代码硬编码 | ⚠️ 需要配置化 |

---

## 详细对比分析

### 1. 节点标识和分组机制 ⭐⭐⭐

#### Envoy 的三层标识体系

```yaml
# Envoy 使用 rc:// 格式编码节点信息
address: "rc://downstream-node:downstream-cluster:downstream-tenant@upstream-cluster:1"
```

**三层标识：**
1. **src_node_id** (downstream-node) - 唯一节点标识
   - 每个Envoy实例必须全局唯一
   - 用于精确路由到特定节点
   
2. **src_cluster_id** (downstream-cluster) - 集群标识
   - 多个节点可共享同一cluster_id
   - 用于负载均衡（请求可分发到cluster内任意节点）
   - **关键约束**: cluster_id 不能与任何 node_id 冲突
   
3. **src_tenant_id** (downstream-tenant) - 租户标识
   - 多租户隔离
   - 资源配额和访问控制

#### 当前实现

```rust
// client/main.rs
let client_id = Uuid::new_v4().to_string();

// 只有单层标识，缺少集群和租户概念
```

**问题：**
- ❌ 无法实现负载均衡（没有cluster概念）
- ❌ 无法多租户隔离
- ❌ 无法灵活路由（只能按client_id精确匹配）

#### 优化方案

```rust
// tunnel-lib/src/identity.rs

/// 节点标识体系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// 节点唯一ID (必须全局唯一)
    pub node_id: String,
    
    /// 集群ID (多个节点可共享，用于负载均衡)
    pub cluster_id: String,
    
    /// 租户ID (多租户隔离)
    pub tenant_id: String,
    
    /// 节点元数据
    pub metadata: HashMap<String, String>,
}

impl NodeIdentity {
    /// 从环境变量或配置文件加载
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(Self {
            node_id: config.node_id.clone()
                .or_else(|| hostname::get().ok()?.into_string().ok())
                .ok_or_else(|| anyhow!("node_id required"))?,
            cluster_id: config.cluster_id.clone()
                .unwrap_or_else(|| "default-cluster".to_string()),
            tenant_id: config.tenant_id.clone()
                .unwrap_or_else(|| "default-tenant".to_string()),
            metadata: config.metadata.clone(),
        })
    }
    
    /// 编码为字符串 (类似Envoy的rc://格式)
    pub fn encode(&self) -> String {
        format!("{}:{}:{}", self.node_id, self.cluster_id, self.tenant_id)
    }
    
    /// 从字符串解码
    pub fn decode(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            bail!("Invalid identity format: {}", s);
        }
        Ok(Self {
            node_id: parts[0].to_string(),
            cluster_id: parts[1].to_string(),
            tenant_id: parts[2].to_string(),
            metadata: HashMap::new(),
        })
    }
    
    /// 验证标识合法性
    pub fn validate(&self) -> Result<()> {
        // 确保 node_id 和 cluster_id 不冲突
        if self.node_id == self.cluster_id {
            bail!("node_id and cluster_id must be different");
        }
        Ok(())
    }
}

/// 客户端注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistration {
    pub identity: NodeIdentity,
    pub capabilities: Vec<String>,  // 支持的协议: ["http", "grpc", "wss"]
    pub labels: HashMap<String, String>,  // 用于高级路由
}
```

**使用示例：**

```rust
// client/main.rs
let identity = NodeIdentity {
    node_id: format!("client-{}", hostname::get()?),
    cluster_id: "edge-cluster-us-west".to_string(),
    tenant_id: "customer-123".to_string(),
    metadata: HashMap::from([
        ("region".to_string(), "us-west-1".to_string()),
        ("zone".to_string(), "az-1".to_string()),
    ]),
};

identity.validate()?;

// 注册时发送完整标识
let registration = ClientRegistration {
    identity: identity.clone(),
    capabilities: vec!["http".into(), "grpc".into()],
    labels: HashMap::from([
        ("env".to_string(), "production".to_string()),
    ]),
};
```

**收益：**
- ✅ 支持负载均衡（请求可路由到cluster内任意节点）
- ✅ 多租户隔离
- ✅ 灵活的路由策略（按node/cluster/tenant/labels）

---

### 2. 动态路由和host_id提取 ⭐⭐⭐

#### Envoy 的动态路由机制

```yaml
# Envoy 使用格式化器动态提取 host_id
reverse_connection_cluster:
  host_id_format: "%REQ(x-computed-host-id)%"
  # 或者组合多个来源
  # host_id_format: "%DYNAMIC_METADATA(routing:target_node)%"
  # host_id_format: "%FILTER_STATE(target_cluster)%"
```

**支持的格式化器：**
- `%REQ(header-name)%` - 从请求头提取
- `%DYNAMIC_METADATA(namespace:key)%` - 从动态元数据提取
- `%FILTER_STATE(key)%` - 从过滤器状态提取
- `%DOWNSTREAM_REMOTE_ADDRESS%` - 使用下游地址
- 支持组合和默认值

**路由逻辑：**
1. 从请求中提取 `host_id`
2. 先查找是否有该名称的 cluster
3. 如果没有，则作为 node_id 查找
4. 从连接池中获取对应的反向连接

#### 当前实现

```rust
// server/data_stream.rs
let routing_info = RoutingInfo::decode(&routing_frame.payload)?;
let host = routing_info.host.as_str();

// 静态规则匹配
let matched_upstream = matcher.match_egress_http_rule(&host)
    .map(|r| r.action_upstream);
```

**问题：**
- ❌ 只能基于静态规则路由
- ❌ 无法动态选择目标节点
- ❌ 无法根据请求内容路由

#### 优化方案

```rust
// tunnel-lib/src/routing.rs

/// 路由目标
#[derive(Debug, Clone)]
pub enum RoutingTarget {
    /// 路由到特定节点
    Node(String),
    /// 路由到集群（负载均衡）
    Cluster(String),
    /// 路由到租户（多租户场景）
    Tenant(String),
}

/// 路由提取器（类似Envoy的formatter）
pub trait RoutingExtractor: Send + Sync {
    fn extract(&self, context: &RoutingContext) -> Result<RoutingTarget>;
}

/// 路由上下文
pub struct RoutingContext {
    pub headers: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
    pub path: String,
    pub method: String,
}

/// 从请求头提取
pub struct HeaderExtractor {
    header_name: String,
    target_type: TargetType,
}

impl RoutingExtractor for HeaderExtractor {
    fn extract(&self, context: &RoutingContext) -> Result<RoutingTarget> {
        let value = context.headers.get(&self.header_name)
            .ok_or_else(|| anyhow!("Header {} not found", self.header_name))?;
        
        match self.target_type {
            TargetType::Node => Ok(RoutingTarget::Node(value.clone())),
            TargetType::Cluster => Ok(RoutingTarget::Cluster(value.clone())),
            TargetType::Tenant => Ok(RoutingTarget::Tenant(value.clone())),
        }
    }
}

/// 基于路径的提取器
pub struct PathExtractor {
    pattern: Regex,
    group_index: usize,
}

impl RoutingExtractor for PathExtractor {
    fn extract(&self, context: &RoutingContext) -> Result<RoutingTarget> {
        let captures = self.pattern.captures(&context.path)
            .ok_or_else(|| anyhow!("Path pattern not matched"))?;
        
        let value = captures.get(self.group_index)
            .ok_or_else(|| anyhow!("Capture group not found"))?
            .as_str();
        
        Ok(RoutingTarget::Node(value.to_string()))
    }
}

/// 组合提取器（支持fallback）
pub struct CompositeExtractor {
    extractors: Vec<Box<dyn RoutingExtractor>>,
}

impl RoutingExtractor for CompositeExtractor {
    fn extract(&self, context: &RoutingContext) -> Result<RoutingTarget> {
        for extractor in &self.extractors {
            if let Ok(target) = extractor.extract(context) {
                return Ok(target);
            }
        }
        bail!("No extractor succeeded")
    }
}

/// 路由管理器
pub struct RoutingManager {
    extractor: Box<dyn RoutingExtractor>,
    client_registry: Arc<ClientRegistry>,
}

impl RoutingManager {
    pub async fn route(&self, context: &RoutingContext) -> Result<Arc<Connection>> {
        // 1. 提取路由目标
        let target = self.extractor.extract(context)?;
        
        // 2. 根据目标类型查找连接
        match target {
            RoutingTarget::Node(node_id) => {
                // 精确路由到节点
                self.client_registry.get_connection_by_node(&node_id)
                    .ok_or_else(|| anyhow!("Node {} not found", node_id))
            }
            RoutingTarget::Cluster(cluster_id) => {
                // 负载均衡到集群内节点
                self.client_registry.get_connection_by_cluster(&cluster_id)
                    .ok_or_else(|| anyhow!("Cluster {} not found", cluster_id))
            }
            RoutingTarget::Tenant(tenant_id) => {
                // 路由到租户（可能需要额外的租户隔离逻辑）
                self.client_registry.get_connection_by_tenant(&tenant_id)
                    .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))
            }
        }
    }
}
```

**配置示例：**

```yaml
# config/server.yaml
routing:
  # 从请求头提取目标节点
  - type: header
    header_name: "X-Target-Node"
    target_type: node
  
  # 从路径提取集群ID
  - type: path
    pattern: "/api/v1/clusters/([^/]+)/.*"
    group_index: 1
    target_type: cluster
  
  # 默认路由到集群
  - type: default
    cluster_id: "default-cluster"
```

**使用示例：**

```rust
// server/ingress_handlers.rs
let routing_context = RoutingContext {
    headers: extract_headers(&request),
    metadata: HashMap::new(),
    path: request.path.clone(),
    method: request.method.clone(),
};

let connection = routing_manager.route(&routing_context).await?;
let (send, recv) = connection.open_bi().await?;
```

---

### 3. 连接池和负载均衡 ⭐⭐⭐

#### Envoy 的连接池设计

```yaml
# Envoy 维护到每个upstream cluster的连接池
reverse_tunnel_listener:
  address: "rc://node:cluster:tenant@upstream-cluster:3"
  # connection_count: 3 表示维护3个连接到upstream-cluster
```

**特点：**
- 每个cluster维护多个连接
- 自动负载均衡
- 连接健康检查
- 连接重连机制

#### 当前实现

```rust
// client/types.rs
pub struct ClientState {
    pub quic_connection: Arc<RwLock<Option<Arc<Connection>>>>,
    // 只有单个连接
}
```

**问题：**
- ❌ 单点故障（连接断开则服务不可用）
- ❌ 无法负载均衡
- ❌ 无法扩展吞吐量

#### 优化方案

```rust
// tunnel-lib/src/connection_pool.rs

/// 连接池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 每个upstream的最小连接数
    pub min_connections: usize,
    /// 每个upstream的最大连接数
    pub max_connections: usize,
    /// 空闲连接超时
    pub idle_timeout: Duration,
    /// 连接最大生命周期
    pub max_lifetime: Duration,
    /// 健康检查间隔
    pub health_check_interval: Duration,
}

/// 连接池条目
struct PooledConnection {
    connection: Arc<Connection>,
    created_at: Instant,
    last_used: Instant,
    active_streams: AtomicUsize,
}

/// 连接池
pub struct ConnectionPool {
    config: PoolConfig,
    pools: Arc<DashMap<String, Vec<PooledConnection>>>,
    endpoint: Endpoint,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig, endpoint: Endpoint) -> Self {
        let pool = Self {
            config,
            pools: Arc::new(DashMap::new()),
            endpoint,
        };
        
        // 启动健康检查任务
        pool.start_health_check();
        
        pool
    }
    
    /// 获取连接（负载均衡）
    pub async fn get_connection(&self, upstream: &str) -> Result<Arc<Connection>> {
        let mut entry = self.pools.entry(upstream.to_string())
            .or_insert_with(Vec::new);
        
        // 1. 尝试复用现有连接
        if let Some(conn) = self.find_available_connection(&entry) {
            conn.last_used = Instant::now();
            conn.active_streams.fetch_add(1, Ordering::Relaxed);
            return Ok(conn.connection.clone());
        }
        
        // 2. 如果未达到最大连接数，创建新连接
        if entry.len() < self.config.max_connections {
            let connection = self.create_connection(upstream).await?;
            let pooled = PooledConnection {
                connection: Arc::new(connection),
                created_at: Instant::now(),
                last_used: Instant::now(),
                active_streams: AtomicUsize::new(1),
            };
            let conn = pooled.connection.clone();
            entry.push(pooled);
            return Ok(conn);
        }
        
        // 3. 等待可用连接（简单实现：选择活跃流最少的）
        let conn = entry.iter_mut()
            .min_by_key(|c| c.active_streams.load(Ordering::Relaxed))
            .ok_or_else(|| anyhow!("No connections available"))?;
        
        conn.last_used = Instant::now();
        conn.active_streams.fetch_add(1, Ordering::Relaxed);
        Ok(conn.connection.clone())
    }
    
    /// 释放连接
    pub fn release_connection(&self, connection: &Arc<Connection>) {
        for mut entry in self.pools.iter_mut() {
            for conn in entry.value_mut().iter_mut() {
                if Arc::ptr_eq(&conn.connection, connection) {
                    conn.active_streams.fetch_sub(1, Ordering::Relaxed);
                    return;
                }
            }
        }
    }
    
    /// 查找可用连接
    fn find_available_connection(&self, connections: &[PooledConnection]) -> Option<&mut PooledConnection> {
        connections.iter_mut()
            .filter(|c| {
                // 检查连接是否仍然有效
                !c.connection.close_reason().is_some() &&
                // 检查是否超过最大生命周期
                c.created_at.elapsed() < self.config.max_lifetime
            })
            .min_by_key(|c| c.active_streams.load(Ordering::Relaxed))
    }
    
    /// 创建新连接
    async fn create_connection(&self, upstream: &str) -> Result<Connection> {
        let addr = upstream.parse()?;
        let connection = self.endpoint.connect(addr, "server")?.await?;
        Ok(connection)
    }
    
    /// 启动健康检查
    fn start_health_check(&self) {
        let pools = self.pools.clone();
        let interval = self.config.health_check_interval;
        let idle_timeout = self.config.idle_timeout;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                
                // 清理过期连接
                for mut entry in pools.iter_mut() {
                    entry.value_mut().retain(|conn| {
                        let is_alive = conn.connection.close_reason().is_none();
                        let not_idle = conn.last_used.elapsed() < idle_timeout;
                        let not_expired = conn.created_at.elapsed() < self.config.max_lifetime;
                        
                        is_alive && (not_idle || not_expired)
                    });
                }
            }
        });
    }
}

/// RAII守卫，自动释放连接
pub struct ConnectionGuard {
    connection: Arc<Connection>,
    pool: Arc<ConnectionPool>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.pool.release_connection(&self.connection);
    }
}

impl Deref for ConnectionGuard {
    type Target = Connection;
    
    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
```

**使用示例：**

```rust
// client/main.rs
let pool_config = PoolConfig {
    min_connections: 2,
    max_connections: 10,
    idle_timeout: Duration::from_secs(300),
    max_lifetime: Duration::from_secs(3600),
    health_check_interval: Duration::from_secs(30),
};

let connection_pool = ConnectionPool::new(pool_config, endpoint);

// 在ClientState中使用连接池
pub struct ClientState {
    pub connection_pool: Arc<ConnectionPool>,
    // 移除单个connection字段
}

// 使用时
let connection = state.connection_pool.get_connection("server:4433").await?;
let (send, recv) = connection.open_bi().await?;
// connection会在drop时自动释放
```

---

### 4. 握手和认证机制 ⭐⭐

#### Envoy 的握手协议

```yaml
# Envoy 使用专门的网络过滤器处理握手
reverse_tunnel_network_filter:
  name: envoy.filters.network.reverse_tunnel
  typed_config:
    ping_interval: 2s
    # 握手验证逻辑
```

**握手流程：**
1. Client发起TCP连接
2. 发送握手请求（包含node_id, cluster_id, tenant_id等）
3. Server验证身份和权限
4. Server返回握手响应（接受/拒绝）
5. 建立双向通道，开始心跳

**安全措施：**
- mTLS双向认证
- 握手请求签名验证
- 权限检查（是否允许连接到指定cluster）

#### 当前实现

```rust
// client/connection.rs
// 直接建立QUIC连接，没有显式握手
let connection = endpoint.connect(server_addr, "server")?.await?;

// 发送注册帧
let register_frame = create_register_frame(&client_id, &group_id);
write_frame(&mut send, &register_frame).await?;
```

**问题：**
- ❌ 缺少握手协议
- ❌ 认证机制简单（只有自签名证书）
- ❌ 无权限验证

#### 优化方案

```rust
// tunnel-lib/src/handshake.rs

/// 握手请求
#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeRequest {
    /// 协议版本
    pub version: u32,
    /// 客户端标识
    pub identity: NodeIdentity,
    /// 认证令牌（JWT或其他）
    pub auth_token: String,
    /// 客户端能力
    pub capabilities: Vec<String>,
    /// 请求时间戳
    pub timestamp: u64,
    /// 签名（防止篡改）
    pub signature: String,
}

/// 握手响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeResponse {
    /// 是否接受
    pub accepted: bool,
    /// 拒绝原因
    pub reject_reason: Option<String>,
    /// 分配的会话ID
    pub session_id: Option<String>,
    /// 服务器配置
    pub server_config: Option<ServerConfig>,
}

/// 握手处理器
pub struct HandshakeHandler {
    auth_provider: Arc<dyn AuthProvider>,
    acl: Arc<AccessControlList>,
}

impl HandshakeHandler {
    pub async fn handle_handshake(
        &self,
        request: HandshakeRequest,
    ) -> Result<HandshakeResponse> {
        // 1. 验证协议版本
        if request.version != PROTOCOL_VERSION {
            return Ok(HandshakeResponse {
                accepted: false,
                reject_reason: Some(format!(
                    "Unsupported protocol version: {}",
                    request.version
                )),
                session_id: None,
                server_config: None,
            });
        }
        
        // 2. 验证签名
        if !self.verify_signature(&request)? {
            return Ok(HandshakeResponse {
                accepted: false,
                reject_reason: Some("Invalid signature".to_string()),
                session_id: None,
                server_config: None,
            });
        }
        
        // 3. 验证认证令牌
        let claims = self.auth_provider.validate_token(&request.auth_token)?;
        
        // 4. 检查权限
        if !self.acl.is_allowed(&request.identity, &claims)? {
            return Ok(HandshakeResponse {
                accepted: false,
                reject_reason: Some("Access denied".to_string()),
                session_id: None,
                server_config: None,
            });
        }
        
        // 5. 验证标识合法性
        request.identity.validate()?;
        
        // 6. 生成会话ID
        let session_id = Uuid::new_v4().to_string();
        
        // 7. 返回成功响应
        Ok(HandshakeResponse {
            accepted: true,
            reject_reason: None,
            session_id: Some(session_id),
            server_config: Some(ServerConfig {
                heartbeat_interval: Duration::from_secs(30),
                max_idle_timeout: Duration::from_secs(300),
            }),
        })
    }
    
    fn verify_signature(&self, request: &HandshakeRequest) -> Result<bool> {
        // 实现签名验证逻辑
        // 例如使用HMAC或RSA签名
        todo!()
    }
}

/// 认证提供者trait
pub trait AuthProvider: Send + Sync {
    fn validate_token(&self, token: &str) -> Result<Claims>;
}

/// JWT认证提供者
pub struct JwtAuthProvider {
    secret: String,
}

impl AuthProvider for JwtAuthProvider {
    fn validate_token(&self, token: &str) -> Result<Claims> {
        // 使用jsonwebtoken库验证JWT
        todo!()
    }
}

/// 访问控制列表
pub struct AccessControlList {
    rules: Vec<AclRule>,
}

impl AccessControlList {
    pub fn is_allowed(&self, identity: &NodeIdentity, claims: &Claims) -> Result<bool> {
        // 检查ACL规则
        for rule in &self.rules {
            if rule.matches(identity, claims) {
                return Ok(rule.allow);
            }
        }
        Ok(false)  // 默认拒绝
    }
}
```

**使用示例：**

```rust
// client/connection.rs
pub async fn establish_connection(
    endpoint: &Endpoint,
    server_addr: SocketAddr,
    identity: &NodeIdentity,
    auth_token: &str,
) -> Result<Arc<Connection>> {
    // 1. 建立QUIC连接
    let connection = endpoint.connect(server_addr, "server")?.await?;
    
    // 2. 打开握手流
    let (mut send, mut recv) = connection.open_bi().await?;
    
    // 3. 发送握手请求
    let handshake_request = HandshakeRequest {
        version: PROTOCOL_VERSION,
        identity: identity.clone(),
        auth_token: auth_token.to_string(),
        capabilities: vec!["http".into(), "grpc".into()],
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        signature: sign_request(&identity, auth_token)?,
    };
    
    let request_bytes = bincode::serialize(&handshake_request)?;
    send.write_all(&request_bytes).await?;
    send.finish()?;
    
    // 4. 接收握手响应
    let mut response_bytes = Vec::new();
    recv.read_to_end(&mut response_bytes).await?;
    let response: HandshakeResponse = bincode::deserialize(&response_bytes)?;
    
    // 5. 检查是否接受
    if !response.accepted {
        bail!("Handshake rejected: {}", 
            response.reject_reason.unwrap_or_else(|| "Unknown reason".into()));
    }
    
    info!("Handshake successful, session_id: {}", 
        response.session_id.unwrap());
    
    Ok(Arc::new(connection))
}
```

---

### 5. 配置管理 ⭐⭐

#### Envoy 的声明式配置

```yaml
# 完全通过YAML配置，无需修改代码
static_resources:
  listeners:
    - name: reverse_conn_listener
      address:
        socket_address:
          address: "rc://node:cluster:tenant@upstream:1"
      filter_chains:
        - filters:
            - name: envoy.filters.network.http_connection_manager
              typed_config:
                route_config:
                  virtual_hosts:
                    - name: backend
                      domains: ["*"]
                      routes:
                        - match:
                            prefix: "/api"
                          route:
                            cluster: backend-cluster
```

#### 当前实现

```rust
// 配置硬编码在代码中
let rules = vec![
    IngressRule {
        match_host: "example.com".to_string(),
        action_client_group: "group-a".to_string(),
    },
];
```

**问题：**
- ❌ 修改配置需要重新编译
- ❌ 无法热更新
- ❌ 配置分散在多处

#### 优化方案

```rust
// config/server.yaml
server:
  listen_addr: "0.0.0.0:4433"
  cert_path: "/etc/tunnel/server.crt"
  key_path: "/etc/tunnel/server.key"
  
  # 节点标识
  identity:
    node_id: "server-1"
    cluster_id: "server-cluster"
    tenant_id: "default"
  
  # 连接池配置
  connection_pool:
    min_connections: 2
    max_connections: 10
    idle_timeout: 300s
    max_lifetime: 3600s
  
  # 路由配置
  routing:
    extractors:
      - type: header
        header_name: "X-Target-Node"
        target_type: node
      - type: header
        header_name: "X-Target-Cluster"
        target_type: cluster
      - type: default
        cluster_id: "default-cluster"
  
  # Ingress规则
  ingress_rules:
    - protocol: http
      match_host: "api.example.com"
      action_client_group: "backend-cluster"
    - protocol: grpc
      match_host: "grpc.example.com"
      match_service: "com.example.Service"
      action_client_group: "grpc-cluster"
  
  # Egress规则
  egress_rules:
    - match_host: "backend.internal"
      action_upstream: "http://backend:8080"
      is_ssl: false
  
  # 认证配置
  auth:
    type: jwt
    secret: "${JWT_SECRET}"  # 支持环境变量
    issuer: "tunnel-server"
  
  # ACL规则
  acl:
    - tenant_id: "customer-123"
      allowed_clusters: ["edge-cluster-*"]
      deny: false
```

```rust
// tunnel-lib/src/config.rs
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub listen_addr: String,
    pub cert_path: String,
    pub key_path: String,
    pub identity: NodeIdentity,
    pub connection_pool: PoolConfig,
    pub routing: RoutingConfig,
    pub ingress_rules: Vec<IngressRule>,
    pub egress_rules: Vec<EgressRule>,
    pub auth: AuthConfig,
    pub acl: Vec<AclRule>,
}

impl ServerConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        // 支持环境变量替换
        let content = Self::expand_env_vars(&content)?;
        let config: Self = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    fn expand_env_vars(content: &str) -> Result<String> {
        // 替换 ${VAR_NAME} 为环境变量值
        todo!()
    }
    
    fn validate(&self) -> Result<()> {
        self.identity.validate()?;
        // 其他验证逻辑
        Ok(())
    }
    
    /// 热更新配置
    pub fn reload(&mut self, path: &str) -> Result<()> {
        let new_config = Self::from_file(path)?;
        *self = new_config;
        Ok(())
    }
}
```

---

## 优化优先级和实施建议

### 第一阶段：核心架构改进（高优先级）

#### 1. 实现三层节点标识体系 ⭐⭐⭐
**工作量**: 4小时  
**收益**: 支持负载均衡、多租户隔离

**任务清单**:
- [ ] 创建 `tunnel-lib/src/identity.rs`
- [ ] 实现 `NodeIdentity` 结构
- [ ] 更新注册协议包含完整标识
- [ ] 更新 `ClientRegistry` 支持按node/cluster/tenant查询

#### 2. 实现连接池 ⭐⭐⭐
**工作量**: 6小时  
**收益**: 高可用、负载均衡、性能提升

**任务清单**:
- [ ] 创建 `tunnel-lib/src/connection_pool.rs`
- [ ] 实现 `ConnectionPool` 和健康检查
- [ ] 更新 `ClientState` 使用连接池
- [ ] 添加连接池监控指标

#### 3. 实现动态路由 ⭐⭐
**工作量**: 5小时  
**收益**: 灵活的路由策略

**任务清单**:
- [ ] 创建 `tunnel-lib/src/routing.rs`
- [ ] 实现 `RoutingExtractor` trait
- [ ] 实现 `HeaderExtractor`, `PathExtractor`
- [ ] 更新 `ServerState` 使用 `RoutingManager`

### 第二阶段：安全和配置（中优先级）

#### 4. 实现握手协议 ⭐⭐
**工作量**: 4小时  
**收益**: 增强安全性、权限控制

**任务清单**:
- [ ] 创建 `tunnel-lib/src/handshake.rs`
- [ ] 实现 `HandshakeHandler`
- [ ] 实现JWT认证提供者
- [ ] 更新连接建立流程

#### 5. 配置文件化 ⭐
**工作量**: 3小时  
**收益**: 易于部署、支持热更新

**任务清单**:
- [ ] 创建 `tunnel-lib/src/config.rs`
- [ ] 定义YAML配置格式
- [ ] 实现配置加载和验证
- [ ] 支持环境变量替换

### 第三阶段：监控和运维（低优先级）

#### 6. 监控指标 ⭐
**工作量**: 3小时  
**收益**: 可观测性

**任务清单**:
- [ ] 添加Prometheus metrics
- [ ] 连接池指标（活跃连接数、空闲连接数）
- [ ] 路由指标（请求数、延迟、错误率）
- [ ] 节点健康指标

---

## 总结

### 关键优化点

1. **三层节点标识** - 支持负载均衡和多租户
2. **连接池** - 高可用和性能提升
3. **动态路由** - 灵活的路由策略
4. **握手协议** - 增强安全性
5. **配置文件化** - 易于部署和管理

### 与Envoy的差异

**保留的优势**:
- ✅ QUIC协议（比HTTP/2更优）
- ✅ 多协议支持（HTTP/1.1, gRPC, WebSocket）
- ✅ 简单的架构（无需复杂的过滤器链）

**需要借鉴的设计**:
- ⚠️ 三层节点标识体系
- ⚠️ 连接池和负载均衡
- ⚠️ 动态路由机制
- ⚠️ 握手和认证协议
- ⚠️ 声明式配置

### 实施建议

**建议顺序**:
1. 先实现三层节点标识（基础）
2. 再实现连接池（高可用）
3. 然后实现动态路由（灵活性）
4. 最后完善安全和配置

**预期收益**:
- **可用性**: 从单点故障到高可用（连接池）
- **性能**: 吞吐量提升2-10倍（连接池+负载均衡）
- **灵活性**: 支持复杂路由场景（动态路由）
- **安全性**: 企业级认证和权限控制（握手协议）
- **可维护性**: 配置与代码分离（配置文件化）
