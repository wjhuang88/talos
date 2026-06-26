# I055: Exploration Ingestion And Citation Workflow

**Status**: Planned
**Created**: 2026-06-26
**Depends On**: I054 exploration storage foundation

## Objective

Make the exploration library usable from Talos workflows by adding permission-aware ingestion and a
citation-preserving synthesis path.

## Published Baseline

### Selected Stories

- RES-001 ingestion and citation workflow.
- WEBFETCH-001 compatibility boundary where existing fetch tools are reused.

### MVP Deliverable

A user can ingest local or fetched text into an exploration run, extract claims, and produce a
synthesis that cites stored source IDs.

### Scope

- Add local-file ingestion path first.
- Reuse existing permission-aware web/fetch tools only through approved boundaries.
- Add claim extraction interface with deterministic test fixtures.
- Add synthesis/citation assembly with evidence/inference distinction.
- Keep ingestion disabled or dry-run where network permission is absent.

### Non-Goals

- No crawler.
- No paid API dependency.
- No document conversion stack beyond existing supported inputs.
- No vector/graph dependency.

### Acceptance

- Given local source text, ingestion creates source chunks and searchable claims.
- Given fetched content through existing permission flow, ingestion records source provenance.
- Given a synthesis, output distinguishes cited evidence from inference.
- Given disabled network tools, exploration still works with local sources.
- Given missing citations, synthesis validation fails.

### Validation Plan

- Local ingestion tests.
- Mock fetch ingestion tests.
- Claim/synthesis citation tests.
- Permission-disabled tests.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/backlog/active/RES-001-exploration-library.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
