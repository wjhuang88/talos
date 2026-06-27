# ARCH-018: TUI App Stream Render Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Iteration**: I065
**Long task**: `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`
**Depends on**: ARCH-016 complete; architecture debt burn-down T3 complete

## Problem

`crates/talos-tui/src/app.rs` still mixes the `Tui` runtime/event loop with stream rendering state
and scrollback line data structures. The stream rendering state has its own tests and dependencies
on Markdown/highlighting helpers, making it a safer first split than event-loop or terminal cursor
logic.

## Scope

- Extract `ScrollbackLine`, `StreamRenderState`, and `SPINNER_FRAMES` into a focused module.
- Preserve existing `crate::app::{ScrollbackLine, StreamRenderState, SPINNER_FRAMES}` imports
  through re-export.
- Do not change stream rendering, Markdown/highlighting, scrollback flushing, frame assembly,
  input handling, terminal cursor behavior, or event-loop behavior.

## Acceptance Criteria

- [x] Owner story and iteration exist before code edits.
- [x] Stream rendering state moves out of `app.rs`.
- [x] Existing `crate::app::*` test/helper imports remain compatible.
- [x] `cargo test -p talos-tui --quiet` passes.
- [x] Workspace gates pass.
- [x] Governance validation passes.
- [x] Remaining TUI app split work is recorded.

## Implementation Notes

- Planned target module: `crates/talos-tui/src/app_stream.rs`.
- `app.rs` should re-export the moved types/constants to keep the migration low-churn.

## Verification Evidence

- 2026-06-27: `cargo check -p talos-tui` passed.
- 2026-06-27: `cargo clippy -p talos-tui -- -D warnings` passed.
- 2026-06-27: `cargo test -p talos-tui --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed.
- 2026-06-27: `git diff --check` passed.

## Residual Architecture Candidates

- `Tui::run` event loop and output queue handling.
- `draw_frame` component assembly and cursor placement.
- `handle_input_event` input state mutation.
- Exit summary formatting.
