# Iteration I077: Month 2 — Plugin, Exec, And Web Security

> Document status: Planned
> Published plan date: 2026-07-01
> Planned objective: Execute weeks 5-8 of the 2026-07-01 replan: plugin MVP security review,
> read-only plugin tool integration if cleared, WEB-001/WEB-005 security review, and direct exec
> permission policy plus implementation if cleared.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: explicit security decisions for plugin/web/exec boundaries with tests for any
> cleared implementation.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| T110 | PLUGIN-001 | In Progress | T46/ADR-032 | Plugin MVP security review |
| T111 | PLUGIN-001 | Planned | T110 | Read-only plugin AgentTool if cleared |
| T112 | WEB-001/WEB-005 | Planned | T42/T47 | Web/browser security review |
| T113 | WEB-001/WEB-005 | Planned | T112 | Hardening fixes |
| T114 | TOOL-016/PERM-001 | Planned | Issue #16 | Exec permission policy |
| T115 | TOOL-016 | Planned | T114 | Direct exec tool if cleared |
| T116 | Replan | Planned | T110-T115 | Month-2 closeout |

### Scope

- Plugin, web/browser, and exec security reviews.
- Implementation only after the relevant security gate clears.

### Non-Goals

- No write-capable plugin tools.
- No remote dashboard access.
- No default-allow process execution.
- No real publish.

### Acceptance

- Given plugin runtime is reviewed, when T111 starts, then permission/provenance gaps are closed or implementation is deferred.
- Given dashboard/browser review completes, when fixes land, then no secret leakage or auth bypass exists.
- Given exec policy is accepted, when `exec` runs, then command, cwd, env, and timeout are permission-gated.

### Planned Validation

- Targeted plugin/dashboard/tools/permission tests
- `cargo test --workspace` at T116
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- Plugin/web/exec owner docs
- ADR or owner-doc decision for exec policy
- Issue #16 status comments
- `docs/BOARD.md`

### Risks And Rollback

- Risk: plugin or exec broadens execution authority. Rollback: keep adapters non-presented and mark blocked.
- Risk: dashboard grows beyond loopback MVP. Rollback: keep API/root index only.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-01 | Planning | Created as Month 2 shell for the replan. |

## Verification Evidence

- Pending.

## Variance And Residuals

- Pending.

## Retrospective

- Pending.
