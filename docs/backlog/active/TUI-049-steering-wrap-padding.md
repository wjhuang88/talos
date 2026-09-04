# TUI-049: Steering Wrap Respects Shared Horizontal Padding

**Status**: Complete / Closed (proposed closeout)

| Field | Value |
|---|---|
| Story ID | TUI-049 |
| Type | TUI / Layout Correctness Story |
| Priority | P1 |
| Status | Complete / Closed |
| Source | [GitHub Issue #267](https://github.com/wjhuang88/talos/issues/267) |
| Selected Iteration | I207 |
| Depends On | Shared scrollback/composer width and padding conventions |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I207 / TUI-049 only: preserve shared left/right display padding for wrapped steering rows, including narrow and Unicode-width cases, with focused tests and terminal evidence. Excludes steering timing/custody, history wrapping, theme, selection, release and CAP-001 text seam work. |
| Claimed At | 2026-09-04 |
| Source Issue | #267 |
| Governance Claim PR | #482 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested serial execution of I207, I208 and I246 on 2026-09-04; independent review remains required for the implementation candidate. |
| Implementation PR | #483 |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | None; I207 is complete. I208 remains separately governed and unclaimed. |

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

## Closeout Checkpoint (2026-09-04)

Implementation PR #483 merged as `ca3b2fa7ffb1ca14b82d1acf6af6be147368e6fe` after exact-head
CI `33861265618` and independent APPROVE comment `5539172892` on source head
`0998b10581867dca92c2b37919825f8d72134766`. A maintainer-run narrowed-terminal trace then showed
the long mixed Chinese/ASCII queued steering entry wrapping with left continuation padding,
right-edge clearance and no display-width overflow.
The Complete / Closed transition is proposed and remains ineffective until the closeout PR reaches
`main`.

## Completion Evidence

- Completion Commit: `ca3b2fa7ffb1ca14b82d1acf6af6be147368e6fe`.
- The closeout status commit does not certify its own implementation.
