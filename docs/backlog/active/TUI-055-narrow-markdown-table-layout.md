# TUI-055: Narrow Markdown Table Layout Integrity

| Field | Value |
|---|---|
| Story ID | TUI-055 |
| Type | TUI / Markdown Layout Story |
| Priority | P1 |
| Status | Intake / Unclaimed |
| Source | [GitHub Issue #280](https://github.com/wjhuang88/talos/issues/280) |
| Selected Iteration | None |
| Depends On | Existing Markdown table projection, Unicode display-cell width and resize reflow contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #280 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Characterize current Markdown table ownership and select one readable narrow-width strategy before preparing a runnable iteration and claim. |

## Identity / Goal / Value

Keep Markdown table row and column relationships readable when terminal width forces cell content to
wrap, including CJK, mixed-width text, emoji and long tokens.

## Scope

- Characterize current table parsing, column allocation, display-width wrapping and resize reflow.
- Decide the minimum useful grid width and one deterministic narrow-width degradation strategy.
- Preserve logical row height, aligned cell boundaries and immutable transcript source content.
- Define focused layout, repeated-resize and real-terminal acceptance matrices.

## Exclusions

- No Markdown source rewrite, persistence change, global line-wrap disable or unrelated code-block,
  list and paragraph redesign.
- No table implementation inside I209/PR #279; this intake registration grants no code authority.

## Acceptance For Intake

- [ ] Current parser/renderer ownership and width inputs are mapped.
- [ ] Grid wrapping and narrow fallback have deterministic display-cell invariants.
- [ ] CJK, emoji, combining marks, empty cells, inline formatting and long-token fixtures are defined.
- [ ] Wide-to-narrow, narrow-to-wide and streaming-adjacent resize cases are defined.
- [ ] One runnable iteration, effective claim and real-terminal matrix exist before implementation.

## Required Reads

- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`
- `crates/talos-tui/src/scrollback_markdown.rs`
- `crates/talos-tui/src/history_projection.rs`
- `crates/talos-tui/src/stream_markdown.rs`
