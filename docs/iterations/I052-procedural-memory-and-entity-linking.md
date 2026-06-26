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
