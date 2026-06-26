# I053: Memory Quality And Release Hardening

**Status**: Planned
**Created**: 2026-06-26
**Depends On**: I052 procedural memory and entity linking

## Objective

Close I019 with reliability, observability, retention dry-run, and release-quality evidence before
the exploration library builds on memory.

## Published Baseline

### Selected Stories

- MEM-001 quality closure.
- MEM-005 memory observability/status integration.
- DATA-001 memory retention dry-run completion.

### MVP Deliverable

Users can inspect memory status, dry-run memory retention, and run the workspace with memory
enabled through deterministic tests and mock-provider runtime evidence.

### Scope

- Add memory status surfaces where appropriate.
- Add memory retention dry-run that respects ADR-016 ADD-only semantics.
- Add contradiction/freshness/confidence display in debug/status output.
- Add migration and corruption-tolerance tests.
- Update I019 from Planned/Active to Review only after all acceptance evidence is recorded.

### Non-Goals

- No destructive memory compaction without explicit apply command and separate acceptance.
- No vector/graph dependency.
- No exploration library implementation in this iteration.

### Acceptance

- Given a memory DB, status reports counts, size, and maintenance hints without exposing hidden
  content.
- Given retention policy, dry-run reports candidates without deleting semantic/procedural rows.
- Given corrupted or missing memory DB, startup degrades with an actionable error or disabled memory
  mode.
- Given I019 acceptance, every checklist item has direct validation evidence or a registered
  residual/change-control note.

### Validation Plan

- Memory status and retention dry-run tests.
- Migration/corruption tolerance tests.
- Mock runtime test with memory enabled.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/iterations/I019-layered-memory-foundation.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/BOARD.md`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-26 | **Activation** | I053 activated. Dependencies met: I052 in Review (entity linking + procedural memory + retrieval boost, commit `951afda`). Scope: memory status reporting (counts/sizes by kind), retention dry-run (no deletion), corruption/missing DB graceful degradation, I019 acceptance closure with evidence. No destructive compaction, no vector/graph. |
| 2026-06-26 | **Implementation** | All acceptance criteria delivered. `MemoryStore::memory_status()` reports counts by kind, evidence/entity counts, DB path/size without exposing content. `MemoryStore::retention_candidates(policy)` dry-run reports candidates with truncated key previews and reasons. CLI `talos memory status` and `talos memory retention` commands degrade gracefully on missing/corrupt DB. 7 tests including corruption tolerance and end-to-end pipeline. |

## Verification Evidence

### Workspace Gates (2026-06-26)

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo test --workspace` — all pass, 0 failures
- `scripts/validate_project_governance.sh .` — 0 warnings

### End-to-End Runtime Evidence (ITERATION-WORKFLOW §3a)

- `talos memory status`: Total items: 2, Semantic: 2, Evidence links: 2, DB size: 48.0 KB.
- `talos memory retention --min-confidence 0.9`: 2 candidates found, dry-run disclaimer shown, no deletion.

### Changed Files

| File | Change |
|---|---|
| `crates/talos-memory/src/lib.rs` | MemoryStatus, RetentionPolicy, RetentionCandidate types; memory_status(), retention_candidates() methods; 7 tests |
| `crates/talos-cli/src/memory_cli.rs` | MemoryCommand::Status and MemoryCommand::Retention CLI handlers with graceful degradation |
