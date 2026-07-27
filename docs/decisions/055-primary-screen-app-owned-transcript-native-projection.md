# ADR-055: Primary-Screen App-Owned Transcript With Native History Projection

- Status: Proposed
- Date: 2026-07-27
- Owners: TUI / runtime maintainers
- Related: TUI-035, I156, ADR-019, ADR-054

## Context

The ADR-054 alternate-screen implementation isolated fixed controls during
resize, but it also hid the startup logo, pre-existing shell history, and the
terminal's native scrollback for the entire interactive session. Maintainer
acceptance established native terminal history as a product requirement and
rejected a second selectable renderer mode.

The earlier primary-screen implementation is not suitable for restoration as
written: it treated width-specific physical rows as durable history and used
DECSTBM plus reverse index for insertion. Talos now has a geometry-free
`TranscriptStore`, so native scrollback can instead be a derived, append-only
projection.

## Decision

Talos has one interactive rendering mode:

1. The TUI remains on the primary screen. It never enters Alternate Screen.
2. `TranscriptStore` remains the sole logical source of conversation facts.
3. Each newly committed transcript entry is projected at the current width
   exactly once and appended to terminal-native scrollback with ordinary
   newline output.
4. Composer, status, preview, queue, tips, and modal panels are rendered only
   in the current bounded inline frame beneath committed history.
5. Runtime history output uses neither DECSTBM, reverse index, nor terminal
   insertion/recovery transactions.
6. The startup splash remains primary-screen scrollback content per ADR-019.
7. Resize changes the current inline frame and future entry projection; it
   never mutates logical transcript facts.

## Consequences

Positive:

- the logo, shell output, and Talos conversation remain available through
  native terminal scrollback;
- mouse, trackpad, copy, selection, and terminal search retain native behavior;
- transcript, export, and session persistence remain geometry-free;
- there is one runtime renderer and no mode-selection surface.

Tradeoff:

- terminal emulators may reflow cells from the current transient inline frame
  before Talos receives a resize event;
- the application can clear and redraw its current frame, but no terminal
  protocol can retroactively identify leaked fixed cells in native scrollback;
- real-terminal acceptance therefore evaluates the supported terminals and
  records residual visual behavior rather than claiming protocol-independent
  fixed-pane isolation.

## Rejected Alternatives

- Keep ADR-054 Alternate Screen as the only mode: rejected because native
  history is a required interaction surface.
- Maintain both Alternate Screen and primary-screen renderers: rejected by the
  maintainer because Talos should expose one predictable interaction model.
- Restore the old DECSTBM/reverse-index renderer: rejected because it persists
  terminal geometry and reintroduces insertion recovery as a correctness
  boundary.
- Treat terminal scrollback as the logical transcript: rejected; it is a
  derived presentation surface only.

## Exit Policy

Talos clears the transient inline frame, restores terminal input modes, and
prints the compact session summary below the already-present native transcript.
It does not dump a duplicate transcript on exit.
