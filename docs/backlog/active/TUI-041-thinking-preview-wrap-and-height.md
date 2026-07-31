# TUI-041: Thinking Preview Wrapping And Dynamic Height

| Field | Value |
|---|---|
| Story ID | TUI-041 |
| Type | TUI / Product Story |
| Priority | P1 |
| Status | Refinement — layout compression and real-terminal matrix require iteration selection |
| Source | [GitHub Issue #69](https://github.com/wjhuang88/talos/issues/69) |
| Selected Iteration | None |
| Depends On | TUI-039 layout continuity; ADR-034 reasoning boundary; ADR-054 renderer |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #69 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Make the generic live preview use one display-width-aware layout plan for wrapping, measured height, bounded tail clipping, and rendering so thinking/stream text does not truncate or corrupt adjacent rows.

## Scope

- Shared preview layout planner for explicit newlines, CJK/wide text, continuation indentation, and styling.
- Bounded `MAX_PREVIEW_LINES` growth and newest-tail clipping marker.
- Compressible layout integration, anchor/follow-tail preservation, and stale-row cleanup.

## Exclusions

- No thinking persistence, full thinking panel, provider protocol change, or keyboard scrolling.

## Dependencies

TUI-039 layout continuity; ADR-034 reasoning boundary; ADR-054 renderer

## Decision Links And Constraints

- Height measurement and rendering use the same planned rows.
- Composer and required modal rows outrank optional preview expansion.
- Preview remains transient and excluded from transcript/export/session.

## Uncertainty And Validation Path

Refine the exact compression priority and terminal matrix, then select a dedicated TUI iteration.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #69.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Refinement.

## Required Reads

- docs/backlog/active/TUI-039-growing-conversation-composer-continuity.md
- docs/decisions/034-reasoning-thinking-boundary.md
- docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md
- crates/talos-tui/src/scrollback.rs
- crates/talos-tui/src/app_layout.rs

## Acceptance For Behavior / Technical Work

- Long and multiline preview wraps and grows from one row to the bounded cap.
- Resize/clear shrinks cleanly with no stale cells.
- Composer/panel placement and anchored/follow-tail history remain stable.
- Focused buffer/layout tests and native terminal walkthrough pass.

## Residual Destination

User-configurable or scrollable preview behavior is a separate follow-up.
