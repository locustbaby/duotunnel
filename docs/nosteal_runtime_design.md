# NoSteal Runtime 与 Cache 命中率优化设计讨论

> 本文记录了 2026-06-11 针对 duotunnel 运行时模型改造的设计讨论，涵盖 NoSteal Runtime 方案选型、共享状态处理、连接分发机制，以及在不改 runtime 前提下可做的 cache hit 优化。

---

## 一、背景：最复杂的 change 是什么

review 中涉及的 change 按复杂度排序：

| 优先级 | 变更 | 预估工作量 |
|---|---|---|
| P0 | ArrayQueue 替换 SegQueue | 1h |
| P0 | sniff 添加 5s 超时 | 30min |
| P1 | egress/http.rs body 路径 copy_from_slice 修复 | 2h |
| P1 | buffer pool capacity 宽松匹配 | 1h |
| P1 | DNS Single-Flight | 4h |
| P1 | Egress L4 连接池 | 1d |
| **P2** | **NoSteal Runtime 改造** | **2-3d** |
| P2 | TLS 证书热重载 | 4h |
| P2 | JoinSet 任务追踪 | 4h |

NoSteal Runtime 是工作量最大、架构影响最深的 change。

---

## 二、NoSteal Runtime 设计决策

### 2.1 改造动机

- **架构对标**：目标是工业级 tunnel 项目，对标 Pingora 的无共享运行时模型
- **性能场景**：多 client 并发接入时，work-steal 导致 per-connection 热数据（buffer、quinn 内部状态）频繁跨核迁移，产生 cache miss
- **两条链路均受影响**：ingress TCP accept 链路 + QUIC Endpoint accept 链路

### 2.2 三种方案对比

| 方案 | 描述 | 优点 | 缺点 |
|---|---|---|---|
| **A. N current_thread + 绑核** | N 个独立 runtime，每个绑定一个物理核 | 极致 cache 亲和性，L1/L2 完全热 | `core_affinity` crate，macOS 支持不完整；弹性差 |
| **B. N current_thread + OS 调度** | N 个独立 runtime，OS 自由调度线程 | 改动最小，消除 work-steal，OS CFS 实际会分散到各核 | 不保证物理核独占，OS 偶尔可能把两个线程放同核 |
| **C. multi_thread + LocalSet** | 保留全局 runtime，用 `spawn_local` 隔离 task | 不改运行时模型 | 要求 `!Send`，现有 task 全是 `Send`，改动最大；效果打折扣（线程仍会 steal） |

**选定方案：B**。理由：
- `build_single_thread_runtime` 函数已存在，可直接复用
- 现有所有 task 均是 `Send`，无需改造 Future 类型
- 与现有 `Arc<ServerState>` 完全兼容
- work-steal 彻底消除，绑核作为可选后续增强

### 2.3 共享状态处理

现有 `Arc<ServerState>`（含 `registry`、`auth_store`、`egress_map` 等）**保持不变**，clone N 份分别传入 N 个 runtime 线程。

- `DashMap`、`ArcSwap` 等并发结构本身 thread-safe，与 runtime 数量无关
- NoSteal 的真正收益是 **per-connection task 不跨线程迁移**，而不是全局状态无锁
- 热重载仍走原有 ArcSwap 路径，N 个线程各自持有的 `Arc<ServerState>` 会在下次 deref 时看到新快照

### 2.4 连接分发机制

**不需要应用层调度器**，依靠 OS 内核的 `SO_REUSEPORT` 做连接分发：

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

现有代码 `build_reuseport_listener` 和 `build_udp_socket` **已经设置了 `SO_REUSEPORT`**，基础设施已就绪。

### 2.5 实现骨架

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

每个 worker 内：
1. 各自调用 `build_reuseport_listener(addr)` 创建独立的 `TcpListener`
2. 各自调用 `build_udp_socket(addr)` + `quinn::Endpoint::new(...)` 创建独立的 QUIC Endpoint
3. 各自跑 accept loop，`tokio::spawn` 派发 connection task（仍要求 `Send`）
4. `Arc<ServerState>` clone 后传入，热重载自动生效

### 2.6 主要实现难点

**不是调度器，而是 quinn Endpoint 的 N 份初始化**：
- 现在 `run_quic_server` 有一次性逻辑（TLS config 构建、`ready` AtomicBool 设置）
- 分片后需确保 **所有 N 个 Endpoint 初始化完成后**再把 `ready` 置 true
- 建议用 `Arc<Barrier>` 让 N 个线程全部到达 barrier 后再继续

### 2.7 热点折衷

SO_REUSEPORT 按连接哈希分发，意味着：
- **多 client 并发**（主场景）：各 client 天然分散到不同 runtime，收益显著
- **单 client 极限吞吐**：该 client 固定在一个 runtime 线程，单核跑满，其他核闲置

> 如果主场景是单 client 极限吞吐，`multi_thread` work-steal 反而更优。
> NoSteal 针对的是多 client 并发场景。

---

## 三、不改 Runtime 就能提升 Cache Hit 的方法

### 3.1 核心问题：buffer 生命周期太短

现有 `copy_buffered` 每次调用都 `take_buffer` → `return_buffer`：

```rust
// tunnel-lib/src/engine/copy.rs
async fn copy_buffered<R, W>(reader, writer, buffer_size) {
    let mut buf = take_buffer(buffer_size);   // 借出
    loop {
        let read = reader.read(&mut buf).await?;
        writer.write_all(&buf[..read]).await?;
    }
    return_buffer(buf, buffer_size);          // 归还到 thread_local 池
}
```

`thread_local! LOCAL_POOL` 的设计意图是"同线程复用 buffer"，但 work-steal 把 task 从线程 A 偷到线程 B 后，线程 B 的 LOCAL_POOL 是冷的，**buffer 跟不上 task**，thread_local 收益丢失。

### 3.2 改进：buffer 跟着 task 走

把 buffer 生命周期从"每次 copy 调用"提升到"整个 relay task 生命周期"：

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

// relay_inner 中持有 buffer
async fn relay_inner<S>(quic_recv, quic_send, stream, initial_data) -> Result<(u64, u64)> {
    let (stream_read, mut stream_write) = tokio::io::split(stream);
    if !initial_data.is_empty() {
        stream_write.write_all(initial_data).await?;
    }
    // buffer 在 task 的 Future 状态机里，随 task 迁移，不归还池
    let mut buf_a = vec![0u8; DEFAULT_RELAY_BUF_SIZE];
    let mut buf_b = vec![0u8; DEFAULT_RELAY_BUF_SIZE];
    let quic_to_stream = copy_buffered_with_buf(quic_recv, stream_write, &mut buf_a);
    let stream_to_quic = copy_buffered_with_buf(stream_read, quic_send, &mut buf_b);
    // 注意：这里不能用 try_join! 因为两个 &mut 不能同时存在
    // 需要改为独立 task 或用 select! 轮询
    ...
}
```

> **注意**：双向 relay 中两个方向各需要一个 `&mut buf`，需要把双向 copy 改为两个独立 async block 或保留 `try_join!` + 各自持有 buffer（可行，因为 `try_join!` 内的两个 future 是独立状态机）。

### 3.3 保持 `try_join!` 不拆 spawn

`bridge.rs` 中双向 relay 用 `try_join!`，**两个方向的 copy future 在同一个 task 内**：

```rust
// 正确：同一 task，一次 steal 移动整体
let a_to_b = copy_buffered_then_shutdown(a_read, b_write, DEFAULT_RELAY_BUF_SIZE);
let b_to_a = copy_buffered_then_shutdown(b_read, a_write, DEFAULT_RELAY_BUF_SIZE);
tokio::try_join!(a_to_b, b_to_a)

// 错误：拆成两个 spawn，两个 task 可被 steal 到不同线程
tokio::spawn(a_to_b);
tokio::spawn(b_to_a);
```

现有实现已经正确，不要改动。

### 3.4 方法总结与收益上限

| 方法 | 有效性 | 说明 |
|---|---|---|
| Buffer 跟 task 走（不借还池） | ✅ 中等收益 | work-steal 后 buffer 跟着迁移，cache 里的数据仍可用 |
| 保持 `try_join!` 不拆 spawn | ✅ 小收益 | 减少 task 碎片，降低 steal 频率 |
| buffer pool capacity 宽松匹配 | ✅ 减少分配 | 已列入 Bug B，不直接影响 cache hit |
| 彻底消除 work-steal | ❌ 不可能 | 必须切换到 NoSteal 方案 |

**结论**：在现有 `multi_thread` 运行时下，**把 buffer 生命周期绑定到 task** 是最高效的 cache 改进，改动量小，不依赖 runtime 改造。NoSteal 是更彻底的解法，但优先级为 P2。

---

## 四、执行顺序建议

```
立即（P0）
  └── Bug B: buffer pool capacity >= 宽松匹配 + 去零化
  └── Bug A: egress/http.rs body 路径改 read_chunk

近期（P1）
  └── buffer 生命周期绑定 task（本文 §3.2）—— cache hit 改进，不改 runtime
  └── DNS Single-Flight
  └── Egress L4 连接池

计划（P2）
  └── NoSteal Runtime 改造（本文 §2）—— 彻底消除 work-steal
  └── JoinSet 任务追踪
  └── TLS 证书热重载
```

---

*记录时间：2026-06-11 | 基于 /grill-me 设计讨论整理*
