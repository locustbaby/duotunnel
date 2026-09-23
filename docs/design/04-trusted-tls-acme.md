# D4 · 可信 TLS（ACME + 自带证书 + 热重载）—— 详细设计

> 承接：12 §6.2 ④、07 §3.3（CA 私钥权限）、TODO-99（证书热重载）。
> **内网定位下**：只需 **BYO 证书加载 + 热重载**（内部 PKI 签发）；
> **ACME 自动签发降为按需**（仅对公网发布的形态才必要）。
> 目标：对公网提供**浏览器可信**的 HTTPS(当前是自签 MITM 证书,不可信)。设计模式,
> 暂不落地。

## 1. 背景与问题

现状 TLS 终止用**进程级 Root CA 动态签发的自签叶证书**(`pki.rs`,
`get_or_create_server_config:281`)——浏览器**不信任**,无法直接对公网发 HTTPS(会告警)。
且更新证书需重启(TODO-99)。要做"安全暴露"必须能:(a) 用公网可信证书(ACME/自带),
(b) 不断连热重载。

**关键区分**:当前自签逻辑服务两个**不同**用途,不要混:
- **ingress 对外 TLS**(本文):server 面向公网客户端出示的证书——需可信;
- **egress MITM**(保留):duotunnel-client + duotunnel-server 为拦截解密上游 TLS 动态签发的证书——自签即可
  (`duotunnel-client/ingress/app.rs` MitmH2、`plugins/tls` 内部)。

本文只改 ingress 对外证书源。

## 2. 设计:可插拔证书源 + ArcSwap 热重载

### 2.1 证书源抽象

```rust
// duotunnel-lib/src/infra/tls_cert.rs (新)
#[async_trait]
pub trait CertSource: Send + Sync + 'static {
    /// 按 SNI 返回可信 ServerConfig(可缓存)。
    async fn server_config(&self, sni: &str) -> Result<Arc<rustls::ServerConfig>>;
}
```
实现:
- `SelfSignedSource`(现状,dev/内网)——包装现有 `pki.rs`;
- `FilesCertSource`(自带 PEM cert+key)——加载磁盘证书,`notify` 监听变更(TODO-99);
- `AcmeCertSource`(Let's Encrypt 等)——见 §2.3。

### 2.2 热重载(不断连)

TLS acceptor 当前每连接 `get_or_create_server_config(&host)` 取 config
(`plugins/tls/mod.rs:83`)。改为从 `ArcSwap<Arc<dyn CertSource>>` 或证书源内部
`ArcSwap<ServerConfig>` 取——**证书更新 = `ArcSwap::store` 新 config**,存量连接用旧
acceptor 不受影响,新连接用新证书。与既有 `RoutingSnapshot` 的 ArcSwap 热重载同模式。

### 2.3 ACME(公网可信)

- crate:`rustls-acme` 或 `instant-acme`(ACME v2)。
- **挑战方式**优先级(自托管):
  - **TLS-ALPN-01**(最干净):挑战在 TLS 握手层(`acme-tls/1` ALPN)完成,无需 HTTP
    路径,复用 443 端口——适合 DuoTunnel;
  - **HTTP-01**:需服务 `/.well-known/acme-challenge/`(80 端口),可加一个内置 handler;
  - **DNS-01**:通配符证书必需,需 DNS provider API。
- **续期**:后台任务在到期前(如剩 30 天)自动续,续到新证书后 `ArcSwap::store` 热切;
- **持久化**:ACME 账户 key + 已签证书落盘(**0600**,复用 07 §3.3 的权限修复),重启
  复用避免重复签发(Let's Encrypt 有速率限制)。

## 3. 配置 schema

```yaml
tls:
  mode: acme            # self_signed(默认/dev) | files | acme
  files:               # mode=files
    cert: /etc/duotunnel/fullchain.pem
    key: /etc/duotunnel/privkey.pem
    watch: true        # notify 热重载(TODO-99)
  acme:                # mode=acme
    provider: letsencrypt        # letsencrypt | letsencrypt-staging | 自定义 directory URL
    email: admin@example.com
    domains: ["app.example.com", "api.example.com"]
    challenge: tls-alpn-01       # tls-alpn-01 | http-01 | dns-01
    cache_dir: /var/lib/duotunnel/acme   # 0600 持久化
```

## 4. 场景覆盖 & Corner Cases

| 场景 | 处理 |
| --- | --- |
| **Let's Encrypt 速率限制** | 测试用 `letsencrypt-staging`;持久化避免重复签发;失败退避重试 |
| **通配符证书** | 仅 DNS-01 支持;需 DNS provider 凭据 |
| **多域名 SNI** | 按 SNI 返回对应证书;ACME 为每域(或 SAN)签发 |
| **续期不断连** | ArcSwap 热切,存量连接用旧 acceptor;续期失败保留旧证书 + 告警 |
| **首次签发窗口** | 证书就绪前 `/healthz` not ready 或回退自签 + 告警,避免空窗对外 |
| **TLS-ALPN-01 与业务 ALPN 冲突** | 挑战期临时响应 `acme-tls/1` ALPN,业务 ALPN 不受影响 |
| **HTTP-01 需要 80 端口** | 若只开 443,用 TLS-ALPN-01;或额外监听 80 仅服务 challenge |
| **证书私钥权限** | 落盘 0600 + 目录 0700(复用 07 §3.3);ACME 账户 key 同 |
| **egress MITM 证书不受影响** | 保持自签(内部用途),与对外证书源解耦 |
| **时钟错误致证书 notBefore/After 误判** | 依赖系统时钟正确;续期留足提前量 |

## 5. 论证 / 备选

- **为何抽象 CertSource 而非直接塞 ACME**:自签(dev)、自带证书(企业已有 PKI)、ACME
  三种需求并存,抽象后按配置切,且 egress MITM 保持独立。
- **为何 TLS-ALPN-01 优先**:自托管常只暴露 443;ALPN 挑战无需额外 HTTP 路径/端口,
  最少运维面。HTTP-01/DNS-01 作为通配符/无 443 控制场景的备选。
- **为何 ArcSwap 热切而非重启**:TODO-99 的诉求;与既有路由热重载同模式,零新机制。
- **备选(仅自带证书 + 外部 certbot)**:可行但把续期/热重载推给运维;内建 ACME 是
  "对标 ngrok/CF 的托管 TLS"体验的关键,值得内建。

## 6. 取舍 / 改动量 / 影响

- **取舍**:内建 ACME 增加依赖与后台续期任务;换取"开箱即公网可信 HTTPS"。
- **改动量**:CertSource 抽象 + acceptor 接入 ArcSwap(~1-2 天);FilesCertSource + 热
  重载(~1 天,TODO-99);AcmeCertSource(TLS-ALPN-01 + 续期 + 持久化,~3-5 天)。
- **影响面**:ingress TLS 终止路径;egress MITM 不动;需覆盖签发/续期/热切/多 SNI/
  持久化测试(ACME 用 staging)。**独立于 D1,可并行**。

## 7. 分阶段(蓝图,暂不落地)

| 阶段 | 内容 | 依赖 |
| --- | --- | --- |
| P1 | CertSource 抽象 + acceptor 走 ArcSwap;SelfSignedSource 包装现状 | — |
| P2 | FilesCertSource + notify 热重载(TODO-99) + 私钥 0600(07 §3.3) | — |
| P3 | AcmeCertSource(TLS-ALPN-01)+ 续期任务 + 持久化 | P1 |
| P4(可选) | HTTP-01 / DNS-01(通配符) | P3 |

## 8. 验收

- [ ] 可用 ACME 对公网发**浏览器可信** HTTPS;
- [ ] 证书续期/更新**不断开存量连接**(ArcSwap 热切);
- [ ] 证书/账户 key 落盘 0600、重启复用(避免重复签发);
- [ ] 自带证书模式 + 文件热重载可用;
- [ ] egress MITM 自签路径不受影响。
