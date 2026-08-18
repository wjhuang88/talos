# TUI-041: Thinking Preview Wrapping And Dynamic Height

| Field | Value |
|---|---|
| Story ID | TUI-041 |
| Type | TUI / Product Story |
| Priority | P1 |
| Status | Review / Claimed |
| Source | [GitHub Issue #69](https://github.com/wjhuang88/talos/issues/69) |
| Selected Iteration | I199 — Review / Claimed |
| Depends On | TUI-039 layout continuity; ADR-034 reasoning boundary; ADR-054 renderer |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | TUI-041 / I199 only: implement and validate the bounded transient preview layout plan, shared measurement/rendering, display-width wrapping, six-row cap, constrained layout compression and cleanup. Excludes TUI-042/#79, TUI-045/#125, persistence, provider protocol and release work. |
| Claimed At | 2026-08-18 |
| Source Issue | #69 |
| Governance Claim PR | #295 |
| Authorization Mode | Independent review |
| Authorization Evidence | Activation PR #295 exact head `883d5cc1`, CI `32114936912`, independent approval `5325926209`, and claim merge `8127fa57`. |
| Implementation PR | #297 |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | Preserve TUI-042/TUI-045 boundaries; close only after PR #297 exact-head CI, maintainer native-terminal acceptance, independent review, merge-time CAS and owner-first closeout. |

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

I199 owns the dedicated runnable correction. It must confirm the exact compression order against
current `app_layout` behavior before production edits and preserve composer plus required-panel
priority. A conflict with ADR-054 or the completed TUI-039 contract blocks implementation rather
than authorizing a renderer redesign.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #69.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Review.

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

## 2026-08-17 Upcoming-Pool Checkpoint

The maintainer explicitly reconfirmed Issue #69 for the upcoming task pool. Its existing owner and
iteration remain authoritative: TUI-041 is Ready and I199 is Planned / Unclaimed in the ordered
mainline sequence. This checkpoint does not activate I199, establish a claim, create an
implementation branch or change the published acceptance target.

## 2026-08-18 Implementation Review Checkpoint

Implementation commit `938c9edb9b3336e81a3b90232a69e0993574bc69` is in PR #297. It
introduces one shared display-width-aware preview layout plan, bounded six-row growth, explicit
clipping, constrained compression and cleanup tests without changing persistence, provider
protocol, Cargo manifests, Desktop, Dashboard or release behavior.

Focused evidence passed at the implementation commit: `cargo test -p talos-tui --locked` (536
unit tests, 2 integration tests and 2 doctests), `cargo clippy -p talos-tui --all-targets --locked
-- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check`. An isolated local PTY
fixture exercised multiline ASCII/CJK growth, narrow wrapping, the six-row cap, clipped-tail
retention and completion cleanup at 80x24 and 40x15. This agent-run fixture does not replace the
required maintainer native-terminal acceptance or independent exact-head review, so TUI-041
remains Review and its Completion Commit remains pending.
