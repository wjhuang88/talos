# TUI inline-by-default refactor (Codex-style)

## Status

Proposal (revised 2026-06-06). Supersedes the 2026-06-06 "modular overhaul"
framing after verifying Codex TUI's actual implementation.

> Before code lands, record an ADR or an iteration plan that:
>
> - Confirms the inline-by-default architecture is the right boundary for Talos
>   (i.e. scrollback-as-transcript is desirable, not just convenient).
> - Lays out the file-by-file split in `crates/talos-tui/src/`.
> - Confirms the public API of `talos-tui` is unchanged or carries a semver bump.
> - Re-validates that the I008 hook-based learning path still observes the same
>   event ordering after the refactor.
> - Verifies that I014's I009-S6 / I010-S9 functionality (provenance markers,
>   `/plugins`, `/copy last`, `/copy all`, `/export <path>`) survives the move.
> - Re-runs `cargo test --workspace` after every sub-slice; refactors that
>   silently break the I008 hook observer are out of policy per ADR-006.

## Motivation (revised 2026-06-06)

The original 2026-06-06 framing described this work as a "Codex-style
modular overhaul" — adopt Codex's 80+ module layout (`history_cell/`,
`keymap.rs`, `bottom_pane/`, etc.) for structural depth. After reading
Codex's source end-to-end on 2026-06-06, that framing was **wrong**: the
real architectural lesson is not "more modules" but **"inline-by-default,
alt-screen only for sub-views"**. The module split is a consequence, not a
goal. A pure structural refactor that preserves `EnterAlternateScreen` would
ship the wrong thing.

### What the user said (and the principle)

> "我们整体的 ui 设计必须是信息流驱动的,不是框架分割的而是行追加的风格"
>
> "我们 ui 整体上一直是当前终端追加内容,只是追加的内容可以是复杂格式或者组件块"

The principle: **the UI is an information flow, line-appended into the
terminal's scrollback**. Cells are not widgets in a frame-segmented layout;
they are line blocks pushed to the host terminal as the conversation grows.
The terminal's native scrollback **is the transcript**.

### What Codex actually does (verified by source read)

`codex-rs/tui/src/tui.rs:init()` carries the canonical statement in its doc
comment:

> Initialize the terminal (**inline viewport; history stays in normal scrollback**)

`init()` does **not** call `EnterAlternateScreen`. The viewport anchor is
the user's current cursor y. Every finalized chat turn is **appended to the
scrollback above the viewport** via `insert_history_lines`, an
escape-sequence operation in `codex-rs/tui/src/insert_history.rs` that uses
`SetScrollRegion(1..viewport.top())` to confine the scroll to the rows
above the viewport. The terminal's scrollback is the transcript; there is
no "dump on exit" step.

Alt-screen is used only for full-screen sub-views (model picker, theme
picker, keymap remapper, onboarding, plugin browser) and is gated by
`alt_screen_enabled: bool` (default `true`, opt-out via
`set_alt_screen_enabled(false)`). The `Tui::enter_alt_screen`/`leave_alt_screen`
methods are no-ops when disabled.

See `docs/reference/codex-tui-architecture.md` for the full evidence and
file:line citations.

### What Talos currently does (and why it is wrong)

`crates/talos-tui/src/app.rs:50-71` (`Tui::new`):

```rust
pub fn new() -> Result<Self> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;  // ← alt-screen
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    ...
}
```

`crates/talos-tui/src/app.rs:614-625` (`impl Drop for Tui` + `restore_terminal`):

```rust
impl Drop for Tui {
    fn drop(&mut self) { let _ = restore_terminal(); }
}

pub(crate) fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;  // ← alt-screen
    Ok(())
}
```

Talos always uses alt-screen, unconditionally. The TUI is a frame-segmented
full-screen app. On exit, `LeaveAlternateScreen` discards everything. The
conversation is gone; the user must `/export <path>` to disk if they want a
record. This is **the** day-to-day UX gap with the Codex model, and it is
not fixable by a structural refactor that preserves the alt-screen — it
requires switching to inline-by-default.

## Reference: Codex TUI module layout (what to actually adopt)

The module split is the consequence of the inline-by-default model, not the
goal. After the architectural switch, the natural module split mirrors
Codex's:

| Codex module | Responsibility | Talos current home | Talos target home |
|---|---|---|---|
| `tui.rs` (top-level `Tui` + `init`/`draw`/modes) | Inline viewport, alt-screen opt-in, draw loop | `app.rs` (god module) | `tui/mod.rs` (split out) |
| `custom_terminal.rs` | Inline-aware `Terminal<B>` (visible_history_rows tracking) | n/a — uses ratatui::Terminal | `tui/custom_terminal.rs` (new, derived from ratatui) |
| `insert_history.rs` | Streaming scrollback push (`SetScrollRegion` algorithm) | n/a | `tui/insert_history.rs` (new) |
| `history_cell/{mod,base,messages,exec,approvals,patches,mcp,plans,search,hook_cell,request_user_input,separators,notices,session}.rs` | Per-type cell renderers producing `Vec<Line<'static>>` | `widgets.rs` (flat widgets) | `history_cell/` (1 trait + per-type impls) |
| `chatwidget.rs` (or `chatwidget/mod.rs`) | Cell stream orchestrator, layout, scroll, dispatch | `app.rs` (god module) | `chatwidget.rs` (orchestrator only) |
| `bottom_pane/{mod,chat_composer,…}.rs` | Composer, file search, slash popup, popup stack | `state.rs` (god module) | `bottom_pane/` (1 composer + 1 popup stack) |
| `tui/event_stream.rs` | `EventBroker` stdin, pause/resume for `$EDITOR` | `app.rs:85, 143` (raw `EventStream::new()`) | `tui/event_stream.rs` (new, with `pause_events`/`resume_events`) |
| `tui/frame_requester.rs` + `tui/frame_rate_limiter.rs` | 120 FPS coalescing redraw actor | `app.rs:86, 144` (hard-coded `Duration::from_millis(50)`) | `tui/frame_requester.rs` (new actor) |
| `tui/job_control.rs` | SIGTSTP suspend, ResumeAction::{RealignInline, RestoreAlt} | n/a | `tui/job_control.rs` (new, Unix only) |
| `tui/keyboard_modes.rs` | Keyboard enhancement flag stack | n/a (uses default crossterm) | `tui/keyboard_modes.rs` (new) |
| `slash_command.rs` | Typed enum of 50+ commands | `state.rs:311-372` (match on `&str`) | `slash_command.rs` (enum + descriptor trait) |
| `keymap.rs` | Context-aware keymap (8 contexts) | `app.rs:280-362` (inline match on `KeyCode`) | `keymap.rs` (1 struct + 8 contexts) |

## Proposed Approach

The refactor is split into 5 sub-slices. **Sub-slice A (inline-by-default
rewrite) is the architectural foundation** and must land first; the others
are dependent on it. Each sub-slice is independently shippable and
preserves the I010 R2/R3 + I014 user-facing behavior.

### Sub-slice A: inline-by-default Tui (architectural foundation)

Goal: switch the TUI from "alt-screen full-screen" to "inline-by-default,
alt-screen only for sub-views". The terminal scrollback becomes the
transcript; no dump-on-exit step is needed.

Touch points:

1. **`crates/talos-tui/src/tui/mod.rs`** (new top-level module):
   - `pub struct Tui` with `terminal: custom_terminal::Terminal`, `event_broker: Arc<EventBroker>`, `frame_requester: FrameRequester`, `pending_history_lines: Vec<PendingHistoryLines>`, `alt_screen_active: Arc<AtomicBool>`, `alt_screen_enabled: bool`, `suspend_context: SuspendContext` (Unix), `terminal_focused: Arc<AtomicBool>`.
   - `pub fn new()` calls `set_modes()` (raw mode + bracketed paste + keyboard enhancement); does **not** call `EnterAlternateScreen`.
   - `pub fn init() -> Result<InitializedTerminal>` uses `CustomTerminal::with_options_and_cursor_position(backend, cursor_pos)` to capture the user's current cursor y as the viewport anchor.
   - `pub fn insert_history_lines(&mut self, lines: Vec<Line<'static>>)` matches Codex's batching / coalescing / `schedule_frame()` pattern.
   - `pub fn enter_alt_screen` / `leave_alt_screen` are gated by `alt_screen_enabled: bool`; default to `true` so existing sub-views (none today) would just work.
   - `pub fn set_alt_screen_enabled(&mut self, enabled: bool)` for future `--no-alt-screen`-style flag.
   - `pub async fn with_restored(mode, f)` for `$EDITOR` handoff (sub-slice E prerequisite).
   - `pub fn draw(&mut self, height, draw_fn)` wraps in `SynchronizedUpdate`, flushes pending history lines, then renders the inline viewport.
   - `pub fn restore` / `restore_after_exit` / `restore_keep_raw` as the inverse of `set_modes`.
   - `impl Drop for Tui` calls `restore_after_exit` (the strong reset).

2. **`crates/talos-tui/src/tui/custom_terminal.rs`** (new, derived from `ratatui::Terminal` under MIT, with attribution in the file header per the original):
   - `pub struct Terminal<B: Backend + Write>` with `viewport_area`, `last_known_screen_size`, `last_known_cursor_pos`, `visible_history_rows: u16`.
   - `pub fn visible_history_rows(&self) -> u16`.
   - `pub(crate) fn note_history_rows_inserted(&mut self, inserted_rows: u16)`.
   - `set_viewport_area` clamps `visible_history_rows.min(area.top())`.
   - Multiple clear operations: `clear`, `clear_after_position`, `clear_scrollback`, `clear_visible_screen`, `clear_scrollback_and_visible_screen_ansi`, `invalidate_viewport`.
   - Display-width handling: `display_width(&str)` strips OSC sequences before measuring columns.

3. **`crates/talos-tui/src/tui/insert_history.rs`** (new):
   - `pub enum HistoryLineWrapPolicy { PreWrap, Terminal }` (Talos can use just `PreWrap`; `Terminal` is the raw-mode path).
   - `pub(crate) enum InsertHistoryMode { Standard, ZellijRaw }` (Talos can start with just `Standard`; `ZellijRaw` is a follow-up if we hear about Zellij bugs).
   - `pub fn insert_history_lines(...)` — the standard-mode algorithm: `SetScrollRegion(1..area.top())`, MoveTo top, print lines, ResetScrollRegion, restore cursor.
   - `pub fn insert_history_lines_with_wrap_policy(...)`.
   - `write_history_line(...)` per-line writer: clears continuation rows, sets fg/bg, writes styled spans with SGR diffing.

4. **`crates/talos-tui/src/chatwidget.rs`** (new orchestrator):
   - The "rendered output" of agent events. Receives `AgentEvent` from the broadcast channel and converts it into a **cell stream**: `Vec<Box<dyn HistoryCell>>` for committed cells, `Option<Box<dyn HistoryCell>>` for the active streaming cell.
   - When a cell finalizes, calls `tui.insert_history_lines(cell.display_lines(width))`.
   - The active cell renders into the inline viewport (not the scrollback).
   - Mirrors Codex `chatwidget.rs`'s shape, scaled to Talos's smaller scope.

5. **`crates/talos-tui/src/history_cell/`** (new module):
   - `mod.rs` — `pub trait HistoryCell { fn display_lines(&self, width: u16) -> Vec<Line<'static>>; fn raw_lines(&self) -> Vec<Line<'static>>; fn desired_height(&self, width: u16) -> u16; ... }`. Default `desired_height` uses `Paragraph::line_count` to account for URL wrapping.
   - `base.rs` — `PlainHistoryCell`, `PrefixedWrappedHistoryCell`, `CompositeHistoryCell`.
   - `messages.rs` — `AssistantMessageCell` (markdown), `UserMessageCell` (prefixed), `SystemMessageCell`, `ErrorMessageCell`.
   - `tool_call.rs` — replaces `widgets::ToolCallBubble`; produces the same `▸ name [marker]` block but as `Vec<Line<'static>>` with provenance marker and optional result.
   - `approval.rs` — replaces `widgets::ApprovalOverlay`; produces the same `⚠ Permission Required` block. (The full overlay rendering is also a `bottom_pane/` modal; this cell is the committed `▸ tool [awaiting approval]` block.)
   - `diff.rs` — replaces `widgets::render_diff`; produces diff-styled `Vec<Line<'static>>`.
   - `mod.rs` impl `Renderable for Box<dyn HistoryCell>` for the active-cell viewport render.

6. **`crates/talos-tui/src/app.rs`** (slimmed to a thin wrapper):
   - The `Tui::new` / `Tui::run` / `Tui::run_with_approval` methods become thin constructors that wire up `ChatWidget` + `Tui` + the broadcast channel.
   - The `render()` function (currently in `app.rs:371-449`) becomes a `chatwidget::render` function.
   - The big `build_chat_text` / `build_input_text` / `build_status_text` helpers (currently in `app.rs:451-612`) split between `chatwidget` (chat text) and `bottom_pane` (input/status).

Acceptance criteria:

- [ ] `talos-tui/src/` layout gains `tui/` (custom_terminal, insert_history, event_stream, frame_requester, frame_rate_limiter, job_control, keyboard_modes), `history_cell/` (1 trait + per-type impls), `chatwidget.rs`, `bottom_pane/`, and keeps `app.rs` as a thin wrapper.
- [ ] `Tui::new()` does **not** call `EnterAlternateScreen`; the viewport anchor is the cursor y at startup.
- [ ] Finalized cells are written to the host terminal scrollback via `SetScrollRegion(1..viewport.top())` + `MoveTo` + `Print("\r\n")` per line + `ResetScrollRegion`.
- [ ] User can `Shift+PageUp` (or scroll wheel) in their terminal and see the entire conversation history. No transcript dump on exit is needed.
- [ ] `restore_terminal` (in `Tui::Drop` or `restore_after_exit`) does **not** call `LeaveAlternateScreen` when `alt_screen_active == false` (i.e. default code path).
- [ ] I008 hook-based learning still observes the same `HookEvent` ordering (verified by `crates/talos-cli/tests/hooks_e2e.rs` at `RUST_LOG=debug`).
- [ ] I014's `/copy last`, `/copy all`, `/export <path>`, `/plugins` still work — they reuse the unchanged `state::transcript_plain_text` / `state::transcript_markdown` / `state::plugin_observation_key` helpers.
- [ ] `cargo test --workspace` passes (652+ tests).
- [ ] Pre-existing 5 × `clippy::collapsible_if` warnings on `talos-tui` are unchanged or reduced.
- [ ] No new dependencies in `talos-tui/Cargo.toml` (the inline-by-default refactor uses ratatui + crossterm, same as today; the only new code is custom Terminal derived from ratatui, MIT-licensed, attributed).

### Sub-slice B: tui/ subdir plumbing (event_stream + frame_requester + job_control)

Goal: replace the hard-coded `Duration::from_millis(50)` render interval and
the raw `EventStream::new()` with the actor-style 120 FPS rate limiter and
the `EventBroker` pause/resume.

Touch points:

1. **`crates/talos-tui/src/tui/event_stream.rs`** (new): `EventBroker` with `Paused`/`Start`/`Running(S)` states, `pause_events`/`resume_events`, `TuiEventStream` that round-robins draw and crossterm. Mirrors `codex-rs/tui/src/tui/event_stream.rs` §4.3.
2. **`crates/talos-tui/src/tui/frame_rate_limiter.rs`** (new): 120 FPS clamp, `clamp_deadline`/`mark_emitted`, 28 lines.
3. **`crates/talos-tui/src/tui/frame_requester.rs`** (new): `FrameRequester` + `FrameScheduler` actor, `schedule_frame`/`schedule_frame_in`, broadcast coalescing.
4. **`crates/talos-tui/src/tui/job_control.rs`** (new, Unix only): `SuspendContext` with `resume_pending`/`suspend_cursor_y`, `suspend(alt_screen_active)` + `prepare_resume_action` + `PreparedResumeAction` enum.
5. **`crates/talos-tui/src/tui/keyboard_modes.rs`** (new): keyboard enhancement flag stack with `pop_stack`/`reset_after_exit`.

Acceptance criteria:

- [ ] `app.rs` no longer creates a raw `crossterm::event::EventStream`; it polls `tui.event_stream()`.
- [ ] The render loop uses `frame_requester.schedule_frame()` and a 120 FPS cap. Stress test: 1000 frames requested in a 100ms window produces ~12 actual draws.
- [ ] `^Z` (Unix) cleanly suspends and resumes the TUI, with inline viewport realignment when inline, alt-screen re-entry when in a sub-view. Verify on macOS `zsh` + `bash`.
- [ ] `with_restored` runs `pbcopy` (or any child process) and the terminal returns to a working TUI state.

### Sub-slice C: bottom_pane / composer

Goal: replace the single-line `state::input_buffer: String` with a
multi-line composer that supports `@`-mention file search and `$`-mention
app references.

Touch points:

1. **`crates/talos-tui/src/bottom_pane/mod.rs`** (new): `BottomPane` view stack, `BottomPaneView` trait, `ChatComposer`, `ApprovalOverlay` (modal), `SlashPopup` (modal).
2. **`crates/talos-tui/src/bottom_pane/chat_composer.rs`** (new): multi-line input with `textarea`-like behavior. Cursor, history (`↑`/`↓`), `@`-mention search (uses `talos-tools::find_files` or a temporary `walkdir`-based path scan), `$-mention` app list.
3. **`crates/talos-tui/src/bottom_pane/popup_stack.rs`** (new): ordered view stack with `push`/`pop`/`is_empty`.
4. **`crates/talos-tui/src/bottom_pane/file_search.rs`** (new): `@`-mention file search popup.

Acceptance criteria:

- [ ] Multi-line input: `Enter` inserts newline when modified (`Shift+Enter`); `Enter` submits when not modified.
- [ ] `@` triggers file search popup; typing after `@` filters; `↑`/`↓` selects; `Enter` inserts the mention; `Esc` cancels.
- [ ] `$` triggers app-reference popup; same UX.
- [ ] Approval overlay opens on `TuiApprovalRequest`; `y`/`a`/`n` selects; `Esc` denies.

### Sub-slice D: slash command framework

Goal: replace the `state::handle_slash_command` match-on-`&str` with a typed
enum + descriptor trait so adding a command doesn't require touching the
match arm.

Touch points:

1. **`crates/talos-tui/src/slash_command.rs`** (new): `pub enum SlashCommand` (`strum` derive), `description`, `command`, `supports_inline_args`, `available_during_task`, `is_visible` methods. `pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)>`.
2. **`crates/talos-tui/src/slash_command/popup.rs`** (new): popup rendering, fuzzy filter, inline args hint.
3. **I014 commands** (`/plugins`, `/copy`, `/export`) get the enum treatment.
4. **TUI-002's 50+ commands** are not all required; Talos can ship 13-15 (its current 13 plus 2-3 more: `/clear`, `/vim` toggle, `/model`).

Acceptance criteria:

- [ ] Adding a new slash command requires 1 enum variant + 1 description arm, not a `handle_slash_command` match arm.
- [ ] Tab completion uses `built_in_slash_commands` and is presentation-order (most-frequent first).
- [ ] I014 commands (`/plugins`, `/copy`, `/export`) survive the migration.

### Sub-slice E: keymap system

Goal: move the inline `KeyCode` match in `app.rs:280-362` into a context-aware
keymap.

Touch points:

1. **`crates/talos-tui/src/keymap.rs`** (new): `pub struct Keymap`, `pub enum KeyContext { App, Chat, Composer, Approval }` (Talos can start with 4 contexts; Codex has 8).
2. **`crates/talos-tui/src/keymap.rs`** builder: `Keymap::new()` with default bindings, `.bind(context, KeyEvent, KeyAction)` for extension.
3. **Vim mode** as a follow-up: `KeyContext::Vim` + `MotionMode`. Not required for the 4-context baseline.

Acceptance criteria:

- [ ] All I014 keybindings (Ctrl+C, Ctrl+K skills, Ctrl+E evolution, Tab completion, Esc, Enter, etc.) keep their current behavior.
- [ ] Adding a new key binding requires a single `.bind(context, key, action)` call.

## Non-Goals

- No new TUI features (no new slash commands beyond the I014 set, no new cells, no new key bindings).
- No user-visible behavior changes for the inline-by-default path — the conversation is now scrollback-preserved instead of discarded, which is a **new** capability but not a breaking change.
- No public API changes to `talos-tui` (or, if any, a semver bump + ADR).
- No migration of `/copy` / `/export` / `/plugins` semantics from I014.
- No changes to I010 R2 run-path convergence (AppServerSession stays).
- No new external dependencies; the inline-by-default refactor is ratatui + crossterm only.

## Alternatives Considered

- **Leave the alt-screen mode as-is and only add a transcript dump on exit (TUI-003 original plan)**. Rejected: the transcript dump is a workaround for the wrong model. Inline-by-default makes the scrollback the transcript by construction, which is what the user actually asked for ("信息流驱动... 行追加的风格"). See `docs/reference/codex-tui-architecture.md` §10.2 for the touch-point comparison.
- **Adopt a third-party TUI framework (e.g. `tui-realm`)**. Rejected: we use ratatui + crossterm directly per `docs/reference/REFERENCE-PROJECTS.md` §956. A framework would add an external dependency that goes against AGENTS.md rule #1.
- **Port `codex-rs/tui` files directly**. Rejected: the I010 R2 event architecture (single-mpsc `AppEvent` + `AppServerSession` seam) is the right boundary for Talos. Direct porting would either duplicate the agent loop or break the `AppServerSession` seam (per ADR-005). We adopt the *layout* and the *inline-by-default model*, not the *implementation*.
- **Pure structural refactor (the original 2026-06-06 framing)**. Rejected: structural depth without the architectural switch ships a frame-segmented Codex-style layout that does not match the inline model. The module split is a consequence of the architecture, not the goal. After the architectural switch, the natural module split mirrors Codex's, so the work is not duplicated.

## Open Questions

- Should `Tui::new` take a `Config` or stay stateless and let `set_alt_screen_enabled` be called separately? (Prefer the latter — keeps `Tui::new` minimal and lets `ChatWidget` decide based on config.)
- Should `BottomPane` own the `TuiState` projection (input buffer, slash popup state) or expose a `View` trait that the `Tui` renders? (Prefer the `View` trait — keeps `BottomPane` testable in isolation.)
- Should the cell stream include a **transcript overlay** (`Ctrl+T`) like Codex? Yes, but it can be a follow-up after sub-slice A lands.
- Should we use `text-area` or `tui-textarea` from crates.io, or hand-roll the composer? Hand-roll: AGENTS.md rule #1 (self-contained capabilities); Codex hand-rolls too.
- Zellij compatibility: do we need `InsertHistoryMode::ZellijRaw` on day one? Defer until we hear about Zellij bugs from a Talos user.

## Dependencies

- **I015 Provider Schema** (R6): provider output format may affect how
  `history_cell/messages.rs` renders. Should land first or in parallel.
- **I016 Portable File/Search** (R7): `@`-mention file search in
  `bottom_pane/file_search.rs` benefits from the native `find_files` tool.
  Should land first or in parallel.
- **I017 Embedded Git** (R8): `history_cell/diff.rs` benefits from `gix`
  output. Should land first or in parallel.
- **ADR-003** (TUI progressive evolution) — anchor for the migration.
- **ADR-005** (TUI event architecture) — boundary on event flow.
- **ADR-006** (event architecture boundary) — single-mpsc bus contract
  must not be violated.
- **I008** (Learning Agent): hook-based learning observes the same event
  stream. The refactor must preserve I008's event ordering. Verified by
  `crates/talos-cli/tests/hooks_e2e.rs` after each sub-slice.

## Scheduling

- **Sub-slice A is the prerequisite for everything else** and is the most
  architecturally significant change. Land it as a small dedicated
  iteration (call it I022 or similar) ahead of the I015-I017 foundations.
- Sub-slices B, C, D, E are independent of each other and of I015-I017.
  They can be picked up as I023+ sub-iterations.
- **TUI-003 (transcript-on-exit)** is **superseded** by sub-slice A — see
  `docs/backlog/active/TUI-003-tui-exit-transcript.md` for the supersede
  note.
- The pure structural refactor (the original 2026-06-06 framing) is no
  longer a separate sub-slice; the structural split happens naturally as
  a consequence of sub-slice A.

## Acceptance Criteria (Iteration-Level)

- [ ] `talos-tui/src/` layout includes `tui/`, `history_cell/`, `chatwidget.rs`, `bottom_pane/`, `slash_command.rs`, `keymap.rs`.
- [ ] Inline-by-default: `Tui::new()` does not call `EnterAlternateScreen`; the viewport anchor is the cursor y at startup; finalized cells are pushed to the scrollback via `SetScrollRegion` + `MoveTo` + `Print` + `ResetScrollRegion`.
- [ ] The terminal's native scrollback contains the full conversation after a clean TUI exit. Verified by manual test: start a session, send 10 messages, run 3 tool calls, `/copy all` (or just look at scrollback), `q` to exit, `Shift+PageUp` in the host terminal.
- [ ] All I014 functionality (provenance markers, `/plugins`, `/copy last`, `/copy all`, `/export <path>`) still works.
- [ ] I008 hook-based learning still observes the same `HookEvent` ordering.
- [ ] Public API of `talos-tui` is unchanged (or carries a semver bump + ADR).
- [ ] `cargo test --workspace` passes with no regressions (652+ tests).
- [ ] Pre-existing 5 × `clippy::collapsible_if` warnings on `talos-tui` are either unchanged or reduced.
- [ ] `docs/iterations/I022-tui-inline-by-default.md` (or successor) records sub-slice outcomes and runtime evidence.

## Risks

- **Event ordering drift** during the cell-stream rewrite can silently break the I008 hook observer. Mitigation: run `hooks_e2e` + `mcp_client_e2e` after every sub-slice; they assert on event strings in stderr at `RUST_LOG=debug`.
- **Public API churn**: even an "architectural" refactor tends to expose types. Mitigation: gate the refactor behind `cargo doc` review + semver check; keep `TuiState` private.
- **I014 regression**: `/copy` and `/export` use `TuiState` private methods (`last_assistant_text`, `transcript_plain_text`, `transcript_markdown`). These are `pub(crate)`; the cell refactor must not need to expose them more widely. Mitigation: keep the same access pattern; do not move transcript state to a public type.
- **Custom Terminal license**: the inline-by-default refactor requires a `CustomTerminal` derived from `ratatui::Terminal` (MIT). The file header must carry the MIT attribution per the original Codex source. Mitigation: copy the header verbatim; record the license provenance in the iteration log.
- **Scrollback-as-transcript contract**: switching to inline-by-default means the user's terminal scrollback now contains the conversation. If the user has tmux/screen with limited scrollback, they may hit the cap. Mitigation: document the behavior; `--redact-on-exit` is a follow-up.
- **TUI-003 (transcript-on-exit) is dissolved by this refactor**: in inline mode, there is no transcript to dump — the scrollback is the transcript. The `/export <path>` workflow is the canonical way to save a transcript to disk. Mitigation: update TUI-003's backlog item to "Superseded by TUI-002 sub-slice A (inline-by-default)"; keep `/export` as the canonical transcript-save path.
