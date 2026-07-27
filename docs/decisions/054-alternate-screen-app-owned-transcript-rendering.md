# ADR-054: Alternate-Screen Application-Owned Transcript Rendering

- Status: Proposed
- Date: 2026-07-27
- Owners: TUI / runtime maintainers
- Related: TUI-035, I156, ADR-035

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
- The old primary-screen insertion recovery helpers and their test-only writer
  seam have been removed. This ADR remains Proposed pending real-terminal
  acceptance.
