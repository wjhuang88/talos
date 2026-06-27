# ARCH-023: TUI App Residual Decomposition

**Status**: Planned
**Priority**: P3
**Created**: 2026-06-27
**Source**: Final audit of `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`

## Problem

`crates/talos-tui/src/app.rs` is reduced from the pre-long-task baseline but remains a large
1118-line TUI actor/rendering module. ARCH-018 extracted stream rendering state, but event-loop,
frame assembly, cursor placement, input handling, and exit summary formatting remain in the root
file.

## Scope

- Map one future behavior-preserving TUI app slice before implementation.
- Prefer pure frame/cursor/output queue helpers before mutation-heavy event-loop changes.
- Preserve viewport behavior, cursor placement, scrollback flushing, input handling, approval UX,
  and session lifecycle behavior.

## Acceptance Criteria

- [ ] A future iteration selects one executable TUI app slice.
- [ ] `app.rs` is reduced materially without behavior changes.
- [ ] `cargo test -p talos-tui --quiet` and workspace gates pass.
- [ ] Visual-risk notes are recorded when frame/cursor behavior is touched.

## Required Reads

- `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`
- `docs/backlog/active/ARCH-018-tui-app-stream-render-decomposition.md`
- `docs/iterations/I065-tui-app-stream-render-decomposition.md`
- `crates/talos-tui/src/app.rs`
