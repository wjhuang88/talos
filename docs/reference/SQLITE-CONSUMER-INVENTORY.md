# SQLite Consumer Inventory

## Policy Boundary

ADR-008 permits exactly five workspace packages to cross directly into the bundled SQLite native
dependency graph. A workspace package is a consumer only when it has a direct resolved edge to a
non-workspace package that transitively reaches `libsqlite3-sys`. Reachability only through another
workspace package is layering, not another consumer.

The repository validator includes normal, build, development, and target-specific resolved edges.
It is implemented in `scripts/validate_sqlite_consumers.py`, runs from both project-governance
validators, and rejects consumer-set, resolved-version, bundled-feature, or `talos-models`
quarantine drift.

## Accepted Consumers

| Package | Classification | Purpose and owning module | Schema and migration surface |
|---|---|---|---|
| `talos-session` | Runtime | Session search/indexing in `sqlite.rs`; durable-session bindings in `durable.rs`; pending-submission/runtime-state journal under `pending_submission/`; session todo storage under `todo/`. JSONL/TLOG transcript data remains authoritative where documented. | `sessions`, `messages_fts`, `forks`, `durable_bindings`, pending journal/runtime-state tables, and todo/dependency tables. The search index adds `workspace_root` idempotently; pending storage has schema version 1. |
| `talos-evolution` | Runtime | Observation, signal, conflict, and learned-pattern persistence in `store.rs`. | `observations`, `patterns`, `conflicts`, `signals`, `turn_observations`, and `schema_version`. Current migration records version 2, adds pattern columns, and hard-resets incompatible v1 observation/pattern data with a warning. |
| `talos-exploration` | Runtime | Research runs, sources/chunks, FTS search, claims/edges, and syntheses in `lib.rs`. | `schema_version` 1, `research_runs`, `sources`, `source_chunks`, `source_chunks_fts`, `claims`, `claim_edges`, and `syntheses`. Opening is idempotent and initializes version 1. |
| `talos-memory` | Runtime | Structured memory, evidence, entity links, FTS retrieval, retention, and associative graph storage in `store.rs` and `graph.rs`. | `schema_version` 3, `memory_items`, `evidence_links`, `memory_fts`, `entities`, `memory_entities`, `memory_graph_nodes`, and `memory_graph_edges`. Migration initializes/upgrades version 3 and creates graph indexes. |
| `talos-models` | Quarantined non-runtime | Historical provider/model/pricing catalog in `store.rs`. The CLI/runtime uses packaged TOML and must not depend on this crate or create `catalog.db`. | `schema_version` 1, `providers`, `models`, `pricing`, and `catalog_meta`. Opening rejects any schema version other than 1. |

The current resolved versions are `rusqlite 0.40.1` and `libsqlite3-sys 0.38.1`, one each. The
validator derives those versions from locked Cargo metadata rather than treating this document as
dependency truth.

## Existing Failure And Migration Evidence

| Package | Confirmed focused evidence | Unproven or intentionally unchanged under I183 |
|---|---|---|
| `talos-session` | Search-index schema creation; pending schema/reopen; corrupt durable-binding database returns a structured error without entering the busy retry; busy/locked retry exhaustion returns a structured error; live locked SQLite sidecars are skipped until released. | Coverage is call-family-specific. I183 does not establish a uniform panic boundary, operation deadline, retry policy, or corrupt/busy guarantee for every session SQLite surface. |
| `talos-evolution` | Migration and v1 hard-reset tests; `rusqlite::Error` is mapped into `EvolutionError`/`StoreError`; hook migration coverage exists. | No focused corrupt, busy/locked, panic-containment, or operation-deadline fixture was found. |
| `talos-exploration` | Schema-creation and idempotent-reopen tests; SQLite errors propagate as `ExplorationError::Database`. | No focused corrupt, busy/locked, panic-containment, or operation-deadline fixture was found. |
| `talos-memory` | Schema and graph-migration tests; corrupt database opening returns an actionable error rather than panicking; missing parent directories are created. | No focused busy/locked, panic-containment, or operation-deadline fixture was found. |
| `talos-models` | Schema creation/idempotence, corrupt-database error propagation, and incompatible-version rejection; CLI no-`catalog.db` tests protect runtime quarantine. | No focused busy/locked or panic-containment fixture exists, and no runtime fallback path may be inferred because this crate is quarantined from runtime composition. |

These differences are inventory facts, not remediation. `ARCH-034-R04-AG11` owns SQLite containment
classification under the parent R04 finding; any timeout, retry, panic, schema, or runtime fallback
change requires a separately claimed child and cannot be implemented under I183/AG-7.

## Validator Operation

Run either platform governance entrypoint; both invoke the same validator and the same nine-case
fixture matrix:

```bash
scripts/validate_project_governance.sh .
pwsh -NoProfile -File scripts/validate_project_governance.ps1 .
```

The fixture matrix protects the accepted graph, workspace-only layering, alternate normal/build/dev
and target-specific boundary edges, missing accepted consumers, duplicate native versions,
`talos-models` dependents, and removal of bundled features.

Python 3 is a governance-only host dependency here: the frozen I183 scope prohibits Cargo manifest
and Rust-source changes, while the standard library provides one JSON graph implementation on Unix
and Windows. It is not linked into or shipped with Talos. If a supported developer or CI environment
no longer provides Python 3, replacement with a repository-native validator requires its own
authorized scope rather than weakening or bypassing this gate.
