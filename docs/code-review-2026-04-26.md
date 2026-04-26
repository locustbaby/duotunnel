# Code Review - 2026-04-26

Scope: Rust workspace under `duotunnel` including `client`, `server`, `tunnel-lib`, `tunnel-service`, and `tunnel-store`.

Verification:

- `cargo test --workspace`
- Result: passed. 75 unit tests ran successfully. Several crates compiled with no tests.

Fix verification:

- `cargo fmt`
- `cargo test --workspace`
- Result: passed after the fixes below.

## Findings

### High - Security

File: `tunnel-service/src/main.rs:50`, `tunnel-service/src/watch/mod.rs:28`

Issue: The ctld watch endpoint is unauthenticated, plaintext, and defaults to `0.0.0.0:7788`.

Detail: Any reachable peer can connect to the watch endpoint and receive routing snapshots plus the token hash cache. Because the endpoint listens on all interfaces by default, an accidental public bind exposes control-plane state.

Fix: Bind to localhost by default, add mTLS or signed bearer authentication for watch clients, and avoid exposing token cache material to unauthenticated peers.

Status: Fixed by changing the default watch bind address to `127.0.0.1:7788` and adding optional watch bearer-token authentication.

### High - Security

File: `config/client.yaml:8`, `config/client.yaml:16`

Issue: The checked-in client config disables TLS verification and includes a real-looking `auth_token`.

Detail: This makes insecure deployment copy-paste likely and risks credential reuse if the sample token is or becomes valid in any environment.

Fix: Replace committed tokens with placeholders, move real tokens to env/local ignored files, and keep `tls_skip_verify: false` in production-facing examples.

Status: Fixed in the root client config by replacing the token with a placeholder and setting `tls_skip_verify: false`.

### High - Stability

File: `server/main.rs:324`

Issue: Server can hang on startup or runtime error while joining background threads.

Detail: `proxy_main` can return when the QUIC task fails, but `shutdown` is not cancelled before `background_thread.join()`. The background hot reload/control-client loops wait on that token, so a bind failure after background startup can deadlock shutdown.

Fix: Cancel `shutdown` before joining background/metrics threads, use a guard/drop path, or start background services only after the QUIC listener binds successfully.

Status: Fixed by cancelling the shared shutdown token before joining background and metrics threads.

### Medium - Correctness

File: `server/listener_mgr.rs:114`

Issue: Listener state is inserted before bind succeeds.

Detail: If `build_reuseport_listener` fails, the manager keeps a map entry for a listener that is not actually running. Future syncs see the port in the map and skip retrying it.

Fix: Bind first, then insert the active listener entry, or remove the placeholder on bind failure.

Status: Fixed by removing the placeholder if listener binding fails.

### Medium - Security

File: `server/handlers/quic.rs:33`

Issue: Login timeout does not cover waiting for the first bidirectional stream.

Detail: An unauthenticated QUIC peer can connect and never open the login stream. The timeout only starts after `accept_bi()` returns, so idle pre-login connections can consume server resources longer than intended.

Fix: Wrap the initial `conn.accept_bi()` in the same login timeout and close idle pre-login connections.

Status: Fixed by applying the login timeout to the initial `accept_bi()` and closing timed-out pre-login connections.

### Medium - Stability

File: `client/main.rs:320`

Issue: Client health readiness is latched true after first successful login.

Detail: On disconnect, `entry_pool.remove(&conn)` runs but `ready` is never reset. `/healthz` can continue returning `200 OK` while no server connection is available.

Fix: Set `ready.store(false, ...)` before leaving `run_client`, then set it true only after a usable connection is registered.

Status: Fixed by resetting readiness to false when the active client connection is removed.

### Low - Performance

File: `tunnel-store/src/sqlite.rs:21`, `tunnel-store/src/sqlite_rules.rs:31`

Issue: SQLite schema relies on `ON DELETE CASCADE`, but the pool never enables `PRAGMA foreign_keys = ON`.

Detail: Repeated `save_routing` deletes parent rows but can leave orphan child rows, causing database growth and stale internal data.

Fix: Enable `PRAGMA foreign_keys = ON` on every SQLite connection or explicitly delete child tables before parent tables.

Status: Fixed by enabling `PRAGMA foreign_keys=ON` in the SQLite pool `after_connect` hook.

### Low - Latency

File: `server/control_client.rs:59`

Issue: ctld reconnect never resumes from the last applied resource version.

Detail: `connect_and_watch` records `last_seen` but always returns `Err`, so the `Ok(version)` branch is unreachable and reconnects send resource version `0`. This forces full snapshot behavior and blocks future delta optimization.

Fix: Return a typed disconnect error carrying `last_seen`, or update `last_version` in the error path.

Status: Fixed by updating `last_version` as snapshots are applied and reusing it on reconnect.

## Summary

| Dimension | Critical | High | Medium | Low |
|---|---:|---:|---:|---:|
| Correctness | 0 | 0 | 1 | 0 |
| Architecture | 0 | 0 | 0 | 0 |
| Security | 0 | 2 | 1 | 0 |
| Performance | 0 | 0 | 0 | 1 |
| Latency | 0 | 0 | 0 | 1 |
| Stability | 0 | 1 | 1 | 0 |
| Maintainability | 0 | 0 | 0 | 0 |
