# ADR-054: Alternate-Screen Application-Owned Transcript Rendering

- Status: Accepted (2026-07-27; reinstated with startup-splash amendment)
- Date: 2026-07-27
- Owners: TUI / runtime maintainers
- Related: TUI-035, I156, ADR-035

> Decision history: the initial implementation was temporarily rejected when
> its startup sequence printed the logo on Primary Screen and then hid it by
> entering Alternate Screen. A primary-screen ADR-055 trial restored native
> history, but the maintainer subsequently selected one Alternate Screen mode.
> This accepted amendment fixes the actual startup defect: enter Alternate
> Screen first, then render the logo in the first full application frame.

## Context

Repeated Alacritty resize leaked fixed hint, composer, and status rows into
primary terminal scrollback. Cleanup after `Event::Resize` cannot reliably
reverse terminal-side reflow that occurs before the event reaches Talos.

The primary-screen inline renderer treats native terminal scrollback and
DECSTBM/reverse-index insertion as history storage while drawing fixed controls
as ordinary terminal cells. Terminal emulators are free to reflow those cells
on a width change, so there is no protocol boundary that identifies fixed UI as
non-history. The current architecture therefore cannot meet TUI-035's resize
isolation acceptance criterion.

## Decision

Talos interactive TUI uses alternate screen. Talos owns the logical transcript
and history scroll state. All visible content is rendered through one full-frame
renderer. No runtime correctness depends on terminal native scrollback, DECSTBM
history insertion, reverse index, or primary-screen reflow behavior.

One terminal-size snapshot is read for each frame and is used for layout,
history projection, drawing, and cursor placement. Resize invalidates a frame;
it never mutates transcript facts.

The startup logo is display-only application state. `TerminalSession` enters
and clears Alternate Screen before the first draw; the full-frame renderer then
draws the logo as a virtual prefix of the history rectangle. The first user
message is appended below that prefix, and increasing projected history
naturally scrolls Logo rows out of the rectangle. The Logo never enters
`TranscriptStore` and is never printed to Primary Screen.

## Rejected Alternatives

- More `ClearType` commands.
- Resize debounce as a correctness mechanism.
- An Alacritty-specific workaround.
- Resetting DECSTBM only after resize.
- Keeping native scrollback while trying to protect fixed bottom rows.
- Maintaining native scrollback and an app-owned transcript as equal sources of truth.

## Consequences

Positive consequences:

- deterministic resize and cross-terminal rendering;
- one rendering model and testable full-frame projection;
- Talos-controlled history reflow and scroll anchors;
- fixed-pane isolation from primary-screen scrollback.

Costs:

- interactive history scroll must be app-owned;
- leaving alternate screen does not naturally leave a live transcript on the
  primary screen;
- history viewport and anchors require explicit implementation.

## Exit Policy

Default: restore primary screen and print a compact session summary. An optional
explicit mode may print the transcript on exit. Talos never mirrors the live
transcript into primary scrollback during interactive execution.

## Implementation Notes

- `TranscriptBlock` stores tool calls and results as logical display facts; only
  `HistoryProjection` applies a current-frame width.
- History scrolling uses a logical entry/row anchor, with a documented nearest
  surviving-entry fallback after reflow.
- `AppLayout` bounds history and the bottom frame allocation for zero and
  short terminal sizes; cursor placement is clamped to that frame.
- Fill tokens are projected scalar-by-scalar and never exceed the viewport.
- `TerminalSession` records each terminal mode transition and rolls back only
  transitions that completed when initialization fails.
- `viewport_splash_lines` is the single Logo representation. It is rendered
  after Alternate Screen entry, reserves one display-only spacer row above the
  wordmark, uses a compact wordmark on narrow terminals, and participates only
  as a virtual frame-history prefix excluded from transcript/session/export
  facts.
- The old primary-screen insertion recovery helpers and their test-only writer
  seam have been removed. Real-terminal acceptance remains an I156/TUI-035
  completion gate rather than an unresolved architecture decision.
- Logical history anchors identify `(entry_id, logical_line, scalar_offset)`;
  rendered rows carry a stable range rather than a width-specific row number.
- Final component rectangles are assigned by `AppLayout`, with composer/status
  reserved before optional preview, queue, tips, and panels.
- Terminal restoration attempts every enabled cleanup action, retains failed
  state for retry, and propagates failure before any primary-screen summary.
- Mouse capture is a tracked terminal lifecycle state. Wheel events move the
  application-owned logical history projection; successful/failed capture
  transitions follow the same exhaustive rollback and retry rules.
- User-message history rows preserve `INPUT_BG` across the full projected row,
  including their padding rows. Projection suppresses synthetic leading
  assistant prefix-only rows without mutating transcript content.
- Rendered anchor ranges use half-open logical intervals. Only the last
  projected row accepts logical-line EOF; semantic empty and fill-only lines
  use an explicit zero-length logical range.
- Logical offsets come from original transcript scalars, never from
  projection-only markers, fill characters, or rendered text length.
- `AppLayout` separates resource allocation priority from visual placement,
  owns above-input/below-input panel placement, and assigns every input cursor
  from its final component rectangle.
- A modal text cursor is visible only when its active semantic field or
  selection row exists in the final panel rectangle. Vertical coordinates are
  never clamped onto another semantic row; the panel renderer and cursor target
  use the same title/instruction/input-row convention.
