# Iteration I070: TUI Exit Summary Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue the two-month architecture optimization by extracting TUI exit
>   summary formatting from `talos-tui/src/app.rs` without changing visible output.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: exit-summary formatting lives in `app_summary.rs`, `Tui` still inserts the
>   generated history lines, and TUI/workspace gates pass.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-025` | `ARCH-023` | In Progress | Two-month architecture plan M3 | Extract exit-summary formatting from the TUI app root. |

## Scope

- Move exit-summary line construction to `crates/talos-tui/src/app_summary.rs`.
- Move estimated-cost calculation with it.
- Keep terminal mutation in `Tui::print_exit_summary`.
- Do not touch draw-frame, cursor, input, or scrollback flush behavior.

## Acceptance

- [x] `app.rs` is materially smaller than the 1118-line baseline.
- [x] `cargo test -p talos-tui --quiet` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | I070 opened from the two-month architecture optimization plan after M3 selected exit-summary formatting as the lowest-risk ARCH-023 residual slice. |
| 2026-06-27 | Extracted exit-summary line construction and estimated cost calculation into `app_summary.rs`. |
| 2026-06-27 | `app.rs` reduced from 1118 to 1005 lines without touching frame/cursor/input paths. |
| 2026-06-27 | Validation passed: `cargo test -p talos-tui --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, and `scripts/validate_project_governance.sh .`. |

## Validation Plan

- `cargo test -p talos-tui --quiet`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Residual Work

ARCH-023 still owns frame assembly, cursor placement, input handling, approval flow, and event-loop
decomposition. Those require separate visual-risk notes if touched.

## Closure State

I070 is complete. The slice intentionally changed only exit-summary module ownership; frame
rendering, cursor placement, viewport sizing, input handling, approval flow, and scrollback
flushing are unchanged.
