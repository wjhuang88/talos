# TODO-002: Todo Mutation Reliability And User Controls

| Field | Value |
|---|---|
| Story ID | TODO-002 |
| Priority | P1 |
| Status | In Progress (SSP120: idempotent create complete) |
| Source | [GitHub Issue #19](https://github.com/wjhuang88/talos/issues/19), [GitHub Issue #33](https://github.com/wjhuang88/talos/issues/33), [GitHub Issue #34](https://github.com/wjhuang88/talos/issues/34) |
| Depends On | `TODO-001`, `CMD-001`, `PERM-002` |

## Problem

Todo tools are useful for planning, but mutation reliability is weak under resume/retry. Duplicate
`todo_create` calls can create repeated records, batch planning requires many tool calls, and TUI
users cannot directly delete a todo even though the agent tool can.

## Acceptance

- `todo_create` is idempotent per session for the same effective title, with tests covering
  retry/resume duplicate creation.
- Batch create/update is supported without breaking the existing single-item tool input shape.
- TUI list/panel output hides full UUIDs by default while retaining enough identity for show/update
  workflows.
- `/todo delete <id>` is available only with an explicit confirmation or equivalent permission
  safeguard.
- README/help text clearly states which `/todo` operations are read-only and which mutate state.

## Non-Goals

- No cross-session global deduplication.
- No automatic deletion of historical duplicate rows without a migration plan.

## Required Reads

- `docs/backlog/active/TODO-001-session-todo-list.md`
- `crates/talos-session/src/todo.rs`
- `crates/talos-cli/src/todo_view.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-tui/src/app.rs`

