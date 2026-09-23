# D6 · 客户端 IP 透传 + 每后端可观测 —— 详细设计

> 承接：09 §4.1、10 §6.4、12 §6.2 ①。**企业内网定位下的第一优先能力**
> （优先级：XFF > 每后端可观测 > 认证[可选插件,默认关闭]）。
> 客户端 IP 透传的 HTTP 实现已在 **D1 §5.1（ForwardedForFilter）**、TCP 实现在
> **11 §4（PROXY protocol）**;本文补齐**每后端/每路由可观测**与 trace 透传。
> 设计模式,暂不落地。

## 1. 背景与问题

- **客户端 IP**:后端看不到真实客户端(09 §4.1)。HTTP 由 D1 的 `ForwardedForFilter`
  注入 XFF;TCP 由 PROXY protocol(11 §4)。**本文不重复,仅登记依赖**。
- **可观测**:两大缺口——(a) **双指标路径**:`MetricsSink` trait(plugin 侧)与
  `runtime/metrics.rs` 的 `metrics::counter!` 宏(请求/连接直调)**并存**,per-backend
  RED 无处统一挂;(b) **无每后端/每路由 RED + 分位、无 trace 透传/request-id 关联**。

## 2. 设计

### 2.1 收敛到统一 MetricsSink（10 §6.4）

- 请求/上游相关指标统一走 `MetricsSink::incr/observe`(`plugin/metrics.rs:11`,签名已
  足够:`&'static str name` + `&[(&'static str,&str)] labels`);
- `runtime/metrics.rs` 的宏保留**进程级 gauge**(连接数/活跃流),请求级迁到 MetricsSink;
- 在 **D1 的 filter 边界**与 **HttpConnector/relay 的上游调用处**统一埋点(有 D1 的
  `started_at` 可算延迟)。

### 2.2 每后端 / 每路由 RED + 分位

```rust
// 在 D1 on_response + 上游调用完成处
metrics.incr("duotunnel_requests_total",
    &[("route", route_name), ("upstream", upstream), ("status_class", "2xx")]);
metrics.observe("duotunnel_upstream_duration_ms", upstream_ms,
    &[("route", route_name), ("upstream", upstream)]);
```
- **label 低基数**:`route`(vhost 规则名)、`upstream`(后端地址,有界)、`status_class`
  (2xx/3xx/4xx/5xx)——**禁 per-IP/per-host/per-request-id**(基数爆炸,09 §4.1 警告);
- 分位:histogram(上游耗时、端到端耗时、首字节),供 outlier(D2)与容量判据(TODO-140)。

### 2.3 request-id / trace 透传

- `request-id`:D1 的 `RequestIdFilter`(生成/透传 `x-request-id`,响应回填);
- `traceparent`(W3C Trace Context):透传下游已有的,或生成新 span-id;注入上游请求头,
  使后端 trace 可关联;
- 结构化访问日志:`{ts, client_addr, method, path, status, upstream, upstream_ms,
  end_to_end_ms, request_id, trace_id}`——client_addr 来自 `RoutingInfo.src_addr`,
  日志复用现有 `TunnelService::logging`/`ConnectionModule::on_complete` seam。

## 3. 配置 schema

```yaml
observability:
  metrics:
    per_backend: true            # 开启 per-upstream/route RED + 分位
    latency_histogram: true
  request_id:
    header: x-request-id
    generate_if_absent: true
  tracing:
    propagate_traceparent: true
  access_log:
    format: json
    mask_client_ip: false        # 合规场景可截断/哈希(13 §6)
```

## 4. 场景覆盖 & Corner Cases

| 场景 | 处理 |
| --- | --- |
| **label 基数爆炸** | 只用有界 label(route/upstream/status_class);禁 per-IP/host |
| **histogram 热路径开销** | 分位埋点有成本;结合 TODO-CR4(异步 metrics)+ 采样;benchmark 验证 |
| **TCP/UDP 无 HTTP 语义** | RED 退化为连接级(bytes/duration/连接数);无 status/路由 label |
| **trace 透传信任** | 透传下游 traceparent 需信任边界(与 XFF 同理,不可信源可伪造) |
| **访问日志 PII** | client_addr 是 PII;`mask_client_ip` 提供截断/哈希(13 §6) |
| **双路径迁移期** | 迁移请求指标到 MetricsSink 时保持指标名兼容,避免 dashboard 断裂 |

## 5. 论证 / 备选

- **为何收敛到 MetricsSink 而非并存**:两套路径导致 per-backend RED 无统一挂点;
  MetricsSink 签名已够(静态名+label),收敛即可,无新抽象(10 §6.4)。
- **为何 per-backend 数据先行**:它是 D2 outlier/重试预算设阈值的**判据前置**(没有
  按后端错误率/延迟,outlier 无从判)——D6 与 D2 的依赖顺序由此确定。
- **备选(继续用宏直调加 label)**:宏是全局静态注册,动态 label(upstream)支持弱且
  与 plugin 侧割裂;统一到 MetricsSink 更一致。

## 6. 取舍 / 改动量 / 影响

- **取舍**:热路径新增计时/埋点(有成本,采样+异步缓解);换取按后端归因 + trace 关联。
- **改动量**:MetricsSink 收敛 + 埋点(~1-2 天,依赖 D1 边界);request-id/trace 透传
  (~1 天,D1 filter);结构化访问日志(~0.5 天,现有 logging seam)。
- **影响面**:指标/日志路径;为 D2 提供判据。**依赖 D1**(埋点边界与 client_addr)。

## 7. 分阶段(蓝图,暂不落地)

| 阶段 | 内容 | 依赖 |
| --- | --- | --- |
| P1 | MetricsSink 收敛 + per-backend/route RED + 分位 | D1 边界 |
| P2 | request-id(D1)+ traceparent 透传 | D1 |
| P3 | 结构化访问日志(+ 可选 IP 脱敏) | 现有 logging seam |

## 8. 验收

- [ ] `/metrics` 有 per-upstream/route 的 RED + 分位直方图,label 有界;
- [ ] 请求指标统一经 MetricsSink,无第二套宏直调承载请求级指标;
- [ ] request-id 生成/透传/回填;traceparent 透传到后端;
- [ ] 访问日志含真实 client_addr、upstream、耗时、request-id;可选 IP 脱敏;
- [ ] per-backend 数据可供 D2 的 outlier/重试预算设阈值。
