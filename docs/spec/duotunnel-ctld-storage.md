# ctld Storage

`duotunnel-ctld/src/storage` is the private persistence boundary of the
control plane. It is not a separate crate or process. The server and client
never depend on it and never access SQLite directly.

## Module layout

```text
duotunnel-ctld/src/storage/
  mod.rs
  rules.rs       RoutingData and RuleStore
  traits.rs      AuthStore and token list types
  token.rs       token generation and hashing
  db/
    mod.rs
    sqlite.rs    SQLite auth store and pool setup
    rules.rs     SQLite routing persistence
```

The shared YAML schema and routing data model live in
`duotunnel-lib/src/config/file.rs`. The ctld storage module owns the SQLite
implementation, migrations, overrides, effective state, and revision state.

## Transaction boundary

All admin mutations execute through the resident ctld process. Token rotation
revokes the old token and inserts the new token in one SQLite transaction.
Configuration override mutations and effective snapshot revision updates use
the same control-plane serialization boundary.

## Dependency boundary

```text
duotunnel-server ──┐
duotunnel-client ──┼── duotunnel-lib
duotunnel-ctld ────┘
        │
        └── internal storage/ ── SQLite
```

Future Etcd or another database backend should implement a control-plane
source or storage adapter inside ctld. It should not be exposed as a new data
plane dependency unless it becomes an independently reused product boundary.
