# TUI-042: No-Op History Scroll State Stability

| Field | Value |
|---|---|
| Story ID | TUI-042 |
| Type | TUI / Bug Story |
| Priority | P1 |
| Status | Refinement — transition helper and resize normalization require iteration selection |
| Source | [GitHub Issue #79](https://github.com/wjhuang88/talos/issues/79) |
| Selected Iteration | None |
| Depends On | TUI-039 completed layout contract; ADR-054; interaction with TUI-041 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #79 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Ensure mouse/touchpad history navigation mutates anchor mode only when the visible frame-history start actually changes, preserving short inline composer layout for non-scrollable history.

## Scope

- One scroll-bounds calculation from rendering metrics.
- Central Noop/Anchored/FollowTail transition helper.
- Normalize impossible anchors after resize/reflow/content/preview changes.
- Mode, layout, cursor, and buffer regression tests.

## Exclusions

- No kinetic scrolling, hit testing, row-step change, horizontal scroll, or renderer redesign.

## Dependencies

TUI-039 completed layout contract; ADR-054; interaction with TUI-041

## Decision Links And Constraints

- A no-op input is a complete state/layout no-op.
- Anchored mode requires real movement; returning to FollowTail requires reaching the actual tail.
- Splash/Logo rows share the same coordinate system as rendering.

## Uncertainty And Validation Path

Refine the normalization point and exact scroll-bound helper ownership before selection.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #79.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Refinement.

## Required Reads

- docs/backlog/active/TUI-039-growing-conversation-composer-continuity.md
- docs/backlog/active/TUI-041-thinking-preview-wrap-and-height.md
- docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md
- crates/talos-tui/src/app.rs
- crates/talos-tui/src/history_projection.rs

## Acceptance For Behavior / Technical Work

- Non-scrollable ScrollUp/ScrollDown preserves FollowTail and composer position.
- Real overflow anchors only after visible movement and returns to tail deterministically.
- Resize/reflow to fully visible history clears stale anchor state.
- Mouse bursts and buffer-level regressions remain stable.

## Residual Destination

Gesture/kinetic enhancements require a separate product Story.
