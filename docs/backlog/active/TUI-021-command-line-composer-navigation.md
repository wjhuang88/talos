# TUI-021: Command-Line Composer Navigation Shortcuts

| Field | Value |
|---|---|
| Story ID | TUI-021 |
| Priority | P3 |
| Status | Planned |
| Source | User feedback 2026-07-02 — input area should support command-line style cursor navigation such as `Ctrl+A` and `Ctrl+E` |
| Depends On | TUI-009, TUI-010 |

## Problem

The TUI composer is the primary interaction surface, but it does not yet feel enough like a normal
terminal command line for cursor navigation. Users expect common readline-style shortcuts such as
line start and line end to work without reaching for arrow/home/end keys.

## Scope

Add command-line style cursor navigation shortcuts to the TUI composer:

- `Ctrl+A`: move cursor to the beginning of the current input line.
- `Ctrl+E`: move cursor to the end of the current input line.
- Preserve existing popup, approval, slash-panel, cancellation, and IME behavior.
- Add tests for empty input, single-line input, multi-line input, and interactions while slash or
  approval panels are active.

## Non-Goals

- No shell history search.
- No Emacs/readline completeness target.
- No keybinding configuration system.
- No change to `Ctrl+C` cancellation/clear behavior from TUI-009.
- No change to slash command execution semantics.

## Acceptance Criteria

- [ ] In normal composer mode, `Ctrl+A` moves the cursor to the start of the current line.
- [ ] In normal composer mode, `Ctrl+E` moves the cursor to the end of the current line.
- [ ] Empty input and single-character input do not panic.
- [ ] Multi-line input uses the current logical line, not always the full buffer start/end.
- [ ] Slash command and approval panels keep their existing key handling priority.
- [ ] TUI keyboard tests cover the shortcuts and priority behavior.

## Required Reads

- `docs/backlog/active/TUI-009-input-and-session-exit-polish.md`
- `docs/backlog/active/TUI-010-slash-command-menu.md`
- `docs/backlog/active/TUI-004-state-model.md`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/state.rs`
