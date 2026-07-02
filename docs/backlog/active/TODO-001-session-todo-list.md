# TODO-001: Session-Level Todo List for Plan Orchestration

| Field | Value |
|-------|-------|
| Story ID | TODO-001 |
| Priority | P2 |
| Status | In Progress — I078/T121-A repository implemented, T121-B tools pending |
| Source | [GitHub Issue #8](https://github.com/wjhuang88/talos/issues/8) |
| Relates To | SESSION-001, MEM-001, CMD-001, TOOL-012 |

## Requirement

Structured task management within Talos sessions. Users view the todo list via slash commands;
the agent creates/updates/deletes items through tools. The list persists with the session and
integrates with the agent prompt for plan-aware orchestration.

## Design Principle

User commands are **read-only** (view, list, stats, export). All mutations (create, update,
delete, dependency) are **agent tools** that go through the permission pipeline.

## Scope

### Data Model
- TodoItem: id, title, description, status (todo/in_progress/completed/blocked), priority
  (low/medium/high/critical), created_at, completed_at, assigned_to_turn, tags
- TodoDependency: parent_id, child_id
- SQLite tables in talos-session or talos-core

### Agent Tools (write operations)
- todo_create, todo_update_status, todo_update, todo_dependency, todo_delete, todo_query (read-only)

### Slash Commands (read-only)
- /todo, /todo list [--filter] [--sort], /todo show <id>, /todo stats, /todo export

### TUI
- Read-only TodoPanel component showing task list with status/priority indicators

### Integration
- Session lifecycle: auto-create empty list on new session, persist on save
- Agent prompt: inject active todo items (top N) into system prompt context
- Cyclic dependency detection on todo_dependency

## Non-Goals

- No user-facing write commands — all writes are agent tools.
- No cross-session todo inheritance (session-scoped only for v1).
- No calendar/scheduling integration.

## Acceptance Criteria

- [ ] TUI displays todo list (read-only)
- [ ] Agent can create/update/delete todos via tools
- [ ] Todo list persists with session
- [ ] Priority and dependency support with cycle detection
- [ ] Slash commands are view-only
- [ ] JSON and Markdown export
- [ ] Unit + integration tests (>80% coverage target)

## Required Reads

- [GitHub Issue #8](https://github.com/wjhuang88/talos/issues/8)
- `docs/backlog/active/SESSION-001-interactive-session-lifecycle.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `crates/talos-core/src/session.rs`
- `crates/talos-session/src/`
- `crates/talos-agent/src/`
- `crates/talos-tui/src/`

## Implementation Phases

### Phase 1: Data Model + Agent Tools
- TodoItem/TodoList structs, SQLite schema, TodoRepository CRUD
- Agent tools (todo_create, todo_update_status, todo_query)
- Cyclic dependency detection
- Tests

I078/T121-A activation (2026-07-02): start with `talos-session` data model, SQLite repository,
CRUD, query, and dependency cycle detection. Agent tool registration is intentionally left to the
next packet after the crate boundary is confirmed, because `talos-tools` does not currently depend
on `talos-session`.

T121-A implementation (2026-07-02): added `talos_session::todo` with `TodoRepository`,
`TodoItem`, status/priority enums, query/update structs, SQLite schema initialization,
session-scoped CRUD/query, dependency edge management, and cycle detection. `SessionManager` now
opens the colocated todo repository through `todo_repository()`.

Validation:

- `cargo test -p talos-session todo`
- `cargo test -p talos-session`
- `cargo clippy -p talos-session -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `scripts/validate_project_governance.sh .`

### Phase 2: TUI + Slash Commands
- TodoPanel component (read-only)
- /todo slash commands (view/list/show/stats/export)
- Session lifecycle integration

### Phase 3: Prompt Integration
- Inject active todo items into agent system prompt
- Budget-bounded, cache-stable
