# I023: TUI State Model — Unified Messages, Tips, and Event-Bus Hook

**User can**: See every message with its lifecycle visible (pending → accepted → streaming → completed), see transient tips auto-expire from the status bar instead of polluting scrollback, and have a structured state model that a future global event bus can subscribe to without touching TUI internals.

## Status: REVIEW (2026-06-11)

Event-driven architecture with `talos-conversation` crate. Codex-style `insert_history` rewrite. Stream-based content delivery with styled scrollback (user messages have Nord bg color with top/bottom padding). 3-column line padding system. Multiline pasted input stays a single user block. Single-row preview stays stable while the TUI holds Markdown tables/code fences/lists/quotes through a stream block classifier. Animated braille spinner with Nord gradient. Native cursor sync to input box. 57 TUI + 53 conversation tests pass.

## Review Remediation Plan (2026-06-11)

Architecture review found that the state model separation is directionally correct,
but the TUI integration still has three boundary issues that must close before
I023 can move from Review to Complete:

1. **Real cancellation**: Ctrl+C in TUI must emit `UserInput::Cancel`, and the
   conversation bridge must translate it into `SessionOp::Interrupt`. Updating
   only UI state is not sufficient because tools or provider work may continue
   after the user sees a cancellation hint.
2. **Reliable state-critical events**: the TUI bridge must not use lossy
   `broadcast` delivery for turn lifecycle events. `TurnStart`, deltas, tool
   events, errors, and completion-derived errors must reach `ConversationEngine`
   through a non-dropping queue; lossy fan-out is allowed only for observers that
   do not own state.
3. **Engine-owned mutation**: external integration code must stop mutating
   `ConversationEngine` fields directly for processing state and queues. The
   engine should expose methods such as `start_user_message`, `enqueue_steering`,
   `cancel_turn`, and `drain_steering_queue` so the ownership boundary matches
   the architecture document.
4. **Multiline input blocks**: pasted multiline input must remain one user
   message block. The input box may grow to multiple rows, submission streams
   the full block, and scrollback renders only the first user line with ` > `;
   continuation lines use the same three-column alignment without repeating
   the prompt marker.
5. **Block-aware line streaming**: `StreamMessage` is a logical message block,
   but terminal history is still flushed line-by-line as complete `\n` lines
   arrive. Prefixes are keyed by the stream-local line index: line 0 uses the
   source prefix (` > `, ` ◆ `, ` # `, ` ! `), and all continuation lines use
   the blank alignment prefix (`   `). The live processing spinner remains a
   preview-only marker and is never written to history.

### Remediation Acceptance

- Pressing Ctrl+C during an active TUI turn sends `SessionOp::Interrupt` to the
  session actor and updates `StatusSnapshot.is_processing` to `false`.
- The TUI bridge from `SessionEvent` to `ConversationEngine` uses a non-lossy
  channel for state-critical events; no `Lagged` recovery path is required in
  the TUI bridge.
- `talos-cli` no longer writes `engine.is_processing` or
  `engine.steering_queue` directly.
- Pasting `line1\nline2` into the TUI input keeps both lines in the input
  buffer; submitting renders one user stream block as ` > line1` followed by
  `   line2`.
- Multiline assistant/system/error/tool streams flush complete lines in real
  time, with only the first line carrying the source prefix and all continuation
  lines aligned with `   `.
- `talos-conversation` has no unused dependency on `talos-permission`.
- Workspace verification remains clean: `cargo fmt --all --check`,
  `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, and
  `cargo test --workspace`.

## Next Slice: Stream Render State

The next implementation step is a behavior-preserving extraction inside
`crates/talos-tui/src/app.rs`: collect stream rendering fields into one
`StreamRenderState` helper that owns the active source, stream-local line count,
incomplete line buffer, and live preview text.

This is intentionally not markdown rendering and not a dynamic-height streaming
viewport. The helper keeps the current contract:

- complete `\n`-terminated lines are converted into scrollback lines immediately;
- incomplete trailing text remains the single-row preview;
- source prefixes are decided by the stream-local line index;
- the processing spinner stays preview-only;
- `InlineTerminal::insert_history` remains a single-line writer.

The purpose is to give future block-aware rendering a local cache boundary
without weakening the stable scrollback/layout strategy.

## Next Slice: Stream Line Emission Boundary

After `StreamRenderState` exists, move the conversion from stream-local lines to
`ScrollbackLine` into that helper. `Tui` should no longer assemble prefixes or
user-message background rows directly; it should only start streams, pass chunks
to the render state, append returned scrollback lines, and flush them through
`InlineTerminal::insert_history`.

Acceptance:

- `StreamRenderState::start` returns any stream-opening scrollback rows needed
  for the source, including the user block top padding row.
- `StreamRenderState::push_chunk` returns prefixed scrollback lines for complete
  `\n`-terminated content only.
- `StreamRenderState::finish` returns the remaining preview line, any
  stream-closing rows such as the user block bottom padding row, and resets the
  helper.
- `Tui` keeps `insert_history` as the only terminal history writer and still
  writes one line at a time.

## Next Slice: Stream Hold Buffer

Add an internal hold-buffer mode to `StreamRenderState` without enabling it for
normal streams yet. The default runtime behavior remains immediate line
emission: complete `\n`-terminated lines are written to pending scrollback as
soon as they arrive, and only the incomplete trailing text remains in preview.

The hold-buffer path exists for future markdown/table rendering, where a logical
block may need complete block text before it can decide how many terminal rows
to emit. This slice only proves the buffer boundary and keeps the terminal
layout stable:

- default stream start uses immediate mode and preserves current behavior;
- hold mode accumulates complete lines internally instead of emitting them from
  `push_chunk`;
- `finish` emits held lines in order, then the remaining preview line, then any
  source-specific closing rows;
- line prefixes are still based on stream-local line indexes;
- `InlineTerminal::insert_history` remains a single-line writer.

## Next Slice: Markdown Block Classifier Design

Before implementing Markdown rendering, add a TUI-side classifier boundary as
described in
[`docs/proposals/tui-stream-markdown-rendering.md`](../proposals/tui-stream-markdown-rendering.md).
The classifier decides whether incoming stream content can be rendered as an
immediate single-line Markdown row or must be held as a structured block.

The preview remains a one-row component. Immediate lines may show their latest
incomplete text in preview. Held blocks hide raw content in preview and instead
show status derived from classifier metadata, such as `rendering table...` or
`receiving code block...`. Finished blocks are rendered into scrollback rows and
inserted through the existing single-line `InlineTerminal::insert_history`
path.

Acceptance:

- Plain text and conservative single-line Markdown render in immediate mode and
  continue flushing complete lines to history as they arrive.
- Tables, fenced code blocks, list blocks, and quote blocks have deterministic
  start/end conditions and expose hold status for preview animation text.
- Code fences suppress table/list/quote recognition until the matching closing
  fence is seen.
- Malformed, oversized, or unterminated blocks fall back to visible plain rows;
  no buffered text is dropped.
- Prefixes remain stream-local: only rendered row 0 gets the source prefix, and
  all continuation rows get the blank three-column prefix.
- Tests cover chunk boundaries split across newline, pipe, backtick, and inline
  delimiter tokens.

Implementation evidence (2026-06-12):

- `crates/talos-tui/src/stream_markdown.rs` adds a deterministic stream block
  classifier with hold status, boundary hints, fallback reasons, table
  alignment, and tests for plain text, tables, code fences, table lookahead, and
  unterminated fences.
- `StreamRenderState` consumes classifier decisions, keeps preview one row,
  shows block status text while holding structured content, and flushes rendered
  rows through the existing `ScrollbackLine` path.
- The first implementation keeps inline Markdown as conservative immediate
  plain text. Rich styled spans remain future work.

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
| S9 | Line padding and styled scrollback | 3-column padding by message type (` > ` user, ` ◆ ` assistant/tool, ` # ` system, ` ! ` error). User messages rendered with Nord Polar Night background (`#3B4252`) and top/bottom padding rows. `ScrollbackLine` carries `text` + optional `bg`. `insert_history` accepts `bg: Option<Color>`, pads lines to full terminal width. Stream separator (blank line) between non-first streams. | ✅ Complete |
| S10 | Animated spinner and cursor sync | 2-char braille spinner with 10-frame animation and Nord color gradient cycling (150ms/frame). Native terminal cursor synced to input box position after each render via `MoveTo` + `Show`. `set_cursor` method on `InlineTerminal`. `restore()` clears viewport content before exiting. | ✅ Complete |

## Execution Evidence

- 110 tests pass (57 TUI + 53 conversation).
- `talos-conversation` crate: `ConversationEngine` owns all business state, 53 tests.
- `talos-tui` crate: event-driven UI with pure state, 57 tests.
- Single-directional information flow: Agent → ConversationEngine → UI via typed async channels (`mpsc::UiOutput`).
- Stream-based content delivery: UI consumes active stream via `next_stream_chunk` in `select!` loop (no spawn task).
- `insert_history` rewritten Codex-style: two branches (non-bottom: `\x1bM` push viewport; bottom: scroll region + `\r\n`), single-line operation, `needs_clear` for clean redraw.
- `insert_history` accepts `bg: Option<Color>` for styled scrollback — lines padded to full terminal width with `SetBackgroundColor`.
- `ScrollbackLine` struct carries `text: String` + `bg: Option<Color>`, enabling per-line background styling.
- User messages rendered with Nord Polar Night background (`#3B4252`) and top/bottom padding rows for visual grouping.
- 3-column line padding system: ` > ` (user), ` ◆ ` (assistant/tool first), `   ` (continuation), ` # ` (system), ` ! ` (error).
- Non-first streams are separated by a blank line when the new stream's first
  non-empty chunk arrives.
- Queued steering input is rendered through `start_user_message` when drained
  after the active turn, before the bridge submits it to the session actor.
- Markdown tables, fenced code blocks, lists, and quotes are detected by a
  TUI-side classifier. Structured blocks are held with one-row preview status
  text, then flushed as visible history rows with no terminal history rewrite.
- Animated 2-char braille spinner with Nord color gradient (10 frames, 150ms/frame) in preview component.
- Native terminal cursor synced to input box position after each render (`MoveTo` + `Show`).
- `restore()` clears viewport content (`MoveTo` + `Clear(FromCursorDown)`) before disabling raw mode.
- Commits: `5c90874` (event-driven architecture), `a669a3e` (insert_history rewrite + preview sync fix), `988fc82` (line padding + error tips), `fc370ce` (animated spinner), pending classifier implementation commit.

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
