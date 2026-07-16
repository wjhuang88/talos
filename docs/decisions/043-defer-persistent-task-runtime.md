# 043: Defer TASK-001 Persistent Task Runtime

> Status: Accepted (Defer)
> Date: 2026-07-16
> Iteration: I132 / P130
> Revised: 2026-07-16 (review v1 — corrected capability assessment)

## Context

TASK-001 asked for an ADR-ready recommendation on persistent resumable task capability before any
engine is implemented. The spike must decide task/turn/session identity, checkpoint storage, crash
recovery, cancellation, retention, cleanup, and permission re-authorization after resume — or
explicitly defer/reject.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|---|---|---|---|
| No global pub/sub bus | Hard | ADR-006 | No |
| No speculative features | Hard | AGENTS.md #7 | No |
| All write-capable tools gated by permissions | Hard | AGENTS.md #4 | No |
| No scheduler, daemon, direct tool path, permission reuse | Hard | P130 non-goals | No |
| Crate public APIs are semver-bound | Hard | AGENTS.md #6 | No |

## Evidence: Reusable Components vs. Unsatisfied Requirements

Existing iterations provide reusable building blocks, but the persistent, multi-phase, resumable
task runtime itself is **not implemented**. Each TASK-001 requirement is assessed below.

| TASK-001 requirement | Status | Existing component | Gap |
|---|---|---|---|
| Session identity (external ID → stable handle) | **Partially satisfied** | I128/ADR-042: opaque external ID → UUID TLOG binding | Session identity exists; no TaskId or task-state entity above session |
| Atomic turn persistence | **Satisfied** (as session primitive) | I128: successful turn TLOG written atomically | This is turn-level persistence, not task-level checkpoint or phase progress |
| Crash recovery for successful turns | **Satisfied** (for committed turns only) | I128: auto-recovery from committed TLOG entries | Only successful turns recover; incomplete/uncommitted turns leave no records — no incomplete-task recovery |
| **Task lifecycle (create/suspend/resume/complete/abort)** | **Not satisfied** | — | No TaskId, TaskState, phase tracking, or task-level lifecycle exists |
| **Phase-level checkpoint storage** | **Not satisfied** | — | TLOG records turn messages, not task phase progress, branch points, or workflow state |
| **Incomplete-task recovery** | **Not satisfied** | — | No mechanism to resume an interrupted multi-phase task from its last completed phase |
| **Durable (cross-restart) scheduling** | **Not satisfied** | I124-I27 scheduler is session-scoped and non-persistent | Scheduled follow-ups are cancelled on process exit; no cross-restart task scheduling |
| Cancellation | **Partially satisfied** | I124-I127: Cancel/Shutdown for scheduled follow-ups; CancellationToken in session actor | Covers scheduler follow-ups and session turns; no task-level cancellation semantic |
| Permission re-authorization after resume | **Partially satisfied** | I124-I127: fresh Allow/Ask/Deny on every scheduled fire | Covers scheduled follow-ups; no task-resume permission re-authorization semantic exists |
| Retention/cleanup | **Partially satisfied** | Session storage: status/cleanup/maintenance commands | Session-level retention; no task-level retention or phase-aware cleanup |

### Key distinction

I128 and I124-I127 provide reusable session persistence, atomic turns, in-process cancellation,
and per-fire permission evaluation as **building blocks**. They do NOT constitute a persistent,
multi-phase, resumable task runtime. The task lifecycle, phase checkpoints, incomplete-task
recovery, and durable scheduling requirements are **unsatisfied**.

## Decision: Defer Implementation

TASK-001's persistent, multi-phase, resumable task runtime is **not implemented**. This ADR defers
implementation because:

1. **No clear product need exists today** for a task runtime beyond what the `LONG-RUNNING-TASK.md`
   SOP provides at the process/documentation level.
2. **P130 non-goals prohibit implementation.** A task engine implies a scheduler, daemon, or
   autonomous execution path — all explicitly excluded by the P130 contract.
3. **AGENTS.md #7** (no speculative features) prevents building a task engine "in case" a need
   appears later.

The existing I128 and I124-I127 components are **reusable foundations** if a future product need
justifies a task runtime, but they are not themselves a task runtime.

## What This Decision Does NOT Approve

- No task engine, TaskId/TaskState, cron service, or autonomous background process.
- No durable (cross-restart) scheduler beyond I124-I127's session-scoped, non-persistent pattern.
- No global event bus or multi-subscriber task notification (ADR-006).
- No permission reuse or bypass on resume.
- No multi-agent orchestration.
- No claim that the capability is "delivered" or "substantially delivered."

## Reversal Trigger

Revisit this defer if **any** of the following holds:

1. A concrete product need requires **cross-restart task lifecycle** (create/suspend/resume/abort
   surviving process restart); **or**
2. A need requires **phase-level checkpoint storage and incomplete-task recovery** (resume from
   last completed phase, not just last successful turn); **or**
3. A need requires **durable scheduling** (tasks that survive process restart and fire
   autonomously); **or**
4. A need requires **task-level permission re-authorization** distinct from session-level
   permission evaluation.

Any such re-evaluation must preserve ADR-006 (no global bus), AGENTS.md #4 (permission gating),
and #7 (no speculative features).

## Proves No Global Bus, Direct Tool Path, or Autonomous Process Required

- **No global bus**: This defer introduces no event flow. A future task runtime would use the
  existing SQ/EQ seam (ADR-005) and TLOG file I/O, not a global bus.
- **No direct tool path**: A future task runtime would execute tools through the normal agent turn
  loop and permission pipeline. No bypass is approved.
- **No autonomous process**: This defer approves no process. Sessions remain host-driven.

## Related

- [ADR-006: Event Architecture Boundary](006-event-architecture-boundary.md)
- [ADR-041: Scheduler Minimal Public API](041-scheduler-minimal-public-api.md)
- [ADR-042: Embedded Durable Runtime Session Boundary](042-embedded-durable-runtime-session-boundary.md)
- [TASK-001 owner doc](../backlog/active/TASK-001-persistent-task-runtime-spike.md)
- [I128](../iterations/I128-embedded-durable-runtime-sessions.md)
- [I124-I127 scheduled follow-ups](../tasks/2026-07-13-scheduled-followups-execution-package.md)
- `docs/sop/LONG-RUNNING-TASK.md`
