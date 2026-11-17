# SQLite API migration consumer

Independent Cargo root for ADR-106. It links all five SQLite consumers with
rusqlite 0.43 and exercises the public typed errors and conversion implementations.
Run with the pinned toolchain:

```sh
cargo run --locked --manifest-path tests/fixtures/sqlite-api-external/Cargo.toml
```

This validates the migrated dependency types, not source compatibility with 0.40.
Downstream callers must update their own rusqlite requirement and mutable index
bindings as described in ADR-106. Real data compatibility has a separate gate.
