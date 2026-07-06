# TUI-022: Todo Panel Checkbox Unification

Type: Product Story
Parent Epic: TODO-001 (Complete) follow-up
Status: Complete (SB110, 2026-07-06)

## Identity / Goal / Value

The structured todo panel (`build_todo_panel_lines`, `crates/talos-tui/src/app.rs:1069-1149`)
still renders rows as `{id} [{status}][{priority}] {title}` bracket text, while the todo tools
and the `/todo` slash view were unified onto Codex-style checkbox rendering via the shared
`talos_session::status_icon` helper (2026-07-03 session refactor). One remaining surface renders
the same data in a different visual language.

Goal: the todo panel uses the same checkbox iconography as the other two surfaces.

## Scope

- Map known status strings to `status_icon` checkboxes in the panel row construction; the
  mapping lives either upstream where `TodoPanelRow` is built (`crates/talos-cli/src/todo_view.rs`)
  or in `build_todo_panel_lines` — pick one place, not both.
- Unknown/unmapped status strings keep the existing `[{status}]` bracket fallback (panel data is
  plain `String`, not an enum).
- Preserve existing colors/priority display unless the checkbox makes `[{priority}]` redundant —
  decide during implementation, note the choice in the story.

## Exclusions

- No changes to todo tool output or `/todo` slash rendering (already unified).
- No new panel interactions.

## Dependencies

- None. `status_icon` is already `pub` in `talos-session`.

## Required Reads

- `crates/talos-tui/src/app.rs` (`build_todo_panel_lines`)
- `crates/talos-cli/src/todo_view.rs`
- `crates/talos-session/src/todo.rs` (`status_icon`)

## Acceptance for behavior

- Given a session with todos in pending/in_progress/completed/cancelled states
  When the todo panel renders
  Then each row shows the same checkbox glyph as the todo tool output for that status, and
  `cargo test -p talos-tui` panel tests assert the new format.

## Implementation (SB110, 2026-07-06)

- Mapping placed upstream in `todo_view.rs::todo_panel_rows()` — uses `talos_session::status_icon()` directly.
- TUI rendering in `build_todo_panel_lines()` uses `row.status` as-is (no bracket wrapping since checkbox icons already include brackets).
- Priority display `[priority]` kept — only `[status]` brackets replaced.
- Commit: `36c14db fix(tui): unify todo panel status icons and add themed diff line backgrounds (#TUI-022, #TUI-023)`
