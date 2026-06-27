# ARCH-024: CLI Inline Mode Decomposition

**Status**: Complete
**Priority**: P3
**Created**: 2026-06-27
**Parent**: ARCH-022 CLI Mode Runner Residual Decomposition
**Selected iteration**: I069

## Problem

`crates/talos-cli/src/mode_runners.rs` still owns inline-mode orchestration after print-mode and
runtime-helper extraction. Inline mode is a self-contained flow with prompt input, runtime Skill
commands, session persistence, and stdout streaming. Keeping it in the residual runner root makes
future CLI session-flow changes harder to review.

## Scope

- Extract `run_inline_mode` and inline `/skills` command handling into
  `crates/talos-cli/src/mode_inline.rs`.
- Preserve config/model/provider overrides, MCP startup, permission-aware tool registration,
  runtime Skill activation, context loading, session resolution, message persistence, Ctrl+C
  interrupt, and shutdown semantics.
- Keep public CLI behavior unchanged.

## Out of Scope

- TUI mode/session command refactors.
- Permission or sandbox semantic changes.
- Runtime Skill feature expansion.
- New dependencies, release, commit, or push.

## Acceptance Criteria

- [x] `mode_runners.rs` delegates inline mode to a focused module.
- [x] `mode_runners.rs` line count is reduced materially from the 1778-line baseline.
- [x] `cargo test -p talos-cli --quiet` passes.
- [x] Workspace quality gates pass.
- [x] Governance validation passes.

## Execution Notes

- `run_inline_mode` and `handle_inline_skills_command` now live in
  `crates/talos-cli/src/mode_inline.rs`.
- `mode_runners.rs` re-exports `run_inline_mode` so existing `main.rs` imports stay stable.
- `mode_runners.rs` dropped from 1778 to 1500 lines.
- Behavior was intentionally unchanged.

## Required Reads

- `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
- `docs/backlog/active/ARCH-022-cli-mode-runner-residual-decomposition.md`
- `docs/iterations/I064-cli-print-mode-decomposition.md`
- `crates/talos-cli/src/mode_runners.rs`
