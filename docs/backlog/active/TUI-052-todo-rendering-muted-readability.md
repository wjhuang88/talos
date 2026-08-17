# TUI-052: Todo Rendering And Muted-Text Readability

| Field | Value |
|---|---|
| Story ID | TUI-052 |
| Type | TUI / Presentation Epic |
| Priority | P1 |
| Status | Intake / Unclaimed |
| Source Issue | #266 |
| Selected Iteration | None |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #266 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Decompose Todo coalescing, in-progress styling and TUI-wide muted-text inventory into non-overlapping runnable children before selection. |

## Identity / Goal / Value

Reduce redundant consecutive Todo snapshots, make the current Todo item scannable, and replace
unreadable routine `DIM` styling with a coherent semantic muted-text treatment.

## Intake Scope

- Preserve operation/session facts while defining presentation-only Todo coalescing boundaries.
- Define semantic, non-color-only in-progress Todo styling.
- Inventory TUI-wide `DIM` use and separate routinely read secondary text from decorative content.

## Exclusions

No Todo persistence/mutation/batch API change, theme-engine redesign or implementation authority.

## Acceptance For Intake

- [ ] Separate runnable children and overlap with existing Todo/TUI owners are recorded.
- [ ] Coalescing boundaries, style contrast and two-terminal visual evidence are testable.
- [ ] Each implementation child receives its own effective claim before coding.
