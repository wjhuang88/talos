# Iteration I064: CLI Print Mode Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue the architecture debt burn-down by extracting print-mode execution
>   from `talos-cli/src/mode_runners.rs` without changing CLI behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: print mode lives in a focused module, `crate::mode_runners::run_print_mode`
>   remains available, and CLI/workspace gates pass.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-017` | Architecture debt burn-down | Promoted by T1 | ARCH-014 complete; long-task T0 complete | Extract `mode_print.rs`. |

## Scope

- Move `run_print_mode` out of `mode_runners.rs`.
- Keep behavior and crate-local call paths unchanged.
- Run CLI targeted tests and workspace gates.

## Acceptance

- [x] `mode_print.rs` exists and owns print-mode execution.
- [x] Existing `crate::mode_runners::run_print_mode` path remains valid.
- [x] `cargo test -p talos-cli --quiet` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | I064 opened from the architecture debt burn-down long task after T0 inventory confirmed `mode_runners.rs` remains the largest production file. |
| 2026-06-27 | Added `mode_print.rs`, moved `run_print_mode`, and re-exported it through `mode_runners.rs` to preserve dispatch imports. |
| 2026-06-27 | `mode_runners.rs` reduced from 1912 to 1778 lines. |
| 2026-06-27 | Full gates passed: `cargo clippy -p talos-cli -- -D warnings`, `cargo test -p talos-cli --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check`. |

## Validation Plan

- `cargo test -p talos-cli --quiet`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Residual Work

After this slice, continue the long task with further CLI flow splits before moving to TUI app and
agent modules.

## Closure State

I064 is complete. This slice intentionally moved only print-mode execution; inline/TUI/session
command flow splits remain under the architecture debt burn-down task.
