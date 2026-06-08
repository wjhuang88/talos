# I023: TUI State Model — Unified Messages, Tips, and Event-Bus Hook

**User can**: See every message with its lifecycle visible (pending → accepted → streaming → completed), see transient tips auto-expire from the status bar instead of polluting scrollback, and have a structured state model that a future global event bus can subscribe to without touching TUI internals.

## Status: PLANNED

Next iteration after I022. Depends on I022 core flip (landed).

## Stories

| Story | Title | Acceptance |
|---|---|---|
| S1 | Define new data types | `MessageRole`, `MessageStatus`, `ChatMessage`, `TipKind`, `Tip`, `TuiStateEvent` defined in `state.rs`. All derive `Debug, Clone`. No `ChatLine` or `status_message` references remain in the type definitions. |
| S2 | Restructure `TuiState` | `TuiState` uses `messages: Vec<ChatMessage>`, `tips: Vec<Tip>`, `streaming_text`, `streaming_scrolled`, `event_tx: Option<mpsc::UnboundedSender<TuiStateEvent>>`. `chat_lines`, `current_turn_text`, `status_message` fields removed. |
| S3 | Migrate state methods | `handle_event`, `handle_ctrl_c`, `append_user_message`, `append_error`, `append_system`, `append_tool_call`, `set_tool_result`, `expire_tips`, `finalize_turn` all use new types. Event emissions on status transitions when `event_tx` is set. |
| S4 | Migrate app.rs | `flush_scrollback`, `extract_new_scrollback_lines`, `finalize_scrollback` use `messages` instead of `chat_lines`. `build_status_text` renders active `Tip`. `handle_input_event` uses `ChatMessage { role: User, status: Pending/Accepted }` for queued/sent messages. `chat_line_to_text_lines` → `message_to_text_lines`. |
| S5 | Migrate tests | All 127+ tests pass. `ChatLine` references replaced with `ChatMessage`. `status_message` assertions replaced with `Tip`. New tests for `Tip` expiry and `TuiStateEvent` emission. |
| S6 | Verification | `cargo check --workspace` clean (0 warnings). `cargo test --workspace` exit 0. `cargo run -p talos-cli` shows correct behavior: messages appear in scrollback, tips auto-expire from status bar, no duplicate content, no rendering glitches. |

## Dependencies

- I022 core flip (landed) — inline-by-default viewport model is prerequisite.

## Decision Gate

Follow `docs/backlog/active/TUI-004-state-model.md` design. Any deviation from the
`ChatMessage`/`Tip`/`TuiStateEvent` model requires updating that backlog item.

Required reading:

- `docs/backlog/active/TUI-004-state-model.md` — full design, acceptance criteria, risks.
- `crates/talos-tui/src/state.rs` — current `TuiState`, `ChatLine`, to be replaced.
- `crates/talos-tui/src/app.rs` — scrollback flush, status bar, input handling, to be migrated.
- `docs/iterations/I022-tui-inline-default.md` — prerequisite iteration record.

## Baseline

- 127 tests pass (`talos-tui`).
- Workspace `cargo test` and `cargo check` exit 0.
- I022 core flip landed; viewport is fixed 4 lines, scrollback flush works.