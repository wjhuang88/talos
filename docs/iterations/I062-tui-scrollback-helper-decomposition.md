# Iteration I062: TUI Scrollback Helper Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue architecture optimization by extracting low-risk helper
>   responsibilities from `talos-tui/src/scrollback.rs` without changing TUI behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: input and status formatting helpers live outside `scrollback.rs`, existing
>   `crate::scrollback::*` call sites keep working, and TUI targeted tests pass.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-015` | Architecture residual cleanup | Promoted from current oversized-module audit | ARCH-014 complete | Extract `scrollback_input.rs` and `scrollback_status.rs`. |

## Scope

- Move input/credential/cursor helpers out of `scrollback.rs`.
- Move status line formatting helpers out of `scrollback.rs`.
- Keep behavior and public crate-local call paths unchanged.
- Run TUI targeted gates and workspace gates.

## Acceptance

- [x] `scrollback_input.rs` exists and owns input/credential/cursor helpers.
- [x] `scrollback_status.rs` exists and owns status formatting helpers.
- [x] Existing `crate::scrollback::*` call sites remain valid.
- [x] `cargo check -p talos-tui` passes.
- [x] `cargo clippy -p talos-tui -- -D warnings` passes.
- [x] `cargo test -p talos-tui --quiet` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | Added `scrollback_input.rs` and moved input line counting, cursor position, credential display, and input text construction out of `scrollback.rs`. |
| 2026-06-27 | Added `scrollback_status.rs` and moved status line, truncation, and cost formatting helpers out of `scrollback.rs`. |
| 2026-06-27 | `scrollback.rs` reduced from 1614 to 1386 lines while preserving `crate::scrollback::*` call paths. |
| 2026-06-27 | TUI targeted check, clippy, and tests passed. |
| 2026-06-27 | Full gates passed: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check`. |

## Validation Plan

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Residual Work

The remaining scrollback module still owns Markdown/history rendering and should be split by a
future owner story. `talos-tui/src/app.rs` and agent-side compaction/prompt/session modules remain
separate architecture cleanup candidates.

## Closure State

I062 is complete. This slice intentionally extracted low-risk input and status helpers only; the
remaining Markdown/history rendering split should use a new owner story to keep visual regression
review focused.
