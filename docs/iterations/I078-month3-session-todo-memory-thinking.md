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
| T122 | TODO-001 | Review | T121-C | Read-only slash/TUI views |
| T123 | REL-002 | Review | T108/T122 | Validation-backed rehearsal |
| T124 | TUI-020 | Review | TUI-004/session docs | Thinking preview separated from history |
| T125 | TODO-001 | Review | T121/T122 | Bounded todo prompt integration |
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
| 2026-07-02 | T122 Activation | Started read-only todo slash/TUI views. Slash commands must route through the active session runtime and remain view/export only. |
| 2026-07-02 | T122 Implementation | Added typed read-only `/todo` slash requests, active-session repository rendering in CLI runtime, JSON/Markdown export text, and TUI todo panel scrollback rendering. |
| 2026-07-02 | T123 Rehearsal | Recorded `docs/tasks/2026-07-02-self-bootstrap-rehearsal-t123-todo-views.md`. Talos generated a workspace validation plan, but Codex remained the primary executor; this is gap evidence, not a REL-002 qualifying session. |
| 2026-07-02 | T124 Implementation | Added transient `ThinkingDelta`/`ThinkingPreview` flow so active thinking is visible in the live preview but excluded from finalized assistant text, scrollback history, JSONL persistence, and resume history. |
| 2026-07-02 | T125 Implementation | Added bounded active todo prompt integration outside the stable cacheable prefix for TUI/inline session actors, including new/resume/fork/model-switch actor rebuilds. |

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
- T122 targeted: `cargo test -p talos-conversation todo` passed.
- T122 targeted: `cargo test -p talos-cli todo` passed.
- T122 targeted: `cargo test -p talos-tui todo_panel` passed.
- T122 crate validation: `cargo test -p talos-conversation` passed.
- T122 crate validation: `cargo test -p talos-cli` passed.
- T122 crate validation: `cargo test -p talos-tui` passed.
- T122 lint: `cargo clippy -p talos-conversation -p talos-cli -p talos-tui -- -D warnings` passed.
- T122 format: `cargo fmt --all -- --check` passed.
- T122 workspace compile: `cargo check --workspace` passed.
- Governance: `scripts/validate_project_governance.sh .` passed with 0 warnings after T122.
- T123 Talos validation plan: `./target/debug/talos validate plan --profile workspace` passed.
- T123 Talos validation plan JSON: `./target/debug/talos validate plan --profile workspace --json`
  passed.
- T124 format: `cargo fmt --all -- --check` passed.
- T124 crate validation: `cargo test -p talos-core` passed.
- T124 crate validation: `cargo test -p talos-conversation` passed.
- T124 crate validation: `cargo test -p talos-session` passed.
- T124 crate validation: `cargo test -p talos-agent` passed.
- T124 crate validation: `cargo test -p talos-cli` passed.
- T124 crate validation: `cargo test -p talos-tui` passed.
- T124 lint: `cargo clippy -p talos-core -p talos-agent -p talos-conversation -p talos-session -p talos-cli -p talos-tui -- -D warnings` passed.
- T124 workspace compile: `cargo check --workspace` passed.
- T125 format: `cargo fmt --all -- --check` passed.
- T125 crate validation: `cargo test -p talos-agent` passed.
- T125 crate validation: `cargo test -p talos-cli` passed.
- T125 crate validation: `cargo test -p talos-session` passed.
- T125 lint: `cargo clippy -p talos-agent -p talos-cli -p talos-session -- -D warnings` passed.
- T125 workspace compile: `cargo check --workspace` passed.
- Governance: `scripts/validate_project_governance.sh .` passed with 0 warnings after T125.

## Variance And Residuals

- No scope variance at activation.
- T121 residual resolved: agent-side write tools now cover create, update status, update fields,
  delete, add dependency, remove dependency, and query.
- T122 is in Review for read-only user views.
- T123 is in Review as a validation-backed rehearsal evidence record. It explicitly does not
  satisfy REL-002 because Codex remained the primary executor and Talos only generated a read-only
  validation plan.
- T124 is in Review for the thinking-preview history boundary. No provider-specific reasoning
  parser was added; this slice only handles the internal event/UI/session boundary.
- T125 is in Review for bounded todo prompt integration. Print mode remains out of scope because it
  does not own a durable session in this slice.

## Retrospective

- Pending.
