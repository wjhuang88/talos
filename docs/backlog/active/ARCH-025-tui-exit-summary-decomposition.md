# ARCH-025: TUI Exit Summary Decomposition

**Status**: Complete
**Priority**: P3
**Created**: 2026-06-27
**Parent**: ARCH-023 TUI App Residual Decomposition
**Selected iteration**: I070

## Problem

`crates/talos-tui/src/app.rs` still owns exit-summary formatting inside the TUI runtime root. This
formatting is independent of frame assembly, cursor placement, input handling, and approval flow,
so keeping it in the root file increases review noise without protecting behavior.

## Scope

- Extract exit-summary line construction and estimated cost calculation into
  `crates/talos-tui/src/app_summary.rs`.
- Keep `Tui::print_exit_summary` as the terminal insertion owner.
- Preserve summary text, colors, spacing, model/duration/turn/token/cost behavior, and terminal
  history insertion order.

## Out of Scope

- Frame assembly, viewport sizing, cursor placement, input handling, and scrollback flushing.
- Exit-summary visual redesign or metric changes.
- New dependencies, release, commit, or push.

## Acceptance Criteria

- [x] `app.rs` delegates exit-summary formatting to a focused helper module.
- [x] `app.rs` line count is reduced materially from the 1118-line baseline.
- [x] `cargo test -p talos-tui --quiet` passes.
- [x] Workspace quality gates pass.
- [x] Governance validation passes.

## Execution Notes

- Exit-summary line construction now lives in `crates/talos-tui/src/app_summary.rs`.
- `Tui::print_exit_summary` remains the terminal history insertion owner.
- `app.rs` dropped from 1118 to 1005 lines.
- Frame rendering, cursor placement, viewport sizing, input handling, approval flow, and scrollback
  flushing were not touched.

## Visual-Risk Note

This slice does not touch frame rendering, cursor placement, or viewport sizing. Visual risk is
limited to preserving the existing exit-summary lines inserted after TUI restore.
