# 历史评审记录（截至 2026-09-04）

> 保留原始判断作为历史记录，其中部分结论已被代码复核否定。当前状态以 [TODO](../todo.md) 和 [2026-09-05 复核](../review-2026-09-05/README.md) 为准。

## 2026-07-30 implementation status

The current control-plane implementation has landed on the review branch:

- YAML/SQLite merge, override/tombstone/clear semantics, schema initialization,
  transactional effective-state commit, and canonical Snapshot hash are in
  place.
- `ConfigSource` now has an associated layer type. YAML remains the default
  file source, while `SqliteConfigSource` is a read-only polling subscriber;
  SQLite writes remain confined to ctld admin mutation transactions. Legacy
  `server_config` is accepted as the YAML base when no explicit YAML source is
  configured, and old normalized routing tables are preserved as SQLite
  overrides with repeatable migration markers. The current coordinator supports
  one YAML source plus one SQLite source; future Etcd-like sources need explicit
  coordinator registration and merge/error-state policy.
- The control wire uses one canonical envelope with a numeric wire version;
  Snapshot/Delta ACK, Resync, target-hash validation, LKG preflight, and server
  runtime-generation publication are implemented. Legacy business V1/V2
  branches are not being reintroduced.
- `duotunnel-ctld` owns the local admin Unix socket. CLI mutations go through
  ctld and do not edit SQLite directly. The server remains a pure watch
  consumer and never reads YAML or SQLite.
- Admin mutations require an idempotency key and commit the key/fingerprint in
  the same SQLite transaction. Configuration/revoke responses are replayable;
  create/rotate bearer tokens use a redacted durable marker plus a bounded
  process-local cache, so restart returns 410 instead of replaying or repeating
  a token mutation.
- CI 8K remains a performance-only case. It does not mutate YAML/SQLite or
  claim to validate Delta consistency; those behaviors belong to control-plane
  tests.
- Admin socket framing still needs a focused hardening pass for duplicate or
  missing `Content-Length`, declared-length/body mismatches, and handler-level
  status tests. The current bounded `0600` Unix socket and request timeout keep
  this as follow-up hardening rather than a current correctness blocker.

The remaining performance-related work is deliberately separated by nature:

| Task | Nature | Current decision |
| --- | --- | --- |
| TODO-140/T5 | Measurement and latency baseline | CI-side CPU contract/cpuset attempt, effective-config snapshot, absolute timing window, dropped iterations and fail-closed gate landed; real runner permission/repeatability and baseline artifacts remain. |
| TODO-141/T6 | Real buffer parameter wiring | Implemented for relay, HTTP header/body, and QUIC stream sniff consumers; performance before/after evidence remains separate from this wiring task. |
| TODO-145/T7 | Profile-guided hot-path optimization | dial9 is observation only; no optimization is accepted without before/after evidence. |
| TODO-146/T8 | Capacity and cross-shard correctness | Capacity validation, exhaustion/high-water metrics, and rotating cross-shard selection landed; fairness/cross-shard acceptance evidence remains. |
| TODO-142/T9 | Overload tail-latency control | Partial: UDP session production admission is wired with an owner-held RAII permit; HTTP/raw/reverse/queue domains remain separate and deferred pending lifecycle and threshold evidence. |

Clarification for the 726 performance review: dial9 remains enabled only for the
existing 8K trace cases. Basic, 3K, and 6K continue to use the release path and
resource sampling without dial9. The recent CI changes improved CPU-role
isolation, absolute sampling windows, collector cleanup, and gates; they did
not expand dial9 coverage. FRPS is assigned the server CPU set and FRPC the
client CPU set in isolate mode.

The existing proven data-plane optimizations remain the baseline: `BytesMut`
with `read_buf`, QUIC `read_chunk`, and TCP-specific owned split. Do not start
io_uring, multi-endpoint, custom runtime, H2 sender-pool, or UDP wire-format
changes without a dial9/Criterion profile showing that the relevant path is a
measured bottleneck.

## 2026-08-21 全项目代码评审（2026-08-24 已复核）

对四个 crate（约 31.5k 行 Rust）做了整体评审：clippy 全绿，283 个单元测试全部通过。
重点精读了认证路径（`quic.rs` Login 握手、`auth.rs`）、数据面缓冲池（`engine/copy.rs`）、
UDP 会话管理、连接池、监听器管理、TLS/PKI 和客户端重连逻辑。**未发现高危或阻断性问题。**

评审确认的亮点（无需行动）：生产代码几乎零 `unwrap`/`expect`（全部位于测试模块）；
PKI 密钥处理有 `O_NOFOLLOW`/uid 校验/fchmod 0600 加固；预认证预算与 UDP 三层限额完整；
无任何无界队列；Login 错误信息不外泄且 retryable 语义正确。

改进项（已在 2026-08-24 复核，是否立项按优先级执行）：

1. **TLS 跳过验证的告警强度** → TODO-152。`tls_skip_verify` 与
   `allow_insecure_fallback` 目前仅有 `warn!` 日志，考虑加更醒目的启动告警。
2. **UDP 容量常量硬编码** → TODO-153。`udp_datagram.rs` 中的单连接/全局会话上限
   与排队 envelope 上限均为 `const`，其他配额已走配置，建议统一。
3. **VhostRouter 通配符 O(n) 扫描** → 已由 TODO-31 覆盖，当前规模无碍，维持原优先级即可。
4. **README 0-RTT 表述与实现的差距** → 已由 TODO-27 覆盖（ticket key 未持久化，
   重启后 0-RTT 必然回退），文档措辞可在 TODO-27 落地时一并修正。

---

## 2026-08-21 深度评审：生命周期 / 数据结构 / 选型（2026-08-24 已复核）

按三个维度精读了核心实现：①进程保活与 worker 派生/cancel/重载/错误传播；
②数据结构与命名；③选型性能。结论：**三层保活设计正确，cancel 语义完备，
自研边界划得准（该用库的都用了），热路径选型几乎都是同类最优。**

发现一项实质缺口与若干可选项：

1. **Registry actor 无自动恢复** → TODO-154（可用性缺口）。actor panic 后不会自动
   重建；register/purge/revoke 会 fail-closed 并返回错误，不会静默继续写入。
2. 命名瑕疵（NegotiatedProtocol、MessageType 数值乱序、死字段等） → TODO-155。
3. 可选性能优化（DefaultHasher→xxhash、UDP envelope 改 Bytes） → TODO-156。
4. 观察项无需行动：`select_across_shards` 全 shard 扫描小规模无碍；
   `background_main` 中 purge JoinHandle 被 drop 不 abort 但因每次重启新建
   runtime 实际无泄漏（易误判，建议加注释）；supervise_component 一处 error!
   缩进错位（cosmetic）。
5. 确认的亮点：错误传播三段式（ErrorKind→Source/Retry/http_status）教科书级；
   Login token 掩码、MAX_LOGIN_BYTES 预认证上限、LKG 双代快照回退、
   fail-closed 令牌吊销均正确。

---

## 2026-08-21 性能专项评审：热路径扫描（2026-08-24 已复核）

对数据面热路径做了逐段扫描。结论：**HTTP/TCP 主链路（TLS→sniff→h1→QUIC relay）
已高度优化，无发现值得动的点**——read_chunk 零拷贝、httparse 零拷贝解析、
SniffPrefix 零复制转发、open_bi 的 now_or_never 快路径、vectored write 透传等均已到位。

发现三处优化候选，按预期收益排序：

1. **UDP 回包泵每包 3 次堆分配+拷贝** → TODO-157。`pump_udp_replies` 每个数据报
   经 `to_vec()`→rkyv→`Bytes::copy_from_slice` 三次分配；每 task 独立缓冲即可归零。
   注意 UDP pps 是当前 k6 基准的未覆盖盲区，按仓库门槛应先补基准再动手。
2. **每请求 host 归一化堆分配** → TODO-158。`VhostRouter::get` 每次调用
   `canonicalize_egress_host` 分配 String；可改栈上 ASCII 小写化零分配命中。
3. **每流 RoutingInfo 序列化 2 次分配+2 次写** → TODO-159。可与 initial_bytes
   合并为单次向量化写。
4. 观察项不动：`select_across_shards` 全 shard 扫描当前规模可忽略；h1 解析缓冲的
   chunk 拷贝是语义必需。

---

## 2026-08-21 抽象与可读性评审（2026-08-24 已复核）

评审了插件体系、trait 边界、泛型使用和代码可读性。结论：**扩展点少而准且全部真实
被使用（6 个内置协议插件无一例外走 PluginRegistry），零 dyn 开销处用泛型、运行时
多态处才用 Box<dyn>（全仓仅 4 处），注释文化解释“为什么”属范本级。**

待确认项：

1. **数据面流中变换无挂点** → TODO-160。ConnectionModule 只有 pre_admission/
   on_complete 两个生命周期端点钩子，header 清洗硬编码在 h1 driver 内。
2. **sniff 管道半硬编码** → TODO-161。SniffPolicy/detectors 每连接重建且不走
   registry/配置；AdmissionReq.token 是预留半成品字段。
3. **可读性债** → TODO-162（return_buffer 的 Option 舞蹈、control_client.rs 四职责
   混杂）+ R1 已并入 TODO-158（VhostRouter::get 两段重复通配扫描与零分配化一并处理）。
4. 不动的：error.rs 样板构造器保留显式风格（IDE 补全友好，见仁见智）；
   handle_quic_connection 359 行内部阶段清晰，拆分收益中等不强求。

---

## 2026-08-21 多视角专项评审（2026-08-24 已复核）

分四路并行扫描：安全攻击面、并发/死锁、配置校验、测试覆盖。结论：
**并发卫生验证通过**——全仓脚本扫描“锁获取跨 .await”模式仅 1 个候选，人工核实为误报
（dns_cache entry 守卫未跨 await）；admin socket 有界+超时+0600；cargo-audit 已在 CI 强制执行；
validate() 聚合式错误报告良好。新增两项记录：

1. **admin HTTP 解析器边界处理与单测缺口** → TODO-163。手写解析不可信输入的代码
   恰是最需要表驱动单测的地方，与已知“admin framing 硬化”事项配对。
2. **配置层缺少防御式重复监听端口校验** → TODO-164。底层 `ServerConfigFile::validate()`
   未直接查重，但当前 control-plane、override 和 snapshot 校验均会在 bind 前拒绝重复端口；
   这不是当前运行时依赖 EADDRINUSE 的未校验问题。
3. 无需行动：DNS single-flight/broadcast 模式正确；MSRV 钉死 1.95.0 带 clippy/rustfmt；
   quic/tls/h2c 认证路径由 ci-helpers 集成测试覆盖，可接受。

---

## 2026-08-24 review 复核结论

已对 TODO-152～TODO-164 逐项核对实现，并运行三个相关 crate 的测试：
`duotunnel-lib` 164 个、`duotunnel-server` 35 个、`duotunnel-ctld` 22 个全部通过。
结论如下：

- TODO-163 属于真实的 admin 请求 framing 正确性与测试缺口，应优先处理。缺失
  `Content-Length` 的 POST 可能在 header 结束处提前返回；重复 `Content-Length` 只取首个值，
  未拒绝冲突值。
- TODO-164 的低层校验遗漏属实，但生效配置会经过 `normalize_and_validate_routing`、override
  合并校验和 snapshot 校验，因此重复 listener 当前会在绑定前被明确拒绝。建议作为防御式校验
  和独立 API 一致性改进，不应按运行时安全缺陷排期。
- TODO-154 的自动重建/健康状态粒度缺口属实；actor 异常后的调用路径会 fail-closed、记录错误，
  不属于 review 中所称的“静默失败”。
- TODO-152、153、155～162 均为已确认的可观测性、配置能力、性能候选、架构扩展或可维护性项，
  其中 UDP/host/stream 相关优化必须先补 benchmark，再根据 before/after 数据实施。

## 2026-09-03 全项目代码与架构深度复核（2026-09-03 结论）

基于对全仓 5 个 crate（约 32.7k 行 Rust）的全面审查与代码具体实现上下文的核实，确认并新增 6 项实质性技术债务与工程改进点（TODO-165 ～ TODO-170），并对前期已有 TODO（TODO-154、TODO-158、TODO-163）进行了精准上下文校正：

1. **Client 端 EntryConnPool Actor 崩溃无自愈机制** → TODO-165。不同于 Server 端具备 `supervise_component` 线程级重启，Client 侧 `EntryConnPool` 的 actor 若 panic，虽然会标记 `actor_alive: false` 并 fail-closed，但因 pool 实例跨重连复用，一旦崩溃将导致客户端永久失效，必须重启进程。
2. **Server 端 QUIC 随机自签证书无法配置固定证书链** → TODO-166。Server 每次启动无条件通过 `generate_self_signed_cert()` 动态生成随机公私钥，使得客户端无法使用持久化的 CA 证书进行严格校验（`tls_skip_verify: false`），生产环境被迫妥协为跳过校验。
3. **H1 Driver 64 Header 上限与假挂起风险** → TODO-167。栈上固定 64 个 header slot，超限时 httparse 返回 `Status::Partial`。代码未检查缓冲区是否已含完整 `\r\n\r\n`，在客户端发完头后会死等 2.5s 超时或打满 8KB 报 431。
4. **H2 Sender 单流重置错误驱逐全局复用连接** → TODO-168。`send_via` 发生任何流错误（如对端发送 `RST_STREAM`）时均无条件清退全局 `H2Sender` 缓存，未区分 Stream 错误与底层 Connection 致命错误，导致高并发多路复用连接雪崩重连。
5. **TCP/QUIC 流中继核心代码重复** → TODO-169。`duotunnel-lib/src/engine/bridge.rs` 与 `relay.rs` 存在两份逻辑近乎完全一致的 relay 实现，需合并收敛。
6. **PhaseOutcome 错误类型擦除导致指标收集脆弱的字符串子串匹配** → TODO-170。结构化 `ErrorKind` 被转为非结构化 `Option<String>`，后续打标时使用长达 30 余行的 `err_lower.contains(...)` 反向倒推，易碎且有多余分配。
7. **历史项上下文校正**：
   - TODO-158（VhostRouter）：确认 `DashMap<String, T>::get` 原生支持 `&str` 查询，内部的 256 字节栈拷贝和 `unsafe from_utf8_unchecked` 纯属多余，重构时可彻底移除。
   - TODO-163（Admin HTTP framing）：确认 `cli.rs` 作为唯一客户端固定发送 `Content-Length: 0`，当前实际路径不触发截断，但协议防护和单测仍需补齐。

## 2026-09-04 独立复审（认证/安全面定向 + 静态检查）

复审范围：token 生命周期、TLS/QUIC 证书链、ctld admin socket、SQLite 存储参数化、VhostRouter、watch 认证、control_client 重连；另跑了 clippy（零警告）与全量测试（除 4 个依赖真实 TCP bind 的用例在禁网沙箱中 EPERM 外全部通过，非项目问题）。

结论：此前的发现与已有 TODO 高度重合，仅新增 1 项（TODO-171），并对 TODO-166 做两点补充：

1. **watch 流 fail-open 加固 → TODO-171（新增）。** `watch_addr` 默认绑定 `127.0.0.1:7788` 且 `watch_token` 可选；当管理员改绑非回环地址且忘配 token 时，仅输出一条 `watch_auth=false` 日志而不拒绝启动，明文 TCP 上任何可达客户端都能拉取完整路由快照。建议 fail-closed：非回环 watch_addr 必须配置 watch_token，否则拒绝启动。
2. **TODO-166 补充 ①：** `README.md` 快速开始中的 `tls_skip_verify: true # set false in production with a real cert` 在 TODO-166 落地前不可执行（服务端无任何配置固定证书的入口），应随 TODO-166 一并修正措辞或提前落地 TODO-166。
3. **TODO-166 补充 ②：** `duotunnel-lib/src/infra/pki.rs` 已有带 O_NOFOLLOW/uid 校验/fchmod 加固的持久化 CA 基础设施（`RootCa::load_or_generate`），目前仅服务于 ingress 动态 TLS 终止；TODO-166 落地时可直接复用，修复成本低于新增独立证书配置。
4. **需求登记：Server HA 与 client-server 关联登记 → TODO-172（新增，2026-09-04 用户需求）。** 多 server 实例对外服务时，需要在 ctld（或共享存储）维护 server↔client 在线关联矩阵（哪个 server 上有哪些 group/client 在线、健康状态），供路由/外部 LB 把 vhost 流量导到持有对应 client 连接的 server，并在单实例故障时导流。TODO-151（server 稳定 identity + 注册信息）是其前置基础；client 侧现仅有单 `server_addr`，多 server 端点故障转移也需一并支持。
5. **其余发现与已有记录重合确认：** admin framing 缺 Content-Length 的行为已包含在 TODO-163（`read_admin_request` 对无 Content-Length 的 POST 在 header 结束处提前返回，实际到 `cli.rs::handle_admin_request` 后被 serde 拒绝或本就不需要 body，无可利用影响，维持加固定位）；VhostRouter 通配符 add 不去重在现有调用链不可达（三处调用点均为快照全量重建 + ctld layer 拒绝重复 key），TODO-158 的简化方案落地后顺带消除；cargo-audit 已在 CI 强制执行（`ci.yml`），无需动作。正面确认：SQL 全参数化、token 仅存 SHA-256 哈希、`subtle` 恒定时间比较、日志脱敏、重连指数退避+抖动、`from_utf8_unchecked` 有前置 ASCII 校验。

