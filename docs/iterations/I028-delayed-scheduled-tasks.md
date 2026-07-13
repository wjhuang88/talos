# I028: Delayed and Scheduled Task Execution

**Status**: Superseded before implementation (2026-07-13)
**Started**: 2026-06-18 (plan opened)
**Depends On**: None (greenfield feature)

> This published baseline is preserved for history. Its `ToolNature::Read, auto-allow` premise for
> mutating scheduling operations conflicts with the current permission architecture. The changed,
> Ask-gated and incrementally runnable acceptance target is owned by I124-I127 and
> `docs/tasks/2026-07-13-four-month-scheduled-followups-plan.md`. Do not activate I028.

## Outcome

Add 4 built-in tools (`delay`, `schedule`, `cancel_scheduled_task`, `list_scheduled_tasks`)
that let the LLM register session-scoped delayed or recurring actions. Trigger action is
message injection via `SessionOp::Submit`; the LLM mediates any subsequent tool calls through
the normal permission pipeline. No external scheduling crate; raw `tokio::time::sleep` +
`tokio::time::interval`.

## Selected Stories

- [ ] #SCHED-001-A: Define `ScheduleCommand` enum + `ScheduledTaskInfo` in `talos-core`
- [ ] #SCHED-001-B: Implement `SchedulerActor` in `talos-agent` (tokio task, task HashMap, sq_tx injection)
- [ ] #SCHED-001-C: Implement 4 tools in `talos-tools` (DelayTool, ScheduleTool, CancelScheduledTaskTool, ListScheduledTasksTool)
- [ ] #SCHED-001-D: Wire scheduler + tools in `talos-cli` composition root
- [ ] #SCHED-001-E: Update system prompt so LLM knows the scheduling tools exist

## Architecture

```
┌─ talos-core/src/session.rs ──────────────────────┐
│  ScheduleCommand enum:                            │
│    OneShot { task_id, message, delay_secs }       │
│    Recurring { task_id, message, interval_secs }  │
│    Cancel { task_id }                              │
│    List { reply: oneshot::Sender<Vec<ScheduledTaskInfo>> } │
│                                                    │
│  ScheduledTaskInfo { id, message, kind, secs }     │
└────────────────────────────────────────────────────┘
           ▲                              │
           │ mpsc::Sender<ScheduleCommand>│
           │                              ▼
┌─ talos-tools/src/scheduler.rs ────────────────────┐
│  DelayTool { tx }       → schedule OneShot         │
│  ScheduleTool { tx }    → schedule Recurring       │
│  CancelScheduledTaskTool { tx } → Cancel           │
│  ListScheduledTasksTool { tx } → List              │
│  All: ToolNature::Read, auto-allow                 │
└────────────────────────────────────────────────────┘
           │ sends ScheduleCommand
           ▼
┌─ talos-agent/src/scheduler.rs ────────────────────┐
│  SchedulerActor {                                 │
│    cmd_rx: Receiver<ScheduleCommand>,             │
│    sq_tx: Sender<SessionOp>,                      │
│    tasks: HashMap<String, JoinHandle<()>>,        │
│    task_meta: HashMap<String, ScheduledTaskInfo>, │
│    cancel_token: CancellationToken,               │
│  }                                                 │
│                                                    │
│  run(): select! on cmd_rx.recv() / cancel_token   │
│  OneShot: tokio::spawn(sleep → sq_tx.send(Submit))│
│  Recurring: tokio::spawn(interval loop → sq_tx)   │
│  Cancel: abort JoinHandle, remove from maps       │
│  List: reply via oneshot channel                  │
└────────────────────────────────────────────────────┘
           ▲ spawned by
           │
┌─ talos-cli/src/main.rs ───────────────────────────┐
│  let (sched_tx, sched_rx) = mpsc::channel(64);     │
│  let scheduler = SchedulerActor::new(              │
│    sched_rx, handle.sq_tx.clone(), cancel_token    │
│  );                                                │
│  tokio::spawn(scheduler.run());                    │
│  // register 4 tools with sched_tx clone           │
│  // in all tool registry builders                  │
└────────────────────────────────────────────────────┘
```

## Risks

- **R1 (timing)**: `tokio::time::interval` has `MissedTickBehavior` semantics — need to set
  `MissedTickBehavior::Delay` so a backed-up scheduler doesn't fire bursts.
- **R2 (injection ordering)**: if the agent is mid-turn when a scheduled task fires, the
  `SessionOp::Submit` sits in the bounded SQ (cap=512) until the current turn completes. This
  is acceptable (matches Claude Code's "defer until between turns" pattern) but should be
  documented.
- **R3 (task ID generation)**: use `uuid` (already a workspace dep) or a simple counter.
  Counter is simpler and sufficient for session-scoped tasks.
- **R4 (cancel during sleep)**: `tokio::time::Sleep` is not directly cancellable via
  `JoinHandle::abort` — but `JoinHandle::abort` DOES cancel the spawned task, which includes
  the sleep. Verify this works in tests.

## Commit Strategy

Five atomic commits, one per story:

1. `feat(core): add ScheduleCommand and ScheduledTaskInfo types (#SCHED-001-A) [model:glm-5.2]`
2. `feat(agent): implement SchedulerActor for delayed/recurring tasks (#SCHED-001-B) [model:glm-5.2]`
3. `feat(tools): add delay, schedule, cancel, list scheduled task tools (#SCHED-001-C) [model:glm-5.2]`
4. `feat(cli): wire scheduler actor and register scheduling tools (#SCHED-001-D) [model:glm-5.2]`
5. `docs(agent): update system prompt for scheduling tools (#SCHED-001-E) [model:glm-5.2]`

## Verification Log

(to be filled as stories land)
