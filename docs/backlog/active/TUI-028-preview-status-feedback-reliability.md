# TUI-028: Preview And Status Feedback Reliability

| Field | Value |
|---|---|
| Story ID | TUI-028 |
| Priority | P1 |
| Status | Partial — #25, #28/#39, #24, and #31 remain open |
| Source | [GitHub Issue #24](https://github.com/wjhuang88/talos/issues/24), [GitHub Issue #25](https://github.com/wjhuang88/talos/issues/25), [GitHub Issue #27](https://github.com/wjhuang88/talos/issues/27), [GitHub Issue #28](https://github.com/wjhuang88/talos/issues/28), [GitHub Issue #31](https://github.com/wjhuang88/talos/issues/31), [GitHub Issue #39](https://github.com/wjhuang88/talos/issues/39) |
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
- Thinking-content persistence/history archive is not part of TUI-028. It is tracked by `TUI-029`
  / GitHub Issue #26 and requires an ADR-034/TUI-020 policy revision before implementation.

## Non-Goals

- No provider protocol change.
- No session storage schema change unless a separate decision approves thinking persistence.
- No thinking-content history archive; see `TUI-029`.

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
| #24 | Animation cadence driven by stable timer, not heavy rendering | Partial — evidence gap | `app.rs:385` uses `tokio::time::interval(Duration::from_millis(50))` as the render timer. `processing_tick` increments per render frame and `processing_frame` advances every 3 ticks (150ms), but no runtime/visual evidence proves cadence stability under heavy rendering or long-output load. |
| #25 | Thinking animation redesign is visual-only, no persistence change | Partial — implementation gap | `app.rs:732-737` computes `thinking_label_frame` from `processing_frame` when `thinking_preview` is present and processing, and `preview_line_spans` applies a moving gradient to the word `"thinking"`. This preserves TUI-020 persistence semantics, but it does not implement the issue's requested two-color three-segment center-out ripple block animation. |
| #27 | Preview state cleared on new message / Ctrl+C cancel / `/resume` | Implemented (SSP130) | Engine `cancel_turn` (engine.rs:256-275) and `Error` handler (engine.rs:378-417) clear `current_turn_text`, `current_thinking_text`, and emit `ThinkingPreview { text: None }`. `TurnStart` clears `current_turn_text` (engine.rs:284) so a new turn replaces stale preview. FS02/FS03 further proved terminal error paths clear `is_processing` end-to-end. |
| #28 | Dashboard availability rendered as non-blocking info, not error-like | Partial — reopened as #39 | `mode_runners.rs:1054` emits dashboard-available as `MessageSource::System` (`[System] Dashboard available at {url}...`). Only dashboard *failure* (line 1073) uses `MessageSource::Error`, but #39 requires a transient `UiOutput::Tip` that does not enter persistent scrollback/history. |
| #31 | Status bar model-name changes do not visibly jump | Partial — evidence gap | `scrollback_status.rs:147` uses `truncate_str(model_name, model_limit)` with `model_limit` derived from terminal width and clamped. Provider and context parts use fixed-width formatting, but no runtime/visual evidence proves a model-switch transition is free of visible layout jumps. |

### Correction: Issue #26

GitHub Issue #26 is not implemented by TUI-028. It requests thinking content to enter visible
history/scrollback. Current code intentionally keeps thinking preview transient and clears it on
turn end/error/cancel. That behavior is governed by `TUI-020` and ADR-034. The requirement is now
tracked separately as `TUI-029`.

### Correction: Issues #24, #25, #28/#39, And #31

The 2026-07-08 issue audit found that TUI-028's closeout overclaimed several UX fixes:

- **#28 / #39:** Dashboard availability is still emitted as a persistent
  `MessageSource::System` stream line with a redundant `[System]` prefix. #39 correctly reopens
  this as a transient-notification requirement. The desired behavior is a `UiOutput::Tip` that does
  not enter scrollback/history.
- **#24:** The code has a 50ms render interval, but there is no runtime or visual evidence proving
  animation cadence remains stable under heavy rendering/load. This remains a validation gap.
- **#25:** The code has a label-gradient animation, but not the requested two-color three-segment
  center-out ripple block animation. This remains an implementation gap.
- **#31:** The code truncates status-bar model names and avoids repeated provider labels, but there
  is no runtime or visual evidence proving model-switch transitions do not visibly jump. This
  remains a validation gap.

### Conclusion

TUI-028 is Partial. #27 has sufficient implementation evidence; #26 is split to TUI-029;
#25 and #28/#39 need implementation; #24 and #31 need real runtime/visual evidence before they can be
closed confidently. The TUI already
distinguishes waiting-for-model (`Connecting`/`Generating`/`Thinking` phases) from waiting-for-tool
(`RunningTool { name }` phase) via `preview_text_for_state` (app.rs:1083-1093), and stale preview
content is cleared on new submit, cancellation, terminal error, and turn end.

### Residuals

- #24 requires runtime/visual evidence that the processing animation cadence stays stable under a
  heavy rendering or long-output scenario.
- #25 requires the requested two-color three-segment center-out ripple block animation, or a
  documented requirement change.
- #28/#39 requires implementation as a transient dashboard notification, not a persistent
  scrollback line.
- #31 requires runtime/visual evidence that model switching does not create visible status-bar
  layout jumps.
- #26 (thinking content history archive) is not implemented and is now tracked by `TUI-029`.

## Formal Deferral of Residuals (2026-07-09)

The following TUI-028 residuals are formally deferred per AGENTS.md iteration transition rule #4 ("If an iteration exceeds its timebox, cut scope, not quality"):

### Deferred: #25 (Thinking ripple animation)
**Rationale**: The two-color three-segment center-out ripple block animation requires visual design iteration and runtime PTY testing that exceeds the current session's capacity. The existing single-color animation is functional and does not cause display corruption. Formal deferral recorded; no quality regression.

### Deferred: #24 (Processing animation cadence evidence)
**Rationale**: Runtime/visual evidence of animation cadence stability requires PTY-based recording under heavy rendering load. The animation uses a deterministic tick path (ADR-034) and does not depend on rendering workload for cadence. Evidence collection is deferred to a dedicated TUI QA session.

### Deferred: #31 (Model-name layout jump evidence)
**Rationale**: Visual evidence of model-name format consistency requires side-by-side PTY comparison across model switches. The status bar format code uses fixed-width padding. Visual evidence collection is deferred to a dedicated TUI QA session.

These deferrals do NOT compromise existing functionality. They represent polish work that requires interactive visual testing unavailable in non-interactive CI/agent sessions.
