# Iteration I124: One-Shot Scheduled Follow-Up

> Document status: Planned
> Published plan date: 2026-07-13
> Planned objective: deliver one safe, session-scoped delayed follow-up through the normal queue
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a user can ask Talos to follow up after a bounded delay, and it fires exactly once

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SF100 | SCHED-001 | Ready | permission matrix | internal command/event contract and labeled source |
| SF101 | SCHED-001 | Ready | SF100 | cancellation-aware one-shot actor |
| SF102 | SCHED-001 | Ready | SF101 | Execute/Ask `delay` tool wired in supported modes |
| SF103 | SCHED-001 | Ready | SF102 | deterministic and fixture-provider runtime proof |

### Scope

- bounded seconds-based one-shot scheduling; session/process lifetime only;
- existing queue, cancellation, registry, permission, and status mechanisms;
- Execute/Ask registration and fresh permission decisions for later tool calls.

### Non-Goals

- recurrence, list/cancel surface, persistence, cron/calendar time, direct tool execution;
- permission defaults, public APIs, session encoding, dependencies, release work.

### Acceptance

- Given a valid delayed request, when it is approved and time advances, then one visibly labeled
  follow-up enters the normal session flow exactly once.
- Given Deny, invalid duration, shutdown, or a closed queue, then no follow-up fires and Talos does
  not panic or retry forever.
- Given the follow-up asks for another tool, then that tool receives its normal independent
  permission decision.

### Planned Validation

- focused agent/tool/CLI tests with paused Tokio time;
- fixture-provider conversation test through the real engine;
- repository standard validation ladder from the program plan.

### Documentation To Update

- `README.md` scheduling section and SCHED-001 owner record;
- iteration index, product backlog summary, and Board.

### Risks And Rollback

- Risk: injection looks like fresh user authority or survives shutdown.
- Rollback: keep the tool unregistered; actor remains unreachable until all permission/lifecycle
  tests pass.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-13 | Planning | Published as Planned; no implementation or activation claimed. |

## Verification Evidence

- Pending activation.

## Variance And Residuals

- Recurrence and operator controls are owned by I125-I126.

## Retrospective

- Pending.
