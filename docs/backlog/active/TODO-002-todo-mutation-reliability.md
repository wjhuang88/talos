# TODO-002: Todo Mutation Reliability And User Controls

| Field | Value |
|---|---|
| Story ID | TODO-002 |
| Priority | P1 |
| Status | In Progress (FS07: /todo delete + batch create repo method complete; batch tool registration residual) |
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

## FS07 Execution Evidence (2026-07-07)

### Implemented

- **`/todo delete <id> --confirm`**: `TodoCommandAction::Delete { id, confirm }` added to
  `talos-conversation/src/types.rs`. Engine `parse_todo_command` handles the `delete` subcommand
  with `--confirm`/`--yes`/`-y` flag parsing. `todo_view.rs::handle_todo_delete` resolves short-ID
  prefixes (minimum 4 chars) against session items, rejects ambiguous matches, and requires
  `--confirm` before calling `repo.delete()`. Without `--confirm`, a guidance message is shown.
  3 engine tests + 3 todo_view tests cover parse, confirm, no-confirm, delete, and ambiguity paths.
- **`TodoRepository::create_batch`**: idempotent batch create added to
  `crates/talos-session/src/todo.rs`. Each item follows the same title-deduplication rule as
  `create`; items within the same batch that share a title deduplicate to the first occurrence.
  4 tests cover distinct creation, in-batch deduplication, against-existing deduplication, and
  empty-input.
- **UUID hiding**: verified already implemented (`todo_view.rs::short_id` takes first 8 chars for
  list/panel rendering; `show` reveals the full UUID for copy/update workflows).

### Residuals

- **Batch agent tool**: `create_batch` repository method is the foundation; a `todo_create_batch`
  agent tool + tool-builder registration is not yet wired. The model currently issues repeated
  `todo_create` calls for batch planning (which is idempotent and safe, just less efficient). A
  future iteration can add the batch tool without changing the single-item tool shape.
- **README help text**: `/todo delete` should be documented in the README slash-command table as
  the first mutating `/todo` subcommand. This is a doc-only residual for the next closeout.

