# Iteration I069: CLI Inline Mode Decomposition

> Document status: Complete
> Published plan date: 2026-06-27
> Planned objective: Continue the two-month architecture optimization by extracting CLI inline
>   mode from `talos-cli/src/mode_runners.rs` without changing user-visible behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: inline mode lives in `mode_inline.rs`, `mode_runners.rs` remains the legacy
>   aggregator for other modes, and CLI/workspace gates pass.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-024` | `ARCH-022` | In Progress | Two-month architecture plan M1 | Extract inline mode and runtime Skill command handling. |

## Scope

- Move `run_inline_mode` to `crates/talos-cli/src/mode_inline.rs`.
- Move inline `/skills` command handling with it.
- Re-export inline mode through `mode_runners.rs` to preserve existing `main.rs` imports.
- Preserve config loading, provider setup, MCP startup, permission-aware tool registration,
  runtime Skill activation, context loading, session persistence, event handling, interrupts, and
  shutdown behavior.

## Acceptance

- [x] `mode_runners.rs` is materially smaller than the 1778-line baseline.
- [x] `cargo test -p talos-cli --quiet` passes.
- [x] Workspace checks pass.
- [x] Governance validation passes.

## Execution Log

| Date | Record |
|---|---|
| 2026-06-27 | I069 opened from the two-month architecture optimization plan after M1 selected inline mode as the lowest-risk ARCH-022 residual slice. |
| 2026-06-27 | Extracted inline mode into `mode_inline.rs` and kept a stable re-export through `mode_runners.rs`. |
| 2026-06-27 | `mode_runners.rs` reduced from 1778 to 1500 lines. |
| 2026-06-27 | Validation passed: `cargo test -p talos-cli --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, and `scripts/validate_project_governance.sh .`. |

## Validation Plan

- `cargo test -p talos-cli --quiet`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --quiet`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Residual Work

After this slice, ARCH-022 still owns remaining TUI/session-command flow decomposition inside
`mode_runners.rs` unless the final line-count audit shows no further review-risk boundary.

## Closure State

I069 is complete. The slice intentionally changed only module ownership; inline input handling,
runtime Skill commands, session persistence, interrupts, event handling, and shutdown behavior are
unchanged.
