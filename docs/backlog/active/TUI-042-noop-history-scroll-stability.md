# TUI-042: No-Op History Scroll State Stability

| Field | Value |
|---|---|
| Story ID | TUI-042 |
| Type | TUI / Bug Story |
| Priority | P1 |
| Status | Active / Claimed |
| Source | [GitHub Issue #79](https://github.com/wjhuang88/talos/issues/79) |
| Selected Iteration | I200 — Active / Claimed |
| Depends On | TUI-039 completed layout contract; ADR-054; interaction with TUI-041 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | TUI-042/I200 only: correct no-op and real-movement frame-history scroll transitions, normalize impossible anchors after current resize/reflow/projection metrics are known, and validate the published focused/full-frame/native-terminal matrix. Excludes kinetic/pixel scrolling, wheel-step changes, hit testing, renderer redesign, transcript/session mutation, TUI-045, TUI-043, provider, persistence and release work. |
| Claimed At | 2026-08-18 |
| Source Issue | #79 |
| Governance Claim PR | #300 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance PR #300 is ineffective until its finalized exact head passes CI, both validators, independent agent technical review with shared-identity limits disclosed, merge-time CAS and merge to `main`. The unattended claim path does not waive I200's published natural-person implementation review or maintainer terminal walkthrough. |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | After #300 merges, implement only from that merge or later current `main`; remain Review until the published exact-head review and mouse/touchpad acceptance gates pass. |

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

I200 owns the isolated state-transition correction. It must choose the smallest normalization point
using the exact projection/layout metrics already used by rendering. I199 is ordered first to reduce
overlap around preview-driven viewport capacity, but a recorded I199 blocked disposition does not
erase I200's independently testable no-op scroll outcome.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #79.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract
changes. Do not present this Story as shipped while its claim is proposed or while implementation
remains Active/Review.

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

## 2026-08-18 Claim Preparation Checkpoint

I199/TUI-041 is Complete/Closed and its preview-capacity behavior is now available to the I200
validation matrix. Exact claim-preparation base is
`main@4acb896e5a76253c50aa2075517edd8b0e53a7f9`; Issue #79 remains open and no overlapping open PR
owns this state-transition correction. The proposed work slice preserves the published acceptance
and exclusions. It does not authorize implementation until the finalized I200 claim reaches
`main`, and it does not transfer TUI-045, TUI-043, provider, persistence or release authority.

## 2026-08-18 Finalized Claim Proposal

PR #300 records the bounded Claimed work slice and proposes TUI-042/I200 as Active. The proposal is
not effective until merge to `main`; no implementation branch exists. The single-maintainer claim
path is limited to governance activation and preserves the published independent natural-person
implementation review and maintainer mouse/touchpad walkthrough gates.
