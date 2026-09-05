# DuoTunnel 问题驱动的优化设计

日期：2026-09-05。代码基线：`7f8fe625bff1f6aa5c7e98924e4f946196a433a0`。状态：**推荐设计，尚未实施**。

[问题证据](README.md) · [当前 TODO](../todo.md) · [项目架构约束](../spec/architecture-guidelines.md)

本文对每个问题综合考虑运行场景、领域含义、调用链、状态与资源生命周期、安全、性能、兼容迁移和验证，最后给出推荐方案。测试不是把问题分成“简单修复”和“架构设计”的依据，而是用来证明设计中的约束。文中的类型、路径、配置项是拟议接口，不表示现有代码已经提供它们。

## 决策总览

| 问题 | 综合判断与推荐方案 | 对应 TODO |
|---|---|---|
| 身份随机/损坏 CA 被覆盖 | 共享安全文件读取能力；隧道身份和 ingress 签发分属两个领域；显式初始化，加载不写盘 | 166、176、152、99、27 |
| watch 暴露及 epoch 信任歧义 | 先安全监听默认；认证/加密与 authority reset 各有明确契约，不因 TCP 重连自动获得换权威权限 | 171、178 |
| H2 缓存跨代串用 | generation 身份贯穿请求、路由和 sender；保持 snapshot 读模型，不建立全局清缓存广播 | 175、52、162 |
| H2 单流错误驱逐连接 | 共享领域化转发策略，错误作用域与 sender 实例身份分开，失效动作和重试动作独立决定 | 168、68、139、143 |
| authority 不一致及 H1 边界误判 | 统一解析契约、保留协议适配差异；先保持语义，再优化分配 | 177、158、167、164 |
| admin framing | ctld 自有 HTTP adapter，使用成熟语法解析器与受限 framing 状态机；mutation 不依赖原始 HTTP 文本 | 163 |
| actor 不可用与恢复承诺 | 保留 release abort；不可恢复 actor 退出上升为进程失败，依靠外部重启，不拼接可能损坏的快照 | 154、165 |
| 指标错误类型丢失 | 在阶段结束时形成统一 outcome，由 adapter 映射稳定标签，保留原始 source | 170、CR-AUDIT-3 |
| relay 重复与资源预算 | 收敛 TCP↔QUIC 核心；保留泛型 TLS 路径；容量所有权覆盖真实 body/relay/session 生命周期 | 169、142、153 |
| UDP/host/frame 热点 | 先统一正确性、所有权与协议上限；每个优化有同口径收益和内存上界 | 140、145、156–159 |
| HA 关联视图 | 稳定身份+带租约的在线视图；小规模以双连提供可用路径，视图不进入每请求依赖 | 172、151 |

## 1. 稳定身份：把加载、初始化、签发和轮换分开

### 上下文与责任

当前 tunnel QUIC 配置直接调用临时证书生成；ingress TLS 则使用进程级 RootCa/cache 动态签发。它们都涉及 PEM 和私钥文件，但信任对象、证书用途和轮换范围不同。直接让 tunnel 复用 ingress RootCa 单例，会让一次 ingress CA 变更影响 tunnel 身份，并把已有“解析失败自动重建”行为带到新的安全边界。

推荐保留现有 crate 层次，在 `duotunnel-lib/src/infra/pki/` 内拆出私有的 `material`、`files`、`issuer`、`cache` 职责；`mod.rs` 保留迁移期入口。公用 `CertificateMaterial` 只包含验证后的证书链/私钥和必要身份元数据，不能暴露任意落盘能力。进程 bootstrap 选择 tunnel TLS identity；ingress issuer/cache 由其自身 owner 创建和注入。无需先拆独立 crate 或定义一个多实现、动态分派的万能证书管理器。

### 行为契约

- `load_identity(paths) -> Result<CertificateMaterial>` 只读取和校验，不生成、不覆盖文件。
- `initialize_identity(destination, names) -> Result<...>` 是显式运维操作；只在目标不存在时创建，使用排他创建防止并发初始化覆盖。
- 已有 cert/key 任意一个缺失、损坏、不可读、不匹配都失败。私钥读取沿用权限、owner、symlink 防护，不通过另一路普通 read 绕过。
- tunnel bootstrap 将材料与传输参数组合为 QUIC server config；材料不是 `QuicTransportParams` 的流控字段。拟议配置放 `server.tunnel_tls`，传输参数继续放 `server.quic`。
- 推荐 `mode: files` 与显式 `mode: development_self_signed`；没有文件配置不隐式降级到不验证身份。client 原有 CA/server_name 配置继续可用。
- 身份材料的有效性与“这个具体 client 应信任谁”分开：服务端检查 key/chain；目标 server_name/SAN 通过严格客户端握手证明。不能把任意完整 PEM 当作可用身份。

### 持久化与轮换

第一批只做安全加载和显式初始化，不因修复覆盖问题自动实现完整 CA 轮换。若支持工具托管的成套更新，使用不可变版本目录保存 cert/key，完成文件和目录持久化后原子切换单个 manifest/pointer；两次独立 rename 不构成 cert/key 的原子事务。失败保留旧材料，禁止仅输出 error 然后使用未持久化的新身份。

外部证书管理器可继续提供两个 PEM 路径。reload 必须把两者作为同一候选校验，失败保留当前有效内存 config；文件事件仅触发重读，不作为“更新已完成”的证明。CA 更换需要分发新 trust root、重叠信任期、观察新连接成功后再撤旧 root，不能仅原子 swap acceptor。

### 取舍、迁移与验证

推荐“共享底层能力、隔离信任领域”，不选“一份 CA 全进程通用”或“所有启动失败都重新生成”。安全默认会改变旧开发配置，因此实现提交必须同步更新 quick start、CI fixture 和样例，提供明确的缺字段诊断；不留静默兼容降级。

契约测试覆盖损坏/单文件缺失/错配/权限/并发初始化，断言失败后文件字节未改变。真实 QUIC 握手覆盖正确 CA/SAN、错误 CA/SAN、重启后身份一致。持久化 fault injection 覆盖写第一文件后失败、切换前退出和切换后重启。用临时目录与实例化 PKI 测试对象隔离全局状态，测试不能操作真实部署凭证。

## 2. watch：传输信任和配置权威转换必须同时明确

### 当前问题及推荐边界

无 token 的非回环 watch 暴露快照；README 快速开始会引导该配置。另一方面，server 在每次 watch 连接建立时把 `authority_reset_allowed` 设为 true，与部分规格中的“不同 epoch 默认拒绝”矛盾。修监听地址不会解决冒充控制端或换 epoch 的语义。

`ctld/bootstrap` 将原始字段解析成经过校验的 `WatchEndpointPolicy`：绑定地址、认证方式、传输要求。`WatchServer` 使用已验证策略提供服务，协议 handler 不再凭 Option<Token> 自行推断部署安全级别。server watch transport 负责认证远端身份；revision policy 只接收明确的权威信息，不认识 socket 细节。

第一步默认回环，非回环无非空 token 拒绝启动；空白 token 不能等价于有效凭证。此步不宣称明文 bearer 已实现完整远程安全。远程生产推荐 TLS（有需要再加 mTLS）或已有受信安全隧道；无需重写现有 DTCP 为 QUIC。明确本地 TCP 同机用户也可能访问，回环不是用户级授权。

### 推荐权威转换策略

- 同 epoch：保持当前 sequence/hash 检查、幂等 ACK 和 Delta base 校验。
- 首次无 LKG/无既有 authority：通过已认证的控制端建立权威；开发回环无认证模式显式注明信任边界。
- 已有 authority 后不同 epoch：默认拒绝；换库/灾备是显式 reset 操作。不能因普通重连自动授权。
- reset 拟采用受控的 expected-old-epoch + target-epoch 许可，而非永远有效的 bool；只允许完整 Snapshot，成功应用并持久化后绑定新 epoch。失败重试只对同一目标幂等，不能变成任意 epoch 通行证。
- reset 的持久记录、LKG 和当前 authority 一起设计：重启不能误回旧 authority，也不能因为内存先消耗许可、磁盘失败就永远无法恢复。日志只记录 epoch/revision/原因，不记录 token。

这是一项建议中的兼容性变更，不在缓存修复提交顺手改变信任策略。保留当前部署可运行的期间，先修完整 cache key，再通过运维说明和迁移验收落地严格 reset。

验证覆盖 IPv4/IPv6 loopback、通配地址、空 token/错 token、同 epoch 重连、不同 epoch 未授权、精确授权、Delta 跨 epoch、reset 途中断线/持久化失败/重启。此处 hash 验证只能证明内容一致，不能代替认证控制端。

## 3. generation：从 wire revision 到请求快照保持身份完整

### 问题机理与建模

H2c route cache 和 H2c/TLS sender cache 使用 sequence，完整 generation 却是 epoch+sequence+hash。不同 epoch 同 sequence、旧 negative cache、迟到旧请求都会影响行为。只在 watch 线程广播 clear 会引入时序窗口，也遗漏未登记的长连接。

推荐 server 内 `GenerationKey { epoch: Arc<str>, sequence: u64 }`，由 RuntimeGeneration 生成并提供只读 accessor；wire 的 `ControlRevision` 保持现有布局，在边界转换，避免热路径每请求复制 epoch String。content hash 保留在 generation 用于验证与诊断，同 epoch/sequence 不同 hash 必须在 apply 层拒绝，不用第三个 cache key 字段掩盖协议错误。

每个请求持有自己的 immutable generation Arc，解析路由、获取 sender、记录 outcome 都使用这次请求的 generation。缓存键为 `(GenerationKey, CanonicalRouteHost)` 和 `(GenerationKey, RouteTarget)`，negative cache 也同样带 generation。

### 并发与生命周期

- 新发布 generation 后到达的新请求不得命中旧代路由。已有请求按已获取的快照完成；安全 revoke 则继续受 token fence/connection retirement 约束，不由缓存取代鉴权。
- 不以“收到请求的 sequence”来覆盖共享 current-generation 字段；旧请求晚执行不应把缓存退回旧代。
- 初版保留现有硬容量上限，删除跨 epoch `sequence - 1` 数字比较。可只保留当前代及仍有需要的旧代条目，但清理不是正确性的前提；generation 匹配才是前提。
- 老请求即使晚插入旧代 entry，也只能命中旧 key，并受容量约束；不能把新代条目变成旧代。淘汰不强制关闭持有中的 sender/流。
- 不把 request generation 自动当成“client 已应用新 upstream 配置”的证据。server→client 的 LoginResp/config 刷新行为需在联调中观察；ACK 只证明 server apply 边界。

### 模块与替代方案

generation 类型放 server `control/generation` 或现有 RuntimeGeneration 所在模块附近，跨协议缓存机制在 server `ingress/forwarding` 内共享。公共 lib 的 H2 sender 负责单 H2 transport，不依赖 ctld revision 或 server registry。

不选“全局 flush 广播”（存在竞态/连接登记成本）；不选“每请求彻底取消 cache”（有性能倒退且仍需 sender ownership）；进程内递增 ID 也可行，但引入第二种 revision 含义且不能直接对应控制日志，当前优先完整 epoch/sequence。

验收除了 key 相等单测，必须保持一条 H2c TCP 连接跨 epoch A/1→B/1，并验证路由改组、删除、None→Some；用两个有效在线 group 排除“旧连接刚好失效”掩盖问题。通过 barrier 控制旧请求晚完成，断言新请求仍用 B；TLS/H2 sender 的 key 同步验收。

## 4. H2：失败域、复用实例与重试是同一条转发链上的不同决策

### 上下文

每个下游连接持有 route/sender cache。公共 lib 里一个 `H2SenderCache` 包装在 QUIC stream 上的 Hyper H2 sender；外层 CachedSender 还持有 SelectedConnection。单个 request 返回 Err 不等于 H2 connection 失效，更不等于 QUIC connection 失效。旧请求与新 sender 可能使用相同 QUIC stable_id。

### 推荐领域接口

在 lib 的 H2 adapter 保留原始 error/source，输出有限的失败作用域：request/stream、H2 transport unavailable、QUIC transport lost、unknown；同时记录失败发生在 open/handshake/send/response-body 哪个阶段。不要仅靠 Hyper 错误文本判断。

server `ingress/forwarding/h2` 抽取 H2c/TLS 真正共享的部分：按 route/generation 选连接、获取 sender、按失败作用域失效、执行受限重试。协议 handler 继续负责 authority/SNI、响应适配和协议 shutdown。先抽共享 policy/cache，再迁移两个调用者，避免顺手统一 TLS/H1 的 pinned-route 语义。

cache 获取返回 `SenderLease`（命名暂定），携带 route key、SelectedConnection 和 sender 实例身份。外层失效比较持有的 H2 cache Arc 身份，而不是只比较 QUIC stable_id；内层继续以实际 SendRequest Arc 做 compare-and-swap。driver 退出只清自己的内层 sender，不把旧 driver 的退出升级成删除外层后来创建的新对象。

| 观察到的事实 | cache/registry 动作 | 请求动作 |
|---|---|---|
| 单流 RST、body producer 错误、请求取消，transport 尚可用 | 保持其他流与 sender | 当前请求按业务语义失败；取消不重试 |
| sender/driver 已终止、H2 无法接新流 | 使对应内层实例不可复用；外层 selected QUIC 可保留并重建 H2 | 仅 replay 合格且预算/总 deadline 允许才重试 |
| GOAWAY/停止接新流 | 对新流更换 H2 sender；旧流自行 drain | 不把整个 QUIC client unregister |
| QUIC close/retired | 按连接实例 retire/unregister；不再分配新流 | 按 replay 条件重新选连接 |
| 不明错误 | 不凭文本扩大到 QUIC 故障；结合 driver/closed 状态处理 | 返回明确 unknown，并观测，不无限 retry |

当前 `is_ready` 是提示而不是持久健康证明；“尚未 ready”也不能直接被设计成“关闭所有关联流”。优先使用现有 driver ownership 和可确认的关闭状态。实现前以锁定 Hyper 版本建立 RST/GOAWAY 行为测试，确认 API 暴露的信息；如果无法可靠解析某个错误作用域，保守返回 unknown，不引入脆弱字符串分类。

### 重试、资源和验证

保留已有 GET/HEAD/OPTIONS + empty/end-stream 白名单；本次不扩大到 POST/gRPC，也不缓存任意 request body。请求总 deadline 跨所有尝试，per-attempt open timeout 不能反复延长总预算。响应头/body 已交付下游后不自动重放；body 失败只在 completion 观测。admission 的 logical request permit 跨重试持有，attempt transport permit 按次释放，避免重复计数或绕开限额。

保留 rebuild mutex 的单飞能力，覆盖 winner 取消后其他 waiter 能继续；不在这次顺手做 CAS+Notify 或多 sender 池。测试使用真实 Hyper peer/QUIC stream 注入 RST、GOAWAY、EOF，另用 barrier 确定制造“旧失败晚于新 sender 插入”。断言未受影响流成功、重建次数有界、同 QUIC stable_id 的新实例不被删除、取消不重试、permit 回零。性能验收测 sender rebuild 次数和 p99，不声称修复前一定发生全局雪崩。

## 5. authority 与 H1：修复解析差异时保留产品语义

当前 canonicalizer 同时处理配置中的 wildcard 和请求 authority；H2c 又自行 split(':')。直接换成同一宽松函数能修 IPv6，但不能让请求输入自动继承配置 wildcard 的权限。

推荐 lib `protocol/authority` 定义两个明确入口：配置规则 `RoutePattern`（可 wildcard）与请求 `RequestAuthority`（不得 wildcard）；都复用 host/port 解析底层。提供规范化的 `CanonicalRouteHost`，cache、single_authority 和 lookup 使用同一语义。强类型负责表示校验结果，不把所有字符串都包装成新类型。

第一步修 H2c IPv6 并用 characterization tests 固定既有行为：大小写、可选端口、bracketed IPv6、空/非法 authority、现有非 ASCII 行为。当前路由忽略端口，应明确保持；如果 single_authority 将来要区分端口，作为独立安全语义变更，不与修 split(':') 混合。IDNA、尾点、IPv6 等价文本的标准化会改变已有配置匹配，需分别定义迁移与兼容规则，不顺手加入。

VhostRouter 已归一化的 String 后面的栈拷贝/unsafe 可移除，并共用一个 lookup 实现。之后才评估 Cow/借用的 ASCII lowercase 快路径；已小写域名可借用，大写/需要变换时拥有数据。不要以“删除 unsafe”声称删除了全部分配。wildcard 优先级是路由产品契约，trie 选型不能替代对重叠规则的定义。

H1 的 64 header 上限返回 TooManyHeaders，当前 400；推荐直接映射 431，数量上限仍为 64，除非兼容需求证明需要扩大。提取纯 `parse_head(bytes, limits)` 返回 NeedMore/Parsed/Rejected，driver 负责读取/超时/写拒绝响应。纯 parser 只在完整头长度范围检查 header byte budget，不能把首包中附带 body 的长度算成 header，也不能接受超上限但已 Complete 的头。覆盖 63/64/65、完整/分片、body 随首包、截断/无效头、重复 Host、CL/TE 与 Expect。

请求解析正确性无需等性能 gate；优化 parser scratch/host allocation 才需要基准。重复 listener 校验共用规则函数，不能让 config/merged routing/snapshot 三份逻辑各自演进。

## 6. admin：受限 HTTP adapter 与 mutation 领域解耦

现有 read_admin_request 在 runtime/app，业务路由在 bootstrap/cli，后者直接解析原始 HTTP 字符串。只补 Content-Length 分支会继续让传输、路由、幂等指纹与命令混在一起。

推荐 ctld 增加私有 `admin/{framing,request,handler,server}` 模块：

- framing：字节输入到完整请求边界；用 httparse 解析语法，自己只实现受限 body framing/预算状态机。
- request：校验 method/path/query/header/body，形成现有 AdminMutation 或只读 query；fingerprint 来自规范化 mutation，不能来自原始 HTTP 字节。
- handler：调用现有 ControlService，映射 AdminErrorKind，不知道 UnixSocket 读写。
- server：0600 socket、连接 permit、绝对 deadline、读写和关闭；bootstrap/cli 仅作为客户端构建请求。

ctld 尚无直接 Hyper 依赖。对现有本地、单请求、无 chunked 的 API，推荐先增加 httparse 直接依赖并保持有限协议面；比扩大到完整 HTTP server 的迁移范围小。若未来明确需要 keep-alive/streaming/public admin，再用 Hyper 替换 adapter，mutation 不动。不能为了少一个依赖继续自己解析 HTTP 语法，也不需要为当前问题加入 web framework。

framing 状态为 ReadingHead→ReadingBody→Complete/Rejected。明确契约：总预算延续 256KiB、绝对请求期限延续 5s、header count 有独立上限；所有 CL 先 decimal/checked_add/预算校验，再读 body。拒绝重复 CL（包括相同值）、TE、TE+CL、非法数值/负数/溢出；POST 要求 CL，GET 缺失 CL 按 0。只 dispatch 声明 body 范围；同一读取中已有尾随数据则拒绝，未来才到达的 pipeline 字节不能靠猜测检测，单请求响应后关闭即可。EOF 不完整则失败。UTF-8/JSON 是 request adapter 的内容规则，不是 socket framing 的先决条件。

迁移保持 CLI 的 CL=0、Idempotency-Key、响应文本和现有 200/错误状态契约，保持 token create/rotate 的持久脱敏 marker 和重启后 410 语义。测试覆盖任意分片边界、多个请求拼接、超长声明、EOF、deadline；mutation spy 断言非法输入从未执行、合法输入只执行一次。再跑 Unix socket 真实 CLI 往返测试，验证 body 上限与错误映射，避免只测 parser 而破坏操作接口。

## 7. actor：可恢复的是明确的运行态，不能恢复未知的部分写入

release 使用 panic=abort；组件 catch_unwind 不能捕获此类 panic。EntryConnPool 跨多个 service 持有 Arc，仅替换 supervisor 局部变量不会替换 listener 引用；registry 则涉及 revoke fence、注册容量和快照一致性。自动“把读快照恢复为 actor”不是可靠的恢复协议。

推荐保持 release abort。对于正常 Err/unexpected actor exit、channel 不可用等可观测故障，actor owner 持有 JoinHandle/失败通知，立即上报 FatalRuntimeFailure，进程 readiness=false、禁止新工作、取消根生命周期、在 deadline 内结束存量后非零退出。外部进程管理器重启完整 composition root。状态变成失败后不可被迟到的 active count/ready 更新重新标成健康。

退出信号必须区分 expected shutdown 与 fatal failure；资源归还即使失败也不能无限等待已死 actor 的 ACK。可以复用 server 已有健康失败通道；client 在 RuntimeEngine 增加该 capability 的 owner。不要为两个调用点先造通用 actor framework，也不要用轮询 bool 来掩盖 JoinHandle 丢失。

只有将来产品明确要求“进程不能重启”，才考虑 unwind 与全组件 ownership 重建：先 fence/停止新入口、关闭旧连接、确认 task/permit 生命周期结束、从控制面重建新的 generation/registry，再重新开放入口。该方案必须证明旧任务不能向新代写状态，不能仅写一个 panic 单测就上线。

验证：在可控测试 owner 中制造 channel close、unexpected exit、正常 shutdown，检查失败只上报一次且不可恢复 ready；用 release 子进程注入 panic/退出验证进程策略，并在部署 fixture 验证实际重启而非仅 readiness。测试进程内 unwind 行为仅证明相应分支，不代表 release。外部重启策略和最大退避是部署文档的一部分。[Cargo panic 策略](https://doc.rust-lang.org/stable/cargo/reference/profiles.html#panic)

## 8. outcome、relay 和 admission：让观测与资源寿命遵守同一契约

PhaseOutcome 的错误不能只剩 String。推荐统一 `OutcomeError { kind: Option<ErrorKind>, phase, source }`，unknown 显式表达；构造入口保证成功与 error 不矛盾。阶段边界提取结构化类型，lib 暴露有限标签语义，Prometheus adapter 才映射 label；显示文本变化不得改变归类。先迁移所有 producer/consumer，再删除字符串倒推，不允许过渡期间多处各自定义优先级。

区分 connection outcome、logical request outcome、attempt outcome、body/relay completion；H2 response headers 返回不是 body 完成。可以先只修现有 connection-level PhaseOutcome 的类型，再按实际 request owner 扩展，不在一次 PR 中重定义所有历史指标。高基数 host/token/error-string 不进入指标 label。

relay 重复代码收敛为一个 TCP↔QUIC implementation，既有两个公共入口暂留 wrapper 以保持参数和计数字义。generic TLS relay 保留独立 split 适配；不因去重把 TCP into_split 退回 generic。初次收敛保持首包目前不计入返回计数的行为，再通过明确新指标契约决定是否将初始字节纳入；不能顺手让 dashboard 口径变化。测试双向数据完整、半关闭、首包仅发送一次、错误/取消、buffer 参数和返回计数。

admission 以 owner 而非函数调用次数计数：HTTP logical request permit 持有到 body EOF/error/drop，重试复用 logical permit；每 attempt 另有 transport permit；raw relay 持有到双向结束；UDP session 与 queued bytes 各自独立。它们可复用已有 AdmissionController，但不能共用一个模糊的“active”预算。body wrapper/RAII guard 是生命周期对象，不是插件功能；优先在协议 adapter 管理，不把 permit 塞进所有业务模型。

UDP 配额配置保持默认并加硬上界、组合内存预算。reload 降低额度不应粗暴 revoke 已有流；推荐拒绝新增直到占用回落，安全 revoke 仍按独立 fence 立即执行。先验证 success/error/drop/cancel/timeout/shutdown/retry 全部归还，再在慢客户端/慢上游下测 p99、RSS 和 rejection。正常路径优化与过载治理的效果不能混在一个吞吐百分比里。

## 9. 性能：先确定所有权与协议语义，再减少成本

### UDP

当前 payload Vec→AlignedVec→owned Bytes 的链路存在。建议保持 wire format 做第一轮实验：验证是否可用 Bytes owner 包装直接持有序列化缓冲、减少最后一次拷贝；对齐、Send/生命周期以及 Quinn 释放时间由测试证明。per-task scratch 只用于未交给 Quinn 的暂存，不能在队列仍持有时复写。buffer pool 必须有最大数量/容量并覆盖 burst、拥塞和取消，零拷贝可能增加峰值持有内存。

发送上限取现有协议 codec cap 与 conn.max_datagram_size 的较小值，并预留真实 envelope overhead；能力不支持和单包 TooLarge 与 connection closed 不同。TooLarge 应按包丢弃并计数，不默认结束整个 session；读取大包需可靠发现截断（例如足够的接收缓冲/平台能力），不能将截断的数据当作完整业务包转发。扩大 wire cap、紧凑 header、分片重组均独立版本化，不夹带进 buffer 优化。[Quinn datagram 契约](https://docs.rs/quinn/0.11.9/quinn/struct.Connection.html#method.send_datagram)

### host 与 routing frame

host 优化继承第 5 节解析契约。routing frame 先测分配/CPU：现有两次 write_all 没有额外 flush。若 vectored，提供完成全部切片的写入循环并处理 0/短写/取消/错误；不得跨 await 借 thread-local scratch。合并到一个 Vec 会复制首包，可能比两次写更差；Quinn 用户态队列写次数也不等于 syscall 次数。

### 实验门槛

沿用 T5/dial9/Criterion；新 profiler 是可选手段。每次只改变一个成本源，记录 build SHA、有效配置、CPU contract、负载窗口、成功率、dropped iterations、p99、RSS 与分配数。UDP 包长/pps/session churn、H1 小请求、H2 单/多连接、bulk relay 分别解释同一改动的代价，不预设一定改进。没有 baseline 的项目保留为 Candidate，不创建“必做优化”承诺。

## 10. HA：在线关联视图、实际可达路径与配置作用域共同设计

已有用户需求包含 server↔client 关联登记，不能用“双连即可”替代该需求。推荐第一期同时提供最小在线视图和双 server 可用路径，完整放置调度与分布式存储后置。

server 拥有隧道会话，因此不是运行态无状态。为 server 注册稳定 server_id，并加每次启动的 incarnation；client 的持久身份与 token/QUIC stable_id 区分，token 会 rotate，QUIC ID 只在连接寿命内有效。先明确视图粒度：group 在线可先满足导流；需要 client 级矩阵时为 client 提供可认证/校验的 identity，不能把用户自报 ID 直接当授权主体。

presence 建议 `{server_id, incarnation, event_seq, group_counts, lease}`，ctld 以服务端接收时间维护租约；乱序/重复事件幂等，旧 incarnation 的 disconnect 不能删除新实例登记。增量事件用于及时性，周期全量 reconciliation 修复丢事件；presence 不混入 routing revision/hash，不因 heartbeat 写一遍 LKG。ctld 重启可把 presence 置 unknown 等待重报，不能把持久旧记录当在线证明。

client 每个目标建独立 `ServerSession`/pool，避免所有目标混入同一个带全局 egress_rules 的 EntryConnPool；当前 run_pool 任意 fatal slot error 会取消所有 slots，多目标不能照搬为“一台认证错误使所有健康目标退出”。本地 listener 由一个 owner 管理，选定目标后取该目标配置和连接；UDP 会话 pin 到具体目标，不能逐包漂移。multi-server 配置、token scope、upstream config 不一致必须显式拒绝/隔离，不能用最后收到的 LoginResp 覆盖所有目标。

默认两台同时连，退避和健康各自独立，连接数/FD/UDP budget 计算双倍成本；现有单 server_addr 迁移为一个 target 的兼容入口，禁止 legacy 与新 targets 混用产生歧义。client 收的是 LoginResp.config，不直接参与 ctld→server Snapshot ACK。

入口侧需要“此 server 上此 group 可用”检查。LB 不支持按 group/host 选路时，presence 视图本身不会修好导流；部署第一期要求配置组在两个 server 都可达，或明确给出支持该能力的路由 adapter。数据面每个请求不能同步查 ctld/SQLite，ctld 故障期间受限的本地视图与探活继续工作，过期状态明确为 unknown/unavailable。

存量 TCP/QUIC 会话随故障断开，应用重连；不承诺透明迁移。HA 验收必须包含流量命中缺 group 的 server、单实例退出/恢复、网络分区、迟到 disconnect、ctld 重启、token rotate、两个目标配置差异和 UDP reply 归属。完整 tenant-scoped Snapshot（151）不是最小 server identity/presence 的前置，避免依赖链无限扩大。

## 11. 提交顺序、完成条件与回滚

以下是逐步落实上述综合方案的提交边界，不是按“能否单测”给问题贴标签。每个行为提交都包含领域约束的测试、调用方迁移和对应文档。

| 提交 | 内容 | 必须保持/证明 | 依赖与回滚边界 |
|---|---|---|---|
| S0 | 当前证据、TODO 校正、设计与复现材料 | 方案不标成已实现，ID 无歧义 | 当前分支；文档可独立审阅 |
| S1a | PKI 只读加载/显式初始化，移除损坏时覆盖 | 不改真实已有身份，错误不写盘 | 176；禁止回退到自动覆盖 |
| S1b | tunnel 稳定身份配置+样例迁移 | 严格验证真实握手通过 | 166 依赖 S1a 底层材料；不暗中恢复 insecure fallback |
| S2 | generation key + 两条 H2 路径接线 | 跨 epoch 同 sequence、迟到旧请求隔离 | 175；无需等待完整 control_client 拆分或 reset 政策 |
| S3 | authority 契约与 H2c/H1 接线 | IPv6、端口/大小写语义、header 上限 | 177/167/158 清理；性能优化另做 |
| S4 | H2 forwarding policy/cache identity 收敛 | 两层失效、一条流失败不影响另一条、重试预算 | 168；共享 code 先等价迁移后改策略，基于 S2 key |
| S5 | admin adapter 与严格 framing | CLI/幂等行为保持、非法输入无 mutation | 163；不顺手变 API/DB schema |
| S6a | watch bind/token 安全默认+文档 | 默认回环，远程无 token 拒绝 | 171 可与前面独立实施 |
| S6b | 远程安全 transport + 显式 authority reset | 身份可信、reset 重启幂等、不降级 | 171/178，依赖身份材料和 S2；部署迁移需操作说明 |
| S7 | actor failure→进程退出、typed outcome | 失败不可恢复 ready、release 子进程行为 | 154/165/170；owner 重构与行为修改分提交 |
| S8 | relay 收敛、逐域 admission | 数据/半关闭/统计保持，所有释放路径正确 | 169/142/153；不改变 wire format |
| S9 | 单项热点实验与最小 HA | 原始 benchmark artifact；真实 group 导流 | 140 等、172；HA presence 独立于性能任务 |

control_client 模块拆分（162）优先提取纯 revision policy、LKG adapter、watch transport、apply/security orchestration；实际 apply owner 保留一个，不能让各子模块自行发布 generation 或操作 readiness。先移动并保持行为，再在对应 S2/S6b 改契约，避免大重构阻塞已确认的 cache bug。

所有行为 PR：相关契约测试 + workspace tests + clippy；涉及网络/系统的测试必须在可绑定端口的环境执行。性能项另附基准，不能把测试通过当收益证据。安全修复不提供“恢复不安全旧行为”的回滚开关；可回退二进制时需说明旧版配置兼容和安全暴露代价。

设计验收：每个问题有推荐方案、未采用方案的理由、真实 owner、失败/取消边界、兼容迁移和可观察的验证结果。保留的开放决策仅限部署输入（外部 LB 能力、证书来源、HA 客户端身份方式/可接受恢复时限）；这些不阻塞 S1–S5 等既有单实例正确性工作。
