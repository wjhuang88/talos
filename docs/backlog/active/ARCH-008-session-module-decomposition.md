# ARCH-008: Session Module Decomposition

**Status**: Planned
**Priority**: P2
**Source**: Architecture decay audit 2026-06-18 (post-ARCH-005)
**Depends on**: ARCH-005 partial complete

## Problem

`crates/talos-session/src/lib.rs` remains at 1737 lines after the ARCH-005 pass. It mixes
session actor logic, JSONL persistence, SQLite indexing, workspace topology, session fork
operations, search/list modes, and test helpers. This makes narrow changes harder to verify and
increases cognitive load for new contributors.

## Scope

Decompose `talos-session/src/lib.rs` without behavior changes:

- `session_actor.rs` — `AppServerSession` struct and `SessionHandle`, the actor run loop,
  turn lifecycle management.
- `jsonl.rs` — JSONL source-of-truth persistence (append, load, replay).
- `topology.rs` — workspace identity (`workspace_sha256`), session naming, `workspace_root`
  column handling, same-basename collision prevention.
- Keep `sqlite.rs` (already extracted; `IndexError`, `SqliteStore`) as-is.
- Keep `lib.rs` as the thin re-export surface: `pub use` for all public types, `mod` declarations.

## Acceptance Criteria

- [ ] `talos-session/src/lib.rs` is ≤400 lines after decomposition.
- [ ] No behavior changes. All existing public types (`Session`, `SessionManager`,
      `AppServerSession`, `SessionHandle`, `SessionConfig`, `SessionOp`, `SessionEvent`) remain
      accessible at the same import paths via `pub use`.
- [ ] `cargo test -p talos-session` passes.
- [ ] `cargo clippy -p talos-session -- -D warnings` passes.
- [ ] Architecture reference updated.

## Verification Notes

Baseline: `talos-session/src/lib.rs` at 1737 lines (2026-06-18 audit).
Compare function inventory before/after: zero functions lost, visibility changes only
`pub(crate)` as required by cross-module access.
