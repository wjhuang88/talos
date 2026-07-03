# 035: TUI Conversation History Scrollback Boundary

## Status

Accepted (2026-07-03)

## Context

The maintainer asked whether keeping conversation history in native terminal scrollback —
printed once above a small ratatui viewport — instead of a ratatui-rendered, re-drawable
history view is an architectural impediment.

ADR-019 already ruled that finalized content renders scrollback-only and permanently rejected a
viewport overlay, but its scope was the splash screen. This record generalizes the boundary to
conversation history and documents the impediment analysis so the question is not re-litigated
per feature.

Mechanism facts (verified 2026-07-03): `flush_pending_scrollback`
(`talos-tui/src/app.rs:664`) drains `pending_scrollback` through
`insert_styled_history` (`talos-tui/src/inline_terminal.rs:283`), which emits raw ANSI
DECSTBM scroll-region sequences plus reverse-index/linefeed to push one line into the terminal
emulator's scrollback buffer. After flush, Talos retains no cell-level record of those lines.
The logical history lives separately in `ConversationEngine.messages` and session JSONL; the
`hydrate_history` path rebuilds scrollback lines from messages at session resume.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Flushed scrollback lines are immutable terminal-emulator state | Hard (consequence of the mechanism) | `inline_terminal.rs` | Only by retaining a render buffer and moving history into a managed surface |
| Logical history is durable and re-renderable independent of the terminal | Hard | `ConversationEngine.messages` + session JSONL + hydrate | No |
| Finalized content renders scrollback-only; viewport is dynamic status | Hard (decided) | ADR-019 | Via its reversal trigger only |
| Terminal-native scroll/search/copy/tmux behavior and post-exit persistence are product behaviors | Soft | I022 inline-by-default direction | Yes |
| Simplicity First / no speculative features | Hard | AGENTS.md | No |

## Reasoning

Known costs of the scrollback-print model (all real, all accepted):

1. No re-wrap on terminal resize — lines wrap at print-time width.
2. No retroactive theme change — printed colors are fixed.
3. No interactive history — no collapse/expand of tool output, no click-to-fork a message, no
   hover; history is bytes, not widgets.
4. Overlays/pickers must be designed to never corrupt already-printed lines.
5. Two rendering vocabularies exist (crossterm segments for history, ratatui for viewport).

Benefits: native terminal scrollback, search, selection/copy, and tmux/screen compatibility for
free; history survives process exit in the user's terminal; O(1) memory in conversation length
with zero retained render state; no scroll-position state machine; the pattern is proven by
Codex CLI and Claude Code. The costs are bounded because the *logical* history is fully owned
and re-renderable (hydrate) — immutability applies to pixels, not data.

Impediment assessment: for everything shipped and planned (streaming turns, tool display,
todo panel prints, session resume via hydrate, export/copy from the logical transcript, retry
and status states in the viewport), the boundary is an enabler, not an impediment — it removes
whole classes of state/lifecycle bugs (ADR-019's rendering-timing coupling) and keeps memory
flat. It becomes an impediment only if the product commits to interactive or mutable history
(cost items 1-3). None of the current backlog requires that.

## Decision

1. Terminal scrollback remains the canonical and only renderer for finalized conversation
   history. A ratatui-rendered history view (alternate-screen or overlay) is rejected for the
   current product direction.
2. Guardrails: do not add a ratatui `Widget` that renders historical messages inside the
   viewport; do not retain per-line render buffers "just in case"; hydrate-from-messages remains
   the only re-render path; new history-surface features must be expressible as one-shot line
   prints.
3. The two-vocabulary split (crossterm history / ratatui viewport) is accepted as the cost of
   the boundary; shared pure data (`ScrollbackLine`) stays renderer-neutral.

## Reversal Trigger

Redesign (retained `ScrollbackLine` buffer + managed history surface, via a new ADR) only when
a committed product requirement needs at least one of:

- collapsible/expandable or otherwise interactive history entries,
- retroactive re-render of history on resize or theme change,
- message-level pointer interactions (select-to-fork, hover detail).

A wish for any of these is not the trigger; a story with acceptance criteria is.

## Related

- [ADR-019: TUI Splash Scrollback-Only Boundary](019-tui-splash-scrollback-boundary.md)
- I022 (inline-by-default TUI), I023 (state model), I024 (resume hydrate)
