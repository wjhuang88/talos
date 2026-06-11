# I023: TUI State Model — Unified Messages, Tips, and Event-Bus Hook

**User can**: See every message with its lifecycle visible (pending → accepted → streaming → completed), see transient tips auto-expire from the status bar instead of polluting scrollback, and have a structured state model that a future global event bus can subscribe to without touching TUI internals.

## Status: REVIEW (2026-06-11)

Event-driven architecture with `talos-conversation` crate. Codex-style `insert_history` rewrite. Stream-based content delivery with styled scrollback (user messages have Nord bg color with top/bottom padding). 3-char ASCII line padding system. Animated braille spinner with Nord gradient. Native cursor sync to input box. 43 TUI + 50 conversation tests pass.

## Stories

| Story | Title | Acceptance | Status |
|---|---|---|---|
| S1 | Define new data types | `MessageRole`, `MessageStatus`, `ChatMessage`, `TipKind`, `Tip`, `TuiStateEvent` defined in `state.rs`. All derive `Debug, Clone`. No `ChatLine` or `status_message` references remain in the type definitions. | ✅ Complete |
| S2 | Restructure `TuiState` | `TuiState` uses `messages: Vec<ChatMessage>`, `tip: Option<Tip>`, `event_tx: Option<mpsc::UnboundedSender<TuiStateEvent>>`. `chat_lines`, `status_message` fields removed. `current_turn_text` retained for streaming. | ✅ Complete |
| S3 | Migrate state methods | `handle_event`, `handle_ctrl_c`, `append_user_message`, `append_error`, `append_system`, `append_tool_call`, `set_tool_result`, `expire_tip`, slash commands all use new types. `emit_event` helper on `TuiState`. Event emissions on status transitions when `event_tx` is set. | ✅ Complete |
| S4 | Migrate app.rs | `flush_scrollback`, `extract_new_scrollback_lines`, `finalize_scrollback` use `messages` instead of `chat_lines`. `build_status_text` uses Nord-styled Span-based layout (no background, dim separators, `S:/F:` queue indicators). `build_input_text` has `❯` prompt (Nord Aurora green). `draw_frame` uses `ViewportLayout` struct with named fields. `InlineTerminal::new(viewport_height)` parameterized. | ✅ Complete |
| S5 | Migrate tests | All 131 tests pass. `ChatLine` references replaced with `ChatMessage`. `status_message` assertions replaced with `Tip`. New tests: `test_tip_auto_expires`, `test_tip_does_not_expire_before_ttl`, `test_emit_event_no_tx_is_noop`, `test_message_roles_are_correct`. | ✅ Complete |
| S6 | Verification | `cargo check --workspace` clean. `cargo test --workspace` exit 0. Runtime verified: messages appear in scrollback with blank-line spacing, tips auto-expire, input area with bg-color style, status bar no background. | ✅ Complete |
| S7 | Event-driven architecture | `talos-conversation` crate: `ConversationEngine` owns business state, emits `UiOutput` via async channels. `talos-tui` owns pure UI state. Single-directional flow: Agent → Engine → UI. Stream-based content delivery (`StreamMessage`). `select!` loop consumes streams directly (no spawn task). | ✅ Complete |
| S8 | Codex-style insert_history | Two-branch `insert_history`: non-bottom (`\x1bM` push viewport) and bottom (scroll region + `\r\n`). Single-line operation. `needs_clear` after each insert for clean viewport redraw. `streaming_preview` unconditionally synced with `stream_buffer`. | ✅ Complete |
| S9 | Line padding and styled scrollback | 3-char ASCII padding by message type (` > ` user, ` ~ ` assistant/tool, ` # ` system, ` ! ` error). User messages rendered with Nord Polar Night background (`#3B4252`) and top/bottom padding rows. `ScrollbackLine` carries `text` + optional `bg`. `insert_history` accepts `bg: Option<Color>`, pads lines to full terminal width. Stream separator (blank line) between non-first streams. | ✅ Complete |
| S10 | Animated spinner and cursor sync | 2-char braille spinner with 10-frame animation and Nord color gradient cycling (150ms/frame). Native terminal cursor synced to input box position after each render via `MoveTo` + `Show`. `set_cursor` method on `InlineTerminal`. `restore()` clears viewport content before exiting. | ✅ Complete |

## Execution Evidence

- 93 tests pass (43 TUI + 50 conversation).
- `talos-conversation` crate: `ConversationEngine` owns all business state, 50 tests.
- `talos-tui` crate: event-driven UI with pure state, 43 tests.
- Single-directional information flow: Agent → ConversationEngine → UI via typed async channels (`mpsc::UiOutput`).
- Stream-based content delivery: UI consumes active stream via `next_stream_chunk` in `select!` loop (no spawn task).
- `insert_history` rewritten Codex-style: two branches (non-bottom: `\x1bM` push viewport; bottom: scroll region + `\r\n`), single-line operation, `needs_clear` for clean redraw.
- `insert_history` accepts `bg: Option<Color>` for styled scrollback — lines padded to full terminal width with `SetBackgroundColor`.
- `ScrollbackLine` struct carries `text: String` + `bg: Option<Color>`, enabling per-line background styling.
- User messages rendered with Nord Polar Night background (`#3B4252`) and top/bottom padding rows for visual grouping.
- 3-char ASCII line padding system: ` > ` (user), ` ~ ` (assistant/tool first), `   ` (continuation), ` # ` (system), ` ! ` (error).
- Non-first streams separated by a blank line.
- Animated 2-char braille spinner with Nord color gradient (10 frames, 150ms/frame) in preview component.
- Native terminal cursor synced to input box position after each render (`MoveTo` + `Show`).
- `restore()` clears viewport content (`MoveTo` + `Clear(FromCursorDown)`) before disabling raw mode.
- Commits: `5c90874` (event-driven architecture), `a669a3e` (insert_history rewrite + preview sync fix), `988fc82` (line padding + error tips), `fc370ce` (animated spinner).

## Dependencies

- I022 core flip (landed) — inline-by-default viewport model is prerequisite.

## Decision Gate

Follow `docs/backlog/active/TUI-004-state-model.md` design. Any deviation from the
`ChatMessage`/`Tip`/`TuiStateEvent` model requires updating that backlog item.

Required reading:

- `docs/backlog/active/TUI-004-state-model.md` — full design, acceptance criteria, risks.
- `crates/talos-conversation/src/engine.rs` — ConversationEngine, handle_agent_event, handle_user_message.
- `crates/talos-conversation/src/types.rs` — StatusSnapshot, StreamMessage, UiOutput, MessageSource.
- `crates/talos-tui/src/app.rs` — Tui struct, select! loop, consume_stream_chunk, handle_ui_output, PreviewComponent.
- `crates/talos-tui/src/inline_terminal.rs` — InlineTerminal, insert_history (Codex-style), set_viewport_area, draw/draw_inner.
- `crates/talos-tui/src/state.rs` — TuiState (pure UI state).

## Baseline

- 127 tests pass (`talos-tui`) at start.
- Workspace `cargo test` and `cargo check` exit 0.
- I022 core flip landed; viewport is fixed 4 lines, scrollback flush works.
