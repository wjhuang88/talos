# Iteration I065: TUI App Stream Render Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue the architecture debt burn-down by extracting TUI stream rendering
>   state from `talos-tui/src/app.rs` without changing TUI behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `ScrollbackLine`, `StreamRenderState`, and `SPINNER_FRAMES` live in a focused
>   module, existing `crate::app::*` imports remain valid, and TUI/workspace gates pass.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-018` | Architecture debt burn-down | Promoted by T3 | ARCH-016 complete; long-task T3 complete | Extract `app_stream.rs`. |

## Scope

- Move stream rendering state and scrollback line value type out of `app.rs`.
- Keep `Tui` runtime behavior and call paths unchanged.
- Run TUI targeted tests and workspace gates.

## Acceptance

- [x] `app_stream.rs` exists and owns stream rendering state.
- [x] Existing `crate::app::{ScrollbackLine, StreamRenderState, SPINNER_FRAMES}` imports remain valid.
- [x] `cargo test -p talos-tui --quiet` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | I065 opened from the architecture debt burn-down long task after T3 identified stream rendering as the safest TUI app split. |
| 2026-06-27 | Added `app_stream.rs`, moved `ScrollbackLine`, `StreamRenderState`, and `SPINNER_FRAMES`, and re-exported them through `app.rs` for compatibility. |
| 2026-06-27 | `app.rs` reduced from 1503 to 1118 lines. |
| 2026-06-27 | Full gates passed: `cargo check -p talos-tui`, `cargo clippy -p talos-tui -- -D warnings`, `cargo test -p talos-tui --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check`. |

## Validation Plan

- `cargo test -p talos-tui --quiet`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Residual Work

After this slice, continue the long task with event-loop/frame/cursor TUI app splits or move to
agent compaction if the app root is sufficiently reduced.

## Closure State

I065 is complete. This slice intentionally moved only stream rendering state; event-loop, frame
assembly, cursor placement, input handling, and exit summary formatting remain separate residuals.
