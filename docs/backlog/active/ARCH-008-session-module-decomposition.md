# ARCH-008: Session Module Decomposition

**Status**: Complete (2026-06-19)
**Priority**: P2
**Source**: Architecture decay audit 2026-06-18 (post-ARCH-005)
**Depends on**: ARCH-005 partial complete

## Problem

`crates/talos-session/src/lib.rs` remained at 1737 lines after the ARCH-005 pass. It mixed
session actor logic, JSONL persistence, SQLite indexing, workspace topology, session fork
operations, search/list modes, and test helpers. This makes narrow changes harder to verify and
increases cognitive load for new contributors.

## Scope

Decompose `talos-session/src/lib.rs` without behavior changes:

- `types.rs` — public session data types (`Session`, `SessionEntry`, `SessionBranch`,
  `SessionInfo`, `SessionMetadata`) and in-memory branch helpers.
- `jsonl.rs` — JSONL source-of-truth persistence (append, load, replay).
- `topology.rs` — workspace identity (`workspace_sha256`), session naming, `workspace_root`
  column handling, same-basename collision prevention.
- `manager.rs` — `SessionManager` disk scanning, resume/list/search/index coordination.
- `error.rs` — `SessionError` public error surface.
- `tests.rs` — existing session unit tests moved out of the public re-export surface.
- Keep `sqlite.rs` (already extracted; `IndexError`, `SqliteStore`) as-is.
- Keep `lib.rs` as the thin re-export surface: `pub use` for all public types, `mod` declarations.

Note: the original audit text incorrectly listed `AppServerSession` and `SessionHandle` under
`talos-session`; those actor types live in `talos-agent` / `talos-core::session` and were not moved
by this no-behavior-change slice.

## Acceptance Criteria

- [x] `talos-session/src/lib.rs` is ≤400 lines after decomposition.
- [x] No behavior changes. Existing `talos-session` public imports (`Session`,
      `SessionManager`, `SessionEntry`, `SessionBranch`, `SessionInfo`, `SessionMetadata`,
      `SessionError`, `SessionIndex`, `IndexError`, `SearchResult`, `ForkInfo`) remain accessible
      through `talos_session::*` re-exports.
- [x] `cargo test -p talos-session` passes.
- [x] `cargo clippy -p talos-session -- -D warnings` passes.
- [x] Architecture reference updated.

## Verification Notes

Baseline: `talos-session/src/lib.rs` at 1737 lines (2026-06-18 audit).

Completion evidence (2026-06-19):

- `talos-session/src/lib.rs`: 45 lines after decomposition.
- New focused modules: `error.rs`, `types.rs`, `jsonl.rs`, `topology.rs`, `manager.rs`,
  `tests.rs`.
- Function inventory preserved:
  - `Session::new`, `append`, `append_event`, `fork`, `with_fork_identity`, `get_branch`,
    `list_branches`, `read_entries`, `read_messages`, `read_events`.
  - `SessionManager::new`, `with_dir`, `sessions_dir`, `create_session`, `get_session`,
    `list_sessions`, `list_workspace_sessions`, `latest_workspace_session`, `resume_session`,
    `search`, `list_recent`, `update_index`.
  - Internal helpers moved: `workspace_dir_name`, `workspace_root_from_dir_name`,
    `scan_file`, `message_parts`, `preview_text`.
- Verification:
  - `cargo test -p talos-session` passed: 55 tests.
  - `cargo clippy -p talos-session -- -D warnings` passed.
