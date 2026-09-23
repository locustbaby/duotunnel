# D5 · 限流 / IP 策略 / 分层 Admission —— 详细设计

> 承接：12 §6.2 ②、09 §5(D)、07 §3.1（未认证连接洪泛）、TODO-142（分层 admission）。
> **内网定位下**保留在主线，但理由由"抗攻击"改为**容量公平**（防单 group 饿死全体）。
> 目标：从"仅 per-connection inflight"升级为**入口全局 → 每租户 → 每连接**三层
> admission,含限流/IP 策略/快速拒绝,并统一治理未认证洪泛与跨租户公平。设计模式,
> 暂不落地。

## 1. 背景与问题

现状过载保护只有 **per-connection**:`open_bi` 的 inflight 慢路径 + pending 上限
(`open_bi.rs`)+ stream semaphore。**缺**:
- 入口对**未认证连接**无上限(07 §3.1,`run_quic_server` 无条件 spawn);
- **无按 IP / 路由 / 租户的限流**(全库无实现);
- **无跨租户公平**——一个 group 可占满共享资源饿死其它(09 §5 D)。

## 2. 分层 Admission 模型（TODO-142）

```
① 入口全局(连接前/未认证)   ② 路由后(可识别 group/租户)     ③ 连接级(现有)
   ├ 未认证连接数 semaphore      ├ per-group 令牌桶/active 预算     ├ inflight 慢路径
   ├ per-IP 令牌桶(登录尝试)     ├ per-route 令牌桶                 ├ pending 上限
   └ QUIC accept gate            └ RateLimitFilter(D1,请求级)      └ stream semaphore
   (07 §3.1)                     (09 §5 D / 本文)                   (现有,保留)
```

**分层理由**:①在鉴权前挡洪泛(最便宜、最外)、②在知道租户身份后做公平与配额、
③兜底单连接。三层各司其职,不互相替代。

## 3. 各层设计

### 3.1 层① 入口全局(连接级,含 07 §3.1)

- **未认证连接 semaphore**:`run_quic_server`（`handlers/quic.rs:47`)对每个 `incoming`
  先 `try_acquire` 一个全局 permit,**拒绝而非排队**(排队仍占内存);permit 覆盖到
  鉴权完成,鉴权后释放(转入已认证)。
- **per-IP 令牌桶**:限登录尝试速率(防单 IP 洪泛);与 QUIC Retry(07 §3.2)配合防
  伪造源。
- **IP 允许/拒绝**:`IpPolicyModule: ConnectionModule`(D3 §2.1,同一 seam)。
- 这层直接关闭 07 §3.1 的未认证 DoS。

### 3.2 层② 路由后 / 每租户(请求级,RateLimitFilter on D1)

```rust
pub struct RateLimitFilter {
    limiter: KeyedLimiter, // 令牌桶,按 key 分桶
    key: RateKey,          // ClientIp | Route | Group | Header(name) | Global
}
#[async_trait] impl HttpFilter for RateLimitFilter {
    fn order(&self) -> i32 { -40 } // 认证之后、业务改写之前(可配)
    async fn on_request(&self, ctx: &mut FilterRequestCtx) -> Result<FilterOutcome> {
        let key = self.key.extract(ctx); // client_addr / route / group / header
        if self.limiter.try_acquire(&key) { Ok(Continue) }
        else { Ok(ShortCircuit(too_many_requests(retry_after))) } // 429 + Retry-After
    }
}
```
- **每租户/group 公平**:key=Group 的桶给每租户独立配额,防噪声邻居;
- 请求级(非连接级)才能治理 H2/keep-alive 单连接多请求。

### 3.3 层③ 连接级(现有,保留)

inflight 慢路径 + pending + semaphore 不变,作为最内层兜底。

### 3.4 限流算法

- **令牌桶**(token bucket):每 key 维护 `{tokens, last_refill}`;`try_acquire` 按
  速率补充、扣 1;支持 `rate`(稳态)+ `burst`(桶容量)。
- **分片/per-core**:桶按 key 哈希分片,减少共享;全局桶用 per-core 近似(同 02 K1)。
- 现成 crate 可选(如 `governor`),或自研轻量桶。

## 4. 配置 schema

```yaml
admission:
  ingress_global:                 # 层①
    max_unauthenticated_conns: 1000
    per_ip_login_rate: 10/s
  rate_limits:                    # 层②(按 key)
    - key: client_ip
      rate: 100/s
      burst: 200
      scope: app.example.com       # 限某路由
    - key: group                   # 每租户公平
      rate: 5000/s
      burst: 10000
    - key: global
      rate: 50000/s
  reject:
    http_status: 429               # 429 | 503
    retry_after_secs: 1
ip_policies:                      # 层①/③ IP 允许拒绝(见 D3)
  admin.example.com: { allow: ["10.0.0.0/8"] }
```

## 5. 场景覆盖 & Corner Cases

| 场景 | 处理 |
| --- | --- |
| **未认证洪泛**(07 §3.1) | 层① semaphore **拒绝新 incoming**;idle 前挂起窗口(180s)由 semaphore 覆盖 |
| **伪造源 IP** | per-IP 桶配合 QUIC Retry(07 §3.2)——无地址验证时每 IP 桶可被绕过 |
| **429 vs 503** | 限流用 429(Too Many Requests)+ Retry-After;过载(层③)用 503;语义区分 |
| **非 HTTP(TCP/UDP)** | 请求级限流仅 HTTP;TCP/UDP 用层①连接级(IP 桶/连接数) |
| **H2 单连多请求** | 层②请求级 filter 天然每请求;层①连接级不足以治理 |
| **突发 vs 稳态** | burst 桶容量吸收突发;rate 控稳态 |
| **白名单** | 受信 IP/路由跳过限流(与 IP 策略 allow 联动) |
| **分布式(多 server)** | 单节点桶不跨 server 共享;多 server 精确限流需共享存储(redis)——**v2**,v1 单节点近似 |
| **计数竞争** | per-key 分片 + per-core 全局近似 |
| **限流 vs 认证顺序** | 可配:认证前限流挡洪泛(省认证);认证后按身份限流(更精细) |

## 6. 论证 / 备选

- **为何分三层而非一个全局限流器**:洪泛在鉴权前(层①)、公平在识别租户后(层②)、
  兜底在连接(层③)——单层无法同时便宜地挡未认证洪泛又做租户公平(TODO-142 的核心)。
- **为何"拒绝而非排队"**(层①):排队占内存 + 把延迟转嫁合法方;`try_acquire` 一次
  原子,热路径可忽略。
- **为何请求级用 D1 filter**:`pre_admission` 是连接级,H2/keep-alive 多请求只准入
  一次;请求级公平必须在 filter 层(10 §6.5 结论)。
- **备选(仅连接级限流)**:挡不住单连接内的请求洪泛,也做不到每请求租户公平;不足。
- **单节点 vs 分布式**:先单节点(自托管常单/少节点);分布式限流(redis)按需 v2。

## 7. 取舍 / 改动量 / 影响

- **取舍**:引入限流状态(桶)+ 三层接线;换取未认证 DoS 防护 + 租户公平 + IP 策略。
  单节点近似(非分布式精确)是 v1 取舍。
- **改动量**:层①(QUIC gate semaphore + per-IP 桶,~1 天,并入 07 §3.1);
  层②(RateLimitFilter + KeyedLimiter,~1-2 天,依赖 D1);IP 策略 module(~0.5 天,D3 共用)。
- **影响面**:QUIC accept 路径 + D1 filter 链;拒绝语义变化(新增 429/503),需覆盖
  各 key 限流、突发、白名单、拒绝语义测试。

## 8. 分阶段(蓝图,暂不落地)

| 阶段 | 内容 | 依赖 |
| --- | --- | --- |
| P1 | 层① 未认证连接 semaphore + per-IP 桶 + IP 策略 module(= 07 §3.1) | ConnectionModule seam |
| P2 | 层② RateLimitFilter(client_ip/route/group/global) | **D1** |
| P3 | 每租户公平预算(group active-stream 预算,TODO-142) | P2 |
| P4(v2) | 分布式限流(共享存储) | P2 |

## 9. 验收

- [ ] 未认证洪泛被层①拒绝(不 OOM/不耗尽 FD);
- [ ] 可按 IP/路由/租户/全局限流,超限 429 + Retry-After;
- [ ] 单租户洪泛不饿死其它租户(每 group 预算);
- [ ] IP 允许/拒绝策略生效;
- [ ] 限流/认证顺序可配;计数 per-core 无热点。
