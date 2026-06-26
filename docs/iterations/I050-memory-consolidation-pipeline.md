# I050: Memory Consolidation Pipeline

**Status**: Planned
**Created**: 2026-06-26
**Depends On**: I049 complete; DATA-001 lifecycle controls complete or change-control exception

## Objective

Turn persisted episodic sessions into semantic memory candidates through a bounded, ADD-only
consolidation pipeline.

## Published Baseline

### Selected Stories

- I019-S2: episodic-to-semantic consolidation schema and batch pipeline.
- MEM-001 consolidation execution boundary.

### MVP Deliverable

A manual or end-of-session consolidation path reads session JSONL, writes semantic memory records
with evidence links, and can be tested without invoking live providers.

### Scope

- Add consolidation job/service boundary in the appropriate memory/session crate layer.
- Read session episodes from JSONL/index without making JSONL secondary.
- Produce semantic memory candidates with provenance.
- Preserve ADD-only conflict behavior and exact dedup.
- Keep the first automatic trigger conservative and disable-able.

### Non-Goals

- No prompt injection yet.
- No procedural memory yet.
- No vector/graph store.
- No unbounded live-provider dependency in deterministic tests.

### Acceptance

- Given a session with user/assistant/tool entries, consolidation creates semantic memory with
  evidence pointing back to session/source references.
- Given duplicate content, exact content hash dedup prevents duplicate rows.
- Given conflicting same-key facts, records are preserved rather than overwritten.
- Given malformed or empty sessions, consolidation degrades without panicking.
- Given disabled consolidation config, no memory writes happen.

### Validation Plan

- Unit tests for consolidation candidate extraction.
- Temp-dir integration tests for session JSONL to memory DB writes.
- Error-path tests for malformed JSONL and missing evidence.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `README.md` memory section if user-visible controls change.
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/iterations/I019-layered-memory-foundation.md`
- `docs/tasks/2026-06-26-data-memory-exploration-two-month-plan.md`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
