# duotunnel 优化与设计方案总汇

> 本文档为 `duotunnel` 架构优化方案的总汇总，整合了对标其他三方实现（wstunnel、pingora、rathole）、NoSteal Runtime 设计决策、Cache 优化细节以及其他各类稳定性与性能优化的全景设计。

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
- **问题**：buffer 在每次 `copy_buffered` 内借还，work-steal 把 task 迁到新线程后 thread_local 池是冷的，buffer 的 cache 利用率归零。
- **解决方案与深入设计**：
  把 buffer 生命周期从"每次 copy 调用"提升到"整个 relay task 生命周期"，让 buffer 跟着 task 走：
  ```rust
  // 新增带外部 buf 参数的版本
  async fn copy_buffered_with_buf<R, W>(
      mut reader: R,
      mut writer: W,
      buf: &mut Vec<u8>,
  ) -> std::io::Result<u64>
  where
      R: AsyncRead + Unpin,
      W: AsyncWrite + Unpin,
  {
      let mut copied = 0u64;
      loop {
          let read = reader.read(buf).await?;
          if read == 0 { break; }
          writer.write_all(&buf[..read]).await?;
          copied += read as u64;
      }
      Ok(copied)
  }
  ```
  在 `relay_inner` 中持有 buffer，借此使得 buffer 在 task 的 Future 状态机里，随 task 迁移，不归还池：
  ```rust
  async fn relay_inner<S>(quic_recv, quic_send, stream, initial_data) -> Result<(u64, u64)> {
      let (stream_read, mut stream_write) = tokio::io::split(stream);
      if !initial_data.is_empty() {
          stream_write.write_all(initial_data).await?;
      }
      let mut buf_a = vec![0u8; DEFAULT_RELAY_BUF_SIZE];
      let mut buf_b = vec![0u8; DEFAULT_RELAY_BUF_SIZE];
      let quic_to_stream = copy_buffered_with_buf(quic_recv, stream_write, &mut buf_a);
      let stream_to_quic = copy_buffered_with_buf(stream_read, quic_send, &mut buf_b);
      // 注意：这里两个方向的 copy future 依然运行在同一个 task 中（通过 try_join!），
      // work-steal 发生时，两个 future 会作为一个整体迁移，保证了 cache 数据的连续性。
      tokio::try_join!(quic_to_stream, stream_to_quic)
  }
  ```
  **不改 Runtime 提升 Cache Hit 的方法对比：**
  | 方法 | 有效性 | 说明 |
  |---|---|---|
  | Buffer 跟 task 走（不借还池） | ✅ 中等收益 | work-steal 后 buffer 跟着迁移，cache 里的数据仍可用 |
  | 保持 `try_join!` 不拆 spawn | ✅ 小收益 | 减少 task 碎片，降低 steal 频率 |
  | buffer pool capacity 宽松匹配 | ✅ 减少分配 | 已列入 P0-B，不直接影响 cache hit |
  | 彻底消除 work-steal | ❌ 不可能 | 必须切换到 NoSteal 方案 |
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
- **问题**：tokio `multi_thread` work-steal 在多 client 并发场景下频繁迁移 task，per-connection 热数据 cache miss。
- **设计决策与深入方案**：
  - **改造动机**：目标是工业级 tunnel 项目，对标 Pingora 的无共享运行时模型。多 client 并发接入时，work-steal 导致 per-connection 热数据（buffer、quinn 内部状态）频繁跨核迁移，产生 cache miss。
  - **三种方案对比**：
    | 方案 | 描述 | 优点 | 缺点 |
    |---|---|---|---|
    | **A. N current_thread + 绑核** | N 个独立 runtime，每个绑定一个物理核 | 极致 cache 亲和性，L1/L2 完全热 | `core_affinity` crate，macOS 支持不完整；弹性差 |
    | **B. N current_thread + OS 调度** | N 个独立 runtime，OS 自由调度线程 | 改动最小，消除 work-steal，OS CFS 实际会分散到各核 | 不保证物理核独占，OS 偶尔可能把两个线程放同核 |
    | **C. multi_thread + LocalSet** | 保留全局 runtime，用 `spawn_local` 隔离 task | 不改运行时模型 | 要求 `!Send`，现有 task 全是 `Send`，改动最大；效果打折扣（线程仍会 steal） |
    
    **最终选定方案：B**。理由是 `build_single_thread_runtime` 已存在可复用，且对现有代码侵入性最小，不要求改造 Future 的 `Send` 约束。
  - **共享状态处理**：
    现有 `Arc<ServerState>` 保持不变，clone N 份分发到各线程。`DashMap`、`ArcSwap` 等并发结构本身是线程安全的，热重载时各线程解引用会看到新快照，不会产生锁冲突或性能瓶颈。
  - **连接分发机制**：
    依靠 OS 内核的 `SO_REUSEPORT` 做连接分发，无需应用层调度器。
    ```
    TCP 端口 :8080
       ├── runtime-0 的 TcpListener fd   ← client A 的 TCP 连接
       ├── runtime-1 的 TcpListener fd   ← client B 的 TCP 连接
       └── runtime-2 的 TcpListener fd   ← client C 的 TCP 连接
             ↑ 内核按 4-tuple 哈希均匀分发，同一 client 的连接亲和到同一 fd

    QUIC 端口 :9000
       ├── runtime-0 的 UDP socket + quinn::Endpoint
       ├── runtime-1 的 UDP socket + quinn::Endpoint
       └── runtime-2 的 UDP socket + quinn::Endpoint
             ↑ 同样由 SO_REUSEPORT 分发 UDP 包，同一 client IP 路由到同一 Endpoint
    ```
  - **实现骨架**：
    ```rust
    // infra/runtime.rs 新增
    pub fn run_nosteal_workers(n: usize, f: impl Fn() + Clone + Send + 'static) {
        let handles: Vec<_> = (0..n).map(|i| {
            let f = f.clone();
            std::thread::Builder::new()
                .name(format!("proxy-worker-{i}"))
                .spawn(move || {
                    let rt = build_single_thread_runtime(&format!("worker-{i}"));
                    rt.block_on(async { f() });
                })
                .expect("spawn worker thread")
        }).collect();
        for h in handles {
            let _ = h.join();
        }
    }
    ```
  - **主要实现难点**：
    `quinn::Endpoint` 的 N 份初始化：目前 `run_quic_server` 包含一次性初始化逻辑，改为分片后需使用 `Arc<Barrier>` 同步，以确保所有 N 个 Endpoint 全部就绪后再将 `ready` 状态置为 `true`。
  - **热点折衷**：
    SO_REUSEPORT 按连接哈希路由，适合多 client 并发场景。若遇到单 client 极限吞吐场景，该 client 会被固定在单个线程中无法利用多核，这属于预期的折衷设计。
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
- 在 P2-A 基础上，通过环境变量或配置文件切换 `Steal / NoSteal` 两种模式，默认保持现有行为。

---

## 四、已确认不做 / 暂缓

| 方案 | 结论 | 原因 |
|---|---|---|
| wstunnel `read_buf` 零拷贝写帧 | ⚠️ 暂不移植 | `quinn::SendStream` 不暴露内部 `BytesMut`，需 fork quinn，风险高收益存疑 |
| rathole 增量 service diff | 暂不需要 | 场景不同，ArcSwap 全量替换已满足需求且更安全 |
| DNS L7 注入（Hyper HttpConnector） | 观察 | 先做 P1-C 的 DashMap 改造，再评估 L7 注入价值 |
| CPU 绑核（core_affinity） | 可选后续 | 依赖 P2-A 完成后再叠加，macOS 支持有限 |

---

## 五、执行优先级与顺序建议

```
立即（本周，P0）        近期（1-2周，P1）          计划（1个月，P2）
───────────────        ────────────────────      ──────────────────────
P0-A ArrayQueue        P1-A body copy_from_slice P2-A NoSteal Runtime
P0-B pool capacity     P1-B buffer → task 绑定   P2-B TLS 热重载
P0-C sniff timeout     P1-C DNS Single-Flight    P2-C JoinSet 追踪
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
