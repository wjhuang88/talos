# TUI-028: Preview And Status Feedback Reliability

| Field | Value |
|---|---|
| Story ID | TUI-028 |
| Priority | P1 |
| Status | In Progress (SSP130: stale preview clear complete) |
| Source | [GitHub Issue #24](https://github.com/wjhuang88/talos/issues/24), [GitHub Issue #25](https://github.com/wjhuang88/talos/issues/25), [GitHub Issue #26](https://github.com/wjhuang88/talos/issues/26), [GitHub Issue #27](https://github.com/wjhuang88/talos/issues/27), [GitHub Issue #28](https://github.com/wjhuang88/talos/issues/28), [GitHub Issue #31](https://github.com/wjhuang88/talos/issues/31) |
| Depends On | `TUI-027`, `TUI-020`, `TUI-024`, `RUNTIME-002` |

## Problem

The preview/status area does not always communicate state clearly. Reported issues include unstable
processing animation cadence, stale preview content after cancellation, dashboard info text that
looks like an error, model-name layout jumps, and thinking display follow-ups.

## Acceptance

- Preview state is cleared when a new user message is committed to history, including after
  Ctrl+C cancellation and `/resume`.
- Processing/thinking animation cadence is driven by a stable timer or equivalent deterministic
  tick path and does not depend on heavy rendering work.
- Dashboard availability is rendered as non-blocking info, not as an error-like line.
- Status bar model-name changes do not visibly jump because of inconsistent formatting.
- Thinking animation redesign is a visual-only slice and must not change persistence semantics.
- Persisting thinking content into history is not implemented unless ADR-034/TUI-020 are explicitly
  revised; until then it remains a decision gap.

## Non-Goals

- No provider protocol change.
- No session storage schema change unless a separate decision approves thinking persistence.

## Required Reads

- `docs/backlog/active/TUI-027-preview-render-order.md`
- `docs/backlog/active/TUI-020-thinking-preview-not-history.md`
- `docs/backlog/active/TUI-024-thinking-title-in-preview.md`
- `docs/decisions/034-reasoning-thinking-boundary.md`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/scrollback_status.rs`

