# 008: Bundled SQLite for Local Storage

## Status

Accepted — amended by I183 to record the exact five-consumer boundary

## Context

Talos introduced SQLite in I006 for session metadata and FTS5 search, and later extended the same
bundled dependency to evolution, exploration, memory, and a quarantined model-catalog crate. The
current implementation has four runtime consumers (`talos-session`, `talos-evolution`,
`talos-exploration`, and `talos-memory`) plus non-runtime `talos-models`.

This raised a governance question: AGENTS.md Hard Constraint #1 says "Rust only. No C/C++
bindings." At the same time, the product needs a local, embedded, crash-resistant database with
FTS5 and no runtime dependency on a system SQLite installation.

The current locked dependency graph confirms:

- Exactly those five workspace packages cross directly into the non-workspace graph that reaches
  `libsqlite3-sys`; `talos-agent`, `talos-cli`, and `talos-runtime` reach it only through workspace
  layering and are not direct consumers.
- `rusqlite/bundled` enables `libsqlite3-sys/bundled`, compiling SQLite into the final binary.
- One `rusqlite 0.40.1` and one `libsqlite3-sys 0.38.1` version are resolved.
- No workspace package depends directly or transitively on quarantined `talos-models`.
- On macOS, `otool -L target/debug/talos` shows no `libsqlite3.dylib` dependency. The binary still
  links platform system libraries/frameworks, but not a system SQLite dynamic library.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|------------|------|--------|-------------|
| Rust-only project; no arbitrary C/C++ bindings | Hard | AGENTS.md #1 | Only by explicit ADR exception |
| No runtime dependency on local SQLite installation | Hard | Local-dev/distribution goal | No |
| Session search needs FTS5 | Hard | I006 acceptance | Maybe, but only by changing the feature |
| Evolution patterns need structured local queries | Hard | I008/ADR-001 | Maybe, but only by changing storage design |
| Keep sessions human-recoverable | Hard | ADR-002 | No; JSONL remains source of truth |

## Reasoning

SQLite is the smallest proven storage engine that satisfies the current query requirements:

- FTS5 search for session content.
- Transactional metadata indexing.
- Structured observation/pattern queries for self-evolution.
- Single-file local persistence with predictable operational behavior.

Using `rusqlite` without `bundled` would make Talos depend on whatever SQLite version is installed
on the target machine. That violates the self-contained distribution goal and risks missing FTS5 or
version-specific behavior.

Using `rusqlite` with `bundled` does introduce a C library through `libsqlite3-sys`, but it does so
as a tightly scoped, statically linked storage engine dependency. The runtime does not need a system
SQLite package, dynamic library, daemon, or service.

Pure-Rust alternatives do not currently meet the same requirements with less risk:

- JSONL-only search is simple but does not provide ranked FTS or efficient metadata queries.
- Pure-Rust embedded databases add migration cost and usually do not provide SQLite-compatible FTS5.
- A custom index would be speculative infrastructure and would duplicate mature database behavior.

## Decision

1. `rusqlite` with `features = ["bundled"]` is an approved exception to AGENTS.md Hard Constraint
   #1 for local storage only.
2. The exception is limited to this exact allowlist:
   - runtime: `talos-session`, `talos-evolution`, `talos-exploration`, and `talos-memory`;
   - quarantined non-runtime: `talos-models`. Accepting its existing store does not authorize a
     workspace package to depend on it or activate `catalog.db` in CLI/TUI/runtime paths.
3. SQLite remains an implementation detail for indexes and structured runtime state; JSONL session
   files remain the source of truth.
4. All crates that use SQLite must use one workspace-wide `rusqlite`/`libsqlite3-sys` version to
   avoid duplicate native `links = "sqlite3"` conflicts.
5. The project must describe this precisely as:
   "SQLite is bundled into the binary; Talos does not require a system SQLite installation. The
   binary is not fully static on macOS/Linux because it may still link platform system libraries."
6. `scripts/validate_sqlite_consumers.py` enforces the exception from parsed
   `cargo metadata --locked` output. A workspace package counts only when it has a direct resolved
   edge to a non-workspace package that transitively reaches `libsqlite3-sys`; workspace-only
   layering does not count. Normal, build, development, and target-specific edges are all included.
   The validator also enforces the exact allowlist, bundled features, one resolved
   `rusqlite`/`libsqlite3-sys` version each, and zero workspace dependents of `talos-models`.

**Rejected alternatives:**

- *Use system SQLite* — reduces build complexity but loses self-contained distribution and version
  control.
- *Replace SQLite with JSONL scans* — acceptable for source-of-truth storage, not for FTS/search
  performance and evolution queries.
- *Build a custom Rust FTS/index layer now* — speculative, higher maintenance, and not needed for
  current iteration scope.

## Reversal Trigger

Revisit this decision if:

- A mature pure-Rust embedded database provides the required FTS/search/query behavior with lower
  operational risk.
- SQLite bundled builds become a major cross-compilation or supply-chain blocker.
- Talos changes its storage requirements so FTS5 and structured evolution queries are no longer
  needed.

## Related

- [ADR-002: Local Storage Architecture](002-local-storage-architecture.md)
- [SQLite Consumer Inventory](../reference/SQLITE-CONSUMER-INVENTORY.md)
- `crates/talos-session/Cargo.toml`
- `crates/talos-evolution/Cargo.toml`
- `crates/talos-exploration/Cargo.toml`
- `crates/talos-memory/Cargo.toml`
- `crates/talos-models/Cargo.toml`
- `crates/talos-session/src/sqlite.rs`
- `crates/talos-evolution/src/store.rs`
- `crates/talos-exploration/src/lib.rs`
- `crates/talos-memory/src/store.rs`
- `crates/talos-models/src/store.rs`
- `scripts/validate_sqlite_consumers.py`
- EVOLUTION.md lesson 10: shared SQLite versions across crates
