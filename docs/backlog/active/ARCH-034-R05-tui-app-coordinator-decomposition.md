# ARCH-034-R05: TUI App Coordinator Decomposition

> Document status: Complete

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F05 |
| Status | Complete |
| Priority | P2 |
| Selected Iteration | I174 (Complete; PR #152) |
| Preserved behavior | Rendering, input, stream ordering, approvals, scrollback, and terminal restore |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Private input, stream/UI-output, and frame-coordination source decomposition behind the current `app` facade with exact `Tui` public paths, lifecycle/select priority, rendering, key/mouse/approval, scrollback, cursor, output, and terminal-restoration preservation. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #151 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #152 |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Closed; any future public API, lifecycle, select-order, rendering, input, output, or terminal behavior change requires a separate TUI story. |

## Problem And Boundary

`talos-tui/src/app.rs` combines `Tui` state, event-loop scheduling, stream consumption, UI-output
dispatch, frame construction, scrolling, and input dispatch in 1,719 production lines. `Tui`
remains the correct public coordinator; its private concerns need source boundaries.

## Scope

- Extract private input, stream/output, and frame coordination helpers while retaining `Tui`.
- Preserve terminal lifecycle, select priorities, state fields, and render order.

## Exclusions

- No visual redesign, key binding, panel behavior, event protocol, public API, or dependency change.

## Acceptance And Validation

- `Tui::run` remains the sole lifecycle coordinator and private modules have one reason to change.
- Existing snapshots, cursor, approval, stream, exit-summary, and terminal tests remain identical.
- Locked workspace, TUI smoke, governance, and diff checks pass.

## Completion Evidence

- Completion Commit: `e4248bfedd17c91aebb24c80c60580fcbcebec62`
- Implementation PR #152 merged at `62b09c277713bea8404ed7ef9c7f50354e5a2e17`.
- Exact-head CI `31148908291` passed Unix/Windows workspace, governance, and rebuilt CLI smoke checks.
- Merge-time CAS confirmed base `49ff4e24bc1072569f145e8eea735e9500094a92`, head `f681599176eca7f3d25801d1da98bdd623a99f5e`, no blocking reviews/comments, and no overlap.

## Rollback / Residual

Revert if frame/event equivalence cannot be shown. Visual behavior changes use a separate TUI story. R04 remains Refinement pending independent security review; R06-R11 remain Ready and separately claimable.
