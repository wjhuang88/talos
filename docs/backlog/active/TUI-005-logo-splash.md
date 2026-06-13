# TUI-005: Logo & Splash Screen

| Field | Value |
|-------|-------|
| ID | TUI-005 |
| Title | TUI Logo & Splash Screen |
| Priority | P2 |
| Status | Complete (in-scope splash); overlay permanently deferred per [ADR-019](../../decisions/019-tui-splash-scrollback-boundary.md) |
| Depends on | I023 (TUI state model — Complete), I022 (inline-by-default — Complete) |
| Blocks | None (TUI-001 completion was delivered via I014) |
| Owner | `crates/talos-tui/src/` |

## Outcome

User sees a branded, product-quality splash on startup: a scrollback-only `TALOS`
block wordmark in Nord Frost colors, a short tagline, capability badges, version text,
a narrow-terminal fallback, and stable startup line boundaries. The splash is printed
before raw mode, remains in native terminal scrollback, preserves the blank line before
the logo, and never renders as a viewport overlay.

## Motivation

Current `print_banner()` is a 3-line plain-text print (`🛡 Talos v0.x` + slogan + newline). It:
- Has no brand identity — emoji + plain text is not recognizable
- Cannot use Nord theme colors (printed before raw mode)
- Overlaps with viewport if cursor is near screen bottom (I022 padding logic compensates but banner itself is unstyled)
- No visual hierarchy — looks like a debug print, not a product

## Historical Design (Superseded)

The original design below proposed a ratatui Canvas helmet, viewport overlay,
auto-dismiss lifecycle, and reusable `LogoWidget`. That plan is superseded by the
scrollback-only boundary in [ADR-019](../../decisions/019-tui-splash-scrollback-boundary.md).
Do not implement the overlay path unless a future ADR reopens it with a real runtime
readiness data source and concrete viewport consumer.

### Architecture Constraint: Inline-by-Default Model

The splash must work **within** the inline-by-default TUI model (I022). It cannot:
- Switch to alt-screen for the splash then switch back (violates ADR-018)
- Block the conversation view indefinitely
- Require a full-screen animation loop that delays user input

Therefore the splash is a **short-lived overlay** that renders inside the existing inline viewport area for ~2 seconds, then collapses to the fixed 4-line viewport. The helmet + typography is printed to stdout scrollback (like current `print_banner()`), but with **styled ANSI output** (Nord colors via crossterm `SetForegroundColor`), and the status section renders briefly in the viewport before auto-dismissing.

### Rendering Layers

```text
┌─────────────────────────────────────────────┐
│                                             │
│        [ Canvas: Helmet Outline ]           │ ← ratatui Canvas, 6-8 lines
│                                             │
│              T A L O S                      │ ← ratatui Text, Frost/Snow colors
│                                             │
│        Bronze Guardian Runtime              │ ← ratatui Text, Bronze color
│                                             │
│  [Precision] [Safety] [Reliability]         │ ← ratatui Line, colored badges
│                                             │
│  v0.x • Rust Edition 2024                   │ ← version info
│                                             │
├─────────────────────────────────────────────┤
│ [✓] Agent Runtime     [✓] Plugin Manager   │ ← brief status, auto-dismiss
│ [✓] Event Bus         [✓] State Store      │
├─────────────────────────────────────────────┤
│ > _                                         │ ← input line (after splash dismiss)
│ Ready • v0.x                                │ ← status bar
└─────────────────────────────────────────────┘
```

### Layer 1: Canvas Helmet

Use `ratatui::widgets::canvas::Canvas` to render a geometric guardian helmet outline.

**What Canvas can do:**
- Lines (arbitrary float coordinates → terminal cell resolution)
- Rectangles (outline only, no fill)
- Circles (outline only, no fill)
- Points (colored dots)
- Custom Shape trait (manual point-by-point painting)
- Resolution: Braille marker = 2×4 per cell (e.g. 80×24 terminal → 160×96 dots)
- HalfBlock marker = 1×2 per cell with **dual color per cell** (fg upper / bg lower)
- Layering via `ctx.layer()`

**What Canvas CANNOT do:**
- No filled shapes (no solid rectangles, circles, polygons)
- No arcs or partial circles (Circle always draws full 360°)
- No curves (only straight Bresenham lines)
- No gradients, anti-aliasing, or stroke width
- No image/bitmap import

**Helmet design approach:**
- Symmetrical side-view silhouette of a bronze guardian helmet
- Use `Marker::Braille` for finest detail (2×4 per cell)
- 5-10 `Canvas::Line` elements forming the outline:
  - Crest ridge (triangle peak)
  - Face guard (horizontal bar)
  - Cheek plates (angled lines)
  - Neck guard (angled back line, approximating curve with short line segments)
  - Nose bridge (vertical center line)
- Bronze color (`#d08770`) for all lines
- Gold accent (`#ebcb8b`) for the crest ridge peak
- Keep it **simple** — 10 lines max, recognizable at 80-col terminal width
- Outline-only is intentional and correct — Canvas has no fill primitives
- For "filled" appearance in future, implement custom `Shape` with scanline fill

**Fallback**: If Canvas resolution is insufficient at narrow terminals (< 80 cols), render a simplified Unicode block-character version using `█▄▀░▓▒` instead. The widget detects terminal width and chooses Canvas vs Unicode block mode.

### Layer 2: Typography

```rust
// "TALOS" — centered, wide-spaced, Frost Blue
Line::from(Span::styled(
    "T A L O S",
    Style::default()
        .fg(Color::Rgb(136, 192, 208))  // Frost Blue #88c0d0
        .add_modifier(Modifier::BOLD),
))

// "Bronze Guardian Runtime" — centered, Bronze
Line::from(Span::styled(
    "Bronze Guardian Runtime",
    Style::default()
        .fg(Color::Rgb(208, 135, 112))  // Bronze #d08770
        .add_modifier(Modifier::ITALIC),
))
```

Both use `ratatui::text::Text<'static>` — no Canvas needed.

### Layer 3: Status Badges

```rust
// Three pill-shaped badges with colored backgrounds
Line::from(vec![
    Span::styled(" Precision ", Style::default().fg(Color::Rgb(136, 192, 208)).add_modifier(Modifier::BOLD)),  // Cyan
    Span::raw("  "),
    Span::styled(" Safety ", Style::default().fg(Color::Rgb(163, 190, 140)).add_modifier(Modifier::BOLD)),     // Green
    Span::raw("  "),
    Span::styled(" Reliability ", Style::default().fg(Color::Rgb(180, 138, 173)).add_modifier(Modifier::BOLD)), // Purple
])
```

### Layer 4: Subsystem Readiness (Auto-dismiss)

Brief status indicators shown in viewport for 2 seconds:
- `[✓] Agent Runtime`
- `[✓] Plugin Manager`
- `[✓] Event Bus`
- `[✓] State Store`

These render as `ChatMessage` (role=System, status=Completed) via I023 state model, with auto-expire via `Tip` TTL mechanism.

### Startup Sequence

```text
Phase 1 (0ms):   print_splash_scrollback() — styled ANSI helmet + typography to stdout
                  (uses crossterm colors, before raw mode)

Phase 2 (enter raw mode):
                  InlineTerminal::new() — padding logic ensures viewport fits

Phase 3 (first frame):
                  viewport renders status badges + subsystem readiness
                  (4-line viewport, reuses existing draw infrastructure)

Phase 4 (2s or first keypress):
                  splash dismisses — viewport transitions to normal input+status
                  emits TuiStateEvent::SplashComplete (via I023 event-bus hook)
```

**Phase 1 is the key architectural change**: Replace `print_banner()` (plain text) with `print_splash_scrollback()` (styled ANSI). This happens **before** raw mode, so it becomes part of the terminal's native scrollback — matching the I022 inline-by-default model.

## Widget Structure

```rust
/// Renders the Canvas helmet + typography as a ratatui widget.
/// Used both in splash viewport (Phase 3) and future alt-screen sub-views.
pub struct LogoWidget {
    width: u16,
    mode: LogoRenderMode,  // Canvas or UnicodeBlock
}

pub enum LogoRenderMode {
    Canvas,        // ratatui Canvas lines (>= 80 cols)
    UnicodeBlock,  // █▄▀░ characters (< 80 cols)
}

/// Prints styled ANSI splash to stdout (Phase 1, before raw mode).
/// Returns the number of lines printed (for InlineTerminal padding calculation).
fn print_splash_scrollback() -> u16;

/// Renders splash status section in viewport (Phase 3).
/// Returns a ratatui Block/Paragraph widget for the status area.
fn build_splash_status(state: &TuiState) -> Paragraph<'static>;
```

## Acceptance Criteria

### AC-1: Styled Splash in Scrollback

**Given** user launches `cargo run -p talos-cli`
**When** TUI starts
**Then** terminal scrollback shows:
- A colored helmet outline (bronze lines on Nord dark background)
- `T A L O S` in Frost Blue (#88c0d0), bold
- `Bronze Guardian Runtime` in Bronze (#d08770)
- Three colored pill badges (Precision/Safety/Reliability)
- Version string
**And** no plain `🛡 Talos v0.x` banner text remains

### AC-2: Auto-dismiss

**Given** splash is displayed
**When** 2 seconds elapsed OR user presses any key
**Then** viewport transitions to normal input+status view
**And** splash content remains in scrollback (not erased)
**And** `TuiStateEvent::SplashComplete` is emitted (if event_tx is set)

### AC-3: Canvas Helmet at Normal Width

**Given** terminal width >= 80 columns
**When** splash renders
**Then** helmet outline uses `ratatui::widgets::canvas::Canvas` with `Line` shapes
**And** helmet is symmetrical and recognizable as a guardian helmet silhouette

### AC-4: Unicode Block Fallback at Narrow Width

**Given** terminal width < 80 columns
**When** splash renders
**Then** helmet uses Unicode block characters (`█▄▀░▓▒`) instead of Canvas
**And** helmet remains recognizable (simplified silhouette)

### AC-5: No Alt-screen Violation

**Given** TUI starts
**When** splash displays and dismisses
**Then** no `EnterAlternateScreen`/`LeaveAlternateScreen` is called
**And** terminal scrollback is preserved (per I022 inline-by-default model)

### AC-6: No Input Delay

**Given** splash is displayed
**When** user types immediately (within 2s window)
**Then** input is processed, splash dismisses, no keystrokes lost

### AC-7: Widget Reusability

**Given** future alt-screen sub-views (help panel, plugin dashboard)
**When** developer wants to show the logo
**Then** `LogoWidget` can be embedded in any ratatui layout without splash-specific logic

## Dependencies

| Dependency | Type | Notes |
|-----------|------|-------|
| I023 (TUI state model) | Hard | `LogoWidget` should emit `TuiStateEvent::SplashComplete`; subsystem status uses `Tip` TTL; splash content stored as `ChatMessage` |
| I022 (inline-by-default) | Hard | Splash must not violate inline-by-default model; no alt-screen |
| ratatui Canvas | Soft | Canvas is available in ratatui 0.30; resolution limits may require Unicode block fallback |
| crossterm `SetForegroundColor` | Soft | Used in Phase 1 for styled ANSI output before raw mode |

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Canvas resolution insufficient for helmet detail | Visual quality | Unicode block fallback for narrow terminals; keep helmet design simple (10 lines max) |
| ANSI color codes don't render in some terminals | Brand fidelity | Test on minimal terminals (xterm, kitty, Windows Terminal); provide `--no-color` fallback |
| Splash delay frustrates power users | UX | 2s max + immediate dismiss on any keypress; make splash duration configurable in config.toml |
| Helmet silhouette unrecognizable at small sizes | Brand identity | Simplify design; add `TALOS` typography as primary identity, helmet as secondary accent |

## Required Reads

- `docs/iterations/I022-tui-inline-default.md` — inline-by-default architecture (no alt-screen)
- `docs/backlog/active/TUI-004-state-model.md` — `ChatMessage`/`Tip`/`TuiStateEvent` model
- `docs/iterations/I023-tui-state-model.md` — I023 iteration plan
- `crates/talos-tui/src/app.rs:499-505` — current `print_banner()`
- `crates/talos-tui/src/inline_terminal.rs:86-94` — viewport padding logic
- `docs/decisions/018-tui-job-control-unsafe.md` — ADR-018 (no alt-screen violation)
- ratatui `widgets::canvas` module docs — Canvas API capabilities

## Scope Boundary

**In scope:**
- Styled ANSI splash to stdout (replacing `print_banner()`)
- `LogoWidget` (Canvas helmet + typography)
- Splash auto-dismiss (2s TTL + keypress)
- Unicode block fallback for narrow terminals
- `TuiStateEvent::SplashComplete` emission (via I023 hook)
- `print_splash_scrollback()` returns line count for InlineTerminal padding

**Out of scope:**
- Animated helmet glow (future enhancement)
- Runtime statistics panel in splash (future enhancement)
- Agent count indicator (future enhancement)
- Alt-screen views (separate backlog item)
- Splash duration configuration in config.toml (future enhancement)
- Startup subsystem check animation (requires state model to have actual subsystem readiness; defer until I023 + runtime integration) — **permanently deferred per [ADR-019](../../decisions/019-tui-splash-scrollback-boundary.md)** (scrollback-only splash; no viewport overlay)

## Sub-slice A: Styled Scrollback Splash Foundation — Landed (2026-06-13)

Implemented the Phase 1 styled ANSI scrollback output:

- **`crates/talos-tui/src/splash.rs`** — new module:
  - `LogoRenderMode` enum (Canvas / UnicodeBlock) with `select_render_mode(width)` threshold logic (≥80 → Canvas, <80 → UnicodeBlock)
  - `print_splash_scrollback()` — Phase 1 styled ANSI output to stdout before raw mode

### Logo redesign (2026-06-13)

The first cut used a crude `/\|`-drawn helmet plus spaced `T A L O S` letters, which
read as a debug print. It was replaced with a product-quality block wordmark:

- Wide (≥80 cols): 6-row ANSI Shadow `TALOS` block wordmark (42 cols, byte-aligned),
  painted with a vertical Nord Frost gradient (NORD10 → NORD9 → NORD8 → NORD7).
- Narrow (<80 cols): 4-row compact block `TALOS` fallback (~26 cols).
- Subtitle `⬡ The watchman never sleeps` (bronze italic) — a myth wink (Talos'
  tireless patrol) that stays language-agnostic to match the general-purpose
  agent-runtime positioning. Badges `Precision · Safety · Reliability` with dimmed
  `·` separators, and a brighter Frost-Blue (NORD9) version line `v0.1.0`.
- Whole splash is left-aligned with a fixed 2-space indent.
- Render-mode selection, gradient, wordmark alignment (42-col invariant), fallback
  width, badge separators, reserved-symbol exclusion, and CRLF line-boundary behavior
  are covered by tests.

- **`crates/talos-tui/src/lib.rs`** — added `mod splash;`
- **`crates/talos-tui/src/app.rs`** — removed old plain-text `print_splash_scrollback()`, now delegates to `crate::splash::print_splash_scrollback()`

Verification: `cargo fmt --all --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, `scripts/validate_project_governance.sh .` all pass.

Correction (2026-06-14): a runtime check showed the first wordmark row could inherit the
previous command's terminal column when launched through `cargo run`, causing the first
row to appear far to the right of the rest of the splash. The fix makes splash output
start with an explicit CRLF, preserves the intended blank line before the logo, and routes
rendering through a testable writer function.

## Closeout (2026-06-13)

The in-scope deliverable — a branded, inline-safe startup splash printed to terminal
scrollback — is delivered and verified. The Phase 3-4 viewport overlay (subsystem
readiness badges + 2s auto-dismiss) is **permanently deferred per
[ADR-019](../../decisions/019-tui-splash-scrollback-boundary.md)** (scrollback-only
splash; no viewport overlay). It is the only remaining TUI-005 item and is recorded
here rather than implemented in this slice.

Acceptance status for the delivered scrollback-splash scope:
- AC-1 (styled splash in scrollback): met — wordmark, gradient, subtitle, badges, version.
- AC-3 (helmet at normal width): superseded — the crude Canvas helmet was replaced by
  the `TALOS` block wordmark, which carries the brand identity more cleanly.
- AC-4 (narrow-width fallback): met — compact block wordmark under 80 columns.
- AC-5 (no alt-screen violation): met — scrollback-only, inline-by-default preserved.
- AC-7 (widget reusability): not applicable to the accepted scrollback-only scope; do
  not claim `LogoWidget` exists unless a future ADR/backlog item reopens viewport logo
  rendering.
- AC-2 / AC-6 (overlay auto-dismiss / no input delay): not applicable to the
  scrollback-only path; they re-open only if the deferred Phase 3-4 overlay is built.

Runtime evidence: the user ran the real `talos` binary on 2026-06-14 and found a
first-row column drift bug when launched through `cargo run`. The corrected implementation
now starts splash output on a fresh CRLF boundary before any styled content.

Verification re-run in the current tree on 2026-06-13: `cargo fmt --all --check`,
`cargo check --workspace`, `cargo clippy --workspace -- -D warnings`,
`cargo test --workspace`, and
`scripts/validate_project_governance.sh .` (0 warnings) all pass.

Correction verification re-run in the current tree on 2026-06-14: `cargo fmt --all --check`,
`cargo check --workspace`, `cargo clippy --workspace -- -D warnings`,
`cargo test -p talos-tui` (82 tests passed), `cargo test --workspace`,
`git diff --check`, and `scripts/validate_project_governance.sh .` (0 warnings) all pass.

README updated to document the new startup splash behavior.

Status: in-scope work Complete; viewport-overlay enhancement deferred per Scope Boundary.
