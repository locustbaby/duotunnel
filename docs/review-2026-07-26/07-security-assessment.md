# 安全性评估（2026-07-26）

## 背景

DuoTunnel 是把私网服务暴露到公网的隧道：**server 的 QUIC 端口与 ingress 端口
直接面向不可信网络**，威胁面等同一台反向代理 / 边界网关。落盘的 CA 私钥、
控制面（ctld watch）、client entry 则构成第二层信任边界。本文所有判断**以代码为准**，
每条结论附 `file:line` 证据；不采信文档/注释的自述行为。

## 问题陈述

在对手可任意构造 QUIC/TCP 字节、可开海量连接、并可能持有（或攻陷）一个合法
client 的前提下，下列边界是否达标：

1. **认证 / 授权边界** —— 未认证方能触达什么？错误信息是否泄露内部细节？
2. **未认证资源消耗（DoS）** —— 连接、流、FD、内存、注册槽是否有未认证方可撑爆的无上限结构？
3. **密钥与凭证处理** —— token、CA 私钥的存储与落盘权限。
4. **传输安全** —— QUIC 反放大 / 地址验证、TLS 校验默认值。
5. **输入解析健壮性** —— rkyv 反序列化、HTTP/1 解析、请求走私面。
6. **多租户隔离** —— 单租户能否拖垮全体（跨租户 DoS / 串扰）。

## 结论速览

**最高危 3 项（即下文实施顺序 1/2/3，均"上线前应关闭"）：**

- **3.3 落盘 CA 私钥无权限控制（高）** —— `pki.rs:225` 以进程 umask（通常 0644）
  写 CA 私钥；一旦泄露即可为任意 host 伪造被 MITM client 信任的证书，整个信任域可仿冒。
- **3.1 未认证连接可无限占用（高）** —— `run_quic_server` 对每个 incoming 无条件
  `spawn`（`handlers/quic.rs:47`），无连接数上限 / 无每 IP 限速 / 无登录失败惩罚；
  可耗尽 FD/内存，或用有效 token 占满全局 4096 槽实现跨租户拒服。
- **4.2 认证内部错误文本回传未认证方（中-高）** —— `handlers/quic.rs:152-157` 把
  `AuthError::Internal`（可能含 DB 路径/SQL）直接回给未认证 peer。

**一句话**：基线（token 摘要化、帧上限、嗅探/登录超时、egress 双层、
ctld watch 的 token 鉴权 + 常量时间比较）已合格，
主要缺口集中在**未认证资源无上限**与**凭证落盘权限**——三者与 01 的 P0 同批、
均不依赖压测即可关闭。

---

## 1. 威胁模型

| 面 | 暴露对象 | 主要威胁 |
| --- | --- | --- |
| QUIC :10086 | 公网任意方 | 未认证连接/流洪泛、登录探测、放大攻击、鉴权绕过 |
| ingress :8080/tcp | 公网任意方 | 慢速攻击、协议解析漏洞、请求走私、跨租户串扰 |
| client entry :8082 | 本机/内网 | egress allowlist 绕过（把隧道当开放代理） |
| ctld watch :7788 | 内网（默认仅回环） | 未授权读取路由/token（控制面）——已有 token 鉴权 + 常量时间比较，仅"绑非回环却未设 token"的误配可绕过（§4.6） |
| 落盘 CA 私钥 | 本机文件系统 | 私钥泄露 → 伪造任意站点证书（MITM 场景） |

---

## 2. 已做对的（基线合格项）

- **Token 存储**：只存 SHA-256 摘要（`crates/duotunnel-store/src/token.rs:12-18`），
  明文 token 32 字节 CSPRNG（`rand::fill`，:5），`dt_` 前缀 + base64url。
  DB 泄露不直接暴露可用凭证 ✅。
- **日志脱敏**：`dt_...` 子串在 `AuthError` 格式化前被掩码（TODO-CR-AUDIT-8 完成）✅。
- **协议嗅探超时**：sniff 包 `timeout(sniff_timeout, ...)`（`dispatcher.rs:38`、
  `crates/duotunnel-client/egress/listener.rs:109`、`core.rs:70`），默认 5s，挡 Slowloris
  嗅探阶段（TODO-CR-AUDIT-18 完成）✅。
- **登录握手超时**：`handlers/quic.rs:69-132` 每步 `timeout(login_timeout)` ✅。
- **消息帧上限**：`MAX_MESSAGE_BYTES=10MiB`（`msg.rs:14,126,154`）收发双向校验，
  防单帧内存爆炸 ✅。
- **UDP datagram 上限**：`MAX_DATAGRAM_BYTES=1200`（`msg.rs:15,195`）✅（但硬编码
  见 §4.4）。
- **egress 双层防御**：client 本地 allowlist（`conn_pool.rs:163`）+ server 端
  vhost 兜底（`crates/duotunnel-server/egress/mod.rs:96` 无匹配即 `route_not_found`）✅。
- **stream 并发上限**：per-connection `Semaphore(max_concurrent_streams)`
  （`connection_handle.rs:36,72-78`），单连接开流被限流 ✅。
- **ctld watch 控制面鉴权**：`WatchServer` 接 `auth_token`
  （`crates/duotunnel-ctld/src/control/watch.rs:25-31`），消息级校验 `WatchRequest.token`
  （`watch.rs:87-89`），且用 `subtle::ConstantTimeEq` 常量时间比较（`watch.rs:141-145`）；
  默认 `watch_addr: 127.0.0.1:7788` 仅回环 ✅（残留低危缺口见 §4.6）。

---

## 3. 高风险发现

### 3.1 未认证连接可无限占用注册槽 → 认证后拒服 `[关联 TODO-146]`

- **现象与证据**：`handlers/quic.rs:60-180` 的连接生命周期是**先 `accept_bi` + 收 Login
  + 鉴权，通过后才 `register`**。但入口 `run_quic_server` 对每个 `incoming` **无条件
  `tokio::task::spawn`**（`handlers/quic.rs:47`），**无并发连接数上限、无每 IP 限速、
  无登录失败惩罚**。注册侧的 inflight slot 表**硬编码 4096 且全局**
  （`registry.rs:89` `new_inflight_table(4096)`，跨所有 group/shard 共享），
  超限时 `register` 返回 "slot table exhausted"（`handlers/quic.rs:171-179`）。
- **攻击场景 / 根因**：两条独立利用路径。
  (a) **未认证洪泛**：开海量 QUIC 连接停在登录前，每连接一个 task + `accept_bi` 等待
  即占 FD/内存/CPU，无任何门槛。(b) **认证后槽耗尽**：单租户用有效 token 反复登录占满
  全局 4096 槽，让**其它租户**的合法 `register` 失败——即跨租户拒服。根因是
  **accept 与 register 之间缺一层"入口全局预算"**：现有 registry actor 的
  `mpsc::channel(1024)`（`registry.rs:90`）只在 register 侧提供背压，洪泛发生在 register 之前；
  `max_concurrent_bidi_streams`（`quic.rs:36`）只限单连接内的流、不限连接数。
- **风险等级**：**高（未认证 DoS）。**
- **方案**：
  1. 入口全局 in-flight 未认证连接数 semaphore——**拒绝新 `incoming` 而非排队**；
  2. 每源 IP 令牌桶，限登录尝试速率；
  3. slot 容量显式化 + 可观测（TODO-146）+ 超限指标；
  4. 本条作为 **TODO-142 分层 admission 的"入口全局预算"层**并入其设计，不单独实现另一套。
- **论证 / 备选**："拒绝而非排队"是关键——排队仍占内存且把延迟转嫁给合法方；用
  semaphore 的 `try_acquire` 在 accept 热路径上做一次原子操作即可，代价可忽略。
  每 IP 桶而非全局桶，避免单一坏 IP 拖垮全局配额。不复用 registry 的 mpsc 背压，
  因为它在鉴权之后、挡不住未认证洪泛。
- **场景覆盖 & Corner Cases**：
  - **FD 上限非 cgroup 派生**：本仓已按 cgroup 感知 CPU（bootstrap 记录 `cgroup_cpu_limit`
    / `effective_runtime_parallelism`），但 `RLIMIT_NOFILE` 不随 cgroup 收敛；容器
    默认 `nofile`（常见 1024）下，未认证洪泛可**先于 CPU 限制**打满 FD。CPUQuota 限住 CPU
    但不限连接数与 pending task 内存，攻击仍成立。
  - **登录前挂起放大持有时间**：`idle_timeout` 默认 180s（`quic.rs:28`）——停在登录前的连接
    在 QUIC 层最长可赖 180s 才被回收；`login_timeout` 仅约束"已进入登录步骤"的耗时，
    连接/task 本身在此之前已存在。入口 semaphore 需覆盖这段窗口。
  - **分离/托管部署**：ctld 只下发路由/token，**不介入 QUIC accept**；managed 模式同样裸奔，
    限流必须落在 server 入口。
  - **多租户**：4096 是全局值，跨租户耗尽是隔离缺口，per-group 配额可作为二级防护。
- **取舍**：入口限流会在极端过载下拒绝**部分合法新连接**（fail-closed）——这是正确取舍：
  牺牲尾部可用性换取不被打垮。阈值需可配并给指标，避免误伤正常突发。
- **改动量 / 影响面**：~1 天。改 `run_quic_server` + 新增全局 semaphore 与每 IP 桶模块；
  热路径每连接一次 `try_acquire`。**并入 TODO-142 入口层**，不新增独立子系统。

### 3.2 QUIC 反放大与 Retry 未确认 `[新发现]`

- **现象与证据**：`create_server_config_with`（`quic.rs:78-90`）构造 `ServerConfig`
  时只设置了 transport 参数（`max_concurrent_bidi_streams` 等），**未显式启用
  `use_retry` / 地址验证**。
- **攻击场景 / 根因**：QUIC 在地址验证完成前对单个初始包的响应字节受 RFC 9000 §8 的
  **3× 反放大**限制约束——quinn 默认遵守该限制，故放大倍率有限；但**未启用 Retry
  意味着不校验源地址即为伪造源地址分配连接状态**，与 §3.1 叠加放大未认证 DoS 面
  （伪造源 IP 让每 IP 桶失效）。
- **风险等级**：**中-高。**
- **方案**：生产配置在 `ServerConfig::transport_config` 之外显式
  `server_config.use_retry(true)`（或等价的 quinn 地址验证选项），并在文档/启动日志
  确认反放大与 Retry 行为。落地前需核对当前 quinn 版本对应 API。
- **论证 / 备选**：Retry 每条新连接加 1 RTT。本场景连接是**少而长命**的 tunnel client
  连接，1 RTT 一次性成本可忽略；若换成海量短连接场景才需权衡。备选（IP 白名单 / 前置
  防火墙）不能替代——攻击来自公网任意方。
- **场景覆盖 & Corner Cases**：
  - **IPv4/IPv6 双栈**：反放大按 path 计，两栈都需覆盖。
  - **NAT / 漫游**：Retry 校验的是"能收到 Retry 令牌的地址"，在 NAT 后仍有效（验证反射地址）；
    client 漫游换 IP 会触发新地址验证，属预期。
  - **Retry 令牌**：quinn 默认管理无状态 Retry 令牌的生成/校验；若版本暴露密钥/有效期
    再评估轮换，默认即可起步。
- **取舍**：开 Retry 略增握手延迟与一点 server 状态；对长命连接不划算的只有握手风暴场景，
  而那正是要防的。净收益为正。
- **改动量 / 影响面**：~0.5 天（含查 quinn API）。仅改 `quic.rs` 配置一处 + 启动日志，
  影响面局限在握手路径。

### 3.3 落盘 CA 私钥无权限控制 `[新发现]`

- **现象与证据**：`infra/pki.rs:225` `std::fs::write(k_path, key_pem)` 写入 CA **私钥**，
  使用进程 umask 默认权限（通常 0644，组/其他可读）。同函数中 `create_dir_all`
  的错误被 `let _ =` 忽略（`pki.rs:212-215`）。该 CA 用于 TLS 终止签发叶证书
  （`plugins/tls/mod.rs:83`），初始化失败时 `RootCa::load_or_generate` 直接
  `expect("failed to initialize root CA")` panic（`pki.rs:86`）。
- **攻击场景 / 根因**：0644 私钥对同机任意本地用户/进程可读；该私钥能为**任意 host**
  签发被 MITM client 信任的叶证书，泄露 = 该信任域内全站可伪造（MITM）。根因是写盘未
  显式设权限、依赖 umask，且目录创建失败被静默吞掉。
- **风险等级**：**高（凭证泄露）。**
- **方案**：私钥写入前用 `OpenOptions::new().mode(0o600).create_new(true)`（Unix），
  父目录 0700；Windows 用 ACL 限制。cert（公钥）可保持 0644。同时把 `create_dir_all`
  的 `let _ =`（:212-215）改为**传播错误**，避免静默写失败再 panic。~15 行。
- **论证 / 备选**：选 **0600 + 目录 0700 + `create_new`** 而非其它：
  - `mode(0o600)` 直接落最小权限，不依赖部署方 umask（容器镜像常见 umask 0022 → 0644）；
  - `create_new(true)` 避免**跟随预置符号链接**写入、并避免截断/覆盖既有 key，
    再生时以 AlreadyExists 走加载分支；
  - 目录 0700 兜住"文件权限对但目录可遍历"的次生暴露。
  备选"仅靠 systemd `UMask=0077`"被否：属部署侧、不可依赖，必须在代码内强制。
- **场景覆盖 & Corner Cases**：
  - **既有 key 不会被追溯收紧**：`mode` 只作用于**新建**文件；若上一版已以 0644 落过 key，
    修复后不会自动 chmod。需一次性迁移（启动时检测并 `set_permissions` 收紧，或运维 chmod），
    否则旧部署仍暴露。
  - **Windows**：Unix `OpenOptionsExt::mode` 在 Windows 不可用/无效，须走 ACL（`icacls`
    或 `windows-acl`），或至少文档强制"Windows 部署自行锁目录"。cert 公钥不受影响。
  - **容器 / bind-mount**：key 若落在宿主 bind-mount 卷上，宿主侧属主/权限决定实际暴露面；
    多容器共享卷会放大暴露。0600 是必要不充分，需配合卷权限。
  - **原子写窗口**：若改为"写临时文件再 rename"，临时文件也须 0600，否则存在短暂 0644 窗口。
  - **create_dir_all 静默失败**：目录创建失败 + `let _ =` 会导致随后写盘失败，再由
    `expect` panic 成不可读信息；传播错误可给出可诊断的启动失败。
- **取舍**：`create_new` 让"key 已存在"从覆盖变为需显式处理的加载分支，逻辑略复杂，
  但换来不被符号链接攻击/不被误覆盖，值得。
- **改动量 / 影响面**：~15 行 + Windows ACL 分支。仅触及 `pki.rs` CA 初始化路径，
  不影响运行时热路径。**独立项，无依赖。**

---

## 4. 中/低风险发现

### 4.1 CI 默认 `tls_skip_verify: true` 的扩散风险 `[新发现]`

- **现象与证据**：`ci-helpers/configs/client.yaml:2` 与 README 快速上手默认关闭证书校验；
  生产 `config/client.yaml:8` 已 `false` ✅；`endpoint.rs:36-42` 命中时已打 `WARN` ✅。
- **攻击场景 / 根因**：风险不在 CI 本身（CI 用自签 CA 属预期），而在**示例被直接复制到生产**
  且无人读 WARN 日志，导致生产静默关闭校验 → 可被 MITM。
- **风险等级**：**低。**
- **方案**：`tls_skip_verify: true` 时启动打**额外醒目告警**，并加可选
  `--i-know-this-is-insecure` 门禁，缺失即拒绝以该配置启动。
- **论证 / 备选**：门禁而非仅日志——日志可被忽略，硬门禁强制显式承认。备选（删掉该开关）
  不可行，CI/自签场景确需它。
- **场景覆盖 & Corner Cases**：门禁必须**可非交互设置**（env var / flag），
  不能用 TTY 交互提示，否则阻断 CI 与自动化部署。
- **取舍**：给合法使用者（CI）增加一次性显式声明成本，换取生产误用被硬挡。
- **改动量 / 影响面**：~0.5 天。改启动参数解析 + `endpoint.rs` 附近，无热路径影响。**独立项。**

### 4.2 认证内部错误文本回传未认证方 `[已追踪 CR-AUDIT-22]`

- **现象与证据**：`handlers/quic.rs:152-157` 对 `AuthError::Internal` 也走
  `LoginResp::failure(e.to_string())`，可能把 DB 路径/SQL/schema 回给未认证 peer。
- **攻击场景 / 根因**：未认证方通过触发内部错误（如构造异常输入压 DB）**回读服务端内部细节**
  用于进一步攻击。根因是错误分类未区分"给对端看"与"仅记日志"。
- **风险等级**：**中-高**（todo 已跟踪，属**应尽快关闭**项）。
- **方案**：对端一律回泛化文案（如 "authentication failed"），完整错误仅服务端 `error!` 记录。
- **论证 / 备选**：泛化对端信息是标准做法；保留服务端日志不损可观测性。
- **场景覆盖 & Corner Cases**：**泄露面主要在 standalone SQLite 模式**——managed 模式下
  `LocalTokenCache::authenticate` 只返回 `InvalidToken/ClientDisabled/TokenRevoked`
  （`crates/duotunnel-server/control/local_auth.rs`），不产生带 DB 细节的 `Internal`；SqliteAuthStore
  才会把 DB 错误升成 `Internal`。修复应覆盖两种 store 的返回映射，别只改一处。
- **取舍**：对端拿不到细粒度失败原因，客户端排障略难——可用错误码（非文本）弥补。
- **改动量 / 影响面**：~0.5 天。改 `handlers/quic.rs:152-157` 登录失败返回映射一处。**独立项。**

### 4.3 UDP session 表无上限 `[关联 TODO-144]`

- **现象与证据**：`udp_datagram.rs:33` server 侧 `DashMap<UdpSessionKey,...>` 按
  (proxy, client_ip, client_port) 建 session，有定时老化（:44-75，空闲 30s、每 10s 驱逐）
  但**无数量上限**。每 session 绑一个临时 `UdpSocket`（`bind 0.0.0.0:0`）+ spawn 一个
  reply pump task；回包路径 `:159 buf[..n].to_vec()` 每包一次分配。
- **攻击场景 / 根因**：持有合法 client 的对端用**变化的源端口**在单个允许的 proxy 上狂建 session，
  每个消耗 1 FD + 1 task + 1 临时端口；**持续洪泛速率可超过 10s 驱逐节奏**，撑大 session 表
  直至 `RLIMIT_NOFILE` / 临时端口段耗尽。
- **风险等级**：**中。**
- **方案**：session 数上限 + LRU 驱逐 + 每客户端配额；复用现有驱逐 loop。并入 TODO-144 的
  "带上限的 session shard"；`:159` 每包分配一并在该项优化。
- **论证 / 备选**：上限 + LRU 而非仅靠 idle 老化——老化挡不住高速新建。每客户端配额限制单个
  被攻陷 client 的爆炸半径。
- **场景覆盖 & Corner Cases**：
  - **爆炸半径**：`UdpSessionManager` 按 `quinn::Connection` 一个（每认证连接一套），
    故直接影响面是"该连接内"，但**FD/临时端口是宿主全局资源**，单个恶意授权 client 仍可拖垮全机、
    波及其它租户。
  - **临时端口段**：`bind 0.0.0.0:0` 每 session 占一个 ephemeral 端口（~28k 上限），
    是 FD 之外的第二道天花板。
- **取舍**：达上限后新 session 被拒/驱逐旧的，极端下影响正常 UDP 会话——可接受，且比 FD 耗尽好。
- **改动量 / 影响面**：~0.5 天。改 `udp_datagram.rs` UDP 转发路径。**并入 TODO-144。**

### 4.4 UDP datagram 1200 硬编码 `[已追踪 TODO-144]`

- **现象与证据**：`msg.rs:15` 固定 1200，未按协商的 QUIC datagram / path MTU 处理；
  大于此的包被**静默丢弃**（`crates/duotunnel-client/egress/udp_listener.rs:83-86`）。
- **攻击场景 / 根因**：非攻击面，属**功能正确性**问题——大 UDP 包（如大 DNS 响应、QUIC-in-UDP）
  被无声吞掉，表现为偶发丢包难排查。
- **风险等级**：**功能正确性（非安全评级）**，todo 已记。
- **方案**：按协商 datagram size / path MTU 动态取值，或至少对超限包记日志而非静默丢。
- **论证 / 备选**：随 TODO-144 一并做，避免为单一常量单独开工。
- **场景覆盖 & Corner Cases**：IPv6 / jumbo 场景 MTU 与 1200 差距更大；静默丢弃对上层协议
  表现为超时而非错误，排障成本高。
- **取舍**：动态 MTU 增加少量复杂度，换取正确性与可诊断性。
- **改动量 / 影响面**：小，随 TODO-144。触及 `msg.rs` + `udp_listener.rs`。

### 4.5 rkyv 反序列化面 `[关联 TODO-CR-AUDIT-20]`

- **现象与证据**：`recv_message`（`msg.rs:146-165`）用 `rkyv::access`（`CheckBytes` 校验）✅ 是
  安全反序列化路径；UDP envelope 解码同样经 `rkyv::access`（`msg.rs:208`）✅。但 QUIC 流首帧
  `RoutingInfo`、UDP envelope 直接来自对端（server 侧来自已认证 client，**但 client 可能被攻陷**），
  其 `src_addr: String` 等字段后续被 `parse::<IpAddr>()`（`tunnel_handler.rs:16`）——解析失败已处理 ✅。
- **攻击场景 / 根因**：CheckBytes 保证 archived 结构合法，但**下游解析逻辑**（IP、host 解析等）
  仍可能在畸形但合法编码的输入上 panic/行为异常；被攻陷 client 是直接的不可信字节来源。
- **风险等级**：**系统性**（不是单点漏洞，是解析健壮性的覆盖度问题）。
- **方案**：把 sniff detectors、msg codec、UDP envelope 纳入 `cargo-fuzz`（TODO-CR-AUDIT-20），
  作为解析健壮性的系统性保证；因直接暴露在不可信字节上，**优先级应升高**。
- **论证 / 备选**：fuzz 是覆盖"合法编码 + 畸形语义"组合的唯一可扩展手段，人工用例覆盖不全。
- **场景覆盖 & Corner Cases**：重点靶：`RoutingInfo`/`UdpDatagramEnvelope` 的 String/Vec 字段
  边界、`parse::<IpAddr>()` 与 host 解析的 panic 面、sniff detector 在截断输入上的行为。
- **取舍**：fuzz 是一次性基建投入，回报是持续回归保护；不改主路径、无运行时成本。
- **改动量 / 影响面**：~1-2 天。新增 fuzz crate/targets，不动生产热路径。**独立项（建议升优先级）。**

### 4.6 ctld watch 端口鉴权 `[已验证 ✅ 基线合格 + 一条低危加固]`

- **现象与证据（已读码确认，鉴权存在）**：
  - `WatchServer::new(svc, bind_addr, auth_token: Option<String>)`
    （`crates/duotunnel-ctld/src/control/watch.rs:25-31`）——token 是构造参数，空串被归一为 `None`；
  - **消息级校验**：读完 `WatchRequest` 后立即比对其 `token` 字段
    （`watch.rs:87-89`），不匹配即 `warn!` + 断开；
  - **常量时间比较**：`tokens_equal` 用 `subtle::ConstantTimeEq`（`watch.rs:141-145`），
    抗计时侧信道 ✅；
  - **配置项已存在**：`config/ctld.yaml` 的 `watch_token`（当前注释掉，:3-5 注释明写
    "Set when exposing watch_addr beyond localhost. Server must pass the same value"）；
  - **默认绑回环**：`watch_addr: "127.0.0.1:7788"`（`config/ctld.yaml:2`），
    server 侧 managed 模式携带 `ctld_token`（`ServerMode::Managed { ctld_token }`，
    `crates/duotunnel-server/bootstrap/mod.rs:35,45` 由 `resolved_ctld_token()` 注入）与之对应。
- **架构说明（值得记录的实际数据流）**：ctld **轮询 SQLite** 感知变更
  （`crates/duotunnel-ctld/src/control/reactor.rs:43` `db_poll_task`），server 作为 watch 客户端
  消费 ctld 的推送流（先 Snapshot、后 Patch）。控制面不是 server 主动查库。
- **残留缺口 / 根因**：鉴权本身没问题，缺的是**启动期强制**——"绑非回环就必须设 token"
  目前**只由 YAML 注释约束**，运维把 `watch_addr` 改成 `0.0.0.0:7788` 而忘了取消注释
  `watch_token` 时，进程会**静默地以无鉴权模式**监听控制面（`auth_token=None` ⇒ §87-89 的
  校验分支整体跳过），此时任意内网方可读全量路由/token 元数据。
- **风险等级**：**低**（默认配置安全；仅误配路径可触发，且需内网可达）。
- **方案**：加**启动校验**——当 `watch_addr` 非回环地址（非 `127.0.0.0/8` / `::1`）
  且 `watch_token` 未设置时**拒绝启动**并给出明确错误，把 YAML 注释里的约定变成硬约束。~15 行。
- **论证 / 备选**：选 fail-closed 拒绝启动而非 WARN——与 §4.1 同理，日志会被忽略，
  而控制面裸奔的后果是全租户路由/token 暴露。备选"强制 mTLS"被否：默认回环部署下属过度设计，
  且 token + 常量时间比较对当前威胁模型已足够；跨主机部署应由部署侧 TLS/网络隔离叠加。
- **场景覆盖 & Corner Cases**：
  - `0.0.0.0` / `[::]` 通配绑定必须判定为**非回环**（最常见的误配形态）；
  - 空串/纯空白 token 已在 `watch.rs:31-36` 归一为 `None`，启动校验须复用同一归一逻辑，
    否则 `watch_token: ""` 会假性通过；
  - 单机（默认回环）部署不受影响，无行为变化。
- **取舍**：误配的分离部署会在启动时失败而非静默降级——正是想要的。
- **改动量 / 影响面**：~15 行，仅 ctld 启动期配置校验路径，不触及 watch 热路径。**独立项。**

### 4.7 请求头注入 / 走私面 `[新发现，需针对性测试]`

- **现象与证据**：`Http1Driver::read_request`（`h1.rs:110-159`）用 httparse +
  `sanitize_request_headers`（`http_utils.rs:75-111`，剥离 hop-by-hop）。**但**未见对
  `Content-Length` 与 `Transfer-Encoding` **同时出现**、**重复 `Content-Length`**、
  `Transfer-Encoding: chunked` 的显式冲突检测。当前 body 按 `Content-Length` 处理
  （`h1.rs:173-177`），响应无条件 chunked（`h1.rs:291`；见 01 §3.2）。
- **攻击场景 / 根因**：CL+TE / TE.TE / 重复 CL 是经典**请求走私**面——当本代理与上游 backend
  对同一请求的边界解析不一致时，攻击者可把第二个请求"藏"进 body，实现越权/缓存投毒。
  httparse 本身不拒绝这些组合，需上层显式判定。
- **风险等级**：**中。**
- **方案**：入站请求显式**拒绝 CL+TE 共存、拒绝重复/冲突 CL**、拒绝非法 TE；作为 §4.5 fuzz
  与针对性测试的重点用例；与 01 §3.2 的 chunked 修复合并做。
- **论证 / 备选**：走私防护的业界共识是**在边界处 fail-closed 拒绝歧义请求**，而非尝试"猜"意图。
- **场景覆盖 & Corner Cases**：
  - **响应侧风险有界**：响应无条件 chunked（`h1.rs:291`）使响应方向走私空间小，重点在**请求方向**
    到 backend。
  - **H2→H1 降级**：h2c 路径转发到 h1 上游时，须确保伪首部→h1 转换不注入 CL/TE
    （HTTP/2 禁止 TE，但转换层需显式 sanitize）。
  - **trailer 注入**：响应 chunked trailer 已处理（`h1.rs:322-335`），转发时应经
    `sanitize_response_headers`，防 trailer 携带敏感/控制头。
- **取舍**：严格拒绝会挡掉少量非常规但"善意"的畸形客户端——安全场景下 fail-closed 是正确默认。
- **改动量 / 影响面**：~0.5 天。改 `h1.rs` 入站解析 + `http_utils.rs`，影响 H1 入站路径。
  **与 01 §3.2 合并。**

---

## 5. 修复优先级、实施顺序与依赖

| 顺序 | 项 | 风险 | 改动量 | 依赖 |
| --- | --- | --- | --- | --- |
| 1 | 3.3 CA 私钥 0600 + 目录 0700 | 高 | ~15 行 | 无 |
| 2 | 3.1 未认证连接限流（全局 semaphore + 每 IP 桶） | 高 | ~1 天 | 并入 TODO-142 入口层 |
| 3 | 4.2 认证错误边界（CR-AUDIT-22） | 中-高 | ~0.5 天 | 无 |
| 4 | 3.2 QUIC Retry/地址验证确认 | 中-高 | ~0.5 天（含查 API） | 无 |
| 5 | 4.7 请求走私防护（CL/TE 冲突拒绝）+ 01§3.2 chunked 修复 | 中 | ~0.5 天 | 与 01 §3.2 合并做 |
| 6 | 4.3 UDP session 上限（TODO-144） | 中 | ~0.5 天 | UDP 场景 |
| 7 | 4.5 fuzz 靶（sniff/codec/udp）（CR-AUDIT-20） | 系统性 | ~1-2 天 | 建议升优先级 |
| 8 | 4.1 tls_skip 门禁强化 | 低 | ~0.5 天 | 无 |
| 9 | 4.6 ctld watch 绑非回环时强制要求 `watch_token`（启动校验） | 低 | ~15 行 | 无（鉴权本身已合格） |

### 独立项 vs 并入既有 TODO

- **独立、可各自成 PR**（不依赖其它改造）：**3.3**（CA 私钥权限）、**4.2**（认证错误边界，CR-AUDIT-22）、
  **3.2**（QUIC Retry）、**4.5**（fuzz 靶，CR-AUDIT-20，建议升优先级）、**4.1**（tls_skip 门禁）、
  **4.6**（watch 非回环启动校验）。
- **并入既有 TODO / 章节，不单独实现**：
  - **3.1 → 并入 TODO-142** 分层 admission 的"入口全局预算"层；slot 可观测部分对齐 **TODO-146**。
  - **4.3 → 并入 TODO-144** 的"带上限的 session shard"；**4.4**（1200 硬编码）同随 TODO-144。
  - **4.7 → 与 01 §3.2**（chunked 修复）合并做。
- **已确认无需大改**：**4.6**（ctld watch）鉴权已实现且用常量时间比较，仅补一条
  "绑非回环必须设 `watch_token`"的启动校验，属低危加固，可随手做。

> **实施节奏**：安全项 **1、2、3**（即 **3.3、3.1、4.2**）应与 01 的 P0 **同批**排入最近一个迭代——
> 它们**均不依赖压测、不依赖架构改造**，是"上线前应关闭"的清单。4/5 紧随，6-8 可并入对应 TODO 的
> 常规节奏，9 是 ~15 行的低危加固（鉴权本身已合格），可搭任意 PR 顺带完成。
