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
Configuration override mutations, materialized routing, effective state, and
effective snapshot revision updates use the same SQLite transaction. The
in-memory snapshot and watch notification are updated only after that
transaction commits.

The SQLite layer stores both the high-priority override metadata and the
materialized effective routing. A materialized value is not itself treated as
an override: source-layer and tombstone records are retained so that clearing
an override can reveal the current YAML value again.

The `admin_idempotency` table stores request keys, fingerprints, operation
names, status and response encoding as part of the mutation transaction. Raw
bearer-token responses are never stored there; token mutations use a redacted
commit marker and rely on the bounded in-process response cache only for
same-process retries. Records older than the 30-day idempotency retention
window are removed during SQLite startup migration; callers must retry an
expired key with a new key.

The merge is resource-key based, not field-level deep merge. YAML is the
default layer; SQLite upserts replace a resource, tombstones hide a YAML
resource, and clearing an override restores the YAML resource. Tokens remain
SQLite-owned.

`SqliteConfigSource` is a read-only observer over the `config_layers` override
row. It polls the committed `source_revision` and publishes a new layer only
when the row changes. It never calls `save_sqlite_layer_on`; that write remains
inside the ctld admin mutation transaction. The source may detect an
unsupported direct SQLite edit, but direct edits are not part of the supported
configuration contract.

The `schema_migrations` table records source migration markers. For an old
database without `config_layers`, existing normalized routing tables are
serialized into the SQLite override layer in one repeatable initialization
transaction. If the old ctld configuration contains `server_config`, its
referenced file is used as the YAML base when no explicit YAML source exists;
the old routing rows remain the higher-priority override. Existing override
payloads are validated and never replaced by a rerun. The markers are
`legacy-routing-to-sqlite-override-v1` and
`legacy-server-config-yaml-base-v1`.

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
