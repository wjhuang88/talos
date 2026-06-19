# TUI-010: Slash Command Menu Below Input

| Field | Value |
|-------|-------|
| Story ID | TUI-010 |
| Priority | P2 |
| Status | Complete (2026-06-20, I037) |
| Depends On | TUI-002 sub-slice C/D; TUI-009; CMD-001 |
| Origin | User feedback 2026-06-18 — implement a Codex-like menu layer when typing `/`, rendered below the input area |

## Problem

Talos has slash commands, but command discovery and selection should not depend
on remembering exact command names or tab completion alone. When the user types
`/` in the composer, the TUI should open a visible command menu near the input
area, similar to Codex.

The menu should feel like part of the composer, not a detached full-screen
view. The desired placement is below the input area.

## Scope

Add a slash command menu layer owned by the TUI input/bottom pane system.

Expected behavior:

- Typing `/` at the beginning of the composer opens the slash command menu.
- The menu is rendered below the input area where terminal geometry permits.
- Typing after `/` filters the command list.
- `Up`/`Down` move selection.
- `Enter` accepts the selected command or runs the command when it needs no
  extra input.
- `Tab` completes the selected command name.
- `Esc` closes the menu without clearing normal composer text.
- `Ctrl+C` follows TUI-009 behavior: clear idle composer input or cancel active
  work, rather than acting as the primary popup close key.
- Commands display a short label and description; commands that require inline
  arguments show an argument hint.
- Commands unavailable in the current state remain hidden or disabled with a
  clear reason.

## Placement Rule

The preferred placement is below the input area. If the terminal does not have
enough space below the composer, the implementation may fall back to an
above-input placement, but this fallback must be deterministic and tested.

The menu must not overwrite scrollback history or tool output. It is an active
input-layer surface, not a transcript message.

CMD-001 owns command executability and availability. TUI-010 must consume that registry and must
not recreate commands from historical roadmap lists. Commands remain hidden or disabled until
their domain owner provides a typed runtime action.

For explicit tool-backed aliases, the command registry references the registered tool and derives
its descriptive/schema metadata from that tool definition. The menu does not enumerate every model
tool as a user command.

## Relationship To Existing TUI Work

TUI-002 already defines the architectural pieces (`bottom_pane/`,
`chat_composer.rs`, `popup_stack.rs`, and `slash_command.rs`). TUI-010 is the
user-facing interaction story for the `/` menu layer. It should reuse the same
command registry as any slash command parser instead of maintaining a separate
hardcoded menu list.

TUI-009 defines related key behavior for `Esc` and `Ctrl+C`; TUI-010 must not
reintroduce `Esc` as a normal input-clear shortcut.

TUI-008 tracks approval dialog placement. After TUI-010 introduces a stable
input-layer popup/menu surface, approval can be evaluated as another view on
the same layer instead of a separate bottom-right overlay. That migration is a
follow-up integration point, not required for the first slash menu slice.

## Non-Goals

- Do not add a large new command catalog as part of this story.
- Do not implement command-specific full-screen views such as model picker or
  session picker unless an existing command already owns that behavior. SESSION-001 now owns the
  lifecycle behavior; any picker added here remains presentation-only.
- Do not migrate approval rendering in the first slash menu slice; keep that
  as the TUI-008 integration path after this layer exists.
- Do not introduce a global event bus. Follow ADR-006 and route through the
  existing TUI event loop and state model.

## Acceptance Criteria

- [x] Typing `/` at composer start opens a menu attached to the input area.
- [x] Menu placement prefers below-input rendering and falls back predictably
      when terminal height is insufficient.
- [x] Filtering updates as the user types after `/`.
- [x] Keyboard navigation supports `Up`, `Down`, `Enter`, `Tab`, and `Esc`.
- [x] `Esc` closes the menu but does not clear ordinary composer text.
- [x] `Ctrl+C` behavior remains consistent with TUI-009.
- [x] Slash command metadata comes from one command registry used by parser,
      completion, and menu rendering.
- [x] The menu does not write transient UI text into scrollback history.
- [x] The layer exposes enough structure for TUI-008 to later render approval
      prompts through the same popup stack.
- [x] TUI tests cover opening, filtering, selection, cancellation, placement
      fallback, and disabled/hidden command states.

## Required Reads

- `docs/backlog/active/TUI-002-codex-overhaul.md`
- `docs/backlog/active/TUI-009-input-and-session-exit-polish.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/backlog/active/SESSION-001-interactive-session-lifecycle.md`
- `docs/proposals/tui-codex-overhaul.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/state.rs`
