# TUI-011: Status Bar & Exit Output Polish

**Status**: In Progress (I039)
**Priority**: P2
**Source**: User request 2026-06-20
**Depends on**: TUI-009 (exit summary baseline) ✅
**Iteration**: [I039 Network Tools & TUI Polish](../iterations/I039-network-tools-tui-polish.md)

## Problem

The status bar and exit output are functional but not polished:

1. **Status bar** shows raw data (`model_name │ 12345 tokens │ S:3`) without visual
   hierarchy or intuitive layout.  Users scanning the bar can't quickly find the
   information they care about.

2. **Exit summary** (landed in TUI-009) is functional but plain — a list of
   `Key: Value` lines with no visual distinction between sections or emphasis
   on the most important metrics.

## Scope

### Status Bar

Redesign the status bar layout for clarity and visual appeal:

- **Left-aligned core info**: model name (truncated with `…` when needed)
- **Center-aligned turn progress**: spinner / processing indicator when active
- **Right-aligned metrics**: token count, queue indicators
- **Visual hierarchy**: model name in accent color, tokens in dim, warnings in
  warning color
- **Compact mode**: when terminal width < 80, collapse to single line with
  abbreviated labels
- **Consistent separators**: use Unicode box-drawing or middle-dot separators
  instead of raw `│`

Current format:
```
claude-sonnet-4 │ 12345 tokens │ S:3
```

Target feel (sketch):
```
⬡ claude-sonnet-4     ◷ processing…     12.3k tokens · ⬡ 3 queued
```

### Exit Output

Polish the exit summary (landed in TUI-009) for visual appeal:

- **Header with brand**: `⬡ Talos session complete` instead of plain `── Session Summary ──`
- **Section grouping**: separate model info, usage stats, and cost with visual
  breaks or indentation
- **Human-readable numbers**: `12.3k tokens` instead of `12345 tokens`
- **Cost with context**: show pricing tier used for estimate (e.g.
  `Est cost: $0.27 (Claude Sonnet @ $3/$15 per 1M tokens)`)
- **Duration in natural format**: `12m 34s` → keep, add hours if > 60m
- **Color**: use theme colors — header in accent, values in status-value,
  labels in dim-text

Target feel (sketch):
```
⬡ Talos session complete ─────────────────────────────

  claude-sonnet-4          12m 34s          8 turns
  45.2k tokens in           8.9k tokens out
  Est cost: $0.27
```

## Non-Goals

- Do not change the underlying data model (`StatusSnapshot`, `Usage`).
- Do not add new metrics — only reformat existing data.
- Do not change the inline terminal rendering architecture.
- Do not add configurable themes or user-customizable layout in this story.

## Acceptance Criteria

- [ ] Status bar renders model name (left), progress indicator (center),
      token/queue counts (right) with visual hierarchy.
- [ ] Status bar collapses gracefully at narrow terminal widths.
- [ ] Exit summary uses branded header, grouped sections, human-readable
      numbers, and theme-appropriate colors.
- [ ] Status bar and exit summary share formatting helpers (number formatting,
      duration formatting, color constants).
- [ ] Existing TUI tests pass; new tests cover compact mode and number
      formatting edge cases.
- [ ] No regression in status bar update performance (redraws at 50ms interval).

## Required Reads

- `crates/talos-tui/src/scrollback.rs` — status bar rendering
- `crates/talos-tui/src/app.rs` — `print_exit_summary()`
- `crates/talos-tui/src/theme.rs` — semantic color constants
- `docs/backlog/active/TUI-009-input-and-session-exit-polish.md`
