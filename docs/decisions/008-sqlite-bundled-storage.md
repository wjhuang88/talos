# 008: Bundled SQLite for Local Storage

## Status

Accepted

## Context

Talos introduced SQLite in I006 for session metadata, FTS5 search, and later I008 evolution
tables. The current implementation uses `rusqlite` with the `bundled` feature in both
`talos-session` and `talos-evolution`.

This raised a governance question: AGENTS.md Hard Constraint #1 says "Rust only. No C/C++
bindings." At the same time, the product needs a local, embedded, crash-resistant database with
FTS5 and no runtime dependency on a system SQLite installation.

The current dependency graph confirms:

- `talos-session` and `talos-evolution` depend on `rusqlite = { version = "0.37", features =
  ["bundled"] }`.
- `rusqlite/bundled` enables `libsqlite3-sys/bundled`, compiling SQLite into the final binary.
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
2. The exception is limited to `talos-session` and `talos-evolution`.
3. SQLite remains an implementation detail for indexes and structured runtime state; JSONL session
   files remain the source of truth.
4. All crates that use SQLite must use one workspace-wide `rusqlite`/`libsqlite3-sys` version to
   avoid duplicate native `links = "sqlite3"` conflicts.
5. The project must describe this precisely as:
   "SQLite is bundled into the binary; Talos does not require a system SQLite installation. The
   binary is not fully static on macOS/Linux because it may still link platform system libraries."

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
- `crates/talos-session/Cargo.toml`
- `crates/talos-evolution/Cargo.toml`
- `crates/talos-session/src/sqlite.rs`
- `crates/talos-evolution/src/store.rs`
- EVOLUTION.md lesson 10: shared SQLite versions across crates
