# TUI-045: Permission Prompt Layout Anchor Stability

| Field | Value |
|---|---|
| Story ID | TUI-045 |
| Type | Bug / TUI / Permission UX Story |
| Priority | P1 |
| Status | Review / Claimed |
| Source | [GitHub Issue #125](https://github.com/wjhuang88/talos/issues/125) |
| Selected Iteration | I197 — Review / Claimed |
| Depends On | Inline TUI scrollback/composer ownership; permission panel state; ADR-054 renderer |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline planning session |
| Work Slice | TUI-045 / I197 only: preserve the logical viewport/composer anchor and prior tail-follow state across the transient permission prompt, queued prompts and resize using focused layout tests and real-terminal evidence. Presentation/layout only; excludes permission semantics, policy/default decisions, request identity, persistence, broad renderer changes and release work. |
| Claimed At | 2026-08-19 |
| Source Issue | #125 |
| Governance Claim PR | #304 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #304 merged to `main` as `0db92cf9` from the exact claim base. This effective claim authorizes only the bounded TUI-045 implementation; exact-head CI, independent Agent technical review, merge-time CAS and deferred human/manual rows remain required. Any protected permission/security scope stops and requires independent security review. |
| Implementation PR | #305 |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Claim #304 is effective at merge `0db92cf9`; implementation PR #305 merged as `d98f37e7` after exact-head CI, Agent technical review and CAS without altering permission semantics. Eligible natural-person and terminal rows remain in VALIDATION-002/I211/Issue #302 while this Story stays Review. |

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

The I197 claim became effective through PR #304 merge `0db92cf9`. Implementation PR #305 merged to
`main` as `d98f37e7` from final head `9fce4f13`, including implementation commit `ff4141ca`; it
remains presentation/layout-only and does not authorize permission-policy or protected-crate
changes.

## 2026-08-19 Implementation Checkpoint

The implementation passed `cargo test -p talos-tui --locked` (535 unit tests, 2 integration tests,
2 doctests), `cargo fmt --all -- --check`, `git diff --check` and the complete release preflight.
Exact-head CI, independent technical review, merge-time CAS and the deferred human/manual rows
remain open; this Story stays `Review`.

## 2026-08-19 Implementation Merge Disposition

PR #305 final head `9fce4f13` merged to `main` as `d98f37e7` after exact-head CI `32204974418`
passed all five jobs, independent Agent technical review `5336592072` approved the exact head with
shared-account identity limits disclosed, and merge-time CAS passed. The implementation remains
presentation/layout-only and preserves permission semantics and request identity. TUI-045 remains
`Review`; only the deferred natural-person and real-terminal rows in Issue #302 / I211 remain open.

## 2026-08-20 I211 Human Validation Failure Disposition

Issue #302 checkpoint `5341637918` did not accept the presentation hierarchy and did not complete
the required terminal matrix. A new-session, non-bottom composer also exposed physical-bottom
permission-panel placement. This Story remains Review; TUI-059 / Issue #330 is the separate
Ready/Unclaimed corrective owner. No permission semantic or implementation authority transfers.
