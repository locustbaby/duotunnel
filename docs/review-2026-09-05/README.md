# 代码与 TODO 独立复核 — 2026-09-05

> 后续已在 `codex/review-optimization-design` 整理 [逐问题优化设计](optimization-design.md) 并校正 [TODO](../todo.md)。本报告保留评审时的证据；设计仍未实施。

审查基线：`7f8fe625bff1f6aa5c7e98924e4f946196a433a0`，开始时工作树干净。本次只增加审查报告与复现材料，没有修改业务实现或直接重写原 TODO。

结论：应优先补齐身份验证、配置代际与故障恢复的正确性，再做热路径优化。现有 TODO 混合了真实问题、已修复事项、过时实现描述、未经测量的性能推断；不能直接作为实施清单。“最工业化”在这里意味着失败语义明确、状态归属清楚、有可复现验收、复杂度与实际负载相称。

## 范围与证据边界

- 按项目要求先使用 CodeGraph，沿调用链检查 QUIC 证书、H1/H2、vhost、watch/apply、连接池与 inflight、UDP、relay、admin framing、指标分类。
- 阅读 `docs/todo.md`、架构规格、726 任务拆分与相关运行时设计记录、README、配置与 CI；历史报告视为背景，不把其中的“已确认”当作当前代码证据。
- `cargo test --workspace --locked`：248 个测试通过（client 27、ctld 22、lib 164、server 35）；其余 workspace binary/doc-test target 无测试。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`：通过。
- 最初沙箱测试被本地端口绑定权限阻止；在允许绑定端口的环境重跑后全量通过，不能把最初 EPERM 算作代码失败。
- 额外用实际编译的库复现了 IPv6 路由归一化问题、CA 损坏文件被替换；用实际 httparse 依赖验证了 64/65/128 个 header 的行为。
- 未运行 Linux 性能基准、全拓扑长稳、故障注入矩阵或 H2 RST/跨 epoch 端到端实验。下文区分代码确定的行为与尚待系统验收的影响，不承诺已发现所有缺陷。

## 1. 应优先处理的代码问题

### F1 · P1：H2 缓存键遗漏 epoch，可能复用另一代控制面的路由

证据：

- `duotunnel-server/ingress/plugins/h2c/mod.rs:213–226` 只读取 `runtime.sequence()`，route cache key 为 `(sequence, host)`；sender cache 同样只含数字 sequence（85 行）。
- `duotunnel-server/ingress/plugins/tls/mod.rs:23` 的 sender cache 也只含数字 sequence。
- `duotunnel-server/bootstrap/mod.rs:139–179` 的 generation 明明同时具有 epoch 和 sequence。
- `duotunnel-server/control/control_client.rs:469` 在每次 watch 建连时允许 authority reset；587–595 行允许不同 epoch 的完整 Snapshot 被应用。
- `listener_mgr.rs` 按 port/kind 复用既有监听器；更换 HTTP 路由不等于关闭所有现存 H2 TCP connection。

触发：现存 H2c 连接曾缓存 epoch A/sequence 1 的 host→group A；控制面换库或重新初始化为 epoch B，并发布 sequence 1 的 host→group B。新请求可能命中旧 `(1, host)` 缓存，即便 RuntimeGeneration 已正确发布。旧缓存的 None 也可能掩盖新添加的路由。普通使用同一持久数据库的 ctld 重启不必然改变 epoch。

建议：cache key 使用完整 `{epoch, sequence}`，或进程内严格递增、每次 publication 唯一的 generation ID；两套 H2 sender cache 一起修改。跨 epoch 不应用 `sequence - 1` 的保留规则。新增“同一 H2 连接、跨 epoch 相同 sequence、路由变更/新增/删除”的集成测试，包含两个 group 同时在线的场景。

置信度：调用链与键冲突确定；尚未做完整网络复现。

### F2 · P1：QUIC 服务身份无法稳定验证（TODO-166 属实）

`duotunnel-lib/src/transport/quic.rs:115–126` 无条件生成临时证书；`pki.rs:601–604` 固定签发 `localhost`。客户端存在 `tls_ca_cert` 和 `tls_server_name`，服务端却没有对应的固定 tunnel cert/key 加载入口。README 的“生产设置 false 并使用真实证书”缺少可执行服务端步骤。

建议：把隧道身份 TLS 与 ingress 动态签证分开，提供 PEM certificate chain/key 配置；显式配置失败必须停止启动，校验私钥匹配与 SAN，并提供严格验证的集成样例。自签模式只作为显式开发模式。不要仅提高 skip-verify 日志级别，也不要为了节省配置代码直接共享 ingress CA 的整个生命周期。

### F3 · P1：持久 CA 损坏会触发重新生成并覆盖文件

`duotunnel-lib/src/infra/pki.rs:200–281` 将读取/解析失败落入生成分支，并向原路径写入新 cert/key；写盘失败仅记录日志，仍返回内存中的新 CA。

实际复现：在 `/tmp` 创建内容损坏的既有 ca.crt/ca.key，调用生产公开入口 `init_cert_cache`；函数正常返回，两文件变成新的 PEM。这里只操作临时 fixture，没有读取或覆盖项目真实凭证。

影响：原先信任旧 CA 的客户端失去信任；部分写成功可能留下不配对的文件；只读部署可能在每次重启得到不同身份。这是可用性/信任连续性问题，不是已经证明的远程利用。

建议：区分“首次显式初始化”和“加载既有身份”。任何已有文件损坏、缺少配对文件或读取失败都应返回错误；证书更新先完整校验、成套持久化，再切换内存状态。生产启动用 Result 传播错误，不依赖 expect。TODO-166 若复用 PKI，应只复用已验证的加载/权限工具，不照搬此 fallback 策略。

### F4 · P1（非回环部署）：watch 未认证暴露，而且 README 会引导用户进入此配置

`duotunnel-ctld/src/control/watch.rs:187` 在未配置 token 时放行；配置校验没有非回环约束。`README.md:83` 的快速开始直接给出 `0.0.0.0:7788`，没有 watch_token。故 TODO-171 不只是“管理员以后可能误配”。

建议：快速开始绑定回环；非回环要求非空认证凭证。远程控制面需要认证服务器身份并加密的通道（例如 TLS/mTLS 或已认证的安全隧道），单加明文 bearer token 只解决访问控制。无需为此重写为 QUIC。回环地址也只是缩小可达范围，并不等于同机所有用户都可信。

验收：IPv4/IPv6 回环、通配地址、空/空白 token、错误 token；以部署实际解析出的监听地址判断。安全默认与证书工作应提升到生产上线前，而不是当前 Low/Medium 排期。

### F5 · P2：明文 H2 的 IPv6 authority 被截断

`duotunnel-server/ingress/plugins/h2c/mod.rs:212`：`host.split(':').next()` 将 `[::1]:8080` 变成 `[`，随后 vhost 查找失败。公共 canonicalizer/router 本身支持 bracketed IPv6，且已有测试。

复现输出：`current_key="[", current_match=None, canonical_match=Some("ipv6-backend")`。

建议：共享 authority 解析/host 归一化路径，cache key、single_authority 比较和实际 lookup 使用相同规范值。补 IPv6 带/不带端口、域名大小写、无效 authority 测试。此处修复无需新建 IPv6 resolver 插件，也不应被 TODO-73 的大需求阻塞。

### F6 · P2：H2 单流错误导致两层缓存清退（TODO-168 部分正确，方案不完整）

- 公共库 `duotunnel-lib/src/proxy/h2_proxy.rs:154–159` 在任意 send_via 错误后清理 sender。
- H2c handler `mod.rs:301–307`、TLS handler `mod.rs:303–309` 又无条件移除外层缓存。
- 外层 invalidate 仅比较 QUIC stable_id；同一 QUIC connection 上创建的新 H2 sender 仍可能被旧请求的迟到错误移除。

所以只修公共库无法解决问题。缓存是每个下游连接所持有的 route/sender cache，不是文档所称的“全局 vhost 缓存”。清缓存不会直接杀死所有已发出的流；额外 QUIC stream/H2 handshake 确定存在，“全局雪崩”和 ALPN 重握手不能据此直接声称。

建议：统一错误作用域（request/stream、H2 connection、QUIC connection）与失效策略；外层以 H2 sender 实例身份或代际做 compare-and-remove，公共库继续保留已有的指针身份保护。正常 GOAWAY 是停止接新流并 drain，不必描述成整个 QUIC 连接致命错误。保留 body 可重放性与重试预算。以并发两个流、一个 RST、迟到失败、旧/新 sender 同 QUIC ID 为验收场景。

### F7 · P2：panic 恢复的设计承诺与 release 构建不一致

`Cargo.toml:21` 为 `panic="abort"`，而 `runtime/supervisor.rs:155` 用 catch_unwind。release panic 直接结束整个进程，无法进入该捕获分支，也不会执行 actor Drop 清理。

因此 TODO-154/165 的“panic 后只剩死 actor、在进程内恢复”并非当前默认 release 的行为；它描述的是 unwind/task 异常退出等另一类故障。actor 异常终止时的健康传播仍值得补，但没有确定的线上 panic 触发点，不能把假设写成已发生故障。

建议：先明确故障模型。优先保持 fail-closed，actor 不可用就让进程失败，交给部署进程管理器重启；要做进程内重建，必须定义 registry/pool、listener、slot/permit、token fence 的整体 ownership 和恢复协议。不能从可能部分更新的 DashMap/ArcSwap 拼回“健康状态”。若选择 unwind，显式更改构建策略并证明隔离成立。release 行为用子进程故障注入验收；普通 cargo test 的 panic 策略不能证明 release 的恢复能力。[Cargo profiles](https://doc.rust-lang.org/stable/cargo/reference/profiles.html)

### F8 · P2：admin framing 加固属实，还有未检查的长度加法

`duotunnel-ctld/src/runtime/app.rs:484–528` 对缺失/非法 CL 退化为 0、重复 CL 只采一项，返回时没有按声明长度切片；520 行 `header_end + expected_len` 未 checked_add，也没有先拒绝大于总请求预算的声明长度。极端 usize 长度在 debug 可溢出 panic，在 release 可回绕。这不是暴露公网的接口：Unix socket 0600 是重要影响范围约束。

建议：用现有 HTTP 库在 UnixStream 上服务，或把严格受限的单请求解析器抽成纯 framing 状态机；拒绝冲突 CL/不支持的 TE、明确 POST 长度要求、提前 checked_add 和预算校验、严格处理尾随字节，保持现有连接数与超时上限。加正常/逐字节分片/非法长度/溢出/提前 EOF/重复头/TE+CL 测试，不仅增加成功路径单测。

## 2. 最近 21 项 TODO 复核

| TODO | 当前判断 | 更合适的处理 |
|---|---|---|
| 152 | 不安全 TLS 开关可观测性项，不能代替身份修复 | 结构化启动日志和固定 label 指标足够；并入 166，不必做大 banner |
| 153 | UDP 配额常量存在 | 配置化保持默认值；同时验证组合预算、队列字节与容量耗尽语义 |
| 154 | actor fail-closed 后恢复能力需补；自动拼快照恢复不是已证明安全的方案 | 按 F7 先确定 process/component 故障边界 |
| 155 | 命名/可读性清理，非功能缺陷 | 不改 wire 数值；NegotiatedProtocol→Session 属风格判断，不必专门立项 |
| 156 | 性能候选，没有量化收益 | 保留 benchmark gate；哈希换型不应仅凭算法名字决定 |
| 157 | payload→rkyv→Bytes 的复制链存在；“per-task buffer 即归零”不成立 | Quinn 持有 owned Bytes，任务栈 buffer 不能发送后立即复用；先测分配再设计 ownership/回收 |
| 158 | String 分配与多余栈拷贝均存在 | 先删除已经归一化 String 后的重复拷贝/unsafe；这不等于零分配。Cow/ASCII 借用快路径另行测试与测量 |
| 159 | 两次组包分配存在；“多一次 flush”错误 | send_message 只有 write_all，没有 flush。vectored write 可能短写，必须完整推进切片；不能只调一次 write_vectored |
| 160 | 扩展需求，不是当前 bug | 真实 header/body filter 需求出现后再做；保持 H1/H2 行为一致 |
| 161 | detector 集合不可配置属实；“每次构建 detector 列表”错误 | SniffRuntime 只存 policy 和 static slice 引用，new 不构建/分配列表。不要以不存在的分配为理由引入 registry |
| 162 | 职责拆分有价值，收益属于维护性 | control_client 可以先纯移动拆模块，再单独改行为；局部 Option 写法低优先级 |
| 163 | framing 与测试缺口属实，CL=0 CLI 限定影响范围 | 按 F8 补完整边界；“所有调用方永远只有 CLI”不应成为解析器契约 |
| 164 | 独立 validate 防御性缺口；有效 routing 已查重复端口 | 保持低优先级；共用校验函数，避免多份规则长期漂移 |
| 165 | actor 生命周期缺口部分属实，panic 论证忽略 release abort | F7；仅换 supervisor 持有的 Arc 不会自动更新所有 listener 的旧 Arc |
| 166 | 属实，生产身份基础 | F2/F3；固定身份加载优先于告警和微优化 |
| 167 | 核心故障机理错误 | 65 headers 返回 Err(TooManyHeaders)，当前立即 400，不是 Partial 等待。可改为显式 431；不要加入猜测式 CRLFCRLF 检测 |
| 168 | 属实但作用域与修复范围需修正 | F6；两层清退、实例身份、GOAWAY drain、RST 隔离一起考虑 |
| 169 | 两份 TCP↔QUIC relay 实现确实重复 | 合并核心、保留兼容 wrapper；保留 TCP into_split 与泛型 TLS fallback 的差异；明确首包是否计入字节统计 |
| 170 | Option<String> 导致字符串反推分类属实 | 传递 ErrorKind/稳定 outcome enum；保留展示文本，unknown 明确兜底；优先级高于纯微优化 |
| 171 | 属实，README 增加了实际暴露可能 | F4；非回环认证与远程传输认证/加密分别验收 |
| 172 | HA 产品需求属实，但 registry 不是唯一或总是最优起点 | 先做 client 双连两个 server + group readiness；复杂拓扑再引入带租约的在线视图 |

TODO-167 本机使用实际 httparse 依赖的探针结果：

```text
64 headers: Ok(Complete(722))
65 headers: Err(TooManyHeaders)
128 headers: Err(TooManyHeaders)
Header size: 32
```

因此文档“128 × 16B = 2KB”也不适用于本机 64 位布局：128 个 Header 为 4KB。是否增加数量上限是产品兼容性/资源预算决定，而不是修复该不存在的 Partial 挂起。

关于 TODO-157，Quinn 的 datagram 上限还随 peer 与 path MTU 改变，不能把 1200 字节当作所有连接的可发送保证。当前 encoder 固定限制 1200，reply pump 直接传播 send_datagram 错误：MTU/TooLarge 类失败应考虑按包丢弃并计数，而不是必然销毁整个 session。此项需 UDP 实验后定案。[Quinn Connection API](https://docs.rs/quinn/0.11.9/quinn/struct.Connection.html)

## 3. 较早 TODO 的过期或不合理建议

| 条目 | 当前代码证据 / 判断 | 处理 |
|---|---|---|
| 110：Drop CAS→fetch_sub | `lb/inflight.rs:199–209` 已经使用 fetch_sub | 标记已实现，删除 CAS 实施步骤 |
| 111：free-list Mutex | 当前 InflightTable 只有 capacity/registered/next_id，已无该 free-list | 原问题已过期；owner Mutex 是另一用途，不要按旧建议删除 |
| CR-AUDIT-5：EntryConnPool 三项乘法 | `conn_pool.rs:160,386` 已按 connection 数及 262144 硬上限校验 | 删除旧溢出复现描述；组合内存预算另列待验收 |
| CR-AUDIT-6：VarInt try_into().unwrap | `transport/quic.rs:50–74` 已返回字段化错误，并有边界测试 | 拆成已修复转换 + 尚缺组合资源预算 |
| 108：根据 stable_id 重算 shard | actor Push 使用调用方 shard_id（conn_pool.rs:188–195） | 不能假设 shard 来源是 stable_id；真要 O(1) remove，可用现有 by_id/显式索引记录实际 shard |
| 109/135：inflight 计数合并 | 两个原子 load 非一致快照；并发情况下误差不应承诺仅低 1 | 明确仅用于近似选路；不要用于硬 admission。合并需保留 pending/active 观测语义 |
| 20：H1 全改 split_to/freeze | read_request 的 body_prefix 已使用 split_to().freeze() | 指定仍有问题的调用点；禁止机械“全部改写”导致大 backing buffer 长期滞留 |
| 85：同步 sync_listeners 阻塞 | 当前 sync_all_listeners 是 async、有 async gate，并将任务投到 proxy runtime | 旧诊断过期；只保留具体 reload/rollback 故障测试任务 |
| 52：H2 connection 固定缓存 Snapshot | H2c 已按每请求 generation 做缓存，且 F1 暴露代际完整性问题 | 不要倒退为整个 H2 生命周期固定旧 Snapshot；按完整 generation 缓存 |
| ENTRY-POOL：删除 mutable Vec 换写锁 | 当前 actor 独占 Vec，读路径 ArcSwap；这是合理的写模型/快照分离 | 未有测量不改 actor 为共享写锁 |
| 27：ticket persistence→0-RTT | 客户端 connect 等待握手完成；代码没有 into_0rtt/early-data 接入 | 持久身份、session resumption、0-RTT 应拆成三个任务，并考虑 early data 重放；现在就修 README，不能等优化落地 |
| 145：必须新增 hotpath-rs | TODO 前言/T7 又明确已有 dial9 足够作第一轮定位 | 收敛为“获得 profile 证据”，不把某个新工具当必选依赖 |
| 22/34/86：通用 split 一律消除 | TCP 已特化，泛型 TLS fallback 仍合理 | 保持；into_split 不是两个独立 TCP 连接/新建两份 socket FD |
| 101：固定 10–20µs 唤醒损耗→自旋 | 没有项目测量证明；忙轮询会与共享 runtime 竞争 CPU | 降为实验，仅在尾延迟归因后考虑 |
| CR4：所有 metrics 改 trace+channel | 异步队列增加分配、背压与丢指标设计 | 不作为默认架构；先测实际 metrics 成本 |
| 36：消灭全部 dyn/Box | 没有实际 profile 证明虚调用主导 | 保留真实扩展边界，避免以代码体积/复杂度换未知收益 |
| CR-AUDIT-20：fuzz 无锁结构 | 字节解析和并发调度不是同类测试 | parser 用 fuzz；并发模型用可控调度/模型测试，系统用故障注入 |
| 24/25/42：multi-endpoint/io_uring/旁路 | 现有证据不支持引入，且部署与迁移复杂度大 | 继续延后，不作为工业化必选项 |

标为 Done 的历史项不应继续与未完成项混排。这里没有把未逐个复现的全部历史记录重新认证为“属实”；未证实的性能效果仍是候选，不能从 248 个测试通过推导出性能或长稳结论。

## 4. HA 方案应如何选择

当前 server 不保存 routing authority，但持有活跃 QUIC 连接和会话，因此是“配置可重建”，不是“运行态无状态”。仅在外部 LB 中多放一个 server，并不能让它访问另一台机器上的 client tunnel。

建议分步：

1. 小规模 HA：client 支持同时向两个 server 建 tunnel，同一 group 在两台都在线；分别限制连接/资源预算。外部入口只向 group ready 的 server 导流。普通多 IP DNS 串行尝试不是这个能力。
2. 明确存量流故障语义：失效 server 上的 TCP/QUIC 流中断，由应用重连；不要许诺跨进程透明迁移。
3. 需要稀疏放置或大规模多租户时，再建立 server→ctld 在线视图：稳定 server_id + 启动 incarnation + lease expiry + heartbeat/event seq，防止旧实例迟到 disconnect 删除新实例状态。
4. 路由配置 revision 与高频 presence 独立；不要每次连接事件都触发完整 routing Snapshot/hash/LKG 持久化。
5. 在线视图使用租约失效与本地 group readiness，两者共同决定导流；普通 node-level HTTP health check 无法表达“这台机器只缺某个 group”。明确实际入口是支持按 host 选健康 upstream 的代理，还是仅四层 LB。
6. registry 可以是观测面，不能成为每请求访问 ctld/SQLite 的同步依赖。ctld 单点、恢复时间和数据持久性单独定义。
7. TODO-172 不必依赖 TODO-151 的完整 tenant-scoped Snapshot 实现；稳定 identity 可拆小任务。重连新 server 后 client 获得的是 LoginResp.config，不是直接参与 ctld→server 的 Snapshot/ACK 协议。

这条路线在两三台 server 的规模通常比先做关联矩阵、调度器和分布式存储更容易验证。更大规模是否需要另外的服务发现组件，由实际部署约束决定。

## 5. 建议实施顺序与验收

| 顺序 | 工作 | 验收证据 |
|---|---|---|
| 1 | 纠正 README 的 watch 暴露与 TLS 指引；固定 tunnel 身份；CA load fail-closed | 严格 TLS 连接、错误 SAN/CA/key 拒绝、损坏文件不覆盖、非回环无 token 拒绝 |
| 2 | 完整 generation cache key；H2c authority 修复；两层 sender invalidation | 长连接跨 epoch/同 sequence、IPv6、并发 RST、迟到错误不删除新 sender |
| 3 | admin framing；结构化错误指标；明确 panic 故障模型 | 长度与分片边界测试、指标不依赖文案、release 子进程恢复测试 |
| 4 | 控制面故障验收 + 分层 admission | 磁盘满/损坏、A→B→C、revoke 与 reload 并发、取消释放、慢 body/relay、明确 ready 状态 |
| 5 | 性能基准与 HA 小闭环 | 相同 CPU contract、吞吐/错误/dropped/p99/RSS、双 server 失效与恢复时间 |
| 6 | 用 profile 选择小优化 | UDP 分配、host normalization、routing frame 每次单独前后比较 |

补充建议：

- 正常处理路径保持 Result，panic 只代表不能安全继续的 invariant 失败；测试必须使用部署实际的 build profile。
- 性能 admission 以资源生命周期为边界：HTTP permit 保持到 body 完成；raw relay 到双向结束；UDP session/queue 分开。不把小整数 counter 当完成全局预算的证据。
- 保留可用的模块分层：ctld 配置权威、immutable generation、actor 写侧、ArcSwap 读侧，以及 TCP/QUIC 的专用 relay。无需为了“工业化”再添加自定义 runtime 或拆一批 crate。
- CI 现已有 audit、coverage artifact；后续价值更大的是关键失败路径的验收门槛，而不是再装同类工具。性能 CI 与配置一致性测试分开。
- 优先确定 H1 chunked/Expect/WebSocket、H2/gRPC、UDP 最大包和源端口语义的支持矩阵。支持范围比“全协议高性能”一类表述更适合实际部署。
- 优化收益按 CPU/request、CPU/GiB、PPS、p99、RSS、丢弃率衡量；保留原始配置与构建 SHA，不用源码上少一次 clone 推导业务收益。

## 6. 文档治理本身已影响实施正确性

- 同一 `TODO-148` 分别指 listener ownership 和 coarse timer；`TODO-149` 分别指 batching 和 Count-Min Sketch。引用已经无法唯一定位。
- 顶部“未完成事项唯一来源”与大量 Done/Discarded 条目矛盾。
- 历史段落“零 unwrap/expect”“全部同类最优”“cancel 完备”等绝对表述没有限定 build profile、场景与验收证据；本轮在 PKI、TLS handler 等处可见生产 expect，且存在明确边界缺陷。
- `server-runtime.md` 描述不同 epoch fail-closed，而当前代码每次 watch 重连都打开 authority-reset 边界。需要选定并文档化实际信任策略。
- README 的 O(1) vhost 应限定 exact match；wildcard 扫描不能一并承诺 O(1)。现有 QUIC connection pool 也与“Connection pool not needed”的笼统措辞不一致；区分 tunnel transport pool 和预建业务 stream。

建议只保留一个当前 backlog，每项字段固定为：唯一 ID、问题/非目标、触发场景、证据代码 SHA、状态、推荐方案、拒绝的方案及原因、验收条件、完成提交。状态区分 `Bug confirmed / Candidate / Implemented / Accepted / Rejected`。历史评审只链接，不反复把旧结论追加到当前文件顶部。
