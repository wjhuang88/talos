# Iteration I279: History Reasoning And Live Activity Presentation

> Document status: Planned
> Published plan date: 2026-09-20
> Planned objective: Deliver #334 history continuation padding, #298 collapsible reasoning history, and #310 live activity status/count headers as one locally converged TUI stage.
> MVP deliverable: Runnable TUI with stable padded history, independently collapsible thinking entries, and truthful live thinking/tool titles and display-row counts.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Proposed TUI-061 / TUI-056 / TUI-057 presentation, projection, interaction, tests and documentation only. Excludes Auto permission/provider fixes, Desktop, release, dependencies and session schema. |
| Claimed At | Not applicable |
| Source Issue | #298 / #310 / #334 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer requested development and closure of all three Issues on 2026-09-20; effective target-branch claim still required. |
| Implementation PR | Not started |
| Last Updated | 2026-09-20 |
| Handoff / Release Condition | Finalize atomic claim on main before committed implementation; converge all three requirements locally before implementation PR. |

## Published Baseline

### Inventory And Selection

- I277 remains Review with human/device acceptance Deferred; no Desktop authority is transferred.
- I249 remains Planned/Unclaimed and unselected; I164 remains Paused/superseded.
- I278 is Complete/Closed. No other Active or Blocked iteration header was found; I162's
  Complete / Review outcome is terminal, not an active review claim.
- GitHub open-PR inventory on 2026-09-20 returned none. Refresh before claim submission/merge.
- Existing uncommitted Auto diagnostics, byte-limit and timing changes remain separately owned
  by PERM-007-F. Preserve them without including them in this implementation candidate.
- Existing uncommitted thinking animation work is preserved; its exact integration/disposition
  must be declared before candidate submission rather than silently mixed into this baseline.

### Selected Stories And Execution Order

1. TUI-061 / #334: shared blank three-column continuation prefix for ordinary and reasoning
   history, Unicode-aware wrapping and resize. Keep logical selection offsets independent of
   synthetic prefix cells.
2. TUI-056 / #298: typed display-safe reasoning entry, independent title, collapsed by default,
   click-without-drag toggles only that entry. Expanded body starts beneath the title.
3. TUI-057 / #310: independent live title with total width-aware body display-row count; newest
   body rows roll beneath it. Title does not consume the accepted body-row allowance.

### Presentation Decisions

- Collapse state belongs to the current TUI presentation only, keyed by stable transcript entry.
  Resume initializes collapsed state; no session schema or stored payload changes.
- Preserve only existing displayable reasoning text; signatures/redacted payloads never render.
- Default copy/export safety and explicit include-thinking export remain unchanged. Synthetic
  prefixes and status counters are presentation, not new transcript payload.
- Header toggles preserve FollowTail or logical anchored position. Drag selection never toggles,
  and click toggles leave no phantom selection. Test resize, scrolling and transcript growth.
- Count total body display rows from the same layout plan used to render, not received newline
  counts. Resize may increase/decrease counts; the visible rolling window is a separate bound.
- Tool titles are per invocation, identified by structured call identity. Never infer progress,
  retry, timeout or execution from text placeholders. Inventory available typed events before
  implementation; absent lifecycle facts must be explicitly documented, never fabricated.
- Live and completed reasoning share a title/body hierarchy but retain independent states.
- Keyboard/accessibility remains an explicit refinement check: first-slice #298 excludes new
  keyboard navigation, but existing keyboard scrolling/selection must remain intact.

### Non-Goals

- No permission/provider policy, retry policy, tool execution order, storage, dependency or
  release changes. No unrelated renderer redesign or hidden-reasoning exposure.
- Do not treat prior live-preview acceptance as acceptance of new history folding.

### Validation And Acceptance

- Focused transcript/projection tests: ASCII/CJK, wide/narrow/reflow, synthetic prefix mapping,
  multiple independent reasoning entries, resume defaults and filtered payload sentinels.
- Mouse tests: header click versus drag, hit testing after resize/PageUp/transcript growth,
  FollowTail and anchored stability, copying and exports unchanged.
- Buffer/state tests: independent live title, total counts versus rolling body, tiny terminals,
  structured tool states and no compatibility placeholder leakage.
- Locked TUI/conversation/affected CLI tests; pinned-toolchain Clippy; required workspace
  preflight before stable code submission. Governance-only changes use governance validation.
- Native-terminal acceptance covers thinking folding, ordinary/reasoning continuation padding,
  CJK resize, click/drag, scroll anchors and live thinking/tool counts. Record actual evidence;
  unavailable acceptance stays Review, not Complete.
- Independent Agent technical review, exact-head applicable CI and merge-time CAS before merge.
- Owner-first closeout cites already-existing implementation evidence and synchronizes all three
  Issues only after their full acceptance is established.

### Documentation Targets And Rollback

Update README TUI guidance, the three Story owners, iteration index and Board after owner facts.
Rollback removes presentation changes without migrating or altering persisted conversations.

## Execution Evidence

- 2026-09-20: read all three remote Issue bodies/comments and local owners. Confirmed existing
  history projection wraps at column zero and transcript lacks a typed reasoning block.
- Planning only: claim, implementation, tests and native acceptance remain pending.
