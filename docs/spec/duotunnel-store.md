# DuoTunnel Store Spec

## Scope

`duotunnel-store` is the persistence layer for DuoTunnel. It provides SQLite-backed implementations for routing rules (`RuleStore`) and authentication (`AuthStore`). Configuration parsing for server bootstrap is also housed here.

## Module Layout

```
crates/duotunnel-store/src/
  lib.rs               (crate root, re-exports all public surface + compat aliases)
  rules.rs             (RoutingData, RuleStore trait, rule domain types)
  traits.rs            (AuthStore trait, AuthError, AuthResult, TokenListEntry)
  token.rs             (token generation and hashing utilities)
  db/                  (database adapter implementations, feature-gated)
    mod.rs
    sqlite.rs          (SqliteAuthStore, open_sqlite_pool — feature = "sqlite")
    rules.rs           (SqliteRuleStore — feature = "sqlite")
  config/              (config file adapter, feature-gated)
    mod.rs             (ServerConfigFile, routing_data_from_server_config — feature = "server-config")
```

## Feature Flags

| Feature | Enables |
|---|---|
| `sqlite` | `db::sqlite`, `db::rules` — SQLite auth and rule stores |
| `server-config` | `config::` — YAML server config file parsing |

Both features are enabled by default in binary targets. Library consumers can opt out.

## Compatibility Aliases

`lib.rs` exposes backward-compat module aliases:

- `pub mod sqlite` → re-exports `db::sqlite::*`
- `pub mod sqlite_rules` → re-exports `db::rules::*`
- `pub mod server_config` → re-exports `config::*`

These preserve existing call sites in `duotunnel-server` and `duotunnel-ctld`. New code should use canonical paths.

## Public Surface (crate root)

- `RuleStore` trait + `RoutingData` and all rule domain types (from `rules`)
- `AuthStore` trait + `AuthError`, `AuthResult`, `TokenListEntry` (from `traits`)
- `ClientStatus`, `TokenStatus` (re-exported from `duotunnel_core`)
- `open_sqlite_pool` (feature = "sqlite", from `db::sqlite`)
- `generate_token`, `hash_token`, `hash_token_bytes` (from `token`)

## Invariants

- Core traits (`RuleStore`, `AuthStore`) and domain types are always available, no feature gate
- SQLite implementations are behind `sqlite` feature; callers that don't need a DB don't pay for it
- Config parsing is behind `server-config` feature; `duotunnel-ctld` and `duotunnel-server` enable this
- `db/` groups all external-storage adapters; `config/` groups all config-file adapters
- Token utilities are internal; only token generation and hashing are public
