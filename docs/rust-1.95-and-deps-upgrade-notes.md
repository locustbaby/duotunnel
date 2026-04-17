# Rust 1.95 And Dependency Upgrade Notes

Last checked: 2026-04-17

## Scope

This note records:

- which direct dependencies in this workspace have newer published versions;
- which upgrades look low-risk vs. migration-heavy;
- which Rust 1.95 features are actually worth using in this codebase.

The observations below are based on:

- local manifests in this repo;
- `cargo update --workspace --dry-run --verbose`;
- targeted dry-runs such as:
  - `cargo update -p tokio --precise 1.52.0 --dry-run`
  - `cargo update -p hyper --precise 1.9.0 --dry-run`

## Rust 1.95: Features Worth Using Here

### 1. `Vec::push_mut`

Useful when building nested routing/config structures and needing to mutate the element immediately after insertion.

Best candidates in this repo:

- `tunnel-store/src/sqlite_rules.rs`

Why it helps:

- reduces temporary locals like `let mut v = Vec::new(); v.push(...);`
- avoids `push(...)` followed by `last_mut().unwrap()` style code
- makes the “append then keep filling the new element” flow more direct

### 2. `if let` guards in `match`

Useful where an arm both matches a variant and conditionally extracts/handles another fallible result.

Best candidates in this repo:

- `tunnel-service/src/service.rs`
- `server/hot_reload.rs`
- selected connection/reload state machines under `server/` and `client/`

Why it helps:

- collapses nested `if let Err(...)` or `if let Ok(...)` inside `match` arms
- improves readability in retry/backoff and reload logic

### 3. `cfg_select!`

Rust 1.95 adds `cfg_select!`, which can replace some `cfg-if`-style branching.

Current recommendation:

- do not force it in right now;
- use it only if platform branching grows further in networking/socket setup code.

Likely candidates if needed later:

- `tunnel-lib/src/transport/tcp_params.rs`
- `tunnel-lib/src/transport/listener.rs`

## Direct Dependency Upgrade Matrix

### Batch A: Recommended First

These are the upgrades with the best expected return for this repo and the lowest migration risk.

| Crate | Current | Observed latest | Recommendation | Why it matters |
| --- | --- | --- | --- | --- |
| `tokio` | `1.49` | `1.52.1` | Upgrade first | Confirmed by dry-run. Tokio 1.51/1.52 adds runtime fixes and improves `spawn_blocking` scalability with a sharded queue. Good fit for this repo’s async-heavy server/client paths. |
| `hyper` | `1.8` | `1.9.0` | Upgrade first | Confirmed by dry-run. The 1.5-1.8 line includes HTTP/1 partial-message parsing/perf fixes and extra HTTP/2 builder knobs. Low-risk for current usage. |
| `hyper-rustls` | `0.27` | `0.27.9` | Upgrade with `hyper`/`rustls` | Keep the HTTP/TLS stack aligned. Mostly ecosystem sync and bugfix value. |
| `rustls` | `0.23` | `0.23.38` | Upgrade with `hyper-rustls` | Small-step crypto/TLS refresh. Worth taking together with the HTTP client/server stack. |
| `tracing-subscriber` | `0.3.22` | `0.3.23` | Upgrade opportunistically | Small and low-risk observability update. |
| `arc-swap` | `1.7` / `1` | `1.9.1` | Upgrade opportunistically | Small 1.x refresh for config/hot-swap paths. |
| `clap` | `4.5` | `4.6.1` | Upgrade opportunistically | CLI polish, low migration risk. |
| `rand` | `0.9` | `0.9.4` | Upgrade opportunistically | Minor bugfix refresh, low-risk. |

Notes:

- `tokio` precise dry-run also pulled newer `libc`, `mio`, `socket2`, and `tokio-macros`.
- `hyper` precise dry-run succeeded cleanly as well.

### Batch B: Valuable, But Needs Focused Migration Work

These upgrades are worth considering, but should be handled in isolated PRs with tests.

| Crate | Current | Observed latest | Recommendation | Why it matters / risk |
| --- | --- | --- | --- | --- |
| `metrics-exporter-prometheus` | `0.16` | `0.18.1` | Upgrade in a dedicated observability PR | Newer exporter stack; current docs expose HTTP listener, push-gateway, protobuf-related features, and native histogram examples. Integration should be re-verified against current metrics wiring. |
| `notify` | `6.1` | `8.2.0` | Upgrade in a dedicated hot-reload PR | This is a cross-major move. Worth it because the repo uses it for config reload, but event semantics/backends should be re-tested on macOS. |
| `tonic` | `0.12` | `0.14.5` | Upgrade together with `prost` and `tonic-health` | Tonic 0.14 brings load-shed support, TLS handshake timeout, TCP keepalive interval/count knobs, and a prost split. Good upside, but codegen/build APIs changed. |
| `tonic-health` | `0.12` | `0.14.5` | Upgrade with `tonic` | Keep gRPC stack consistent. |
| `prost` | `0.13` | `0.14.3` | Upgrade with `tonic` | Required for the current tonic line. Build/codegen changes need review. |
| `tokio-tungstenite` | `0.24` | `0.29.0` | Upgrade in a CI helpers PR | Newer tungstenite line includes multiple dependency/security/perf updates, but buffering and close behavior changed across the intervening releases. |
| `webpki-roots` | `0.26` | `1.0.7` | Upgrade after TLS regression check | Newer CA bundle is attractive, but this is a cross-major move and trust store loading should be re-tested. |
| `socket2` | `0.5` | `0.6.3` | Upgrade with networking batch | Useful to align with newer Tokio internals. Since this crate is also used directly in `tunnel-lib`, local socket code should be compiled/tested carefully. |

### Batch C: Defer Unless There Is A Specific Reason

| Crate | Current | Observed latest | Recommendation | Why |
| --- | --- | --- | --- | --- |
| `sha2` | `0.10` | `0.11.0` | Defer | Cross-major but no obvious repo-level payoff right now. |
| `bincode` | `1.3` | `3.0.0` | Defer strongly | Large API jump. This repo uses bincode in protocol/message paths; changing codec generation/compatibility is not a casual dependency bump. |

## Suggested Upgrade Order

1. Move the workspace toolchain baseline to Rust 1.95.
2. Land the Rust 1.95 code cleanups:
   - `Vec::push_mut`
   - `if let` guards in `match`
3. Upgrade the low-risk async/networking stack:
   - `tokio`
   - `hyper`
   - `hyper-rustls`
   - `rustls`
   - `tracing-subscriber`
   - `arc-swap`
   - `clap`
   - `rand`
4. Re-run unit tests and CI helpers.
5. Then split the higher-risk items into dedicated PRs:
   - gRPC stack: `tonic`, `tonic-health`, `prost`
   - reload/FS watcher stack: `notify`
   - metrics stack: `metrics-exporter-prometheus`
   - TLS roots / websocket helpers: `webpki-roots`, `tokio-tungstenite`

## Practical Optimization Ideas For This Repo

### Use Rust 1.95 language/library additions

- Refactor `tunnel-store/src/sqlite_rules.rs` to use `Vec::push_mut` when constructing grouped listener/upstream structures.
- Refactor `tunnel-service/src/service.rs` to use `if let` guards inside `match` for cleaner debounce/backoff flow.

### Favor the Tokio + Hyper refresh first

This repo is dominated by:

- async IO;
- QUIC/TCP accept loops;
- HTTP/1 + HTTP/2 proxying;
- background tasks and coordination.

Because of that, the `tokio` and `hyper` upgrades are the most likely to give real stability/perf benefit with the least code churn.

### Keep the TLS stack coherent

When touching:

- `hyper`
- `hyper-rustls`
- `rustls`
- `webpki-roots`

upgrade them as a small batch rather than one-by-one. That reduces the chance of spending time on mismatched versions and feature flags.

## References

- Rust 1.95 blog: <https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/>
- Rust 1.95 release notes: <https://doc.rust-lang.org/stable/releases.html#version-195-2026-04-16>
- Tokio changelog: <https://docs.rs/crate/tokio/latest/source/CHANGELOG.md>
- Hyper releases: <https://github.com/hyperium/hyper/releases>
- Tonic releases: <https://github.com/hyperium/tonic/releases>
- Tonic crate docs: <https://docs.rs/crate/tonic/latest>
- tokio-tungstenite changelog: <https://docs.rs/crate/tokio-tungstenite/latest/source/CHANGELOG.md>
- notify crate docs: <https://docs.rs/crate/notify/latest>
- metrics-exporter-prometheus docs: <https://docs.rs/metrics-exporter-prometheus>
- webpki-roots crate docs: <https://docs.rs/crate/webpki-roots/latest>
