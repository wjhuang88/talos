# Iteration I078: Month 3 — Session Orchestration, Todo, Memory, And Thinking

> Document status: Active (2026-07-02)
> Published plan date: 2026-07-01
> Planned objective: Execute weeks 9-12 of the 2026-07-01 replan: slash panel auto-execute,
> session todo foundations, self-bootstrap rehearsal with validation loop, thinking preview without
> durable history pollution, and bounded todo prompt integration.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: session-level orchestration features that support real self-bootstrap work without
> corrupting durable history or prompt budgets.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| T120 | TUI-016 | Review | TUI-010/CMD-001 | Slash smart auto-execute |
| T121-A | TODO-001 | Review | SESSION-001 | Todo data model + repository |
| T121-B | TODO-001 | Review | T121-A | Initial agent todo tool API |
| T121-C | TODO-001 | Review | T121-B | Remaining todo mutation tools |
| T122 | TODO-001 | Planned | T121-C | Read-only slash/TUI views |
| T123 | REL-002 | Planned | T108/T122 | Validation-backed rehearsal |
| T124 | TUI-020 | Planned | TUI-004/session docs | Thinking preview separated from history |
| T125 | TODO-001 | Planned | T121/T122 | Bounded todo prompt integration |
| T126 | Replan | Planned | T120-T125 | Month-3 closeout |

### Scope

- Slash command UX improvement.
- Session todo storage, tools, read-only views, and bounded prompt integration.
- Thinking content history boundary.
- Self-bootstrap rehearsal using validation loop.

### Non-Goals

- No cross-session todo inheritance.
- No unbounded prompt injection.
- No persistence of transient thinking as normal history.

### Acceptance

- Given slash menu selection, when Enter is pressed, then direct commands execute and parameterized commands fill input.
- Given todo tools run, when persisted, then dependencies are acyclic and session-scoped.
- Given thinking streams, when finalized, then history and resume do not replay thinking text.
- Given rehearsal completes, when evidence is reviewed, then validation was run by Talos or the remaining gap is explicit.

### Planned Validation

- Targeted TUI/session/agent/todo tests
- `cargo test --workspace` at T126
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- TODO-001, TUI-016, TUI-020 owner docs
- Issue #7, #8, #15 status comments
- Self-bootstrap evidence record

### Risks And Rollback

- Risk: todo prompt injection disrupts cache prefixes. Rollback: keep prompt integration disabled.
- Risk: thinking separation loses final output. Rollback: retain only final assistant message path.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-01 | Planning | Created as Month 3 shell for the replan. |
| 2026-07-02 | Activation | Activated after I077/T116 closeout. First packet is T120 slash smart auto-execute. |
| 2026-07-02 | Inventory | Non-terminal iteration inventory before activation: I078 Planned -> activated; I079 Planned -> remains planned; I058/I047/I057 Review -> legacy review rows left untouched; I046 Planned -> legacy stale row left untouched. |
| 2026-07-02 | T120 Implementation | Slash panel Enter now runs DirectExecution commands and fills the composer for RequireInput commands; Tab remains completion-only. |
| 2026-07-02 | T121-A Activation | Started TODO-001 Phase 1 with `talos-session` data model/repository first; agent tool registration deferred to T121-B to avoid an unreviewed `talos-tools` -> `talos-session` dependency. |
| 2026-07-02 | T121-A Implementation | Added `talos_session::todo` repository, SQLite schema, CRUD/query, dependency edge management, cycle detection, and `SessionManager::todo_repository()`. |
| 2026-07-02 | T121-B Implementation | Added initial agent todo tools (`todo_create`, `todo_update_status`, `todo_query`) in `talos-session` and registered them in print/TUI registries through permission-aware wrappers. |
| 2026-07-02 | T121-C Implementation | Added remaining mutation tools (`todo_update`, `todo_delete`, `todo_add_dependency`, `todo_remove_dependency`) with registry coverage and dependency cycle test coverage. |

## Verification Evidence

- T120 targeted: `cargo test -p talos-tui slash_menu` passed.
- T120 targeted: `cargo test -p talos-conversation complete_slash_command` passed.
- T120 crate validation: `cargo test -p talos-tui` passed.
- T120 crate validation: `cargo test -p talos-conversation` passed.
- T120 lint: `cargo clippy -p talos-tui -p talos-conversation -- -D warnings` passed.
- T120 workspace compile: `cargo check --workspace` passed.
- Governance: `scripts/validate_project_governance.sh .` passed with 0 warnings.
- T121-A targeted: `cargo test -p talos-session todo` passed.
- T121-A crate validation: `cargo test -p talos-session` passed.
- T121-A lint: `cargo clippy -p talos-session -- -D warnings` passed.
- T121-A format: `cargo fmt --all -- --check` passed.
- T121-A workspace compile: `cargo check --workspace` passed.
- Governance: `scripts/validate_project_governance.sh .` passed with 0 warnings after T121-A.
- T121-B targeted: `cargo test -p talos-session todo` passed.
- T121-B registry: `cargo test -p talos-cli registry` passed.
- T121-B lint: `cargo clippy -p talos-session -p talos-cli -- -D warnings` passed.
- T121-B format: `cargo fmt --all -- --check` passed.
- T121-B workspace compile: `cargo check --workspace` passed.
- Governance: `scripts/validate_project_governance.sh .` passed with 0 warnings after T121-B.
- T121-C targeted: `cargo test -p talos-session todo_tools` passed.
- T121-C crate validation: `cargo test -p talos-session` passed.
- T121-C registry: `cargo test -p talos-cli registry` passed.
- T121-C lint: `cargo clippy -p talos-session -p talos-cli -- -D warnings` passed.
- T121-C format: `cargo fmt --all -- --check` passed.
- T121-C workspace compile: `cargo check --workspace` passed.
- Governance: `scripts/validate_project_governance.sh .` passed with 0 warnings after T121-C.

## Variance And Residuals

- No scope variance at activation.
- T121 residual resolved: agent-side write tools now cover create, update status, update fields,
  delete, add dependency, remove dependency, and query. T122 read-only user views remain pending.

## Retrospective

- Pending.
