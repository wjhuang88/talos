# TUI-013: Preview Spinner Single-Block Alignment

**Status**: Complete
**Priority**: P3
**Source**: User request 2026-06-26
**Iteration**: Next-iteration sidecar, implemented 2026-06-28

## Problem

The TUI preview row (shown while the agent is processing) uses a two-character braille
spinner prefix: `preview_spinner_padding()` in `scrollback.rs` renders two `SPINNER_FRAMES`
characters side by side (a "lead" and a "chase" index). Every other row prefix in the
scrollback uses a single character (e.g. `▸`, `⬡`). The two-block spinner makes the preview
row visually misaligned with the rest of the scrollback column structure.

## Scope

Change the preview spinner from two rotating braille blocks to a single block so the
preview row aligns with other rows.

### Required behavior

1. **Single-character spinner**: `preview_spinner_padding()` returns a single
   `SPINNER_FRAMES[frame]` character instead of two. The returned padding string has
   exactly one spinner glyph with the same leading space.

2. **Alignment**: The preview row's text content starts at the same column as other
   scrollback rows that use single-character prefixes.

3. **Animation preserved**: The spinner still cycles through `SPINNER_FRAMES` on each
   frame tick; only the block count changes from 2 to 1.

4. **Color**: The spinner color indexing continues to work (return the frame index so
   the caller can select from `processing_spinner` color array).

### Non-goals

- No change to the `SPINNER_FRAMES` array itself (the 10 braille glyphs stay).
- No change to status bar spinner (`◷ processing…`) — that is a separate surface.
- No change to hold-preview rendering (`HOLD_PREVIEW`).

## Acceptance

- Given the agent is processing and the preview row is visible,
  When the spinner animates,
  Then exactly one braille spinner character is shown as the prefix (not two).

- Given other scrollback rows with single-character prefixes,
  When the preview row is rendered alongside them,
  Then the text content columns align.

- Given the existing TUI tests,
  When `preview_spinner_padding` is called,
  Then the returned string contains exactly one `SPINNER_FRAMES` character.

## Completion Notes

Implemented 2026-06-28:

- `preview_spinner_padding()` now renders exactly one spinner glyph while preserving the
  three-column prefix width with a trailing space.
- The returned color index follows the single rendered frame.
- `preview_spinner_uses_single_block` covers the one-glyph prefix and color index behavior.

## Dependencies

- None blocking.

## Decision links and constraints

- ADR-018 (TUI inline-by-default layout)

## State/status owners

- Backlog: `docs/backlog/active/TUI-013-preview-spinner-single-block.md`
- Board: add to Next/Later when prioritized

## User-facing documentation

- Not required — visual-only change, no CLI/config surface.

## Required Reads

- `crates/talos-tui/src/scrollback.rs` (`preview_spinner_padding`, lines 78-89)
- `crates/talos-tui/src/app.rs` (spinner usage in viewport render)
- `crates/talos-tui/src/theme.rs` (`processing_spinner` color array)
