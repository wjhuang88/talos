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
| 2026-06-26 | **Activation** | I050 activated. Dependencies met: I049 in Review (DATA-001 user-facing lifecycle controls operational, commit `20f9b3e`). Scope: episodic-to-semantic consolidation pipeline in `talos-memory` with a deterministic `RuleBasedExtractor` (no provider dependency), `EpisodeExtractor` trait for future LLM-based extraction, `ConsolidationConfig` with disable flag, CLI command `talos memory consolidate`, and tests covering all acceptance criteria. No prompt injection, no procedural memory, no vector/graph — those are I051/I052 scope. |
| 2026-06-26 | **Implementation** | All acceptance criteria delivered: consolidation pipeline with `EpisodeExtractor` trait + deterministic `RuleBasedExtractor`, `consolidate_episodes()` ADD-only pipeline with evidence links, `ConsolidationConfig` (default disabled), CLI `talos memory consolidate [--session <UUID>]`. 6 unit tests covering all 5 acceptance scenarios. Runtime smoke verified with real session (2 candidates extracted, 2 inserted, 2 evidence links; second run deduped all). |

## Verification Evidence

### Workspace Gates (2026-06-26)

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo test --workspace` — all pass, 0 failures
- `scripts/validate_project_governance.sh .` — 0 warnings

### End-to-End Runtime Evidence (ITERATION-WORKFLOW §3a)

- `talos memory consolidate --session 924e9af8...`: extracted 2 candidates, inserted 2, created 2 evidence links.
- Second run on same session: 0 inserted, 2 duplicates skipped (content-hash dedup verified).
- `talos storage status` after consolidation: memory.db 48.0 KB, Memory items: 2.
- `talos memory consolidate` on empty latest session: "Session ... has no entries." (graceful degradation).

### Changed Files

| File | Change |
|---|---|
| `crates/talos-memory/src/consolidation.rs` | NEW: SessionEpisode, MemoryCandidate, EpisodeExtractor trait, RuleBasedExtractor, ConsolidationConfig/Report, consolidate_episodes(), 6 unit tests |
| `crates/talos-memory/src/lib.rs` | Added `pub mod consolidation` + re-exports |
| `crates/talos-memory/Cargo.toml` | Added `uuid` dependency |
| `crates/talos-cli/src/memory_cli.rs` | NEW: MemoryCommand::Consolidate CLI handler |
| `crates/talos-cli/src/main.rs` | Added `mod memory_cli`, `Memory` variant, dispatch |
