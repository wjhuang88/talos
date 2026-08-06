# ARCH-034-R05: TUI App Coordinator Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F05 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | Not selected |
| Preserved behavior | Rendering, input, stream ordering, approvals, scrollback, and terminal restore |

## Problem And Boundary

`talos-tui/src/app.rs` combines `App` state, event-loop scheduling, stream consumption, UI-output
dispatch, frame construction, scrolling, and input dispatch in 1,719 production lines. `App`
remains the correct public coordinator; its private concerns need source boundaries.

## Scope

- Extract private input, stream/output, and frame coordination helpers while retaining `App`.
- Preserve terminal lifecycle, select priorities, state fields, and render order.

## Exclusions

- No visual redesign, key binding, panel behavior, event protocol, public API, or dependency change.

## Acceptance And Validation

- `App::run` remains the sole lifecycle coordinator and private modules have one reason to change.
- Existing snapshots, cursor, approval, stream, exit-summary, and terminal tests remain identical.
- Locked workspace, TUI smoke, governance, and diff checks pass.

## Rollback / Residual

Revert if frame/event equivalence cannot be shown. Visual behavior changes use a separate TUI story.
