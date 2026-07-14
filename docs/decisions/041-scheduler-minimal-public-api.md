# 041: Scheduler Minimal Public API Boundary

**Status**: Accepted (2026-07-14)
**Decision authority**: Maintainer direction during I124 third-review closure

## Context

I124 implements session-scoped scheduled follow-ups in `talos-agent`. The scheduler module owns
command/event types, an actor, and a delay tool. The CLI composition roots (`talos-cli`) need to:

1. Create the delay tool before building the tool registry (two-phase composition).
2. Spawn the scheduler actor after the session provides `sq_tx`.

These two operations cross the crate boundary: `talos-cli` must call into `talos-agent` to obtain
the tool and spawn the actor. Without any public API, the CLI cannot wire the scheduler.

The I124 published baseline said "crate-private scheduler commands/events without changing public
semver-bound APIs." The command/event types (`ScheduleCommand`, `ScheduledTaskInfo`, etc.) are
indeed `pub(crate)`. However, two items must be public for cross-crate composition.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Crate public APIs are semver-bound | Hard | AGENTS.md #6 | No |
| No speculative features | Hard | AGENTS.md #7 | No |
| Preserve published iteration baselines; append changes | Hard | AGENTS.md / CHANGE-CONTROL | No |
| Program non-goal: no new public API | Soft | scheduled-followups program plan | Yes, only with explicit maintainer direction and an ADR |
| Stop and request direction before changing public API | Hard process gate | scheduled-followups program plan | Satisfied on 2026-07-14 |
| SF100 baseline: "crate-private contract" | Soft | I124 plan | Yes, through appended change control |
| CLI needs cross-crate factory | Hard | Architecture | No |

## Reasoning

The simplest approach satisfying all Hard constraints: expose exactly two items from
`talos-agent`:

- `create_delay_tool_and_scheduler() -> (Arc<dyn AgentTool>, PendingSchedulerActor)` — factory
  returning the delay tool as a trait object (concrete type hidden) and a pending actor.
- `PendingSchedulerActor` — public struct with a private field and one public method
  `spawn(self, sq_tx, cancel_token) -> JoinHandle`.

All other scheduler types (`SchedulerHandle`, `DelayTool`, `ScheduleCommand`,
`ScheduledTaskInfo`, `SchedulerActor`, etc.) remain `pub(crate)`. The CLI never names these
types — it receives `Arc<dyn AgentTool>` and wraps it in the mode-appropriate permission wrapper.

### Why not alternatives?

- **Put the factory in `talos-core`**: `talos-core` cannot depend on `talos-agent` (architecture
  rule: `talos-core` depends on nothing). The delay tool's implementation lives in `talos-agent`.
- **Put the factory in `talos-cli`**: `talos-cli` would need access to `DelayTool::new()` and
  `SchedulerHandle`, which are `pub(crate)`. Exposing those is a larger API expansion.
- **Use a trait in `talos-core`**: over-engineering for a single factory function.

## Decision

Approve two additive public items on `talos-agent`:

1. `pub fn create_delay_tool_and_scheduler() -> (Arc<dyn AgentTool>, PendingSchedulerActor)`
2. `pub struct PendingSchedulerActor` (with private field, `pub fn spawn()`)

Module visibility: `mod scheduler` (private module), items re-exported via
`pub use scheduler::{create_delay_tool_and_scheduler, PendingSchedulerActor}`.

Semver treatment: additive for 0.x — no existing public item is changed or removed.

This is a narrowly scoped change to the published I124/program non-goal, not a replacement of the
published baseline. The objective, acceptance criteria, dependency boundary, and user-visible
deliverable are unchanged. On 2026-07-14, after the third I124 review identified this unresolved
variance, the maintainer directed the agent to fix the blockers and close the iteration. That
direction satisfies the program plan's stop condition and accepts only these two composition
exports. No other scheduler type or future I125-I127 surface is pre-approved.

## Reversal Trigger

Revisit if `talos-runtime` should own the scheduler factory instead (SDK embedding path), or if
I125-I127 require additional public surface that would warrant a different boundary.
