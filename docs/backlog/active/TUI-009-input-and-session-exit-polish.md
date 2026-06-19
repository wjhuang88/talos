# TUI-009: Input Clear And Session Exit Summary

| Field | Value |
|-------|-------|
| Story ID | TUI-009 |
| Priority | P2 |
| Status | Complete (2026-06-19) |
| Depends On | TUI-004 |
| Origin | User feedback 2026-06-18 — input clearing should use Ctrl+C, and exit should print Codex-style session resource usage |

## Problem

Two small TUI behaviors still feel inconsistent with the intended Codex-style
interaction model:

1. The input composer uses `Esc` as a clear action. This conflicts with the
   more common role of `Esc` as a popup/overlay cancel key.
2. Exiting the TUI does not leave a concise session summary in terminal
   scrollback. Users should be able to see what the session consumed without
   opening logs or session files.

## Scope

This is a TUI polish story, not a session persistence redesign.

### Input clear behavior

- `Ctrl+C` clears the input buffer when the app is idle and the composer has
  non-empty input.
- `Ctrl+C` keeps its current cancellation behavior while a turn or tool call
  is running.
- `Ctrl+C` may keep the existing double-press exit behavior when the app is
  idle and the composer is empty.
- `Esc` should no longer clear normal composer input. It remains available for
  context-specific cancellation such as closing popups, dialogs, overlays, or
  approval UI.

### Exit summary behavior

On clean TUI exit, append a concise resource usage summary to terminal
scrollback. The summary should be informational and non-blocking.

Minimum useful fields:

- session id or session path when available
- model/provider when available
- elapsed session duration
- number of user turns
- number of tool calls
- token usage when provider usage is available
- cache read/write usage when provider usage is available
- estimated cost only when the provider exposes enough information to compute
  it honestly

Unavailable values should be omitted or rendered as `unavailable`; they must
not be guessed.

## Non-Goals

- Do not dump the full transcript on exit. The inline scrollback already is the
  transcript by design.
- Do not print hidden tool contents such as full `read` output.
- Do not introduce a global event bus. Follow ADR-006 and the existing
  single-consumer event loop boundary.
- Do not persist new analytics or telemetry outside the local session model.

## Acceptance Criteria

- [x] In an idle session with non-empty composer input, pressing `Ctrl+C`
      clears the composer and does not exit the app.
- [x] In an active turn, pressing `Ctrl+C` still triggers the existing
      cancellation path.
- [x] In an idle session with empty composer input, the existing exit affordance
      remains available and clearly communicated.
- [x] Pressing `Esc` does not clear normal composer text.
- [x] On clean exit, terminal scrollback contains a compact session usage
      summary.
- [x] Exit summary never includes secrets, hidden tool output, or full file
      contents.
- [x] Missing usage fields degrade gracefully without panics.
- [x] TUI keyboard and state tests cover the new key behavior and exit summary
      formatting.

## Required Reads

- `docs/backlog/active/TUI-004-state-model.md`
- `docs/backlog/active/TUI-002-codex-overhaul.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/state.rs`
- `crates/talos-conversation/src/types.rs`
