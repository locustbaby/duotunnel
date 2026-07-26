# D3 · 最终用户认证（OIDC / mTLS / IP 策略）—— 详细设计

> 承接：12 §6.2 ⑤（**可选插件，默认关闭**）、07（当前只认隧道 agent token）。
> 目标：对**访问暴露服务的最终用户**做鉴权(而非仅隧道 client)。挂在 D1 HttpFilter +
> TLS 层 + ConnectionModule 上。设计模式,暂不落地。

## 0. 定位说明（2026-07-26 复审，**优先于全文**）

**目标场景是"企业内部服务，不暴露到公网"，因此本文所设计的能力：**

1. **默认不启用**——保持当前行为（只认隧道 agent token），**零配置开销**，
   不给内网部署增加任何必填项；
2. **作为可选插件按需启用**——用户明确配置 `auth_policies` / `ip_policies` 才生效；
   未配置时 filter 链为空、走空链快路径（D1 §2.2），无热路径成本；
3. **企业内网场景下非必需**——访问者已在企业边界内，强制边缘鉴权收益低、配置负担高；
4. **本轮的唯一硬要求 = 抽象可插入**：D1 的 `HttpFilter`、现有 `ConnectionModule`、
   TLS acceptor 这三个接入层要**留好位置**，使下文任一机制都能以"加一个 impl +
   一段配置"落地，**而不是现在实现 OIDC/JWT/mTLS**。

**本文的技术设计全部保留**——它是"**当有人 opt-in 时怎么做**"的蓝图，
以及"**抽象要留成什么形状才插得进去**"的验证依据。降级的是优先级，不是设计。

内网定位下的整体优先级：**XFF（真实来源 IP）> 每后端可观测 > 认证（本文，可选）**。

## 1. 背景与问题

现状鉴权只有**隧道 agent token**(`token.rs`,client↔server 握手)。若把服务暴露到
公网,**对访问者零鉴权**——任何人只要能到达 ingress 端口即可访问后端。对标 Cloudflare
Access / ngrok 的最终用户认证,这是公网暴露形态下的核心缺失。
**但在"企业内部服务、不暴露公网"的目标定位下,这一缺口不构成阻塞项**：默认无认证是
可接受且期望的行为(§0);下文机制按 opt-in 提供给需要更严格内部隔离的部署
(如把 admin 面板限制在特定网段/需要 mTLS 的合规场景)。

## 2. 三种认证机制 → 三个接入层

| 机制 | 适用 | 接入层 | 说明 |
| --- | --- | --- | --- |
| **IP 允许/拒绝策略** | 所有协议(含 TCP/UDP) | `ConnectionModule::pre_admission`（现有 seam) | 连接级,最便宜,先做 |
| **mTLS 客户端证书** | TLS/HTTPS | rustls acceptor（`plugins/tls/mod.rs`）| 证书即身份,强认证 |
| **OIDC / JWT / forward-auth** | HTTP/HTTPS | `AuthFilter: HttpFilter`（D1) | Web 访问者鉴权(重定向/Bearer) |

### 2.1 IP 策略(ConnectionModule,最先做)

```rust
pub struct IpPolicyModule { allow: IpNet集合, deny: IpNet集合 }
#[async_trait] impl ConnectionModule for IpPolicyModule {
    fn order(&self) -> i32 { -100 } // 最早
    async fn pre_admission(&self, req: &AdmissionReq) -> Result<PhaseResult> {
        // 命中 deny 或不在 allow → PhaseResult::Reject{403}
    }
}
```
复用现有 `ConnectionModule` seam(`module.rs:14`),零新抽象。连接级,对所有协议生效。

### 2.2 mTLS(TLS 层)

- 现状:`get_or_create_server_config`（`pki.rs:281`)构造 `rustls::ServerConfig`
  用 `with_no_client_auth()`(`quic.rs:80` 类似)。
- 改造:可配 `WebPkiClientVerifier`(信任的 CA)→ 要求/可选客户端证书;
  `tls/mod.rs:87` accept 后从连接取 peer cert → 提取身份(CN/SAN)→ 注入
  `ServerCtx`/`FilterRequestCtx` 供后续策略/日志。
- 与 D4 的证书源解耦:mTLS 是**验证下游客户端证书**,D4 是**server 自身对外证书**。

### 2.3 OIDC / JWT(AuthFilter on D1)

两种模式:
- **Bearer/JWT 校验**(API 场景):`on_request` 取 `Authorization: Bearer`,验签
  (JWKS 缓存)、校验 iss/aud/exp → 失败 `ShortCircuit(401)`;成功可注入身份 header。
- **Forward-auth / OIDC 重定向**(浏览器场景,类 oauth2-proxy/CF Access):
  - 无有效会话 cookie → `ShortCircuit(302 → IdP authorize)`;
  - 回调 `/callback` 由一个内置路由处理:换 code→token、验 id_token、下发**签名会话
    cookie**、302 回原 URL;
  - 后续请求 cookie 有效 → `Continue`(可注入 `X-Auth-User` 等)。

```rust
pub struct OidcAuthFilter { issuer, client_id, jwks_cache, session_key, mode: Bearer|Forward }
#[async_trait] impl HttpFilter for OidcAuthFilter {
    fn order(&self) -> i32 { -50 } // 认证在最外(限流之前或之后可配)
    async fn on_request(&self, ctx: &mut FilterRequestCtx) -> Result<FilterOutcome> {
        // Bearer: 验 JWT;Forward: 验 cookie,无则 302
        // 失败 → ShortCircuit(401/302)
    }
}
```

## 3. 配置 schema

> **默认值 = 全部缺省**：`auth_policies` / `ip_policies` **不配置即不启用**，
> 不生成任何 filter/module 实例，链为空。以下是 opt-in 时的写法。

```yaml
# 按路由/vhost 绑定认证策略（可选，缺省=无认证）
auth_policies:
  api.example.com:
    type: jwt
    issuer: https://idp/
    audience: my-api
    jwks_uri: https://idp/.well-known/jwks.json
  app.example.com:
    type: oidc_forward
    issuer: https://idp/
    client_id: ...
    client_secret_ref: env:OIDC_SECRET
    session_ttl: 8h
  admin.example.com:
    type: mtls
    client_ca: /etc/duotunnel/client-ca.pem
ip_policies:
  admin.example.com: { allow: ["10.0.0.0/8"] }
```

route→policy 映射注入对应 filter/module 实例。

## 4. 场景覆盖 & Corner Cases

| 场景 | 处理 |
| --- | --- |
| **TCP 透传 / UDP** | 无 HTTP,认证用 **mTLS 或 IP 策略**(L4);OIDC/JWT 仅 HTTP |
| **健康检查/回调端点豁免** | `/healthz`、OIDC `/callback` 需绕过认证(路径白名单) |
| **JWKS/introspection 每请求开销** | JWKS **缓存 + TTL**;introspection 结果缓存,避免每请求打 IdP |
| **会话 cookie 安全** | 签名(HMAC/JWT)+ HttpOnly+Secure+SameSite;签名 key 轮换 |
| **JWT 时钟偏移** | 校验 exp/nbf 留 leeway |
| **mTLS 与 TLS 终止顺序** | 客户端证书在 TLS 握手取,早于 HTTP;身份注入 ctx 供 filter/日志 |
| **认证 filter fail 策略** | **fail-closed**(出错=拒绝);与 XFF(fail-open)相反 |
| **认证 vs 限流顺序** | 可配:认证前限流=省认证开销挡洪泛;认证后限流=按身份限流 |
| **登出** | 清会话 cookie + (可选)IdP 登出重定向 |

## 5. 论证 / 备选

- **为何 forward-auth 而非在每后端做认证**:统一在边缘认证是反代/LB 的价值(后端无需
  各自实现);与 CF Access/oauth2-proxy 同模式。
- **为何 IP 策略用 ConnectionModule 而非 filter**:IP 是连接级、对所有协议生效、最便宜,
  现有 seam 正好;HTTP 级细粒度(按路由)再叠 filter。
- **为何 mTLS 在 TLS 层**:证书是传输层身份,必须在握手取;放 filter 太晚。
- **备选(只做 JWT 不做 forward-auth)**:JWT 适合 API,浏览器场景需重定向流;两者都要,
  按路由选。

## 6. 取舍 / 改动量 / 影响

- **取舍**:引入会话/JWKS 状态 + 回调路由;换取边缘统一认证。
- **改动量**:IP 策略 module(~0.5 天,现有 seam);mTLS(~1-2 天,rustls verifier +
  身份提取);JWT filter(~1-2 天);OIDC forward-auth(~3-5 天,重定向+会话+回调,最重)。
- **影响面**:依赖 **D1**(filter 层)与 D4 的 TLS 接入点;认证路径新增,需覆盖各机制
  的通过/拒绝/豁免测试。

## 7. 分阶段(蓝图,暂不落地)

**所有阶段均为 opt-in**：不在主线路线图内,**默认关闭**,仅当用户明确需要时才启用;
每阶段的验收都必须包含"**未配置时行为与今天逐字节一致**"。
P0 是本轮唯一的**抽象义务**,其余按需。

| 阶段 | 内容 | 默认 | 依赖 |
| --- | --- | --- | --- |
| **P0（本轮唯一必做）** | **只验证插入点存在**：D1 filter 链能注册一个拒绝型 filter、`ConnectionModule::pre_admission` 能 `Reject`、TLS acceptor 处能取到 peer cert 位点 | — | D1 |
| P1（opt-in） | IP 允许/拒绝 `ConnectionModule` | 关闭 | 现有 seam |
| P2（opt-in） | mTLS 客户端证书验证 + 身份提取 | 关闭 | D4 的 TLS 接入 |
| P3（opt-in） | JWT/Bearer `AuthFilter` | 关闭 | D1 |
| P4（opt-in） | OIDC forward-auth(重定向+会话+回调) | 关闭 | D1 + 会话 key 管理 |

## 8. 验收

- [ ] **未配置任何 auth/ip policy 时：无认证、无 filter 实例、无热路径开销**
  （内网默认形态，必须回归为零变化）;
- [ ] **抽象可插入性**：以"加一个 impl + 一段配置"即可挂上 IP 策略 / mTLS / JWT /
  OIDC 之一，**不改动核心转发路径**（本轮的硬要求）;
- [ ] 以下为 **opt-in 启用后**才适用的验收：
  - [ ] 可按路由要求 IP 策略 / mTLS / JWT / OIDC 之一;
  - [ ] 未认证访问被正确拒绝(401)或重定向(302 到 IdP);
  - [ ] 健康/回调端点豁免;JWKS/introspection 有缓存;
  - [ ] mTLS 客户端身份可用于策略与日志;
  - [ ] 认证 filter fail-closed;认证/限流顺序可配。
