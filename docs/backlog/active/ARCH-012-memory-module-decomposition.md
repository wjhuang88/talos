# ARCH-012: Architecture Corrosion Audit And Memory Module Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I059
**Source**: User request for a standalone architecture optimization task focused on corrosion
judgment and oversized module splitting
**Depends on**: ARCH-011 watchlist discipline; ADR-016 memory boundary; I050-I053 memory delivery

## Problem

Post-release memory work grew `crates/talos-memory/src/lib.rs` into a 2141-line module combining
public domain types, SQLite storage, schema migration, entity extraction, prompt formatting, hidden
output filtering, and tests. That shape is architecture corrosion: unrelated future changes would
touch the same file, making review riskier and increasing the chance that storage, prompt, and
entity behavior leak across boundaries.

## Corrosion Judgment

| Signal | Evidence | Judgment |
|---|---|---|
| Oversized module | `talos-memory/src/lib.rs` was 2141 lines, largest source file in the workspace. | Concrete split candidate. |
| Responsibility mixing | Storage schema, retrieval scoring, prompt injection, entity extraction, retention status, and tests lived together. | Boundary corrosion present. |
| User-facing behavior risk | Memory prompt injection and hidden-output filtering are safety-sensitive and should not be mixed with SQLite maintenance code. | Split before adding more memory features. |
| Watchlist relation | ARCH-011 remained observation-only; memory was not listed there because the growth happened after the prior audit. | Promote to a new owner story instead of editing ARCH-011. |

## Scope

- Split `talos-memory/src/lib.rs` into focused modules without changing public `talos_memory::*`
  imports.
- Keep ADD-only memory semantics, FTS retrieval, entity linking, prompt formatting, hidden-output
  filtering, and retention dry-run behavior unchanged.
- Move tests out of the crate root so production module boundaries stay visible.
- Record current oversized-module inventory and residual candidates for future architecture work.

## Non-Goals

- No schema change.
- No behavior change to memory retrieval, retention, prompt injection, or consolidation.
- No broad workspace refactor.
- No decomposition of unrelated large files in this slice.

## Acceptance Criteria

- [x] `talos-memory/src/lib.rs` reduced from a god module to a small re-export root.
- [x] Storage, types, entity extraction, prompt formatting, and tests have separate modules.
- [x] Public imports such as `talos_memory::MemoryStore`, `talos_memory::MemoryKind`,
  `talos_memory::format_memory_prompt`, and `talos_memory::extract_entities` remain valid.
- [x] `cargo test -p talos-memory` passes after the split.
- [x] Residual oversized-module inventory is recorded for follow-up selection.

## Implementation Notes

- `lib.rs`: crate documentation plus module declarations and public re-exports.
- `types.rs`: memory/domain DTOs and error types.
- `store.rs`: SQLite-backed `MemoryStore`, migration, retrieval scoring, maintenance, retention
  candidate selection, and private SQLite helpers.
- `entities.rs`: deterministic entity extraction.
- `prompt.rs`: memory prompt config, formatting, hidden-output filtering.
- `tests.rs`: existing crate tests migrated out of the root.

`MemoryStore.conn` is now `pub(crate)` so crate-internal tests can continue to inspect schema and
entity-linking tables after moving out of `store.rs`. It remains unavailable to downstream crates.

## Verification Evidence

- 2026-06-27: `cargo test -p talos-memory` passed: 48 unit tests, 0 failures.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace` passed on rerun. First full run had a transient
  `mcp_client_e2e` evidence assertion; targeted rerun and second full run passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed with 0 warnings.
- 2026-06-27: `git diff --check` passed.

## Residual Architecture Candidates

The next architecture pass should select only one candidate with concrete change pressure:

- `crates/talos-config/src/lib.rs` (2083 lines): configuration schema, load/save, masking, and tests
  are still mixed.
- `crates/talos-cli/src/mode_runners.rs` (2062 lines): runtime modes, inline flow, memory prompt
  wiring, and session orchestration remain dense.
- `crates/talos-tui/src/scrollback.rs` (1614 lines): still tracked by ARCH-011; promote only when
  display work requires it.
- `crates/talos-tui/src/app.rs` (1503 lines): future input/rendering separation candidate.
