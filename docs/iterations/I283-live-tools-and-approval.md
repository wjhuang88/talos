# Iteration I283: Desktop Tools Approval And Cancellation

> Document status: Planned
> Plan date: 2026-09-22
> Target window: 2026-09-29 to 2026-10-05
> Planned objective: A Desktop user can execute real file/shell work, understand Auto decisions, allow or deny exact requests, and cancel safely.
> Baseline rule: once committed, preserve scope and append execution facts; changed targets use a new ID.
> MVP deliverable: A Desktop user can execute real file/shell work, understand Auto decisions, allow or deny exact requests, and cancel safely.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | DESKTOP-001-D5 / I283: Desktop Tools Approval And Cancellation; planned scope only |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable until activation is effective |
| Authorization Mode | Not applicable |
| Authorization Evidence | I282 implementation merged as `aaa4c015`; activation remains proposed in #601 |
| Implementation PR | Not started |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Establish an effective serial child claim before implementation; no release or new permission authority |

## Published Baseline

### Selected Story And Dependencies

[DESKTOP-001-D5](../backlog/active/DESKTOP-001-D5-live-tools-and-approval.md), parent DESKTOP-001;
Planned / Unclaimed, not yet Ready or activated.
Prerequisites: I282 implementation merged and technical gates passed; deferred human rows tracked; current permission/Auto contracts.
Follow [the four-week task](../tasks/2026-09-22-desktop-four-week-delivery.md) for
serial scheduling, inventory, resource limits, authorization and deferred acceptance. Dates do not
activate implementation. Resolve readiness, then use one atomic claim/activation transition.

### Scope

- Compose existing Runtime tools and permission policy; show call identity, progress, output and real terminal outcome without duplicating execution authority.
- Bridge existing scoped ApprovalHandler and explanation surfaces to an accessible localized UI; preserve existing choice semantics and explicit Deny.
- Expose existing Auto decision/reason when provided, including model-not-consulted and human-needed states. Do not expand Auto or add a second classifier.
- Bind UI responses to one pending request and task; stale, duplicate, closed-channel or cancelled replies cannot authorize later work.
- Make explicit cancel work during model, tool and approval states; distinguish interruption requested from confirmed stopped, with no silent replay.

### Non-Goals

New permission policy, shell safety heuristics, sandbox fallback changes, CLI/TUI behavior changes, multi-client approvals or persistent broad grants.

### Acceptance

- Given a read-only tool call, display its true call/result association and exactly one terminal result.
- Given an Ask request, show tool, bounded scope and available explanation; Once/Session/Deny produce existing Runtime semantics without UI-invented grants.
- Given a denied write or cancelled prompt, no write occurs; late/double replies cannot approve another request.
- Given Auto evaluates an eligible request, the UI exposes the actual decision status without claiming a model was consulted when it was not.
- Given a running tool or pending approval, cancel and shutdown terminate observation/request lifetimes safely; tests prove no replay of side effects.

### Planned Validation

Deterministic fake tools/provider tests for Allow/Deny/Once/Session, Auto visibility, concurrent same-name IDs, stale replies, cancellation and shutdown; protected permission/security/API review; native rows H2-H3.

Use the pinned toolchain and --locked. Before a stable code candidate, run focused checks,
./scripts/release_preflight.sh, both governance validators, diff/secret review and applicable
independent Agent-role review. Exact-head remote CI validates the stable stage. Planning-only
changes do not require Rust builds. Required human rows remain Review until accepted.

### Documentation To Update

- Desktop crate README with delivered launch/behavior instructions (create in I282).
- README.md and README.zh-CN.md; site English/Chinese capability instructions where affected.
- Child owner, this iteration, four-week task, parent summary, indexes and #29 acceptance evidence.

### Risks And Rollback

Approval identity/cancellation races are protected blockers. Keep tool mode unavailable if failed; do not fall back to unconditional allow.
Preserve working mock/preceding stage. Disable incomplete live features rather than weakening
safety. Roll back only the failing slice; no destructive cleanup of user sessions or workspaces.

## Actual Activation And Execution

2026-09-22: activation proposal prepared after I282 implementation merge `aaa4c015`; owner
remains Unclaimed until the governance PR reaches `main`.

## Verification Evidence

Implementation checks: not run; activation only, no implementation exists for this child.
Planning checks are recorded centrally in the four-week task.

## Completion Evidence

Completion Commit: pending.
Only already-existing implementation/evidence commits may close this iteration.

## Variance And Residuals

None yet. Carry eligible human rows to #29 / I285 without transferring protected security gates.
