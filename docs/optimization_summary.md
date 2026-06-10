# duotunnel 优化方案总汇

> 汇总自 `duotunnel_review.md`、`best_practices_review.md`、`deep_arch_comparison.md`、`nosteal_runtime_design.md` 四份分析文档。

---

## 一、P0 — 立即修复（正确性 / 安全性）

### P0-A：ArrayQueue 替换 SegQueue
- **文件**：[copy.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/copy.rs)
- **问题**：`return_buffer` 里 `SegQueue::len()` 是 O(N) 遍历，每个 buffer 归还时都调用，高并发下 CPU 暴涨
- **方案**：换 `crossbeam_queue::ArrayQueue`（有界，O(1)，满了直接 drop，不需要 `len()` 判断）
- **预计工时**：1h

### P0-B：buffer pool capacity 宽松匹配 + 去零化
- **文件**：[copy.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/copy.rs)
- **问题一**：`buf.capacity() == buffer_size` 精确匹配，不同 buffer_size 调用方导致池命中率极低
- **问题二**：`resize(buffer_size, 0)` 无意义零化，后续立即被 `reader.read()` 覆盖
- **方案**：改为 `capacity >= buffer_size` + `unsafe { buf.set_len(buffer_size) }`
- **预计工时**：1h

### P0-C：sniff 增加 5 秒超时（Slowloris 防御）
- **文件**：[sniff.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/protocol/sniff.rs)
- **问题**：`SniffRuntime::sniff` 无绝对时间超时，攻击者 1 字节/秒发包可永久挂住 accept worker
- **方案**：在 `sniff` 调用外包裹 `tokio::time::timeout(Duration::from_secs(5), ...)`，或在 `SniffPolicy` 中增加 `timeout: Option<Duration>` 字段
- **预计工时**：30min

---

## 二、P1 — 高收益性能优化

### P1-A：egress/http.rs body 路径去除多余 `copy_from_slice`
- **文件**：[egress/http.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/egress/http.rs) L200–203
- **问题**：HTTP body 流式转发用 `recv.read(&mut buf)` + `Bytes::copy_from_slice()`，每个 chunk 都做一次堆分配；而 header 阶段已正确用 `recv.read_chunk()` 返回零拷贝 `Bytes`
- **方案**：body unfold 循环改用 `recv.read_chunk(max, true)` 直接产出 `chunk.bytes`
- **预计工时**：2h

### P1-B：buffer 生命周期绑定到 task（cache hit 改进）
- **文件**：[copy.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/copy.rs)、[relay.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/engine/relay.rs)
- **问题**：buffer 在每次 `copy_buffered` 内借还，work-steal 把 task 迁到新线程后 thread_local 池是冷的，buffer 的 cache 利用率归零
- **方案**：新增 `copy_buffered_with_buf(reader, writer, buf: &mut Vec<u8>)` 版本，由 `relay_inner` 持有 buffer，跟随 task 生命周期，不归还池
- **预计工时**：3h

### P1-C：DNS Single-Flight + DashMap 替换全局锁
- **文件**：[dns_cache.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/infra/dns_cache.rs)
- **问题**：全局 `tokio::sync::Mutex<()>` 导致所有域名的 DNS 解析串行；每次写入全量 clone HashMap
- **方案**：换 `DashMap` 分桶并发 + `broadcast::Sender` 实现 Single-Flight（相同域名并发只解析一次）
- **预计工时**：4h

### P1-D：Egress L4 连接池（上游 TCP 连接复用）
- **文件**：新增 `tunnel-lib/src/proxy/upstream_pool.rs`
- **问题**：每次请求都重新建立 TCP 连接到上游，TLS 握手延迟 50ms+
- **方案**：参考 Pingora 三层结构：`DashMap<key, Arc<UpstreamNode>>` + `ArrayQueue<TcpStream>` 热队列（8 slot）+ `idle_poll` EOF 检测
- **预计工时**：1d

### P1-E：registry.rs 死锁风险修复
- **文件**：[registry.rs](file:///Users/sexy/Documents/GitHub/duotunnel/server/ingress/registry.rs)
- **问题**：`replace_or_register` 在持有 `groups` DashMap 分桶写锁期间再取 `clients` 写锁，嵌套顺序可能引发死锁
- **方案**：先在短生命周期 scope 内操作 `groups` 并释放锁，再操作 `clients`，遵循单锁原则
- **预计工时**：2h

---

## 三、P2 — 架构增强

### P2-A：NoSteal Runtime（消除 work-steal cache miss）
- **文件**：[infra/runtime.rs](file:///Users/sexy/Documents/GitHub/duotunnel/tunnel-lib/src/infra/runtime.rs)，server/client 入口
- **问题**：tokio `multi_thread` work-steal 在多 client 并发场景下频繁迁移 task，per-connection 热数据 cache miss
- **方案**：N 个 `current_thread` runtime + N 个独立 OS 线程，连接分发靠已有的 `SO_REUSEPORT`（`build_reuseport_listener` 和 `build_udp_socket` 均已配置）
- **适用场景**：多 client 并发（不适用单 client 极限吞吐，后者 multi_thread 更优）
- **核心难点**：quinn Endpoint 从 1 份改 N 份初始化，`ready` 标志需用 `Arc<Barrier>` 同步
- **详细设计**：见 [nosteal_runtime_design.md](file:///Users/sexy/Documents/GitHub/duotunnel/docs/nosteal_runtime_design.md)
- **预计工时**：2-3d

### P2-B：TLS 证书热重载
- **文件**：新增 `tunnel-lib/src/infra/tls_reloader.rs`
- **问题**：证书轮换需要重启进程
- **方案**：参考 wstunnel 的 `TlsReloader`，用 `notify::recommended_watcher` 监听文件变更 + `AtomicBool` 标志位，每次建新连接前检查是否需要重建 TLS acceptor
- **预计工时**：4h

### P2-C：JoinSet 任务生命周期追踪
- **文件**：spawn 调用点分散于各模块
- **问题**：`tokio::spawn` 各任务无生命周期管理，组件关闭时子任务可能泄漏
- **方案**：参考 wstunnel 的 `JoinSetTokioExecutor`，引入 `tokio_util::task::TaskTracker` 或 `JoinSet`，确保父任务退出时级联 cancel 子任务
- **预计工时**：4h

### P2-D：可选 NoSteal 运行时配置项
- 在 P2-A 基础上，通过环境变量或配置文件切换 `Steal / NoSteal` 两种模式，默认保持现有行为

---

## 四、已确认不做 / 暂缓

| 方案 | 结论 | 原因 |
|---|---|---|
| wstunnel `read_buf` 零拷贝写帧 | ⚠️ 暂不移植 | `quinn::SendStream` 不暴露内部 `BytesMut`，需 fork quinn，风险高收益存疑 |
| rathole 增量 service diff | 暂不需要 | 场景不同，ArcSwap 全量替换已满足需求且更安全 |
| DNS L7 注入（Hyper HttpConnector） | 观察 | 先做 P1-C 的 DashMap 改造，再评估 L7 注入价值 |
| CPU 绑核（core_affinity） | 可选后续 | 依赖 P2-A 完成后再叠加，macOS 支持有限 |

---

## 五、执行优先级矩阵

```
立即（本周）          近期（1-2周）          计划（1个月）
────────────────     ────────────────────    ──────────────────────
P0-A ArrayQueue      P1-A body copy_from_    P2-A NoSteal Runtime
P0-B pool capacity   P1-B buffer → task      P2-B TLS 热重载
P0-C sniff timeout   P1-C DNS Single-Flight  P2-C JoinSet 追踪
                     P1-D Egress L4 连接池
                     P1-E registry 死锁修复
```

---

## 六、duotunnel 已有的超越三方之处（保持不动）

| 特性 | 说明 |
|---|---|
| P2C 最小 inflight 负载均衡 | 优于 wstunnel/pingora/rathole 的轮询/FIFO |
| Jitter 指数退避重连 | 防惊群效应，优于 rathole 的简单 backoff |
| ArcSwap 零停机热重载 | 比 rathole 增量 diff 更安全，实现更简洁 |
| CachePadded InflightSlot | 避免 false sharing，三方均无此优化 |
| ConstantTime Token 比较 | `subtle::ct_eq` 防时序旁路攻击 |
| EMFILE 退避保护 | 文件描述符耗尽自动休眠，防 accept 死循环 |
| JitterBackoff 分类重试 | Fatal/Transient 分类，防无意义重连风暴 |

---

*汇总时间：2026-06-11 | 涵盖所有 review 文档结论*
