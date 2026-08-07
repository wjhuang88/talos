# ARCH-034-R05: TUI App Coordinator Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F05 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | I174 (Planned; Claim PR #151) |
| Preserved behavior | Rendering, input, stream ordering, approvals, scrollback, and terminal restore |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Private input, stream/UI-output, and frame-coordination source decomposition behind the current `app` facade with exact `Tui` public paths, lifecycle/select priority, rendering, key/mouse/approval, scrollback, cursor, output, and terminal-restoration preservation. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #151 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if the split requires any public API, lifecycle, select-order, rendering, input, output, or terminal behavior change. |

## Problem And Boundary

`talos-tui/src/app.rs` combines `Tui` state, event-loop scheduling, stream consumption, UI-output
dispatch, frame construction, scrolling, and input dispatch in 1,719 production lines. `Tui`
remains the correct public coordinator; its private concerns need source boundaries.

## Scope

- Extract private input, stream/output, and frame coordination helpers while retaining `App`.
- Preserve terminal lifecycle, select priorities, state fields, and render order.

## Exclusions

- No visual redesign, key binding, panel behavior, event protocol, public API, or dependency change.

## Acceptance And Validation

- `Tui::run` remains the sole lifecycle coordinator and private modules have one reason to change.
- Existing snapshots, cursor, approval, stream, exit-summary, and terminal tests remain identical.
- Locked workspace, TUI smoke, governance, and diff checks pass.

## Rollback / Residual

Revert if frame/event equivalence cannot be shown. Visual behavior changes use a separate TUI story.
