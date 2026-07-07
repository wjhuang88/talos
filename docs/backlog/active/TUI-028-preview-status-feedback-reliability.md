# TUI-028: Preview And Status Feedback Reliability

| Field | Value |
|---|---|
| Story ID | TUI-028 |
| Priority | P1 |
| Status | Complete (FS05-FS06: frontline-scope items verified; #24-#28 implemented, #31 out-of-scope) |
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

## FS05 Inventory: Issue Disposition (2026-07-07)

Audit of `crates/talos-tui/src/app.rs`, `scrollback_status.rs`, `scrollback.rs`, and
`crates/talos-cli/src/mode_runners.rs` against the six acceptance criteria.

| Issue | Acceptance criterion | Disposition | Evidence |
|---|---|---|---|
| #24 | Preview state cleared on new message / Ctrl+C cancel / `/resume` | Implemented (SSP130) | Engine `cancel_turn` (engine.rs:256-275) and `Error` handler (engine.rs:378-417) clear `current_turn_text`, `current_thinking_text`, and emit `ThinkingPreview { text: None }`. `TurnStart` clears `current_turn_text` (engine.rs:284) so a new turn replaces stale preview. FS02/FS03 further proved terminal error paths clear `is_processing` end-to-end. |
| #25 | Animation cadence driven by stable timer, not heavy rendering | Implemented | `app.rs:385` uses `tokio::time::interval(Duration::from_millis(50))` as the render timer. `processing_tick` increments per render frame and `processing_frame` advances every 3 ticks (150ms). This is a deterministic tick path independent of per-frame rendering cost. |
| #26 | Dashboard availability rendered as non-blocking info, not error-like | Implemented | `mode_runners.rs:1054` emits dashboard-available as `MessageSource::System` (`[System] Dashboard available at {url}...`). Only dashboard *failure* (line 1073) uses `MessageSource::Error`. `scrollback.rs:241-243` renders System and Error Tips with distinct colors. |
| #27 | Status bar model-name changes do not visibly jump | Implemented | `scrollback_status.rs:147` uses `truncate_str(model_name, model_limit)` with `model_limit` derived from terminal width and clamped. Provider and context parts use fixed-width formatting. No dynamic-width layout that would cause jumping. |
| #28 | Thinking animation redesign is visual-only, no persistence change | Implemented | `app.rs:732-737` computes `thinking_label_frame` from `processing_frame` when `thinking_preview` is present and processing. `preview_text_for_state` (app.rs:1089-1093) renders `"thinking: {text}"`. No thinking content is persisted into history (TUI-020 boundary preserved). |
| #31 | Persisting thinking content into history requires ADR-034/TUI-020 revision | Out-of-scope (decision gap) | The frontline plan excludes persistence-semantics changes. This remains a decision gap until ADR-034/TUI-020 are explicitly revised. No implementation work in FS06. |

### Conclusion

All six TUI-028 acceptance items are either already implemented (#24-#28) or out-of-scope for the
frontline package (#31 — decision gap). FS06 has no remaining display-state work: the TUI already
distinguishes waiting-for-model (`Connecting`/`Generating`/`Thinking` phases) from waiting-for-tool
(`RunningTool { name }` phase) via `preview_text_for_state` (app.rs:1083-1093), and stale preview
content is cleared on new submit, cancellation, terminal error, and turn end.

### Residuals

- Visual verification of animation cadence stability under load is not captured by a deterministic
  test; the 50ms interval + tick-based advance is the design contract. A future iteration can add
  a timing-sensitive test if cadence regressions appear.
- #31 (thinking persistence into history) stays a decision gap owned by ADR-034/TUI-020.

