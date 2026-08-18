# TUI-042: No-Op History Scroll State Stability

| Field | Value |
|---|---|
| Story ID | TUI-042 |
| Type | TUI / Bug Story |
| Priority | P1 |
| Status | Review / Claimed |
| Source | [GitHub Issue #79](https://github.com/wjhuang88/talos/issues/79) |
| Selected Iteration | I200 — Review / Claimed |
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
| Authorization Evidence | Claim PR #300 exact head `c70dcfa7` passed CI `32144285868`, independent agent review `5329269096`, merge-time CAS `5329300644` and merged as `356dc3c5`. The shared-identity agent review is not represented as a distinct natural person and does not waive I200's published implementation review or terminal walkthrough. |
| Implementation PR | #301 |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | PR #301 merged after exact-head CI, independent Agent technical review and merge-time CAS. PR #303 proposes that, on merge, TUI-042 stays Review while VALIDATION-002/I211/Issue #302 owns the deferred natural-person review and maintainer mouse/touchpad acceptance; no further I200 implementation authority transfers. |

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

## 2026-08-18 Implementation Review Checkpoint

PR #301 submits implementation commit `3afeeb2859a441ef7e1b7628ff4b5b83b974210d` from the
effective #300 claim merge. Rendering-derived bounds now reject target-equals-current mutations,
centralize Noop/Anchored/FollowTail outcomes and clear anchors that become impossible after
height, CJK width reflow or preview-capacity changes. Focused full-frame tests preserve input
buffer/cursor/history state and cover exact fit, one-row overflow and repeated boundaries.

The Story is Review, not Complete. Exact-head CI, independent technical review, merge-time CAS,
independent natural-person exact-head review and maintainer mouse/touchpad walkthrough remain
mandatory. No excluded renderer, transcript/session, provider, persistence or release behavior is
authorized.

## 2026-08-18 Deferred Human Validation Change Control

PR #301 exact head `8a58cb2d56c2607a6c2ee383bed086f08e374811` passed CI
`32149762367`, received independent Agent technical approval `5330234992` with its
non-natural-person identity limit disclosed, passed merge-time CAS and merged as `9628e183`.

The maintainer directed that unavailable natural-person review and mouse/touchpad acceptance be
batched later instead of blocking the ordered long task. The original acceptance remains
unchanged and unpassed. PR #303 proposes transferring those two evidence rows to
VALIDATION-002/I211/Issue #302 while TUI-042 stays Review. That ownership transfer and the
separately scoped I197 preparation become effective only after this change reaches `main`.
