# Codex TUI Architecture (Reference)

> **Status**: Reference. Describes what Codex TUI **is**, not what Talos should be. Pair
> with `docs/proposals/tui-codex-overhaul.md` (what Talos should adopt) and
> `docs/backlog/active/TUI-002-codex-overhaul.md` (how to land the adoption).

This document is the authoritative record of Codex TUI's verified implementation
as of the 2026-06-06 source read of `https://github.com/openai/codex` (`codex-rs/tui/src/`).
Line numbers and function names reference the source at that snapshot.

## TL;DR — The Core Principle

**Codex TUI is inline-by-default. Alt-screen is opt-in for sub-views only.**

This is the single most important fact and the source of all subsequent design choices:

```text
┌─Screen──────────────────────────────────────────┐
│ ↑ history rows above viewport (terminal scrollback) │
│ ↑ history rows (continued)                          │
│ ↑ history rows (continued)                          │
│╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌│ ← SetScrollRegion(1..viewport.top())
│╭─Inline viewport───────────────────────────────╮│
││  composer / input / active status indicators ││
│╰───────────────────────────────────────────────╯│
└──────────────────────────────────────────────────┘
```

The viewport sits **at the cursor's current y** when TUI starts. Every finalized
chat turn is **appended to the scrollback above the viewport** via escape-sequence
operations, not a normal ratatui draw. The terminal's native scrollback **is the
transcript**. There is no "dump on exit" step — the scrollback already has
everything.

Alt-screen is used only for full-screen sub-views (onboarding, model picker,
plugin browser, theme picker, keymap remapper). When alt-screen is active the
viewport fills the terminal; when it is not, the viewport is the bottom line
where the user types.

## 1. The Custom Terminal — `custom_terminal.rs`

**File**: `codex-rs/tui/src/custom_terminal.rs` (~880 lines)
**License**: Derived from `ratatui::Terminal` (MIT, Florian Dehau + Ratatui Developers)

Codex does **not** use `ratatui::Terminal` directly. It has a custom fork that
adds inline-viewport support. The file header is explicit:

```rust
// This is derived from `ratatui::Terminal`, which is licensed under the following terms:
// The MIT License (MIT)
// Copyright (c) 2016-2022 Florian Dehau
// Copyright (c) 2023-2025 The Ratatui Developers
```

The struct signature is the smoking gun:

```rust
pub struct Terminal<B>
where B: Backend + Write,
{
    backend: B,
    buffers: [Buffer; 2],
    current: usize,
    pub hidden_cursor: bool,
    pub viewport_area: Rect,
    pub last_known_screen_size: Size,
    pub last_known_cursor_pos: Position,
    /// Count of visible history rows rendered above the viewport in inline mode.
    visible_history_rows: u16,
}
```

The `visible_history_rows: u16` field has the doc comment **"Count of visible
history rows rendered above the viewport in inline mode."** This is the only
in-line scrollback tracking state in the entire TUI.

The viewport anchor is the **cursor y at TUI startup**:

```rust
viewport_area: Rect::new(
    /*x*/ 0,
    cursor_pos.y,    // ← viewport top = user's current cursor y
    /*width*/ 0,
    /*height*/ 0,
),
```

The two key API methods for inline history are:

```rust
pub fn visible_history_rows(&self) -> u16 { self.visible_history_rows }

pub(crate) fn note_history_rows_inserted(&mut self, inserted_rows: u16) {
    self.visible_history_rows = self
        .visible_history_rows
        .saturating_add(inserted_rows)
        .min(self.viewport_area.top());
}
```

`note_history_rows_inserted` is called by `insert_history_lines` after pushing
lines into the scrollback. `set_viewport_area` clamps
`visible_history_rows.min(area.top())` so a viewport shrink cannot desync the
counter.

The custom terminal also exposes **multiple explicit clear operations** that
normal `ratatui::Terminal` does not:

| Method | Purpose |
|---|---|
| `clear()` | Clear viewport only |
| `clear_after_position(pos)` | Clear from position to end of screen |
| `clear_scrollback()` | Clear scrollback only (uses `ClearType::Purge`) |
| `clear_visible_screen()` | Clear visible screen, not scrollback |
| `clear_scrollback_and_visible_screen_ansi()` | Hard-reset with explicit ANSI |
| `invalidate_viewport()` | Force full repaint on next draw |

The clear-after-position and scrollback-aware clear variants are what allow
inline mode to recover when a normal ratatui draw would not work (e.g. the
viewport has been scrolling up while history accumulates below).

## 2. `insert_history.rs` — The Streaming API

**File**: `codex-rs/tui/src/insert_history.rs` (~430 lines)

This is the file that turns a finalized chat cell into **bytes written to the
terminal scrollback**. The doc comment is explicit:

> Inserts finalized history rows into terminal scrollback.
>
> Codex uses the terminal scrollback itself for finalized chat history, so
> inserting a history cell is an escape-sequence operation rather than a normal
> ratatui render.

### 2.1 The wrap policies

```rust
pub enum HistoryLineWrapPolicy {
    PreWrap,     // Codex's adaptive pre-wrap (preserves URLs, wraps mixed content)
    Terminal,    // terminal soft-wrap (no pre-wrap, terminal handles)
}
```

`PreWrap` is the default. It is URL-aware: lines that contain URL-like tokens
stay intact (so terminal emulators can match them as clickable links), mixed
URL+prose lines get adaptive wrapping that keeps URLs unsplit, and plain prose
flows through adaptive wrap. `Terminal` is used for raw scrollback mode (where
copy/paste fidelity matters more than display quality).

### 2.2 The insert modes

```rust
pub(crate) enum InsertHistoryMode {
    Standard,     // normal scroll-region-based push
    ZellijRaw,    // workaround for Zellij's scroll-region bug
}
```

`ZellijRaw` exists because Zellij does not constrain soft-wrapped continuation
rows to the Codex scroll region, so its raw path appends history through the
terminal and reserves blank rows for the next viewport draw.

### 2.3 The Standard mode algorithm (the core of inline mode)

```rust
pub(crate) fn insert_history_hyperlink_lines_with_mode_and_wrap_policy<...>(
    terminal: &mut crate::custom_terminal::Terminal<B>,
    lines: Vec<HyperlinkLine>,
    mode: InsertHistoryMode,
    wrap_policy: HistoryLineWrapPolicy,
) -> io::Result<()> {
    let screen_size = terminal.backend().size().unwrap_or(Size::new(0, 0));
    let mut area = terminal.viewport_area;
    let mut should_update_area = false;
    let last_cursor_pos = terminal.last_known_cursor_pos;

    // 1. Pre-wrap each line according to the wrap policy.
    let wrap_width = area.width.max(1) as usize;
    let mut wrapped = Vec::new();
    let mut wrapped_rows = 0usize;
    for line in &lines {
        let line_wrapped = match wrap_policy { /* ... */ };
        wrapped_rows += line_wrapped.iter()
            .map(|l| l.width().max(1).div_ceil(wrap_width))
            .sum::<usize>();
        wrapped.extend(line_wrapped);
    }
    let wrapped_lines = wrapped_rows as u16;

    match mode {
        InsertHistoryMode::Standard => {
            let writer = terminal.backend_mut();

            // 2. If the viewport is not at the bottom, scroll it down to make room.
            //    Don't scroll past the bottom of the screen.
            let cursor_top = if area.bottom() < screen_size.height {
                let scroll_amount = wrapped_lines.min(screen_size.height - area.bottom());
                let top_1based = area.top() + 1;
                queue!(writer, SetScrollRegion(top_1based..screen_size.height))?;
                queue!(writer, MoveTo(0, area.top()))?;
                for _ in 0..scroll_amount {
                    queue!(writer, Print("\x1bM"))?;   // Reverse Index
                }
                queue!(writer, ResetScrollRegion)?;
                area.y += scroll_amount;
                should_update_area = true;
                area.top().saturating_sub(1)
            } else {
                area.top().saturating_sub(1)
            };

            // 3. Limit the scroll region to the rows ABOVE the viewport.
            //    This is the inline-by-default magic: only the rows above the
            //    viewport are inside the scroll region, so when we add lines
            //    inside this area, only the rows in this area will be scrolled.
            queue!(writer, SetScrollRegion(1..area.top()))?;

            // 4. Move cursor to the bottom of the scroll region, add lines.
            queue!(writer, MoveTo(0, cursor_top))?;
            for line in &wrapped {
                queue!(writer, Print("\r\n"))?;
                write_history_line(writer, line, wrap_width)?;
            }
            queue!(writer, ResetScrollRegion)?;

            // 5. Restore the cursor.
            queue!(writer, MoveTo(last_cursor_pos.x, last_cursor_pos.y))?;
        }
        InsertHistoryMode::ZellijRaw => { /* Zellij workaround path */ }
    }

    // 6. Update the viewport area if needed.
    if should_update_area {
        terminal.set_viewport_area(area);
    }

    // 7. Tell the terminal how many history rows were inserted.
    if wrapped_lines > 0 {
        terminal.note_history_rows_inserted(wrapped_lines);
    }
    Ok(())
}
```

The diagram in the source is worth quoting directly:

```text
┌─Screen───────────────────────┐
│┌╌Scroll region╌╌╌╌╌╌╌╌╌╌╌╌╌╌┐│
│┆ ┆│
│┆ ┆│
│┆ ┆│
│█╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┘│
│╭─Viewport───────────────────╮│
││ ││
│╰────────────────────────────╯│
└──────────────────────────────┘
```

This is the **whole** of inline-by-default. The scroll region is the inline
view above the viewport. New history is written there, scrolling upward.
Viewport stays put.

### 2.4 The `write_history_line` per-line writer

```rust
fn write_history_line<W: Write>(writer: &mut W, line: &HyperlinkLine, wrap_width: usize) -> io::Result<()> {
    let physical_rows = line.width().max(1).div_ceil(wrap_width) as u16;

    // Clear continuation rows for wide lines.
    if physical_rows > 1 {
        queue!(writer, SavePosition)?;
        for _ in 1..physical_rows {
            queue!(writer, MoveDown(1), MoveToColumn(0))?;
            queue!(writer, Clear(ClearType::UntilNewLine))?;
        }
        queue!(writer, RestorePosition)?;
    }

    // Set fg/bg from the line style.
    queue!(writer, SetColors(Colors::new(...)))?;
    queue!(writer, Clear(ClearType::UntilNewLine))?;

    // Write styled spans, emitting modifier diffs (SGR) only when needed.
    write_spans(writer, decorated.iter())
}
```

`write_spans` is a SGR-aware span writer: it tracks current fg/bg/modifier
state and only emits SGR codes when they change. This is the per-cell
efficiency win that lets the inline approach scale to long sessions.

## 3. `history_cell/` — The Cell Trait and Concrete Cells

**Files**: `codex-rs/tui/src/history_cell/` (17 modules + tests + snapshots)

The unit of conversation display is a `HistoryCell` trait. The trait produces
**lines as data**, not rendered widgets:

```rust
pub(crate) trait HistoryCell: std::fmt::Debug + Send + Sync + Any {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>>;
    fn raw_lines(&self) -> Vec<Line<'static>>;
    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> { ... }
    fn display_lines_for_mode(&self, width: u16, mode: HistoryRenderMode) -> Vec<Line<'static>> { ... }
    fn display_hyperlink_lines_for_mode(&self, width: u16, mode: HistoryRenderMode) -> Vec<HyperlinkLine> { ... }
    fn desired_height(&self, width: u16) -> u16 { ... }
    fn desired_height_for_mode(&self, width: u16, mode: HistoryRenderMode) -> u16 { ... }
    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> { ... }
    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> { ... }
    fn desired_transcript_height(&self, width: u16) -> u16 { ... }
    fn is_stream_continuation(&self) -> bool { false }
    fn transcript_animation_tick(&self) -> Option<u64> { None }
}

pub(crate) enum HistoryRenderMode {
    Rich,    // styled colors, hyperlinks
    Raw,     // plain text for copy/paste
}
```

The default `desired_height` uses `Paragraph::line_count` to measure rows
**after** ratatui's viewport-level character wrapping. This is critical for
long URL lines: the logical line count would undercount; the paragraph
measurement reflects what the user actually sees.

`transcript_animation_tick` is for time-based visuals (spinner, shimmer).
Returning `None` means the transcript is stable; returning `Some(tick)` lets
the transcript overlay (`Ctrl+T`) keep up with the main viewport.

### 3.1 Concrete cell types

| Module | Purpose |
|---|---|
| `messages.rs` | User/assistant text messages |
| `exec.rs` | Tool execution cells (`$ command`, output, exit code) |
| `approvals.rs` | Approval prompt cells |
| `patches.rs` | Patch / diff cells |
| `mcp.rs` | MCP tool call cells |
| `plans.rs` | Plan list updates |
| `session.rs` | Session header, configuration summary |
| `notices.rs` | Update available, system notices |
| `search.rs` | Web search results |
| `hook_cell.rs` | Hook execution cells |
| `request_user_input.rs` | User-input request cells |
| `separators.rs` | Horizontal rule separators |
| `base.rs` | `PlainHistoryCell`, `WebHyperlinkHistoryCell`, `PrefixedWrappedHistoryCell`, `CompositeHistoryCell` |

### 3.2 The base cells

`base.rs` provides the building blocks:

- **`PlainHistoryCell`** — wraps `Vec<Line<'static>>`; the simplest cell.
- **`WebHyperlinkHistoryCell`** — wraps lines + URL hyperlink metadata.
- **`PrefixedWrappedHistoryCell`** — wraps text with `initial_prefix` and
  `subsequent_prefix` indents (used for `> user` style messages).
- **`CompositeHistoryCell`** — composes multiple cells with blank lines
  between (used for multi-section tool call cells).

### 3.3 The `Renderable` impl for `Box<dyn HistoryCell>`

```rust
impl Renderable for Box<dyn HistoryCell> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let hyperlink_lines = self.display_hyperlink_lines(area.width);
        let lines = visible_lines(hyperlink_lines.clone());
        let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
        let y = if area.height == 0 {
            0
        } else {
            let overflow = paragraph.line_count(area.width)
                .saturating_sub(usize::from(area.height));
            u16::try_from(overflow).unwrap_or(u16::MAX)
        };
        // Active-cell content can reflow dramatically during resize/stream updates.
        // Clear the entire draw area first so stale glyphs from previous frames
        // never linger.
        Clear.render(area, buf);
        paragraph.scroll((y, 0)).render(area, buf);
        mark_buffer_hyperlinks(buf, area, &hyperlink_lines, usize::from(y));
    }
    fn desired_height(&self, width: u16) -> u16 {
        HistoryCell::desired_height(self.as_ref(), width)
    }
}
```

The `Box<dyn HistoryCell>` is the in-flight **active cell** (the one currently
streaming). It renders into a viewport cell (with overflow scroll), not into
the scrollback. When the active cell finalizes, its `Vec<Line<'static>>` is
flushed to the scrollback via `insert_history_lines`.

## 4. The `tui/` Subdir — Terminal Plumbing

**Files**: `codex-rs/tui/src/tui/{event_stream,frame_rate_limiter,frame_requester,job_control,keyboard_modes,terminal_stderr}.rs`

This is where the terminal is wrapped. Six small files, each a single concern.

### 4.1 `frame_rate_limiter.rs` (28 lines)

```rust
pub(super) const MIN_FRAME_INTERVAL: Duration = Duration::from_nanos(8_333_334);  // 120 FPS

pub(super) struct FrameRateLimiter {
    last_emitted_at: Option<Instant>,
}

impl FrameRateLimiter {
    /// Returns `requested`, clamped forward if it would exceed the maximum frame rate.
    pub(super) fn clamp_deadline(&self, requested: Instant) -> Instant {
        let Some(last_emitted_at) = self.last_emitted_at else {
            return requested;
        };
        let min_allowed = last_emitted_at
            .checked_add(MIN_FRAME_INTERVAL)
            .unwrap_or(last_emitted_at);
        requested.max(min_allowed)
    }

    pub(super) fn mark_emitted(&mut self, emitted_at: Instant) {
        self.last_emitted_at = Some(emitted_at);
    }
}
```

This is the 120 FPS rate limiter. It's a 28-line pure helper, intentionally
small and easy to test in isolation.

### 4.2 `frame_requester.rs` (~200 lines, the actor)

`FrameRequester` is a **handler-side** of an actor/handler pair with
`FrameScheduler`. The doc comment is explicit about the design:

> Internally it spawns a [`FrameScheduler`] task that coalesces many requests
> into a single notification on a broadcast channel used by the main TUI event
> loop. This follows the actor-style design from "Actors with Tokio" with a
> dedicated scheduler task and lightweight request handles.

```rust
#[derive(Clone, Debug)]
pub struct FrameRequester {
    frame_schedule_tx: mpsc::UnboundedSender<Instant>,
}

impl FrameRequester {
    pub fn new(draw_tx: broadcast::Sender<()>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let scheduler = FrameScheduler::new(rx, draw_tx);
        tokio::spawn(scheduler.run());   // ← actor task spawned here
        Self { frame_schedule_tx: tx }
    }

    pub fn schedule_frame(&self) {
        let _ = self.frame_schedule_tx.send(Instant::now());
    }

    pub fn schedule_frame_in(&self, dur: Duration) {
        let _ = self.frame_schedule_tx.send(Instant::now() + dur);
    }
}
```

`FrameScheduler::run` is a tokio task that:
1. Receives a target draw time on its mpsc.
2. Calls `rate_limiter.clamp_deadline(target)` to enforce 120 FPS.
3. Coalesces multiple targets into the **earliest** deadline.
4. Sleeps until the deadline, then sends one draw notification on the
   broadcast channel.
5. Continues to the next request, recomputing the sleep target so coalesced
   requests fire once.

The full test suite (`tests.rs` in the same file) verifies: single immediate
trigger, delayed trigger, multi-request coalescing, mixed immediate+delayed
coalescing, 120 FPS clamp, no over-clamping future draws, multi-delayed
coalescing to the earliest.

### 4.3 `event_stream.rs` — The EventBroker

This file is where stdin ownership lives. The doc comment is explicit about
the motivation:

> The motivation for dropping/recreating the crossterm event stream is to enable
> the TUI to fully relinquish stdin. If the stream is not dropped, it will
> continue to read from stdin even if it is not actively being polled (due to
> how crossterm's EventStream is implemented), potentially stealing input from
> other processes reading stdin, like terminal text editors. **This race can
> cause missed input or capturing terminal query responses** that the other
> process expects to read.

`EventBroker` is a shared state with three modes:

```rust
enum EventBrokerState<S> {
    Paused,                  // Underlying event source dropped
    Start,                   // New event source will be created on next poll
    Running(S),              // Event source is currently running
}
```

`pause_events()` → `Paused`. `resume_events()` → `Start` (creates source on
next poll). The `TuiEventStream` polls the broker; when paused, it polls a
`watch::Receiver<()>` instead so `resume_events` can wake it.

`TuiEventStream::poll_next` round-robins between draw and crossterm events
("approximate fairness + no starvation via round-robin"). This is the
multiplex that lets the main loop receive both event sources.

### 4.4 `job_control.rs` — Suspend/Resume and Alt-Screen Toggle

`SuspendContext` is a `Clone` type with `Arc`/`Atomic` internals, designed to
be moved into the `'static` event stream without borrowing `self`:

```rust
#[derive(Clone)]
pub struct SuspendContext {
    resume_pending: Arc<Mutex<Option<ResumeAction>>>,
    suspend_cursor_y: Arc<AtomicU16>,
}
```

`suspend(alt_screen_active: &Arc<AtomicBool>)` is the **alt-screen-aware**
suspend:

```rust
pub(crate) fn suspend(&self, alt_screen_active: &Arc<AtomicBool>) -> Result<()> {
    if alt_screen_active.load(Ordering::Relaxed) {
        // Leave alt-screen so the terminal returns to the normal buffer while
        // suspended; also turn off alt-scroll.
        let _ = execute!(stdout(), DisableAlternateScroll);
        let _ = execute!(stdout(), LeaveAlternateScreen);
        self.set_resume_action(ResumeAction::RestoreAlt);
    } else {
        self.set_resume_action(ResumeAction::RealignInline);
    }
    let y = self.suspend_cursor_y.load(Ordering::Relaxed);
    let _ = execute!(stdout(), MoveTo(0, y), Show);
    suspend_process()
}
```

This is the key insight: the **alt-screen flag** (`alt_screen_active: Arc<AtomicBool>`)
determines whether `^Z` exits the alt-screen before yielding to SIGTSTP.
Resume realigns the inline viewport if we were inline, or re-enters alt-screen
if we were in a sub-view.

The atomic boolean is shared with `TuiEventStream` and the `Tui` itself, so
suspend decisions are made from the same source of truth.

### 4.5 `keyboard_modes.rs` and `terminal_stderr.rs`

`keyboard_modes.rs` is the keyboard enhancement flags manager (disambiguates
`Enter` vs `Shift+Enter`, etc.). It pairs with `set_modes()` in `tui.rs`:

```rust
pub fn set_modes() -> Result<()> {
    ensure_virtual_terminal_processing()?;
    execute!(stdout(), EnableBracketedPaste)?;
    enable_raw_mode()?;
    // Enable keyboard enhancement flags so modifiers for keys like Enter
    // are disambiguated. chat_composer.rs is using a keyboard event
    // listener to enter for any modified keys to create a new line that
    // require this. Some terminals (notably legacy Windows consoles) do
    // not support keyboard enhancement flags. Attempt to enable them,
    // but continue gracefully if unsupported.
    keyboard_modes::enable_keyboard_enhancement();
    let _ = execute!(stdout(), EnableFocusChange);
    Ok(())
}
```

`terminal_stderr.rs` is a "stderr guard" that suppresses terminal-interactive
output during inline mode (some host processes leak escape sequences to
stderr that would corrupt the inline viewport).

## 5. `tui.rs` — The Tui Top-Level

**File**: `codex-rs/tui/src/tui.rs` (~1000 lines)

The top-level `Tui` struct is the public surface:

```rust
pub struct Tui {
    frame_requester: FrameRequester,
    draw_tx: broadcast::Sender<()>,
    event_broker: Arc<EventBroker>,
    pub(crate) terminal: Terminal,
    pending_history_lines: Vec<PendingHistoryLines>,
    ambient_pet_image_state: crate::pets::PetImageRenderState,
    pet_picker_preview_image_state: crate::pets::PetImageRenderState,
    alt_saved_viewport: Option<ratatui::layout::Rect>,
    #[cfg(unix)]
    suspend_context: SuspendContext,
    alt_screen_active: Arc<AtomicBool>,
    terminal_focused: Arc<AtomicBool>,
    enhanced_keys_supported: bool,
    notification_backend: Option<DesktopNotificationBackend>,
    notification_condition: NotificationCondition,
    is_zellij: bool,
    alt_screen_enabled: bool,   // ← when false, enter_alt_screen() is a no-op
    _stderr_guard: terminal_stderr::TerminalStderrGuard,
}
```

### 5.1 `init()` — inline-by-default startup

```rust
/// Initialize the terminal (inline viewport; history stays in normal scrollback)
pub(crate) fn init() -> Result<InitializedTerminal> {
    if !stdin().is_terminal() { ... }
    if !stdout().is_terminal() { ... }
    set_modes()?;
    flush_terminal_input_buffer();
    set_panic_hook();
    // ... bounded startup probe for cursor position, default colors, keyboard enhancement
    let tui = CustomTerminal::with_options_and_cursor_position(backend, cursor_pos)?;
    let stderr_guard = terminal_stderr::TerminalStderrGuard::install()?;
    Ok(InitializedTerminal {
        terminal: tui,
        enhanced_keys_supported,
        stderr_guard,
    })
}
```

The doc comment is the canonical statement: **"Initialize the terminal
(inline viewport; history stays in normal scrollback)"**. `init()` does
**not** call `EnterAlternateScreen`. The viewport starts at the user's
current cursor y.

### 5.2 `set_modes` / `restore` / `restore_after_exit` / `restore_keep_raw`

The three restore variants cover the three ways TUI can exit:

```rust
pub fn restore() -> Result<()> {
    restore_common(RawModeRestore::Disable, KeyboardRestore::PopStack)
}

pub fn restore_after_exit() -> Result<()> {
    let mut first_error =
        restore_common(RawModeRestore::Disable, KeyboardRestore::ResetAfterExit).err();
    if let Err(err) = terminal_stderr::finish() {
        first_error.get_or_insert(err);
    }
    match first_error { Some(err) => Err(err), None => Ok(()) }
}

pub fn restore_keep_raw() -> Result<()> {
    restore_common(RawModeRestore::Keep, KeyboardRestore::PopStack)
}
```

`restore_after_exit` is the "we're really done" path: it does a stronger
keyboard reset (`ResetAfterExit`) so the parent shell recovers even if the
terminal missed the stack pop. It's called from the panic hook.

The `restore_common` does the inverse of `set_modes`: virtual terminal
processing, keyboard enhancement stack pop (or reset), bracketed paste off,
focus change off, raw mode off, cursor style reset + show.

### 5.3 `enter_alt_screen` / `leave_alt_screen` — opt-in sub-views

```rust
pub fn enter_alt_screen(&mut self) -> Result<()> {
    if !self.alt_screen_enabled {
        return Ok(());   // ← no-op when disabled
    }
    let _ = execute!(self.terminal.backend_mut(), EnterAlternateScreen);
    let _ = execute!(self.terminal.backend_mut(), EnableAlternateScroll);
    if let Ok(size) = self.terminal.size() {
        self.alt_saved_viewport = Some(self.terminal.viewport_area);
        self.terminal.set_viewport_area(Rect::new(0, 0, size.width, size.height));
        let _ = self.terminal.clear();
    }
    self.alt_screen_active.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn leave_alt_screen(&mut self) -> Result<()> {
    if !self.alt_screen_enabled {
        return Ok(());   // ← no-op when disabled
    }
    let _ = execute!(self.terminal.backend_mut(), DisableAlternateScroll);
    let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    if let Some(saved) = self.alt_saved_viewport.take() {
        self.terminal.set_viewport_area(saved);
    }
    self.alt_screen_active.store(false, Ordering::Relaxed);
    Ok(())
}
```

`alt_screen_enabled` defaults to `true` but is **explicitly settable**:

```rust
pub fn set_alt_screen_enabled(&mut self, enabled: bool) {
    self.alt_screen_enabled = enabled;
}
```

The `alt_saved_viewport` is the inline viewport rectangle; when alt-screen is
entered, the viewport is expanded to the full screen, and when leaving, the
inline viewport is restored. The `alt_screen_active: Arc<AtomicBool>` is shared
with `SuspendContext` and `TuiEventStream` so suspend and event mapping
decisions see the same source of truth.

### 5.4 `insert_history_lines` — the public TUI API

```rust
pub fn insert_history_lines(&mut self, lines: Vec<Line<'static>>) {
    self.insert_history_lines_with_wrap_policy(lines, HistoryLineWrapPolicy::PreWrap);
}

pub fn insert_history_lines_with_wrap_policy(
    &mut self,
    lines: Vec<Line<'static>>,
    wrap_policy: HistoryLineWrapPolicy,
) {
    self.insert_history_hyperlink_lines_with_wrap_policy(
        plain_hyperlink_lines(lines),
        wrap_policy,
    );
}

pub(crate) fn insert_history_hyperlink_lines_with_wrap_policy(
    &mut self,
    lines: Vec<HyperlinkLine>,
    wrap_policy: HistoryLineWrapPolicy,
) {
    if lines.is_empty() {
        return;
    }
    if let Some(last) = self.pending_history_lines.last_mut()
        && last.wrap_policy == wrap_policy
    {
        last.lines.extend(lines);   // ← coalesce with same-policy batch
    } else {
        self.pending_history_lines
            .push(PendingHistoryLines { lines, wrap_policy });
    }
    self.frame_requester().schedule_frame();
}
```

The pattern is **batch, coalesce, schedule**. Cells call
`insert_history_lines(lines)`; the TUI buffers them in
`pending_history_lines`, coalesces same-wrap-policy batches, and asks for a
frame. On the next frame, `flush_pending_history_lines` calls
`crate::insert_history::insert_history_hyperlink_lines_with_mode_and_wrap_policy`
with the correct Zellij/Standard mode, then clears the buffer.

### 5.5 `with_restored` — running external programs (e.g. `$EDITOR`)

```rust
pub async fn with_restored<R, F, Fut>(&mut self, mode: RestoreMode, f: F) -> R
where F: FnOnce() -> Fut, Fut: Future<Output = R>,
{
    // 1. Pause crossterm events to avoid stdin conflicts.
    self.pause_events();

    // 2. Leave alt screen if active to avoid conflicts with external program.
    let was_alt_screen = self.is_alt_screen_active();
    if was_alt_screen {
        let _ = self.leave_alt_screen();
    }

    // 3. Restore terminal modes (Full = also disable raw mode, KeepRaw = keep raw).
    if let Err(err) = mode.restore() { ... }
    if let Err(err) = terminal_stderr::pause() { ... }

    // 4. Run the external program.
    let output = f().await;

    // 5. Re-suppress stderr, re-enable TUI modes, flush input buffer.
    if let Err(err) = terminal_stderr::resume() { ... }
    if let Err(err) = set_modes() { ... }
    flush_terminal_input_buffer();

    // 6. Re-enter alt screen if we left it.
    if was_alt_screen {
        let _ = self.enter_alt_screen();
    }

    self.resume_events();
    output
}
```

This is the safe way to hand off the terminal to a child process. It pauses
the crossterm EventStream (so the child owns stdin), restores terminal modes
so the child can use the terminal normally, and re-applies TUI modes on
return. The alt-screen state is round-tripped so sub-views survive the handoff.

### 5.6 `draw` — the synchronized update wrapper

```rust
pub fn draw(
    &mut self,
    height: u16,
    draw_fn: impl FnOnce(&mut custom_terminal::Frame),
) -> Result<()> {
    // 1. Prepare resume action (if ^Z was pressed).
    #[cfg(unix)]
    let mut prepared_resume = self.suspend_context
        .prepare_resume_action(&mut self.terminal, &mut self.alt_saved_viewport);

    // 2. Precompute any viewport updates that need a cursor-position query
    //    before entering the synchronized update.
    let mut pending_viewport_area = self.pending_viewport_area()?;

    ensure_virtual_terminal_processing()?;

    // 3. Wrap the entire draw in a SynchronizedUpdate so the terminal
    //    applies all changes atomically (no flicker).
    stdout().sync_update(|_| {
        // 4. Apply resume action.
        if let Some(prepared) = prepared_resume.take() {
            prepared.apply(&mut self.terminal)?;
        }

        // 5. Apply pending viewport area (resize-driven).
        let terminal = &mut self.terminal;
        if let Some(new_area) = pending_viewport_area.take() {
            terminal.set_viewport_area(new_area);
            terminal.clear()?;
        }

        let size = terminal.size()?;
        let mut area = terminal.viewport_area;
        area.height = height.min(size.height);
        area.width = size.width;
        if area.bottom() > size.height {
            terminal.backend_mut()
                .scroll_region_up(0..area.top(), area.bottom() - size.height)?;
            area.y = size.height - area.height;
        }
        if area != terminal.viewport_area {
            clear_for_viewport_change(terminal, area)?;
            terminal.set_viewport_area(area);
        }

        // 6. Flush pending history lines INTO the scrollback.
        Self::flush_pending_history_lines(
            terminal,
            &mut self.pending_history_lines,
            self.is_zellij,
        )?;

        // 7. Track inline cursor y for ^Z.
        #[cfg(unix)] { ... self.suspend_context.set_cursor_y(...); }

        // 8. Draw the active viewport (with draw_fn).
        terminal.draw(|frame| { draw_fn(frame); })
    })?
}
```

The `SynchronizedUpdate` wrap means the terminal applies the entire frame
atomically. The history flush happens **inside** the synchronized update, so
the user sees the history row appear and the viewport reflow together, with
no intermediate state.

`draw_with_resize_reflow` is the feature-gated variant that skips the legacy
viewport-scroll path and lets transcript reflow rebuild the scrollback before
the frame is rendered. It's the path used when the user resizes the terminal.

## 6. `chatwidget/` — The Conversation Orchestrator

**Files**: `codex-rs/tui/src/chatwidget/` (mod + 11 submodules)

The chat widget is **the renderer of the active viewport**: it consumes the
cell stream, lays out cells vertically, scrolls, and dispatches events. It
talks to `Tui` via `tui.frame_requester().schedule_frame()` and
`tui.insert_history_lines(lines)`.

The pattern is:

1. Agent event arrives (text delta, tool call, tool result, turn end, etc.)
2. ChatWidget updates its `cells: Vec<Box<dyn HistoryCell>>` and
   `active_cell: Option<Box<dyn HistoryCell>>`.
3. If the active cell finalized (turn end), ChatWidget calls
   `tui.insert_history_lines(cell.display_lines(width))` to push it into the
   scrollback.
4. ChatWidget asks for a redraw: `frame_requester.schedule_frame()`.
5. The next frame, the TUI flushes pending history lines into the scrollback
   (via `insert_history::insert_history_lines`) and renders the active cell
   into the inline viewport.

## 7. `bottom_pane/` — The Composer

**Files**: `codex-rs/tui/src/bottom_pane/` (mod + multiple submodules)

The bottom pane is the **inline viewport content**. It hosts the chat
composer (multi-line input, history, @-mention file search, $/-mention app
references), the approval overlay, the slash command popup, and the popup
stack (model picker, theme picker, keymap remapper, etc.).

In Codex, the "bottom pane" is **the inline viewport rectangle** — exactly
the bottom of the screen, height negotiated with the TUI. When the bottom
pane is showing, the inline viewport is filled with the composer + status
indicators; when it's hidden (during streaming), the viewport is collapsed
to a single line and the active cell's tail occupies the rest.

## 8. `slash_command.rs` — The Command Framework

**File**: `codex-rs/tui/src/slash_command.rs` (~200 lines)

Codex has 50+ slash commands, defined as a single `enum`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Model, Ide, Permissions, Keymap, Vim, ElevateSandbox, SandboxReadRoot,
    Experimental, AutoReview, Memories, Skills, Hooks, Review, Rename, New,
    Archive, Resume, Fork, App, Init, Compact, Plan, Goal, Agent, Side,
    Btw, Copy, Raw, Diff, Mention, Status, DebugConfig, Title, Statusline,
    Theme, Pets, Mcp, Apps, Plugins, Logout, Quit, Exit, Feedback, Rollout,
    Ps, Stop, Clear, Personality, Realtime, Settings, TestApproval, MultiAgents,
    MemoryDrop, MemoryUpdate,
}
```

The framework is **enum-driven**, not trait-object driven. Each command has:

- `description() -> &'static str` — user-visible description
- `command() -> &'static str` — kebab-case command name (default serialization)
- `supports_inline_args() -> bool` — does this command take args after `/`?
- `available_in_side_conversation() -> bool` — `/side`-context only?
- `available_during_task() -> bool` — can run while a turn is in progress?
- `is_visible() -> bool` — conditional visibility (debug-only, macOS-only, etc.)

The presentation order is **the enum order**; do-not-alpha-sort is a
deliberate comment. `built_in_slash_commands()` filters by `is_visible` and
returns `(name, command)` pairs for the popup.

## 9. `keymap.rs` — Context-Aware Keybindings

**File**: `codex-rs/tui/src/keymap.rs` (~1100 lines)

The keymap is a context-aware dispatch system. A `KeyEvent` is matched
against the **current context** (App, Chat, Composer, Editor, Vim, Pager,
List, Approval) and translated into a `KeyAction`. The composer supports
Vim mode (toggleable via `/vim`).

The full context set is:

| Context | When |
|---|---|
| `App` | Top-level, no specific view |
| `Chat` | Chat viewport is focused |
| `Composer` | Input line is focused |
| `Editor` | External editor is open (e.g. for `/review` arguments) |
| `Vim` | Vim mode is active in the composer |
| `Pager` | Pager overlay is visible |
| `List` | A list picker is visible |
| `Approval` | Approval overlay is visible |

## 10. Mapping Codex Architecture to Talos Current State

Talos's TUI as of I014 (2026-06-06):

| Talos file | Lines | Codex equivalent | Notes |
|---|---:|---|---|
| `app.rs` | 625 | `tui.rs` + `app.rs` (Codex) + `chatwidget.rs` (Codex) | Talos is one god-module; Codex splits into 3 |
| `state.rs` | 580 | `chatwidget.rs` (Codex) + slice of `tui.rs` | Talos is one god-module; ChatWidget + Tui split |
| `widgets.rs` | 305 | `history_cell/{base,messages,exec,approvals,patches,…}.rs` | Talos has flat widgets; Codex has per-type cells |
| `lib.rs` | 24 | (no direct eq) | Talos public API; ~20 lines of re-exports |
| `sidebar.rs` | 168 | `bottom_pane/` slice (sidebar may be in a different module) | Talos has skill sidebar; Codex has no equivalent |
| `clipboard.rs` | 184 | `clipboard_copy.rs` + `clipboard_paste.rs` | Talos merged; Codex split |
| `evolution.rs` | 141 | (Codex has no equivalent — `codex_otel` runtime metrics is separate) | Talos-specific |
| `export.rs` | 148 | (Codex has `/rollout` instead, no `/export` to file) | Talos-specific |
| `theme.rs` | (skipped, presumed ~200) | `style.rs` + `theme_picker.rs` | Talos has fixed Nord; Codex has picker |
| `tests.rs` | (skipped) | `chatwidget/tests.rs` + 11 per-cell `tests.rs` | Talos has 1 test module; Codex has per-cell snapshot tests |

Total: 11 files, ~2,200 lines (estimated from known).

### 10.1 The architecture gaps

| Concern | Codex approach | Talos current state | Gap |
|---|---|---|---|
| **Inline viewport** | `custom_terminal.rs` + `insert_history.rs` | `app.rs:60` calls `EnterAlternateScreen` unconditionally | **Critical** — wrong model, all content discarded on exit |
| **History cells** | `history_cell/` with 14 cell types + trait | Flat `widgets.rs::ToolCallBubble` + `widgets.rs::ApprovalOverlay` | Major — 1 widget per type vs 1 trait + 14 cells |
| **Frame rate limiting** | `frame_rate_limiter.rs` + `frame_requester.rs` actor | Hard-coded `Duration::from_millis(50)` interval in `app.rs:86, 144` | Significant — no coalescing, no 120 FPS cap |
| **EventBroker stdin** | `event_stream.rs` with `pause_events`/`resume_events` | Plain `EventStream::new()` in `app.rs:85, 143`; no pause/resume | Significant — blocks `$EDITOR` integration |
| **Bottom pane / composer** | `bottom_pane/` (multi-line, @-mentions, $-mentions, popup stack) | Single-line input in `state.rs:171-220` (`input_buffer: String`) | Significant — no multi-line, no mention search |
| **Slash commands** | 50+ commands via `strum` enum + `built_in_slash_commands()` | 13 commands via `SLASH_COMMANDS` const slice + match in `state.rs:316-372` | Moderate — needs `enum` + descriptor trait |
| **Alt-screen management** | `enter_alt_screen`/`leave_alt_screen` opt-in via `alt_screen_enabled: bool` | Unconditional `EnterAlternateScreen` at `app.rs:60` | **Critical** — must become opt-in |
| **Suspend/resume** | `tui/job_control.rs` with `SuspendContext`, `ResumeAction`, `PreparedResumeAction` | None | Moderate — needed if we want `^Z` to work |
| **Keyboard enhancement** | `tui/keyboard_modes.rs` | `crossterm::event::KeyEvent` (no enhancement) | Moderate — blocks `Enter` vs `Shift+Enter` disambiguation |
| **Terminal stderr guard** | `tui/terminal_stderr.rs` | None | Minor — needed if a child process leaks escape codes |
| **Zellij scroll-region bug** | `InsertHistoryMode::ZellijRaw` path in `insert_history.rs` | None | Minor — needed for Zellij users |
| **Public API of `talos-tui`** | n/a | `lib.rs` re-exports 5 types: `Tui`, `SkillInfo`, `SkillSidebar`, `ApprovalState`, `nord` | Stable for now; refactor must not break it |

### 10.2 The "inline-by-default" rewrite impact

Switching from `EnterAlternateScreen` to inline mode touches:

| Touch point | Current line | New behavior |
|---|---|---|
| `Tui::new()` `app.rs:60` | `execute!(stdout, EnterAlternateScreen, EnableMouseCapture)` | `set_modes()` only (raw mode + bracketed paste + keyboard enhancement); no `EnterAlternateScreen` |
| `Tui::Drop` `app.rs:614-618` | `restore_terminal()` | `restore_after_exit()` (stronger keyboard reset) |
| `restore_terminal()` `app.rs:621-625` | `disable_raw_mode` + `LeaveAlternateScreen` + `DisableMouseCapture` | `set_modes` inverse; `LeaveAlternateScreen` only if `alt_screen_active` |
| `Tui::run` `app.rs:84-131` | Whole-screen `Layout::vertical` rendering | Bottom-only `TuiState` projection; cell stream appends to scrollback via `insert_history_lines` |
| `state::transcript_plain_text` `state.rs:477-503` | `chat_lines: Vec<ChatLine>` → String | Unchanged (the source of truth for `/copy all` and `/export`) |
| `state::handle_slash_command` `state.rs:316-372` | Pushes system messages to `chat_lines` | Unchanged (these become `Text` cells; `insert_history_lines` flushes) |
| `state::handle_event` `state.rs:282-309` | Appends to `chat_lines` | Replace with **cell stream**: `insert_history_lines` for committed; `Box<dyn HistoryCell>` for active |

The "transcript dump on exit" story (TUI-003) is **dissolved** by this
refactor. There is no transcript to dump — the scrollback already has it.

## 11. References

### Code

- [`codex-rs/tui/src/tui.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui.rs) — top-level Tui, init/draw/insert_history_lines/alt_screen/with_restored
- [`codex-rs/tui/src/custom_terminal.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/custom_terminal.rs) — inline-viewport Terminal
- [`codex-rs/tui/src/insert_history.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/insert_history.rs) — streaming scrollback push
- [`codex-rs/tui/src/history_cell/mod.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/history_cell/mod.rs) — `HistoryCell` trait
- [`codex-rs/tui/src/history_cell/base.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/history_cell/base.rs) — `PlainHistoryCell`, `PrefixedWrappedHistoryCell`, `CompositeHistoryCell`
- [`codex-rs/tui/src/chatwidget/`](https://github.com/openai/codex/tree/main/codex-rs/tui/src/chatwidget) — ChatWidget + cell stream orchestrator
- [`codex-rs/tui/src/bottom_pane/`](https://github.com/openai/codex/tree/main/codex-rs/tui/src/bottom_pane) — composer, popups, approval overlay
- [`codex-rs/tui/src/tui/event_stream.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui/event_stream.rs) — `EventBroker` stdin
- [`codex-rs/tui/src/tui/frame_requester.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui/frame_requester.rs) — frame-rate-limited redraw actor
- [`codex-rs/tui/src/tui/frame_rate_limiter.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui/frame_rate_limiter.rs) — 120 FPS clamp
- [`codex-rs/tui/src/tui/job_control.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui/job_control.rs) — SIGTSTP + alt-screen toggle
- [`codex-rs/tui/src/tui/keyboard_modes.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui/keyboard_modes.rs) — keyboard enhancement flags
- [`codex-rs/tui/src/tui/terminal_stderr.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui/terminal_stderr.rs) — stderr suppression
- [`codex-rs/tui/src/keymap.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/keymap.rs) — context-aware keymap (8 contexts)
- [`codex-rs/tui/src/slash_command.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/slash_command.rs) — 50+ command enum
- [`codex-rs/tui/src/oss_selection.rs`](https://github.com/openai/codex/blob/main/codex-rs/tui/src/oss_selection.rs) — alt-screen selection widget (sub-view)

### Talos counterparts

- `docs/reference/REFERENCE-PROJECTS.md` §687-741 — TUI section in the reference list (this doc supersedes its "Full-screen ratatui TUI" framing)
- `docs/proposals/tui-codex-overhaul.md` — Talos TUI refactor proposal (revised framing)
- `docs/backlog/active/TUI-002-codex-overhaul.md` — Talos TUI refactor backlog item (revised)
- `docs/backlog/active/TUI-003-tui-exit-transcript.md` — supersede: inline-by-default refactor subsumes this
- `docs/iterations/I014-tui-completion.md` — most recent TUI iteration (alt-screen baseline)
- `docs/decisions/003-tui-progressive-evolution.md` — TUI evolution anchor
- `docs/decisions/005-tui-event-architecture.md` — TUI event architecture boundary
- `docs/decisions/006-event-architecture-boundary.md` — single-mpsc bus contract
- `crates/talos-tui/src/app.rs:50-71` — current Tui::new (the alt-screen + raw mode init that must change)
- `crates/talos-tui/src/app.rs:614-625` — current Tui::Drop + restore_terminal
- `crates/talos-tui/src/state.rs:477-503` — transcript_plain_text / transcript_markdown (reused by inline mode)
- `crates/talos-tui/src/widgets.rs:36-131` — ToolCallBubble widget (becomes ToolCallHistoryCell in the new model)
- `crates/talos-tui/src/widgets.rs:140-235` — ApprovalOverlay widget (becomes ApprovalHistoryCell + bottom_pane modal)
