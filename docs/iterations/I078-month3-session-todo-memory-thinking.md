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
| T120 | TUI-016 | In Progress | TUI-010/CMD-001 | Slash smart auto-execute |
| T121 | TODO-001 | Planned | SESSION-001 | Todo data model + agent tool API |
| T122 | TODO-001 | Planned | T121 | Read-only slash/TUI views |
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

## Verification Evidence

- Pending for T120.

## Variance And Residuals

- No scope variance at activation.

## Retrospective

- Pending.
