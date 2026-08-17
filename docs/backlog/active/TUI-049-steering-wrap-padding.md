# TUI-049: Steering Wrap Respects Shared Horizontal Padding

| Field | Value |
|---|---|
| Story ID | TUI-049 |
| Type | TUI / Layout Correctness Story |
| Priority | P1 |
| Status | Planned / Unclaimed |
| Source | [GitHub Issue #267](https://github.com/wjhuang88/talos/issues/267) |
| Selected Iteration | I207 |
| Depends On | Shared scrollback/composer width and padding conventions |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #267 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Establish an effective claim and bind the width contract to the shared production allocator. |

## Identity / Goal / Value

Long steering content must wrap inside the same public left and right padding used by the rest of
the conversation surface, so continuation lines never touch the terminal edge.

## Scope

- Identify the authoritative display-width allocator for steering rows and continuation lines.
- Apply both shared horizontal padding constraints, including narrow and Unicode-width inputs.
- Preserve existing prefixes, styles, cursor behavior and scrollback ownership.

## Exclusions

No global theme redesign, terminal resize policy change, selection semantics change, or unrelated
history/tool-output wrapping.

## Acceptance For Future Implementation

- Given steering text wider than the available row, every wrapped line remains within the shared
  left/right padding contract.
- Exact-boundary, one-column-too-wide, narrow-terminal, ASCII and CJK cases render without edge
  contact, truncation or overflow.
- Existing composer and history layout tests remain green, with a real-terminal narrow-width trace.

## Required Reads

- `docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`
- `docs/backlog/active/TUI-032-composer-multiline-wrap.md`
- `docs/backlog/active/TUI-035-narrow-viewport-and-resize-rendering-robustness.md`
- `docs/iterations/I142-composer-multiline-wrap.md`
