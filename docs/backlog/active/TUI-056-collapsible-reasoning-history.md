# TUI-056: Collapsible Reasoning History

| Field | Value |
|---|---|
| Story ID | TUI-056 |
| Type | TUI / History Interaction Story |
| Priority | P2 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #298](https://github.com/wjhuang88/talos/issues/298) |
| Selected Iteration | None |
| Depends On | TUI-029 reasoning history archive; ADR-034 reasoning boundary; ADR-054 renderer |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #298 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | Decide the interaction and archive-projection boundary, then prepare a separate runnable iteration and effective claim. I199/TUI-041 grants no implementation authority. |

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
