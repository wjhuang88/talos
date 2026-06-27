# ARCH-022: CLI Mode Runner Residual Decomposition

**Status**: Planned
**Priority**: P3
**Created**: 2026-06-27
**Source**: Final audit of `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`

## Problem

`crates/talos-cli/src/mode_runners.rs` is reduced from the pre-long-task baseline but remains a
large 1778-line orchestration module. ARCH-014 and ARCH-017 extracted runtime helpers and print
mode, but inline/TUI/session-command flow boundaries still need future slices.

## Scope

- Map the remaining CLI mode-runner flows into focused modules before implementation.
- Prefer behavior-preserving extraction of inline mode, TUI mode setup, session command handling,
  or shared MCP/session setup helpers.
- Preserve CLI behavior, permission setup, model/config loading, MCP startup, and runtime Skill
  activation semantics.

## Acceptance Criteria

- [ ] A future iteration selects one executable CLI flow slice.
- [ ] `mode_runners.rs` is reduced materially without behavior changes.
- [ ] `cargo test -p talos-cli --quiet` and workspace gates pass.
- [ ] README/docs are updated only if user-visible behavior or architecture claims change.

## Required Reads

- `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`
- `docs/backlog/active/ARCH-014-cli-mode-runtime-helper-extraction.md`
- `docs/backlog/active/ARCH-017-cli-print-mode-decomposition.md`
- `docs/iterations/I061-cli-mode-runtime-helper-extraction.md`
- `docs/iterations/I064-cli-print-mode-decomposition.md`
- `crates/talos-cli/src/mode_runners.rs`
