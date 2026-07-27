# TUI-035: Narrow-Viewport And Resize Rendering Robustness

| Field | Value |
| --- | --- |
| Story ID | TUI-035 |
| Type | Product / Rendering Story |
| Priority | P1 |
| Status | In Progress (2026-07-26; I156 Active) |
| Source | Maintainer bug reports 2026-07-24: narrow tool-call visibility and fixed-pane resize isolation |
| Parent Epic | None (follow-up to TUI-034 adaptive-width work) |
| Depends on | TUI-034 (adaptive history width; shipped the tool-RESULT wrap but missed tool-CALL lines and resize); TUI-025 (tool argument one-line fit); ADR-035 |
| Blocks | None |
| Selected Iteration | I156 (Active) |

## Current Architecture Authority

The amended ADR-054 is the sole current implementation authority. Talos uses
one Alternate Screen full-frame renderer: `TranscriptStore` owns geometry-free
conversation facts, `HistoryProjection` owns current-width rows and logical
scroll anchors, and history plus fixed components are rendered together.

The startup sequence enters and clears Alternate Screen before drawing the
Logo in the first application frame. The Logo is display-only, never enters
`TranscriptStore`, and yields to conversation history after the first
committed entry. ADR-019 is Superseded and ADR-055 is Rejected.

Resize recomputes the full frame from one size snapshot and never mutates
transcript facts. Runtime DECSTBM, reverse index, native-history insertion, and
pending terminal insertion recovery remain removed.

## Current Technical / Governance Checklist

- [x] geometry-free `TranscriptBlock` with current-frame ToolCall/ToolResult projection
- [x] one Alternate Screen app-owned full-frame renderer
- [x] Logo rendered after Alternate Screen entry in the first application frame
- [x] full and compact Logo modes covered without Transcript pollution
- [x] fixed UI excluded from the transcript
- [x] bounded fill projection and geometry-free ToolCall/ToolResult storage
- [x] component-level `AppLayout`, panel-local cursor coordinates, and clipped modal input hiding
- [x] exhaustive, retryable Alternate Screen lifecycle restore
- [x] no DECSTBM, reverse index, native-history projection, or legacy insertion identifiers
- [ ] real-terminal acceptance matrix
- [ ] Completion Commit

## Current Alternate-Screen Logo Automated Evidence

- Implementation commits `9c87d0f` and `635bc29`.
- `TranscriptStore` remains geometry-free and authoritative.
- The first empty full frame contains the complete wide Logo at 80 columns and
  the compact Logo at narrow widths, with one display-only spacer row above the
  wordmark.
- The Logo is not a transcript fact and disappears when the first committed
  conversation block becomes the history projection.
- Alternate Screen entry failure aborts initialization; restore remains
  exhaustive and retryable.
- Focused TUI validation: 439 passed, 0 failed.
- Locked workspace validation: 2470 passed, 0 failed across 62 test
  binaries/doc-test groups.
- Real-terminal acceptance remains pending.

## Historical / Superseded Original Fix Plan

The following plan describes the superseded primary-screen DECSTBM
implementation attempt. It is retained only as historical evidence and is not
current implementation authority. Its Fix 1/Fix 2/Fix 3, exclusions, source
references, and tests do not describe the amended ADR-054 full-frame runtime.
ADR-055's later primary-screen experiment is also superseded and rejected.

### Original Problem

Two distinct symptoms share one root architecture: the inline terminal renders
scrollback by printing each history line individually through a DECSTBM
scroll-region + reverse-index mechanism (`inline_terminal.rs::insert_styled_history`),
keyed on `screen_size`, with no full-screen buffer diff. This is not robust when
the viewport WIDTH changes or is very small.

#### Symptom A — tool-call summary line blank at narrow width

When a tool is invoked, its one-line summary (`→ bash, command: ...`) is built by
`build_tool_display.rs::build_tool_call_scrollback_line` (tool_display.rs:454).
Unlike the tool-RESULT path, which TUI-034 made width-aware:

| Line type | Builder | viewport_width passed? |
| --- | --- | --- |
| tool RESULT | `build_tool_result_scrollback_lines(&display, icon, color, viewport_width)` (app.rs:632) | yes — renderer-accounted wrap (TUI-034) |
| tool CALL | `build_tool_call_scrollback_line(&display)` (app.rs:621) | **no** — fixed `TOOL_CALL_ARGS_BUDGET_CHARS = 180` char budget |

The tool-call line is only wrapped later by the generic
`app_stream.rs::wrap_scrollback_line` at flush time. That function has an edge
defect for a 3-cell prefix (` → `):

- Continuation prefix guard (app_stream.rs:90):
  `prefix_width > 0 && prefix_width <= 3 && prefix_width < width`. When the
  viewport `width <= 3`, the ` → ` prefix (width 3) fails `prefix_width < width`,
  so continuation indent is dropped.
- At very small widths the summary is shredded into dozens of 1–2 char rows, each
  inserted individually via the DECSTBM path; combined with the missing
  width-aware build, the visible summary collapses to near-empty. TUI-034's
  acceptance ("width overflow creates renderer-accounted continuation rows") was
  only satisfied for tool RESULT, not tool CALL.

#### Symptom B — resize (shrink) duplicates hint bar with staircase fill bars

Shrinking a wide window leaks the bottom hint/status pane into scrollback, one
copy per intermediate width, each with a dark background bar that grows/shrinks
row by row (staircase). Verified cause chain:

1. `app.rs:1253` — `Event::Resize(_, _) => {}`: resize is a **no-op**. No forced
   redraw, no clear, no scroll-region reset. Size changes are only observed
   lazily on the next `draw()`.
2. `inline_terminal.rs:253-259` — `draw()` recomputes
   `screen_size = backend.size()` and `area.width = screen_size.width` each frame,
   so width changes are noticed only frame-by-frame during the drag.
3. `inline_terminal.rs:62-64` — `viewport_change_requires_clear = previous != next`;
   any width change flags a clear.
4. `inline_terminal.rs:231-246` — `set_viewport_area` only issues
   `Clear(ClearType::FromCursorDown)` when **height** shrinks
   (`area.height < self.viewport_area.height`). A pure **width** shrink resets the
   double buffers but does **not** clear the already-drawn region, so stale wide
   bottom-pane rows survive on the grid and get pushed up into scrollback by the
   next DECSTBM insert.
5. Fill width is stale-width-derived: `flush_pending_scrollback` (app.rs:798-808)
   calls `append_fill_segment(..., self.terminal.screen_size().width, ...)`
   (scrollback_markdown.rs:336-359), which computes
   `avail = target_width - segments_width - trailing` and repeats the fill glyph
   to `avail`. Because each intermediate resize frame has a different
   `screen_size().width`, the fill length differs per frame → the staircase.
6. `wrap_scrollback_line` (app_stream.rs:78) returns early for any line with
   `fill.is_some()`, so fill-bearing hint/status lines are never re-wrapped to the
   new width — they are emitted at whatever stale width they were built with.

#### Shared root

`insert_styled_history`'s per-line DECSTBM scroll model depends on `screen_size`,
does no whole-screen diff, ignores `Event::Resize`, does not clear on width
shrink, and computes fill from a possibly-stale width. Width-stable + wide cases
work (early-return hits, no overflow); width-changing / very-narrow cases fail.

### Original Goal / Value

Tool-call summaries stay visible and correctly wrapped at any width, and
shrinking the window never leaks the bottom pane into scrollback or paints
staircase fill bars. History remains renderer-accounted (not
terminal-autowrap-dependent) across resize, matching the TUI-034 intent for the
lines it missed.

### Architecture Correction (2026-07-27)

Manual Alacritty acceptance failed: repeated resize leaked fixed hint, composer,
and status rows into primary terminal scrollback. This is evidence that the
primary-screen DECSTBM approach cannot meet this Story's existing fixed-pane
isolation acceptance. Under I156 change control, implementation is corrected to
ADR-054's alternate-screen, application-owned transcript and full-frame
renderer. The Story remains In Progress; no completion claim is authorized until
the real-terminal matrix passes.

Automated implementation evidence: `a3074ad` provides the app-owned transcript, history
projection, full-frame renderer, alternate-screen lifecycle, and app-owned scroll state. It does
not close this Story: visual acceptance on real terminals is still required.

### Original Scope

#### Fix 1 — width-aware tool-call summary line

- Give `build_tool_call_scrollback_line` a `viewport_width` parameter and produce
  renderer-accounted continuation rows using the same display-width-aware wrap
  the tool-RESULT path uses (`wrap_to_display_width`), with the ` → ` first row
  and continuation-indent subsequent rows, preserving tool-name/args styling.
- Pass `self.terminal.screen_size().width` at the call site (app.rs:621),
  mirroring the tool-RESULT call (app.rs:632).

#### Fix 2 — continuation-prefix edge in `wrap_scrollback_line`

- Correct the app_stream.rs:90 guard so a 3-cell prefix does not silently drop
  continuation indent when `width <= 3`, and so extremely narrow widths degrade
  predictably (defined below) instead of shredding a line into unusable
  fragments.
- Define a minimum usable content width floor for wrapped rows (consistent with
  the tool-RESULT `budget = viewport - prefix_len` handling) so a 1–3 column
  viewport produces a bounded, legible result rather than dozens of empty rows.

#### Fix 3 — resize is handled; bottom pane never leaks; fill uses live width

- `Event::Resize` (app.rs:1253) must trigger a controlled full redraw of the
  viewport for the new size: reset the DECSTBM scroll region and clear the
  region the bottom pane occupied so stale wide rows cannot be pushed into
  scrollback.
- On a **width** shrink (not just height), `set_viewport_area` /the redraw path
  must clear the previously drawn bottom-pane area, symmetric to the existing
  height-shrink clear (inline_terminal.rs:236-241).
- Fill segments must be computed from the authoritative current-frame width at
  emit time; a line that is being (re)committed after a resize must not carry a
  fill length derived from a prior width. Consider not committing fill-bearing
  bottom-pane lines to scrollback at all (they are viewport-fixed, not history).

#### Correctness invariant (both symptoms)

The bottom hint/status/composer pane is viewport-fixed UI redrawn every frame. It
MUST NEVER appear in scrollback history. Any path that moves a viewport-fixed pane
row into `insert_history`/`insert_styled_history` is a defect.

### Historical Explicit Exclusions

Historical exclusion — superseded after real-terminal failure proved that the
original architecture could not satisfy unchanged fixed-pane isolation.

- No rewrite of the scrollback storage model or a switch away from the
  DECSTBM/inline-terminal approach; this is a targeted robustness fix.
- No change to tool RESULT wrapping (TUI-034 already correct) beyond sharing its
  helper.
- No change to summary content policy (`TOOL_CALL_ARGS_BUDGET_CHARS`,
  head/tail/omitted, TUI-015/TUI-025 semantics) except making the tool-call line
  width-aware.
- No new `unsafe`, no native dependencies.
- `/export`, `/copy`, and resume/transcript paths are unaffected (they do not go
  through these viewport render functions).

### Historical Design / Security Constraints

- Reuse `wrap_to_display_width` and the tool-RESULT prefix/budget conventions;
  do not add a second wrapping algorithm.
- All width math must use `unicode_width` display cells (CJK = 2), never
  `char`/byte counts, matching TUI-034/TUI-032.
- External-process output already flows through these renderers; wrapping/clearing
  must not panic on any width (0, 1, 2, 3, huge) — bound every `saturating_sub`
  and division. Constraint 9 (deps must not crash the process) applies:
  a pathological width must degrade to a safe bounded render, never a panic.
- Redraw-on-resize must not double-emit committed scrollback history (only the
  viewport is redrawn; already-scrolled history stays put).

### Historical Acceptance

Behavior:

- Given a tool call whose summary exceeds the viewport, when rendered at width 40,
  80, 120, 160, then the summary is fully present across renderer-accounted
  continuation rows (no blank line, no lost text), with ` → ` on the first row
  and aligned continuation indent after.
- Given a viewport width of 1, 2, or 3 columns, when a tool-call summary is
  rendered, then the render is bounded and does not panic, does not emit dozens of
  empty rows, and retains as much leading content as the width allows.
- Given CJK-heavy tool-call arguments, when wrapped, then breaks occur on
  display-cell boundaries (2 cells per CJK char), never mid-cell.
- Given a wide window that is shrunk to narrow (continuous drag through
  intermediate widths), when the resize completes, then the bottom hint/status
  bar appears exactly once (in the viewport), with zero duplicated hint rows in
  scrollback and zero staircase fill bars.
- Given the same shrink, when inspecting scrollback, then no
  fill-bearing bottom-pane line was committed to history at any intermediate
  width.
- Given a height-only shrink, when it occurs, then the existing correct behavior
  (bottom-pane clear) is preserved (no regression).
- Given a widen (grow) operation, when it occurs, then history and bottom pane
  render correctly with no truncation or leftover artifacts.

Historical technical / governance:

- [x] `cargo test --workspace --locked` includes: tool-call wrap at 40/80/120/160;
      tool-call render at width 1/2/3 renderer-accounted (each output row display
      width ≤ viewport, no terminal-autowrap dependency); CJK tool-call wrap with
      1-cell substitution at width 1; builder→renderer chain width-constraint test;
      physical row accounting via `prepare_history_rows`; the `resize_clear_action`
      decision matrix (width-shrink → ClearViewportRows, height-shrink →
      ClearFromCursorDown, grow → None, zero-height safety).
- [ ] A resize/shrink end-to-end test asserting the hint bar is not committed to
      scrollback and no fill-bearing viewport line enters history; height-shrink
      rendering regression. NOT yet automated: the no-leak guarantee is
      architectural (the bottom pane renders via `terminal.draw()`/InlineFrame and
      never calls `insert_history`) plus the `resize_clear_action` unit tests; the
      full PTY/buffer e2e assertion is deferred to the manual Alacritty gate below.
- [x] `cargo fmt --all`, `cargo clippy --workspace --locked -- -D warnings`,
      `scripts/validate_project_governance.sh .`, and `git diff --check` clean.
- [x] Owner status here and the Board mirror synchronized.
- [ ] Manual Alacritty walkthrough (wide→narrow drag, and a width-1..3 extreme)
      recorded as the Ready→Complete human gate, consistent with TUI-034's
      real-terminal acceptance requirement.

### Historical Resolved Decisions (2026-07-24)

1. **One story, two symptoms** — Symptom A (tool-call blank) and Symptom B
   (resize hint duplication + staircase fill) share the inline-terminal
   width/resize root and are fixed and verified together.
2. **Reuse, don't reinvent** — tool-call wrapping reuses the tool-RESULT
   `wrap_to_display_width` path from TUI-034; no second algorithm.
3. **Resize gets real handling** — `Event::Resize` becomes a controlled full
   viewport redraw with scroll-region reset and bottom-pane clear on width shrink
   (currently only height shrink clears).
4. **Invariant** — the viewport-fixed bottom pane must never enter scrollback;
   fill is computed from the live current-frame width only.
5. **Priority P1** — user-visible corruption on a common interaction (resize) and
   lost tool-call visibility; raised above the P2 default for polish items.

### Historical Root-Cause Reference (verified lines)

- `crates/talos-tui/src/tool_display.rs:454` — `build_tool_call_scrollback_line`
  (no `viewport_width`; fixed 180-char budget).
- `crates/talos-tui/src/app.rs:621` vs `:632` — tool-call vs tool-result call
  sites (missing vs present `viewport_width`).
- `crates/talos-tui/src/app_stream.rs:76-127` — `wrap_scrollback_line`; line 78
  early-returns on `fill.is_some()`; line 90 continuation-prefix guard.
- `crates/talos-tui/src/app.rs:1253` — `Event::Resize(_, _) => {}` (no-op).
- `crates/talos-tui/src/inline_terminal.rs:62-64` —
  `viewport_change_requires_clear = previous != next`.
- `crates/talos-tui/src/inline_terminal.rs:231-246` — `set_viewport_area` clears
  only on height shrink.
- `crates/talos-tui/src/inline_terminal.rs:253-277` — `draw()` recomputes
  `screen_size`/`area.width` each frame.
- `crates/talos-tui/src/inline_terminal.rs:334-401` — `insert_history` /
  `insert_styled_history` DECSTBM scroll-region insert.
- `crates/talos-tui/src/scrollback_markdown.rs:336-359` — `append_fill_segment`
  (`avail = target - width - trailing`, repeated to fill).
- `crates/talos-tui/src/app.rs:798-808` — fill computed from
  `self.terminal.screen_size().width` at flush time.

### Historical Required Reads

- `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
- `docs/iterations/I156-tui-narrow-viewport-resize.md`
- `docs/backlog/active/TUI-034-adaptive-history-width-and-tool-output-visibility.md`
- `docs/backlog/active/TUI-025-tool-argument-line-fit-display.md`
- `docs/backlog/active/TUI-015-head-tail-truncation.md`
- `docs/backlog/active/TUI-032-composer-multiline-wrap.md`
- `docs/decisions/035-tui-history-scrollback-boundary.md` (ADR-035)
- `crates/talos-tui/src/tool_display.rs`
- `crates/talos-tui/src/app_stream.rs`
- `crates/talos-tui/src/inline_terminal.rs`
- `crates/talos-tui/src/scrollback.rs`
- `crates/talos-tui/src/scrollback_markdown.rs`
- `crates/talos-tui/src/app.rs`

### Historical Minimum Validation

## Automated Full-Frame Invariant Evidence — 2026-07-27

- Tool calls and results are stored as logical `TranscriptBlock` facts and are
  formatted only by the current-frame history projection.
- Anchored history tracks a logical transcript row, rather than a physical
  bottom offset, across append and resize.
- Fill-only rows, multi-cell fills, and CJK width-one degradation are projection
  concerns and do not mutate the transcript.
- Bounded layout/cursor and transactional alternate-screen lifecycle tests pass.
- The test-only DECSTBM insertion recovery architecture was removed.

TUI-035 remains **In Progress** until the real-terminal matrix is complete.

## Final Anchor And Coordinate Automated Evidence — 2026-07-27

- Wrapped-row anchors use half-open logical intervals; logical-line EOF,
  semantic empty lines, and fill-only lines have explicit matching rules.
- Logical offsets are derived from original scalar consumption, including
  width-one CJK projection-only markers.
- Tool-call argument and multiline tool-result anchors remain on the same
  logical content through the resize matrix.
- PageUp/PageDown consume the final history rectangle height, and real key
  entry-point tests cover PageUp, PageDown, Ctrl+Home, Ctrl+End, tail, and
  empty-history behavior.
- Component height priority is separate from AboveInput/BelowInput visual
  ordering. Composer padding, modal panels, and status have exact relational
  layout tests at extreme terminal heights.
- Credential/provider cursor positions are panel-local and converted through
  the final panel rectangle; missing or zero-sized panels hide the cursor.
- Modal cursor rows are strict: a clipped credential field, provider entry
  field, or second protocol option hides rather than vertically clamping onto a
  non-input row. Provider-wizard Confirm has no text cursor. Test-terminal
  cursor visibility and position are asserted through the formal draw path.
- Full-frame rendering at `0x0`, `1x1`, `1x2`, `2x1`, `2x2`, `3x3`, `5x2`,
  and `20x3` is safe. Fixed preview, tip, queue, composer, and modal state does
  not enter the transcript.
- Implementation/test commits: `6c32d09`, `18648a6`, `1688e6f`, `ad67fc5`.
  Focused TUI validation: 437 passed. Locked workspace validation: 2464 passed.
  Real-terminal
  acceptance remains pending.

### Historical Test Evidence

- Unit tests for tool-call wrap at 40/80/120/160 and extreme widths 1/2/3.
- CJK tool-call wrap boundary test.
- Resize/shrink test: assert bottom hint bar text is not present in committed
  scrollback and no fill-bearing viewport line entered history; height-shrink
  regression test.
- Locked fmt / check / clippy / test and
  `scripts/validate_project_governance.sh .`; `git diff --check`.
- Manual Alacritty wide→narrow drag + width-1..3 extreme walkthrough evidence.
