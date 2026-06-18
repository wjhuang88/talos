# I030: Session Module Decomposition

**Status**: Planned
**Target Window**: Week 1 of next month plan
**Depends On**: I029 complete

## Outcome

Complete ARCH-008 by decomposing `crates/talos-session/src/lib.rs` into focused modules without
behavior changes. This protects the core session/state boundary before adding more scheduled task,
memory, remote, and protocol work.

## Selected Stories

- [ ] #ARCH-008-A: Inventory public/private items and establish a before/after function map
- [ ] #ARCH-008-B: Extract workspace/session topology helpers into `topology.rs`
- [ ] #ARCH-008-C: Extract JSONL source-of-truth persistence into `jsonl.rs`
- [ ] #ARCH-008-D: Extract `AppServerSession`, `SessionHandle`, and actor loop into `session_actor.rs`
- [ ] #ARCH-008-E: Keep `lib.rs` as the public re-export surface and update architecture docs

## Acceptance Criteria

- [ ] `crates/talos-session/src/lib.rs` is <=400 lines.
- [ ] Existing public imports remain valid through `pub use`.
- [ ] No behavior changes are intentionally introduced.
- [ ] Function inventory before/after shows zero lost functions.
- [ ] `cargo test -p talos-session` passes.
- [ ] `cargo clippy -p talos-session -- -D warnings` passes.
- [ ] `cargo check --workspace` passes.

## Risks

- Session actor code is central to TUI, print mode, resume, and future scheduler injection.
- JSONL and SQLite responsibilities are adjacent; this iteration should move JSONL only and keep
  `sqlite.rs` as-is.
- Visibility changes must be limited to `pub(crate)` where cross-module access requires it.

## Verification Log

(to be filled as stories land)
