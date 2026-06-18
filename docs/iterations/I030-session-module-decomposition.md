# I030: Session Module Decomposition

**Status**: Complete (2026-06-19)
**Target Window**: Week 1 of next month plan
**Depends On**: I029 complete

## Outcome

Complete ARCH-008 by decomposing `crates/talos-session/src/lib.rs` into focused modules without
behavior changes. This protects the core session/state boundary before adding more scheduled task,
memory, remote, and protocol work.

## Selected Stories

- [x] #ARCH-008-A: Inventory public/private items and establish a before/after function map
- [x] #ARCH-008-B: Extract workspace/session topology helpers into `topology.rs`
- [x] #ARCH-008-C: Extract JSONL source-of-truth persistence into `jsonl.rs`
- [x] #ARCH-008-D: Extract session manager/index coordination into `manager.rs`
- [x] #ARCH-008-E: Keep `lib.rs` as the public re-export surface and update architecture docs

`AppServerSession` and `SessionHandle` were listed in the initial plan from stale audit language,
but they already live outside `talos-session` (`talos-agent` / `talos-core::session`). This
iteration corrected that boundary rather than moving unrelated actor code.

## Acceptance Criteria

- [x] `crates/talos-session/src/lib.rs` is <=400 lines.
- [x] Existing public imports remain valid through `pub use`.
- [x] No behavior changes are intentionally introduced.
- [x] Function inventory before/after shows zero lost functions.
- [x] `cargo test -p talos-session` passes.
- [x] `cargo clippy -p talos-session -- -D warnings` passes.
- [x] `cargo check --workspace` passes.

## Risks

- Session actor code is central to TUI, print mode, resume, and future scheduler injection.
- JSONL and SQLite responsibilities are adjacent; this iteration should move JSONL only and keep
  `sqlite.rs` as-is.
- Visibility changes must be limited to `pub(crate)` where cross-module access requires it.

## Verification Log

2026-06-19:

- Decomposed `talos-session/src/lib.rs` from 1737 lines to 45 lines.
- Added focused modules:
  - `error.rs` for `SessionError`.
  - `types.rs` for public session data types and in-memory branch helpers.
  - `jsonl.rs` for append/read/replay/preview JSONL behavior.
  - `topology.rs` for workspace directory identity helpers.
  - `manager.rs` for `SessionManager`, disk scanning, resume/list/search/index coordination.
  - `tests.rs` for the existing session unit tests.
- Preserved public imports through `talos_session::*` re-exports.
- Verification:
  - `cargo test -p talos-session` passed: 55 tests.
  - `cargo clippy -p talos-session -- -D warnings` passed.
  - `cargo check --workspace` passed.
