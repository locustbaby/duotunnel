# Runtime Tuning: Endpoint-per-Thread + mpsc Dispatch

**Status**: Design (2026-04-08)
**Owner**: server side
**Targets**: TODO-56, TODO-41, the `non_tracking::Mutex::lock → futex_wait` dominating dial9 trace under 8K QPS bench

---

## 1. Why

The current server runtime is `multi_thread` + N tokio task quinn endpoints (SO_REUSEPORT). dial9 trace at 8K QPS shows:

- 30 + 14 = **44 sched events (39% of all blocking)** stuck in `non_tracking::Mutex::lock → futex_wait`, on `RecvStream::poll_read_buf` and `SendStream::poll_write`
- Cause: tokio's work-stealing scheduler can poll the same `quinn::Connection`'s streams from multiple worker threads concurrently. quinn's per-`Connection` state Mutex serializes them; the loser of every CAS goes into `futex_wait`.
- Adding more QUIC connections (TODO-16, already at 4) only **statistically dilutes** the contention; it does not eliminate the futex path. quinn upstream issue [#1433](https://github.com/quinn-rs/quinn/issues/1433) confirms `current_thread` runtime cuts CPU ~71% on the same workload.

The client side has already moved to **endpoint-per-OS-thread + `current_thread` runtime** and shows no contention on its side. The server hasn't, because the server has an extra constraint: external HTTP/TCP requests come in via the **ingress listener port (e.g. 8080)**, whose target tunnel client may live on any of N endpoint threads. Pure SO_REUSEPORT does not solve this — the kernel hashes by 4-tuple, which is independent of the target hostname.

## 2. Architecture

```
                            ┌────── shared state (Arc) ───────────┐
                            │  RoutingTable (ArcSwap)              │
                            │  ClientRegistry (DashMap, sharded)   │
                            │  Placement (DashMap<GroupId, idx>)   │
                            │  Cancel token (CancellationToken)    │
                            └─────────────────────────────────────┘
                                          ▲
                                          │ Arc<ServerState>
        ┌─────────────────────────────────┼─────────────────────────────────┐
        │                                 │                                 │
   thread 0                          thread 1                          thread N-1
   ┌──────────────────────────┐     ┌──────────────────────────┐     ┌──────────────────────────┐
   │ current_thread runtime   │     │ current_thread runtime   │     │ current_thread runtime   │
   │                          │     │                          │     │                          │
   │  quinn::Endpoint #0      │     │  quinn::Endpoint #1      │     │  quinn::Endpoint #N-1    │
   │   (SO_REUSEPORT :10086)  │     │   (SO_REUSEPORT :10086)  │     │   (SO_REUSEPORT :10086)  │
   │                          │     │                          │     │                          │
   │  TCP listener            │     │  TCP listener            │     │  TCP listener            │
   │   (SO_REUSEPORT :8080)   │     │   (SO_REUSEPORT :8080)   │     │   (SO_REUSEPORT :8080)   │
   │                          │     │                          │     │                          │
   │  mpsc Receiver<TcpStream>│     │  mpsc Receiver<TcpStream>│     │  mpsc Receiver<TcpStream>│
   │                          │     │                          │     │                          │
   │  spawn_local relay tasks │     │  spawn_local relay tasks │     │  spawn_local relay tasks │
   └────────┬─────────────────┘     └────────┬─────────────────┘     └────────┬─────────────────┘
            │                                │                                │
            └────────────── shared Vec<UnboundedSender<TcpStream>> ───────────┘
                            (each thread holds the full vector)
```

Each OS thread is a self-contained "endpoint actor": owns a quinn endpoint (private state, no Mutex contention), accepts external TCP locally (kernel SO_REUSEPORT distributes connections), and either handles the request locally (self-hit) or forwards the `TcpStream` to the owner thread via a single mpsc send.

### 2.1 Tunnel client lifecycle

1. Tunnel client opens UDP to `:10086`. Kernel hashes 4-tuple → picks one of N quinn endpoints.
2. Endpoint thread T accepts the QUIC connection, completes login, learns `group_id`.
3. Thread T inserts `placement[group_id] = T` into the shared `Placement` map.
4. Thread T also calls the existing `state.registry.register(client_id, group_id, conn)`.
5. On disconnect, thread T removes both entries.

`placement` is a `DashMap<GroupId, u32>`. Reads from the dispatcher are sharded atomic loads — no global lock.

### 2.2 External request lifecycle (the hot path)

1. External user opens TCP to `:8080`. Kernel hashes 4-tuple → picks one of N TCP listeners. Call this thread D ("dispatcher").
2. Thread D accepts TCP, peeks 4 KiB to extract Host header (existing `extract_host_from_http` / SNI logic).
3. Thread D queries `routing.load().http_routers[port][host]` → `(group_id, routing_info)`.
4. Thread D queries `placement.get(&group_id)` → `owner_idx`.
5. **Self-hit (`owner_idx == D`)**: spawn the relay locally on thread D's runtime via `tokio::task::spawn_local`. Call `state.registry.select_client_for_group(&group_id)` to obtain the `quinn::Connection` (which lives on D), then `conn.open_bi()` — same thread, no cross-runtime, no Mutex contention.
6. **Miss (`owner_idx != D`)**: `senders[owner_idx].send(WorkItem { stream, host, routing_info, group_id, peer_addr, port }).ok()`. Thread D moves on to next accept.
7. Thread `owner_idx` has a long-running consumer task `while let Some(item) = rx.recv().await` that, on each item, spawns a local relay task. From this point all I/O on this stream is local to `owner_idx`.

`WorkItem` carries the already-peeked Host and resolved `routing_info` so the owner thread does not re-parse — peek + parse cost stays on the dispatcher thread.

### 2.3 Service lifecycle (process-level)

```
main()
 └── run_with_dial9 / run_with_tokio
       └── outer current_thread runtime, hosts:
             • signal handler task → cancel.cancel()
             • orchestrator: oversees N worker OS threads via mpsc<WorkerEvent>
       └── orchestrator runs to completion → drop(outer runtime) → graceful_shutdown(30s)
```

Orchestrator phases:

1. **Spawn**: for each `i in 0..N`, build a `current_thread` runtime, spawn an OS thread that runs `rt.block_on(EndpointWorker::run(i, state, cancel, event_tx, all_senders))`. Collect the std `JoinHandle`s into `worker_joins: Vec<JoinHandle<Result<()>>>`.
2. **Wait Built**: orchestrator awaits N `WorkerEvent::Built` messages over `event_rx`. Once all reported, set `ready.store(true)` so healthz returns 200.
3. **Run**: orchestrator parks on `select! { _ = cancel.cancelled() => …, ev = event_rx.recv() => … }`. `WorkerEvent::Stopped(Err)` is logged but does NOT broadcast cancel — single failure does not kill the fleet.
4. **Drain**: when cancel fires, orchestrator broadcasts via dropping the cancel sub-tokens (or each worker observes the parent token). Each worker enters its drain phase: closes TCP listener (no new accept), drains in-flight stream tasks up to `drain_timeout`, then exits.
5. **Join**: after all workers signal `Stopped`, orchestrator runs `tokio::task::spawn_blocking(move || { for h in worker_joins { let _ = h.join(); } })` to reap the OS threads without blocking the async context.
6. **Shutdown**: outer runtime drops, dial9 `graceful_shutdown(30s)` runs, traces flushed.

This mirrors pingora's `NoStealRuntime::shutdown_timeout` (oneshot send + std join), avoiding the "blocking-in-async" anti-pattern that the current `b520862` workaround papers over with `multi_thread` outer runtime.

## 3. Cross-runtime cost analysis

| Operation | Where it runs | Crosses runtime/thread? |
|---|---|---|
| TCP `listener.accept` | dispatcher thread D | no |
| TCP `peek` (Host extraction) | D | no |
| `routing.load()` | D (atomic load) | no |
| `placement.get()` | D (DashMap shard atomic) | no |
| **self-hit**: `spawn_local(relay)` | D | no |
| **self-hit**: `conn.open_bi()` | D (local quinn::Endpoint) | no |
| **self-hit**: forward join! both halves | D | no |
| **miss**: `tx.send(item)` | D → owner_idx | **one mpsc wakeup ~50–100 ns** |
| **miss**: `rx.recv()` returns | owner_idx | no (waker is local) |
| **miss**: `spawn_local(relay)` on owner | owner_idx | no |
| **miss**: `conn.open_bi()` | owner_idx (local Endpoint) | no |
| **miss**: stream poll_read / poll_write (entire stream lifetime) | owner_idx | no |
| **miss**: TCP poll on the moved TcpStream | owner_idx (registered to owner's reactor on first poll) | no |
| quinn::Connection state Mutex | only ever locked by one thread | **never contended → futex_wait = 0** |

**Result**: zero continuous cross-runtime cost. At most one mpsc hop per accepted external stream, only on the (N-1)/N statistical fraction where the kernel hash didn't put the request on the right thread. Self-hit case (1/N statistically) pays absolutely nothing.

This is the **theoretical lower bound** under the (tokio + quinn) constraint set, given that:
- quinn::Connection's driver is bound to a specific runtime — we cannot migrate connections.
- the kernel's SO_REUSEPORT hash uses 4-tuple — it cannot match by Host header (eBPF SO_REUSEPORT BPF can rewrite the hash, but it sees only L3/L4, not the post-TLS Host).
- spinning to avoid the wakeup would burn one full core per worker — net loss.

## 4. Files touched

| File | Change |
|---|---|
| `server/main.rs` | Replace `tokio::spawn` × N endpoint loop with N `std::thread::spawn` × `current_thread` runtime. Add orchestrator (mpsc<WorkerEvent>). Replace blocking handle-collection with `spawn_blocking(join)`. Outer runtime can revert to `current_thread` after this. |
| `server/state.rs` | Add `pub placement: DashMap<Arc<str>, u32>` (group_id → owner thread idx). Add `pub ingress_senders: OnceLock<Arc<[mpsc::UnboundedSender<WorkItem>]>>`. |
| `server/handlers/quic.rs` | After successful tunnel client login, call `state.placement.insert(group_id, my_thread_idx)`. On disconnect, remove. |
| `server/handlers/http.rs` | Change `run_http_listener` to be per-thread: bind via `build_reuseport_tcp_listener`, accept locally, peek Host, lookup placement, dispatch via senders[owner] **or** spawn_local on self-hit. |
| `server/handlers/tcp.rs` | Same per-thread + dispatch pattern as http.rs (the simpler L4 path). |
| `server/listener_mgr.rs` | `sync_listeners` becomes a per-thread operation. Each thread holds its own listener set; control_client / hot_reload update them via a small command channel. (Or: share `ListenerManager` and have each thread filter by self.) |
| `tunnel-lib/src/...` | `build_reuseport_tcp_listener` already exists from the earlier client refactor — reuse. |
| `runtime-tune.md` | This doc. |

Out of scope for this refactor:
- Changing `proxy::forward_*` (the relay implementations): they don't care which runtime they run on as long as they're polled by one thread.
- Changing `ServerState.registry` shape: `select_client_for_group` already returns a clone-able `quinn::Connection`. The conn's driver lives on the owner thread, but cloning + handing the clone around is fine — `open_bi()` from any thread is what we want to avoid, and dispatch ensures `open_bi` is always called from the owner.
- Re-thinking config push / Ping / control stream: they follow the same affinity (control stream lives on the conn's owner thread), no extra work needed.

## 5. Risk register

| Risk | Mitigation |
|---|---|
| `placement` write lag (client login → orchestrator can dispatch) | Dispatcher fallback: if `placement.get()` returns `None`, fall back to `state.registry.select_client_for_group()` and `open_bi` cross-runtime once (current behavior), log a counter. Should be near-zero in steady state. |
| `placement` stale on client reconnect (old idx, new conn on different thread) | On disconnect remove first, on register insert second; insert is the writer's last action so dispatcher sees consistent (`Some(new)` or `None`). |
| Ingress TCP listener bind racing with `sync_listeners` hot-reload | `ListenerManager` already debounces; new code path keeps the same debounce, just per-thread. |
| Single client × single conn (8K bench) — only one server thread does work | Acceptable: bench reflects realistic single-client throughput. CI should bump `client.threads` and `server.threads` to the same N to reflect production. Track in TODO. |
| `tokio::task::spawn_local` in `current_thread` runtime requires `LocalSet`? | No — `current_thread` runtime supports `spawn_local` directly via its built-in LocalSet. Tokio docs: "`current_thread` runtime allows spawning local tasks without an explicit LocalSet". Verify with a smoke test. |
| `WorkItem` carries `TcpStream` across threads — registered to dispatcher's reactor? | `TcpStream` is `Send`. Until first `poll_read`, it's not registered to any reactor. Move via mpsc, first poll on owner registers to owner's reactor. One-time cost only. |
| dial9 trace on the dispatcher thread vs owner thread — which has the events? | Both, since both are wrapped by `TracedRuntime` (one per OS thread, see Section 6). |

## 6. dial9 trace coverage (in scope of this PR)

`client/main.rs` currently wraps only the outer `multi_thread` runtime with `TracedRuntime`, while all real work happens on per-thread `current_thread` runtimes that dial9 cannot see → client trace is empty. The server side has the same problem after this refactor.

**Pass 1 (this PR)**: dial9 mode is a single-thread profiling mode.
- When `DIAL9_TRACE_PATH` is set, force `effective_threads() = 1` on both client and server.
- The single OS thread IS the outer `TracedRuntime`'s only worker (built via `Builder::new_current_thread()` wrapped by `TracedRuntime`). All quinn endpoint, ingress listener, dispatcher, and relay tasks `spawn_local` on it.
- Trade-off: dial9 mode does not exercise the multi-thread dispatch path. We accept this — bench under dial9 is for profiling the *per-thread hot path*, not the cross-thread dispatch latency. Production runs without dial9 use the full N-thread architecture.

**Pass 2 (follow-up PR)**: wrap each per-thread runtime with its own `TracedRuntime`, write `client-trace.{i}.bin` / `server-trace.{i}.bin`, post-process to merge. Requires confirming `dial9-tokio-telemetry` supports multiple `TracedRuntime` instances in one process — needs library audit.

## 7. Validation plan

1. Cargo build green.
2. Existing unit + integration tests green.
3. Push branch to `locustbaby/duotunnel`, run CI 8K QPS bench job.
4. Pull dial9 server trace from CI artifacts, open in viewer.
5. Confirm: `non_tracking::Mutex::lock → futex_wait` schedule events drop from ~39% of total to ~0%.
6. Confirm: p99 latency at 8K QPS drops vs main baseline.
7. Confirm: SIGTERM → graceful flush of dial9 trace (`.active` → `.0.bin`) within 30s, no `.active` leftover.

## 8. Rollback plan

Single revert commit on `server/main.rs` brings back `tokio::spawn` × N. `placement` field on `ServerState` is benign (unused) and can stay. mpsc senders go unused.

## 9. Post-merge follow-ups

- TODO: pass 2 for dial9 trace coverage (multiple `TracedRuntime`).
- TODO: deterministic placement (consistent hash by `group_id`) for cache locality + predictability.
- TODO: ingress affinity hints — when LB sticky, kernel hash already gives a poor man's affinity; if LB rotates, consider eBPF SO_REUSEPORT BPF to pin by hostname after a peek (experimental).
- TODO: bench `client.threads = server.threads = 4` and `= 8` to characterize scaling.
