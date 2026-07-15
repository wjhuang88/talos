# TUI-030: Composer Input History

| Field | Value |
|---|---|
| Story ID | TUI-030 |
| Type | UI behavior story |
| Priority | P3 |
| Status | Refinement |
| Source | [GitHub Issue #37](https://github.com/wjhuang88/talos/issues/37) |
| Depends On | `TUI-004`, `TUI-010` |

## Goal

Let a user navigate previously submitted composer input with Up and Down without losing the draft they were editing.

## Scope

- Define empty/non-empty composer, multiline, boundary, and pre-navigation draft-restoration behavior.
- Keep the first delivery process-local and in memory.
- Add state tests for navigation, duplicate submissions, draft restoration, and slash-command and approval priority.

## Acceptance

- Up selects older entries and Down moves toward newer entries.
- Moving past the newest entry restores the exact unsubmitted draft.
- Composer history does not alter transcript persistence, approvals, or command dispatch.

## Non-Goals

- No cross-session or on-disk composer-history persistence, search, or new UI panel.

## Required Reads

- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/input.rs`
- `crates/talos-tui/src/tests.rs`
- `docs/backlog/active/TUI-004-state-model.md`
- `docs/backlog/active/TUI-010-slash-command-menu.md`
