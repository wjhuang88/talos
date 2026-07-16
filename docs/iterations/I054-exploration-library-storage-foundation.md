# I054: Exploration Library Storage Foundation

**Status**: Complete (2026-06-29)
**Created**: 2026-06-26
**Depends On**: I053 memory quality and I019 Review/Complete evidence

## Objective

Open I020 by adding local exploration-library storage for research runs, sources, chunks, claims,
claim edges, and syntheses.

## Published Baseline

### Selected Stories

- RES-001 storage schema foundation.
- ADR-017 SQLite/FTS5 implementation slice.

### MVP Deliverable

A library-level exploration store persists research runs, sources, chunks, claims, and syntheses
with FTS5 search and provenance tests.

### Scope

- Add or extend a crate/module for exploration storage.
- Use SQLite/FTS5 under ADR-008 and ADR-017.
- Store sources, chunks, claims, claim edges, syntheses, caveats, and unresolved questions.
- Link claims/syntheses to source IDs.
- Keep network ingestion out of this first storage iteration.

### Non-Goals

- No web search/fetch integration yet.
- No vector/graph DB.
- No automatic memory consolidation from exploration artifacts.

### Acceptance

- Given a research run, the store persists sources, chunks, claims, and synthesis rows.
- Given source chunks, FTS search returns bounded matches.
- Given a synthesis, citations reference stored source or claim IDs.
- Given missing citation targets, insertion fails or reports validation error.
- Given no external service, all tests pass offline.

### Validation Plan

- Schema migration tests.
- FTS search tests.
- Citation integrity tests.
- Offline integration tests in temp dirs.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/RES-001-exploration-library.md`
- `docs/iterations/I020-exploration-library.md`
- `docs/BOARD.md`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-26 | **Activation** | I054 activated. Dependencies met: I053 in Review (I019 quality gate closed, commit `e745e2c`). Scope: new `talos-exploration` crate with SQLite/FTS5 schema for research_runs, sources, source_chunks, claims, claim_edges, syntheses. Citation integrity enforcement. No network ingestion, no vector/graph, no memory consolidation. |
| 2026-06-26 | **Implementation** | All acceptance criteria delivered. New `talos-exploration` crate (17th workspace member) with full schema: research_runs, sources, source_chunks (FTS5), claims, claim_edges, syntheses. `ExplorationStore` with open/open_memory/migrate, CRUD for all entities, FTS5 chunk search, citation integrity validation (claims validate chunk existence, syntheses validate source existence, edges validate both claims). 8 tests including schema migration, FTS bounded search, citation validation failures, full round-trip, idempotent reopen. |

## Verification Evidence

### Workspace Gates (2026-06-26)

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo test --workspace` — all pass, 0 failures
- `scripts/validate_project_governance.sh .` — 0 warnings

### Changed Files

| File | Change |
|---|---|
| `crates/talos-exploration/Cargo.toml` | NEW: crate manifest matching talos-memory deps |
| `crates/talos-exploration/src/lib.rs` | NEW: ExplorationStore with full schema, CRUD, FTS5 search, citation integrity, 8 tests |
| `Cargo.toml` | Added `crates/talos-exploration` to workspace members |
