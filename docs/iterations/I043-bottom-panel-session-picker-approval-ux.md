# I043: Bottom Panel Generalization, Session Picker, Approval UX

> Document status: Active
> Published plan date: 2026-06-23
> Planned close date: 2026-07-05 (≈ 2 weeks)
> Planned objective: Generalize the TUI bottom panel from slash-command-only
>   to a reusable overlay that hosts multiple picker types. Ship an interactive
>   session picker for `/resume`. Resolve I042 technical debt (R1 interrupt_tx,
>   R2 model_context_limit). Improve the permission approval dialog prominence.

## Selected Stories

| Story | Priority | Outcome |
|---|---|---|
| Bottom Panel Generalization | P0 | `SlashMenuState` → `BottomPanelState` with `PanelKind` enum; `SlashMenuComponent` → `BottomPanelComponent` that renders slash commands or session items or future types |
| Session Picker | P0 | `/resume` (no arg) opens bottom panel with workspace-scoped sessions; Up/Down navigate, Enter selects, Esc cancels |
| R1: interrupt_tx continuity | P1 | Conversation loop's interrupt sender follows session switches via `sq_tx_watch` |
| R2: model_context_limit | P1 | Read `context_limit` from provider config for the active model |
| TUI-008: Approval Dialog UX | P1 | Approval prompt moves from easy-to-miss corner to a prominent centered/below-input position |

## Execution Order

```
Week 1:
  Bottom Panel Generalization ─── 2-3 days
         │
         └── Session Picker ─── 2-3 days (builds on generalized panel)
         ∥
  R1 + R2 fixes ─── 1 day (independent, can parallelize)

Week 2:
  TUI-008 Approval UX ─── 2-3 days (reuses the generalized panel infra)
  Closure + verification ─── 1 day
```

## Scope

### Bottom Panel Generalization

- `SlashMenuState` → `BottomPanelState` with a `kind: PanelKind` field
- `PanelKind` enum: `SlashCommand { registry_items }` | `SessionPicker { sessions }`
- `SlashMenuComponent` → `BottomPanelComponent` that dispatches render based on `PanelKind`
- Placement logic (`slash_menu_placement`, `slash_menu_rows`) stays shared — rename to `bottom_panel_placement` / `bottom_panel_rows`
- All existing slash menu tests pass unchanged after rename
- `TuiState` methods updated: `open_slash_menu` → `open_slash_command_panel`, add `open_session_picker`

### Session Picker

- `UiOutput::SessionPicker(Vec<SessionPickerItem>)` added to `talos-conversation`
- `SessionPickerItem { ordinal, timestamp, message_count, preview }`
- `handle_session_resume` None branch: query sessions, sort, send `UiOutput::SessionPicker(items)` instead of text
- TUI `handle_ui_output`: receive `SessionPicker`, call `state.open_session_picker(items)`
- Bottom panel renders session items: `1. 2026-06-22 19:20 — 15 messages — "preview..."`
- Input handling in SessionPicker mode: Up/Down navigate, Enter sends `/resume <N>` as UserInput::Message, Esc closes panel
- No filtering (unlike slash commands which filter by query)

### R1: interrupt_tx continuity

- `run_conversation_loop` currently takes `interrupt_tx: mpsc::Sender<SessionOp>` as a fixed clone
- Change: conversation loop reads `interrupt_tx` from `sq_tx_watch_rx` (same watch channel user persister uses)
- OR: add a separate `interrupt_tx` update path alongside the existing `bridge_rx_update`
- Simplest: pass `sq_tx_watch_rx` into conversation loop, use `borrow().clone()` for interrupt sends

### R2: model_context_limit from config

- Read `context_limit` from `config.providers[provider].models[model].context_limit`
- Fall back to 128_000 if not configured
- Pass resolved value into handler functions instead of hardcoded constant

### TUI-008: Approval Dialog UX

- Current: approval renders inline in scrollback (easy to miss)
- New: approval renders as a prominent overlay below the input line (similar to bottom panel placement)
- Uses the same placement infrastructure as the bottom panel
- Shows tool name, summary, and choice keys (y/n/a/d) clearly
- Does NOT use the bottom panel state machine (different interaction model — approval is modal, panel is navigable)

## Non-Goals

- Fuzzy search in session picker (substring is enough for MVP)
- Multi-column session info display
- Approval rule editing UI (still config-only)
- Changing the approval choice model (y/n/a/d stays)

## Acceptance

- `/resume` opens an interactive bottom panel with session list
- Up/Down navigates the list, Enter resumes the highlighted session
- Esc closes the panel without action
- Bottom panel code is shared between slash commands and session picker
- Ctrl+C interrupts the correct (current) actor after session switch
- `model_context_limit` reflects provider config
- Approval dialog is more visually prominent than current inline scrollback
- `cargo test --workspace` passes
- `cargo clippy --workspace -- -D warnings` clean

## Verification

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Unit tests for `BottomPanelState` (slash command + session picker modes)
- Unit tests for session picker rendering and navigation
