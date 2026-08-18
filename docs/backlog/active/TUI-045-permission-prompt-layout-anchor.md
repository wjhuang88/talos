# TUI-045: Permission Prompt Layout Anchor Stability

| Field | Value |
|---|---|
| Story ID | TUI-045 |
| Type | Bug / TUI / Permission UX Story |
| Priority | P1 |
| Status | Ready — I197 Planned / Unclaimed |
| Source | [GitHub Issue #125](https://github.com/wjhuang88/talos/issues/125) |
| Selected Iteration | I197 — Planned / Unclaimed |
| Depends On | Inline TUI scrollback/composer ownership; permission panel state; ADR-054 renderer |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #125 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | Establish an effective I197 claim on `main`, then implement from that claim merge or later current `main`; do not alter permission semantics. Per-child CI, Agent technical review and CAS remain merge gates; the natural-person and terminal rows may be recorded in VALIDATION-002/I211/Issue #302 while this Story stays Review. |

## Identity / Goal / Value

When an inline TUI permission request opens, preserve the user's conversation viewport, composer logical anchor and prior tail-follow state so a security-sensitive choice appears next to the current interaction instead of forcing an unconditional jump to the physical terminal bottom.

## Scope

- Treat scrollback viewport, composer anchor, permission panel and tail-follow state as distinct concerns.
- Snapshot a logical pre-prompt layout anchor when the first permission request opens.
- Render the permission panel adjacent to the current composer with the minimum deterministic local reflow needed for full visibility.
- Restore the logical anchor and prior follow-tail state after Allow, Deny, cancel, timeout or error.
- Reuse the same anchor across queued permission prompts without blank-row growth or progressive bottom drift.
- Recompute a logical anchor under terminal resize rather than restoring stale absolute coordinates.

## Exclusions

- No permission-policy/default-decision change, authorization identity change, transcript persistence of the panel, global composer-bottom rule, broad renderer rewrite or non-interactive approval change.

## Decision Links And Constraints

- Permission UI remains transient presentation state, not transcript content.
- Focus transfer cannot be implemented as a viewport reset.
- The triggering request identity, ordering, timeout and tool execution semantics remain unchanged.
- Required choices must remain fully visible; insufficient space permits only the minimum deterministic viewport adjustment.

## Uncertainty And Validation Path

I197 owns the isolated runnable correction. During implementation it must choose the smallest
logical-anchor representation consistent with the current ADR-054 renderer, preserve anchored
versus follow-tail behavior, and prove compression/minimum-reflow plus resize behavior with focused
layout tests and real-terminal evidence.

## State / Status Owners

- Story scope and acceptance: this file.
- Remote discussion: Issue #125.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## Acceptance For Future Implementation

- A non-bottom composer and visible triggering context remain spatially stable when the permission prompt opens and space is sufficient.
- Insufficient space causes only the minimum adjustment required to reveal all choices.
- Resolution restores the prior logical viewport/composer relationship and tail-follow state.
- Repeated prompts, multiline composer, wrapped descriptions, narrow/short terminals and resize remain usable without duplicate rows, cursor artifacts or progressive drift.
- Permission semantics and request identity remain unchanged.
- Focused state/layout tests and the complete real-terminal matrix from Issue #125 pass.

## Residual Destination

General panel docking, user-configurable overlay placement or a broad terminal layout rewrite require separate design owners.

## 2026-08-18 Deferred Human Validation Timing

The maintainer selected the long-task Deferred Human Validation Mode. I197's objective, permission
visibility/fail-closed boundary and full acceptance matrix are unchanged. If implementation reaches
its exact-head CI, independent Agent technical review and CAS gates without touching permission
policy or protected crates, its natural-person review and terminal matrix may be added to Issue
#302 for I211. TUI-045 remains Review until those rows pass; policy or security-scope expansion
cannot use this deferral.
