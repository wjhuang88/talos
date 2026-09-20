# TUI-056: Collapsible Reasoning History

> Document status: Complete

| Field | Value |
|---|---|
| Story ID | TUI-056 |
| Type | TUI / History Interaction Story |
| Priority | P2 |
| Status | Complete |
| Source | [GitHub Issue #298](https://github.com/wjhuang88/talos/issues/298) |
| Selected Iteration | I279 |
| Depends On | TUI-029 reasoning history archive; ADR-034 reasoning boundary; ADR-054 renderer |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I279 presentation-only delivery of this Story; no permission, provider, persistence or execution changes. |
| Claimed At | 2026-09-20 |
| Source Issue | #298 |
| Governance Claim PR | #579 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested all three Issues on 2026-09-20 under standing single-maintainer mode; independent Agent review and exact-head checks required before merge. |
| Implementation PR | #580 |
| Last Updated | 2026-09-20 |
| Handoff / Release Condition | Complete through #580; I279 final delivery ledger records passing acceptance, exact-head CI and independent Agent review. |

## Completion Evidence (2026-09-20)

Completion Commit: 52276f552496b160e064fd54ed7471bc84d3f559

PR #580 delivered this Story on main. Full local release preflight and all six
exact-head CI checks (35502971132) passed; independent Agent review 5749013208
and merge-time CAS 5749088682 bind the implementation candidate. Maintainer native
acceptance passed, including the final completed-tool-body dedup correction.
See [I279 final delivery ledger](../../iterations/I279-tui-history-and-live-activity.md)
for requirement-specific tests, observations, identity limits and ADR-080 migration.
No remaining acceptance in this Story. The original intake and dated selection
below are preserved as historical provenance, not unfulfilled activation gates.

## Identity / Goal / Value

Make completed reasoning history compact by default while preserving an explicit, discoverable way
to inspect the archived reasoning body when needed.

## Proposed Scope

- Keep `thinking` as an independent title row in completed history.
- Default the completed reasoning body to a collapsed state.
- Expand or collapse the body through an explicit mouse interaction on the title row.
- Render expanded content below the title rather than as a same-line label.
- Preserve the existing display-safe reasoning archive and answer/tool ordering.

## Required Decisions Before Ready

- Define whether collapse state is projection-only, session-local or persisted; do not infer a
  persistence change from the interaction request.
- Define keyboard and accessibility parity for the mouse interaction.
- Define selection, copy, export, resize and resume behavior for collapsed and expanded states.
- Confirm the change is compatible with ADR-034 or record the required decision revision first.

## Exclusions

- No change to the live transient thinking preview owned by TUI-041/I199.
- No provider protocol, reasoning persistence, session schema or default export change.
- No iteration selection, claim or implementation authorization from this intake record.

## Acceptance For Refinement

- [ ] Collapse-state ownership and lifecycle are explicit.
- [ ] Mouse, keyboard, accessibility, selection and copy behavior are testable.
- [ ] History ordering, Markdown projection, resize and resume regression cases are defined.
- [ ] ADR-034 compatibility is confirmed or a decision update is prepared.
- [ ] One runnable iteration and effective Collaboration Claim exist before implementation.

## Required Reads

- `docs/backlog/active/TUI-029-thinking-history-archive.md`
- `docs/backlog/active/TUI-041-thinking-preview-wrap-and-height.md`
- `docs/decisions/034-reasoning-thinking-boundary.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`

## 2026-09-20 Unified Delivery Selection

I279 selects #298/#310/#334 as one locally converged TUI stage. PR #579 proposes this claim;
no implementation authority exists before merge. Historical intake text above remains provenance;
I279's explicit projection-only, per-entry, line-count and acceptance decisions govern execution.
Existing I277 deferred acceptance, I249 Planned and I164 Paused dispositions are preserved.
