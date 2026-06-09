# I023: TUI State Model — Unified Messages, Tips, and Event-Bus Hook

**User can**: See every message with its lifecycle visible (pending → accepted → streaming → completed), see transient tips auto-expire from the status bar instead of polluting scrollback, and have a structured state model that a future global event bus can subscribe to without touching TUI internals.

## Status: REVIEW (2026-06-09)

All stories implemented. 131 tests pass (`talos-tui`). Viewport layout refactored to `ViewportLayout` struct with auto-computed height.

## Stories

| Story | Title | Acceptance | Status |
|---|---|---|---|
| S1 | Define new data types | `MessageRole`, `MessageStatus`, `ChatMessage`, `TipKind`, `Tip`, `TuiStateEvent` defined in `state.rs`. All derive `Debug, Clone`. No `ChatLine` or `status_message` references remain in the type definitions. | ✅ Complete |
| S2 | Restructure `TuiState` | `TuiState` uses `messages: Vec<ChatMessage>`, `tip: Option<Tip>`, `event_tx: Option<mpsc::UnboundedSender<TuiStateEvent>>`. `chat_lines`, `status_message` fields removed. `current_turn_text` retained for streaming. | ✅ Complete |
| S3 | Migrate state methods | `handle_event`, `handle_ctrl_c`, `append_user_message`, `append_error`, `append_system`, `append_tool_call`, `set_tool_result`, `expire_tip`, slash commands all use new types. `emit_event` helper on `TuiState`. Event emissions on status transitions when `event_tx` is set. | ✅ Complete |
| S4 | Migrate app.rs | `flush_scrollback`, `extract_new_scrollback_lines`, `finalize_scrollback` use `messages` instead of `chat_lines`. `build_status_text` uses Nord-styled Span-based layout (no background, dim separators, `S:/F:` queue indicators). `build_input_text` has `❯` prompt (Nord Aurora green). `draw_frame` uses `ViewportLayout` struct with named fields. `InlineTerminal::new(viewport_height)` parameterized. | ✅ Complete |
| S5 | Migrate tests | All 131 tests pass. `ChatLine` references replaced with `ChatMessage`. `status_message` assertions replaced with `Tip`. New tests: `test_tip_auto_expires`, `test_tip_does_not_expire_before_ttl`, `test_emit_event_no_tx_is_noop`, `test_message_roles_are_correct`. | ✅ Complete |
| S6 | Verification | `cargo check --workspace` clean. `cargo test --workspace` exit 0. Runtime verified: messages appear in scrollback with blank-line spacing, tips auto-expire, input area with bg-color style, status bar no background. | ✅ Complete |

## Execution Evidence

- 131 tests pass (`cargo test -p talos-tui --lib`).
- `ViewportLayout` struct with `ROWS` const array and auto-computed `HEIGHT` (6 lines): tips, gap, input_pad_top, input, input_pad_bot, status.
- `InlineTerminal::new(viewport_height)` takes dynamic height from `ViewportLayout::HEIGHT`.
- Input area: background color `#3B4252` (Nord Polar Night), no borders, `❯` prompt in `#A3BE8C` (Nord Aurora green).
- Tips: `TipKind`-driven colors (green/purple/red/cyan) displayed in hints row; status bar no longer shows tips.
- Status bar: no background color, Nord Snow gray `#81A1C1` values, dim `│` separators, `S:/F:` queue format.
- Messages separated by blank lines in scrollback (both `extract_new_scrollback_lines` and `finalize_scrollback`).
- `ChatLine` completely removed (no dead code).

## Dependencies

- I022 core flip (landed) — inline-by-default viewport model is prerequisite.

## Decision Gate

Follow `docs/backlog/active/TUI-004-state-model.md` design. Any deviation from the
`ChatMessage`/`Tip`/`TuiStateEvent` model requires updating that backlog item.

Required reading:

- `docs/backlog/active/TUI-004-state-model.md` — full design, acceptance criteria, risks.
- `crates/talos-tui/src/state.rs` — `TuiState`, `ChatMessage`, `Tip`, `TuiStateEvent`.
- `crates/talos-tui/src/app.rs` — `ViewportLayout`, `draw_frame`, `build_input_text`, `build_status_text`.
- `crates/talos-tui/src/inline_terminal.rs` — `InlineTerminal::new(viewport_height)`.

## Baseline

- 127 tests pass (`talos-tui`) at start.
- Workspace `cargo test` and `cargo check` exit 0.
- I022 core flip landed; viewport is fixed 4 lines, scrollback flush works.
