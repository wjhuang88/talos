# 043: Defer TASK-001 Persistent Task Runtime — Capability Substantially Delivered

> Status: Accepted (Defer)
> Date: 2026-07-16
> Iteration: I132 / P130

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

## Evidence: Existing Infrastructure Coverage

### I128 / ADR-042 (Embedded Durable Runtime Sessions)

| TASK-001 requirement | I128 delivery |
|---|---|
| Task/turn/session identity | External ID → UUID TLOG binding; session ID is the stable task identity. Host owns only the opaque external ID. |
| Talos-owned checkpoint storage | TLOG IS the checkpoint. Successful turns are written atomically (temp file + rename). Committed entries have stable IDs for replay. |
| Crash recovery | Runtime auto-recovery from committed TLOG entries. Failed, interrupted, denied, and uncommitted turns leave NO durable messages — clean recovery boundary. |
| Retention/cleanup | Session storage has `status`, `cleanup`, and `maintenance` commands. Workspace-scoped topology enables targeted cleanup. |

### I124-I127 / ADR-041 (Scheduled Follow-Ups)

| TASK-001 requirement | I124-I127 delivery |
|---|---|
| Cancellation | `Cancel` and `Shutdown` lifecycle operations with Execute/Ask semantics. CancellationToken in session actor. |
| Permission re-authorization after resume | Fresh permission evaluation on every fire — no permission reuse. Resumed write-capable actions receive a new Allow/Ask/Deny decision through the normal permission pipeline. |

### Permission re-authorization analysis

When a durable session is resumed after a crash:
1. The session replays committed TLOG messages as **read-only context** (history).
2. The next turn is a **fresh agent turn** with fresh tool execution.
3. Each tool execution goes through the **normal permission pipeline** (Allow/Ask/Deny).
4. Therefore, permission re-authorization happens naturally — the resumed context is history; new
   actions require fresh permission decisions. No separate re-authorization mechanism is needed.

## Decision: Defer

**TASK-001 is deferred.** The persistent resumable task capability is substantially delivered by
the combination of I128 (durable runtime sessions) and I124-I127 (scheduled follow-ups). A separate
"task engine" would:

1. **Duplicate existing infrastructure.** I128 already provides checkpoint storage, crash recovery,
   and session identity. I124-I127 already provides cancellation and permission re-authorization.
2. **Violate P130 non-goals.** A task engine implies a scheduler, daemon, or autonomous execution
   path — all explicitly excluded.
3. **Introduce speculative complexity.** No present need exists for a capability beyond what I128
   and I124-I127 deliver (AGENTS.md #7).

The long-running task pattern is already served by the `LONG-RUNNING-TASK.md` SOP, which operates
at the process/documentation level — checkpoints in task docs, not in a task engine. This is the
correct level of abstraction for Talos's current architecture.

## What This Decision Does NOT Approve

- No task engine, cron service, or autonomous background process.
- No new scheduler beyond I124-I127's existing Execute/Ask pattern.
- No global event bus or multi-subscriber task notification (ADR-006).
- No permission reuse or bypass on resume.
- No multi-agent orchestration.

## Reversal Trigger

Revisit this defer if **all** of the following hold:

1. A concrete need exists for task orchestration that **cannot** be expressed as a durable session
   (I128) plus scheduled follow-ups (I124-I127); **and**
2. The need requires cross-session coordination, multi-step workflow with branching, or persistent
   task state beyond what TLOG provides; **and**
3. The proposed design preserves ADR-006 (no global bus), AGENTS.md #4 (permission gating), and
   #7 (no speculative features).

## Proves No Global Bus, Direct Tool Path, or Autonomous Process Required

- **No global bus**: Session replay uses TLOG reads (file I/O), not event subscriptions. The SQ/EQ
  seam (ADR-005) handles all turn communication. No new event flow is introduced.
- **No direct tool path**: Resumed sessions execute tools through the normal agent turn loop and
  permission pipeline. No bypass exists.
- **No autonomous process**: Sessions are host-driven. The host decides when to resume, what
  external ID to use, and whether to allow the next turn. Talos does not self-schedule.

## Related

- [ADR-006: Event Architecture Boundary](006-event-architecture-boundary.md)
- [ADR-041: Scheduler Minimal Public API](041-scheduler-minimal-public-api.md)
- [ADR-042: Embedded Durable Runtime Session Boundary](042-embedded-durable-runtime-session-boundary.md)
- [TASK-001 owner doc](../backlog/active/TASK-001-persistent-task-runtime-spike.md)
- [I128](../iterations/I128-embedded-durable-runtime-sessions.md)
- [I124-I127 scheduled follow-ups](../tasks/2026-07-13-scheduled-followups-execution-package.md)
- `docs/sop/LONG-RUNNING-TASK.md`
