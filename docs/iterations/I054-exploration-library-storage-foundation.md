# I054: Exploration Library Storage Foundation

**Status**: Planned
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
