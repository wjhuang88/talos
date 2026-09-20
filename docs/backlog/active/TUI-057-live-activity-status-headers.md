# TUI-057: Dynamic Live Activity Status Headers

| Field | Value |
|---|---|
| Story ID | TUI-057 |
| Type | TUI / Product Story |
| Priority | P2 |
| Status | Review — native presentation acceptance passed; I279 claim effective through #579 merge 9fa4ae52 |
| Source | [GitHub Issue #310](https://github.com/wjhuang88/talos/issues/310) |
| Selected Iteration | I279 |
| Depends On | TUI-041 live-thinking layout; TUI-043 placeholder suppression; typed tool lifecycle events; ADR-034; ADR-054 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I279 presentation-only delivery of this Story; no permission, provider, persistence or execution changes. |
| Claimed At | 2026-09-20 |
| Source Issue | #310 |
| Governance Claim PR | #579 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested all three Issues on 2026-09-20 under standing single-maintainer mode; independent Agent review and exact-head checks required before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-09-20 |
| Handoff / Release Condition | Claim effective through #579 merge 9fa4ae52. I279 owns local convergence, exact-head CI/review and native-terminal acceptance before closure. |

## Identity / Goal / Value

Give live reasoning and tool activity one coherent status-title hierarchy that communicates ongoing
progress without turning transient compatibility text into durable transcript content.

## Proposed Scope

- Keep one independent title/status row above each live activity body.
- Show a dynamically updated line count derived from the shared width-aware display-row plan.
- Keep the newest bounded body rows rolling below the title; the title is outside the body cap.
- Apply compatible hierarchy and styling to live thinking and structured tool activity while
  preserving separate reasoning and tool state machines.
- Drive tool status only from typed lifecycle events, never `Calling tools...` or
  `Calling tools…` compatibility text.

## Required Decisions Before Ready

- Define how total planned display-row count changes across resize/reflow and how that differs from
  the visible rolling-window size.
- Define aggregate versus per-call presentation for sequential or concurrent tools.
- Inventory which typed events can truthfully represent queued, running, approval-waiting,
  succeeded, failed, cancelled, retrying and timed-out states; unavailable facts must not be
  inferred in the renderer.
- Define live-to-finalized transition behavior without changing Markdown, tool history,
  transcript persistence, copy/export or session replay.
- Confirm how the visual title relates to TUI-056 completed-history folding without combining the
  two interactions into one implementation slice.

## Exclusions

- No provider protocol, permission, tool execution, retry-policy or persistence change.
- No exposure of hidden reasoning, signatures or redacted payloads.
- No replacement or expansion of TUI-043/I201 placeholder suppression.
- No implementation iteration, branch or authorization from this intake record.

## Acceptance For Refinement

- [ ] Counter and resize semantics are deterministic and testable.
- [ ] Thinking and tool state ownership is explicit, with typed-event availability inventoried.
- [ ] Narrow-width, CJK, rolling-window, FollowTail and anchored-history behavior is specified.
- [ ] Finalized Markdown/tool-history and persistence invariants have regression cases.
- [ ] TUI-056/#298 and PROVIDER-006/#278 dependencies are mapped without hidden scope transfer.
- [ ] One runnable iteration and effective Collaboration Claim exist before implementation.

## State / Status Owners

- Story status and refinement decisions: this file.
- Remote requirement state: GitHub Issue #310.
- Compact planning view: `docs/backlog/PRODUCT-BACKLOG.md`.
- Derived operating view: `docs/BOARD.md`.

## User-Facing Documentation

Update TUI user documentation only in a future implementation iteration when observable behavior
ships. This intake changes no runtime behavior.

## Required Reads

- `docs/backlog/active/TUI-041-thinking-preview-wrap-and-height.md`
- `docs/backlog/active/TUI-043-tool-placeholder-suppression.md`
- `docs/backlog/active/TUI-056-collapsible-reasoning-history.md`
- `docs/backlog/active/PROVIDER-006-bounded-retry-progress-contract.md`
- `docs/decisions/034-reasoning-thinking-boundary.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`

## 2026-09-20 Unified Delivery Selection

I279 selects #298/#310/#334 as one locally converged TUI stage. PR #579 proposes this claim;
no implementation authority exists before merge. Historical intake text above remains provenance;
I279's explicit projection-only, per-entry, line-count and acceptance decisions govern execution.
Existing I277 deferred acceptance, I249 Planned and I164 Paused dispositions are preserved.
