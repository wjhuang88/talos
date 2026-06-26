# I052: Procedural Memory And Entity Linking

**Status**: Planned
**Created**: 2026-06-26
**Depends On**: I051 bounded prompt injection

## Objective

Add procedural memory and lightweight entity linking so Talos can retrieve project-specific
patterns and code/concept context without adding external NLP or vector dependencies.

## Published Baseline

### Selected Stories

- I019-S4: contradiction/provenance metadata and retrieval quality.
- MEM-001 entity-linking refinement.
- Procedural memory extraction for recurring user/project patterns.

### MVP Deliverable

Procedural memories and entity links are stored, retrieved, and injected as advisory context with
tests proving they do not approve or bypass permissions.

### Scope

- Add entity tables or reuse existing memory schema if already present.
- Extract file/code entities from tool call arguments and tree-sitter-backed symbol data where
  available.
- Extract simple concept/file/url entities with deterministic regex.
- Add procedural pattern records with provenance and last-accessed/decay metadata.
- Boost retrieval by entity overlap.

### Non-Goals

- No external NLP runtime.
- No procedural memory authority over permissions, sandbox, or write decisions.
- No automatic self-modifying behavior.

### Acceptance

- Given memories with shared file/code entities, retrieval ranks relevant records higher.
- Given recurring procedural patterns, procedural records are ADD-only and evidence-backed.
- Given permission decisions, procedural memory remains advisory and cannot auto-allow writes.
- Given stale procedural records, ranking decay affects retrieval without deleting data.
- Given missing or malformed tool metadata, extraction safely skips.

### Validation Plan

- Entity extraction unit tests for files, URLs, symbols, and malformed inputs.
- Retrieval-ranking tests for entity overlap.
- Procedural memory storage/retrieval tests.
- Permission-boundary regression test proving no auto-allow path.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/iterations/I019-layered-memory-foundation.md`
- `README.md` if runtime behavior becomes user-visible.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-26 | **Activation** | I052 activated. Dependencies met: I051 in Review (bounded prompt injection with hidden-output guard, commit `7d0e8ee`). Scope: entity tables schema migration + deterministic regex-based entity extraction (files/URLs/code symbols) + entity linking on insert + entity overlap retrieval boost + procedural memory via existing `MemoryKind::Procedural` + permission-boundary regression test. No external NLP, no permission authority. |
| 2026-06-26 | **Implementation** | All acceptance criteria delivered. Schema v2 adds `entities` + `memory_entities` tables. `extract_entities()` uses std-only scanning for URLs, file paths, CamelCase/snake_case code symbols. Entity linking automatic on `insert()` (non-fatal). Retrieval boost: `0.5 × entity_overlap_count` added to final_score. FTS5 query escaping prevents syntax errors on special chars. 8 tests including permission-boundary regression. No new dependencies. |

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
| `crates/talos-memory/src/lib.rs` | Entity tables schema v2, EntityKind/Entity types, extract_entities(), entity linking on insert, entity overlap retrieval boost, FTS5 query escaping, 8 tests |
